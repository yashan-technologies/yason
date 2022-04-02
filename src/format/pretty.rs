//! PrettyFormatter

use crate::format::{FormatResult, Formatter, WriteExt};
use crate::{DataType, Value};
use std::fmt;

struct PrettyOptions<'a> {
    indent: usize,
    newline_in_empty: bool,
    newline_in_nested: bool,
    kv_delimiter: &'a [u8],
}

impl<'a> PrettyOptions<'a> {
    #[inline]
    const fn new(indent: usize, newline_in_empty: bool, newline_in_nested: bool, kv_delimiter: &'a [u8]) -> Self {
        Self {
            indent,
            newline_in_empty,
            newline_in_nested,
            kv_delimiter,
        }
    }
}

pub struct PrettyFormatter<'a> {
    options: PrettyOptions<'a>,
    cur_indent_level: usize,
    has_value: bool,
}

impl<'a> PrettyFormatter<'a> {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            options: PrettyOptions::new(2, true, true, b" : "),
            cur_indent_level: 0,
            has_value: false,
        }
    }
}

impl Formatter for PrettyFormatter<'_> {
    #[inline]
    fn write_object_value<W: fmt::Write>(
        &mut self,
        key: &str,
        value: &Value,
        first: bool,
        writer: &mut W,
    ) -> FormatResult<()> {
        self.begin_object_key(first, writer)?;
        self.write_string(key, writer)?;
        self.end_object_key(writer)?;
        self.begin_object_value(writer)?;

        if matches!(value.data_type(), DataType::Object | DataType::Array) && self.options.newline_in_nested {
            writer.write_bytes(b"\n")?;
            indent(self.cur_indent_level, self.options.indent, writer)?;
        }

        self.write_value(value, writer)?;
        self.end_object_value(writer)
    }

    #[inline]
    fn begin_array<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        self.cur_indent_level += 1;
        self.has_value = false;
        writer.write_bytes(b"[")?;
        Ok(())
    }

    #[inline]
    fn end_array<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        self.cur_indent_level -= 1;

        if self.options.newline_in_empty || self.has_value {
            writer.write_bytes(b"\n")?;
            indent(self.cur_indent_level, self.options.indent, writer)?;
        }
        writer.write_bytes(b"]")?;
        Ok(())
    }

    #[inline]
    fn begin_array_value<W: fmt::Write>(&mut self, first: bool, writer: &mut W) -> FormatResult<()> {
        if first {
            writer.write_bytes(b"\n")?;
        } else {
            writer.write_bytes(b",\n")?;
        }
        indent(self.cur_indent_level, self.options.indent, writer)
    }

    #[inline]
    fn end_array_value<W: fmt::Write>(&mut self, _writer: &mut W) -> FormatResult<()> {
        self.has_value = true;
        Ok(())
    }

    #[inline]
    fn begin_object<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        self.cur_indent_level += 1;
        self.has_value = false;
        writer.write_bytes(b"{")?;
        Ok(())
    }

    #[inline]
    fn end_object<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        self.cur_indent_level -= 1;
        if self.options.newline_in_empty || self.has_value {
            writer.write_bytes(b"\n")?;
            indent(self.cur_indent_level, self.options.indent, writer)?;
        }
        writer.write_bytes(b"}")?;
        Ok(())
    }

    #[inline]
    fn begin_object_key<W: fmt::Write>(&mut self, first: bool, writer: &mut W) -> FormatResult<()> {
        if first {
            writer.write_bytes(b"\n")?;
        } else {
            writer.write_bytes(b",\n")?;
        }
        indent(self.cur_indent_level, self.options.indent, writer)
    }

    #[inline]
    fn begin_object_value<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(self.options.kv_delimiter)?;
        Ok(())
    }

    #[inline]
    fn end_object_value<W: fmt::Write>(&mut self, _writer: &mut W) -> FormatResult<()> {
        self.has_value = true;
        Ok(())
    }
}

#[inline]
fn indent<W: fmt::Write>(level: usize, indent: usize, writer: &mut W) -> FormatResult<()> {
    const SPACE_BUF: [u8; 200] = [b' '; 200];
    writer.write_bytes(&SPACE_BUF[..level * indent])?;
    Ok(())
}
