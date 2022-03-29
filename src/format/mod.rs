//! Formatter.

use crate::{Array, DataType, Number, Object, Value, Yason, YasonError};
use decimal_rs::DecimalFormatError;
pub use pretty::PrettyFormatter;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

mod pretty;

/// Possible errors that can arise during formatting.
#[derive(Debug)]
pub enum FormatError {
    FmtError(fmt::Error),
    NumberFormatError(DecimalFormatError),
    YasonError(YasonError),
}

impl Display for FormatError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::FmtError(e) => write!(f, "{}", e),
            FormatError::NumberFormatError(e) => write!(f, "{}", e),
            FormatError::YasonError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for FormatError {}

pub type FormatResult<T> = std::result::Result<T, FormatError>;

impl From<fmt::Error> for FormatError {
    #[inline]
    fn from(e: fmt::Error) -> Self {
        FormatError::FmtError(e)
    }
}

impl From<YasonError> for FormatError {
    #[inline]
    fn from(e: YasonError) -> Self {
        FormatError::YasonError(e)
    }
}

pub trait Formatter {
    #[inline]
    fn format<W: fmt::Write>(&mut self, yason: &Yason, writer: &mut W) -> FormatResult<()> {
        match yason.data_type()? {
            DataType::Object => {
                let object = unsafe { Object::new_unchecked(yason) };
                self.write_object(&object, writer)
            }
            DataType::Array => {
                let array = unsafe { Array::new_unchecked(yason) };
                self.write_array(&array, writer)
            }
            DataType::String => {
                let str = unsafe { yason.string_unchecked()? };
                self.write_string(str, writer)
            }
            DataType::Number => {
                let num = unsafe { yason.number_unchecked()? };
                self.write_number(&num, writer)
            }
            DataType::Bool => {
                let bool = unsafe { yason.bool_unchecked()? };
                self.write_bool(bool, writer)
            }
            DataType::Null => self.write_null(writer),
        }
    }

    #[inline]
    fn write_value<W: fmt::Write>(&mut self, value: &Value, writer: &mut W) -> FormatResult<()> {
        match value {
            Value::Object(val) => self.write_object(val, writer),
            Value::Array(val) => self.write_array(val, writer),
            Value::String(val) => self.write_string(val, writer),
            Value::Number(val) => self.write_number(val, writer),
            Value::Bool(val) => self.write_bool(*val, writer),
            Value::Null => self.write_null(writer),
        }
    }

    #[inline]
    fn write_null<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b"null")?;
        Ok(())
    }

    #[inline]
    fn write_bool<W: fmt::Write>(&mut self, value: bool, writer: &mut W) -> FormatResult<()> {
        let s = if value { "true" } else { "false" };
        writer.write_bytes(s.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn write_number<W: fmt::Write>(&mut self, value: &Number, writer: &mut W) -> FormatResult<()> {
        value.format_to_json(writer).map_err(FormatError::NumberFormatError)
    }

    #[inline]
    fn write_string<W: fmt::Write>(&mut self, value: &str, writer: &mut W) -> FormatResult<()> {
        self.begin_string(writer)?;
        format_escaped_str(value, writer)?;
        self.end_string(writer)
    }

    #[inline]
    fn write_object<W: fmt::Write>(&mut self, value: &Object, writer: &mut W) -> FormatResult<()> {
        self.begin_object(writer)?;

        let mut iter = value.iter()?;
        if let Some(entry) = iter.next() {
            let (key, value) = entry?;
            self.write_object_value(key, &value, true, writer)?;
        }
        for entry in iter {
            let (key, value) = entry?;
            self.write_object_value(key, &value, false, writer)?;
        }

        self.end_object(writer)
    }

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
        self.write_value(value, writer)?;
        self.end_object_value(writer)
    }

    #[inline]
    fn write_array<W: fmt::Write>(&mut self, value: &Array, writer: &mut W) -> FormatResult<()> {
        self.begin_array(writer)?;

        let mut iter = value.iter()?;
        if let Some(val) = iter.next() {
            self.write_array_value(&val?, true, writer)?;
        }
        for val in iter {
            self.write_array_value(&val?, false, writer)?;
        }

        self.end_array(writer)
    }

    #[inline]
    fn write_array_value<W: fmt::Write>(&mut self, value: &Value, first: bool, writer: &mut W) -> FormatResult<()> {
        self.begin_array_value(first, writer)?;
        self.write_value(value, writer)?;
        self.end_array_value(writer)
    }

    #[inline]
    fn begin_string<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b"\"")?;
        Ok(())
    }

    #[inline]
    fn end_string<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b"\"")?;
        Ok(())
    }

    #[inline]
    fn begin_array<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b"[")?;
        Ok(())
    }

    #[inline]
    fn end_array<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b"]")?;
        Ok(())
    }

    #[inline]
    fn begin_array_value<W: fmt::Write>(&mut self, first: bool, writer: &mut W) -> FormatResult<()> {
        if !first {
            writer.write_bytes(b",")?;
        }
        Ok(())
    }

    #[inline]
    fn end_array_value<W: fmt::Write>(&mut self, _writer: &mut W) -> FormatResult<()> {
        Ok(())
    }

    #[inline]
    fn begin_object<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b"{")?;
        Ok(())
    }

    #[inline]
    fn end_object<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b"}")?;
        Ok(())
    }

    #[inline]
    fn begin_object_key<W: fmt::Write>(&mut self, first: bool, writer: &mut W) -> FormatResult<()> {
        if !first {
            writer.write_bytes(b",")?;
        }
        Ok(())
    }

    #[inline]
    fn end_object_key<W: fmt::Write>(&mut self, _writer: &mut W) -> FormatResult<()> {
        Ok(())
    }

    #[inline]
    fn begin_object_value<W: fmt::Write>(&mut self, writer: &mut W) -> FormatResult<()> {
        writer.write_bytes(b":")?;
        Ok(())
    }

    #[inline]
    fn end_object_value<W: fmt::Write>(&mut self, _writer: &mut W) -> FormatResult<()> {
        Ok(())
    }
}

pub struct CompactFormatter;

impl CompactFormatter {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl Formatter for CompactFormatter {}

pub struct LazyFormat<'a> {
    yason: &'a Yason,
    pretty: bool,
}

impl<'a> LazyFormat<'a> {
    #[inline]
    pub const fn new(yason: &'a Yason, pretty: bool) -> Self {
        Self { yason, pretty }
    }
}

impl fmt::Display for LazyFormat<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pretty {
            let mut fmt = PrettyFormatter::new();
            fmt.format(self.yason, f).map_err(|_| fmt::Error)
        } else {
            let mut fmt = CompactFormatter::new();
            fmt.format(self.yason, f).map_err(|_| fmt::Error)
        }
    }
}

const BB: &[u8] = b"\\b"; // \x08
const TT: &[u8] = b"\\t"; // \x09
const NN: &[u8] = b"\\n"; // \x0A
const FF: &[u8] = b"\\f"; // \x0C
const RR: &[u8] = b"\\r"; // \x0D
const QU: &[u8] = b"\\\""; // \x22
const BS: &[u8] = b"\\\\"; // \x5C
const SS: &[u8] = b"\\/"; // \x2F
const __: &[u8] = b"";

// Lookup table of escape sequences. A value of b"x" at index i means that byte
// i is escaped as "x" in Yason. A value of b"" means that byte i is not escaped.
static ESCAPE: [&[u8]; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    __, __, __, __, __, __, __, __, BB, TT, NN, __, FF, RR, __, __, // 0
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, SS, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

#[inline]
fn format_escaped_str<W: fmt::Write>(value: &str, writer: &mut W) -> FormatResult<()> {
    let bytes = value.as_bytes();

    let mut start = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        let escape = ESCAPE[byte as usize];
        if escape == __ {
            continue;
        }

        if start < i {
            writer.write_bytes(&bytes[start..i])?;
        }
        writer.write_bytes(escape)?;
        start = i + 1;
    }

    if start != bytes.len() {
        writer.write_bytes(&bytes[start..])?;
    }

    Ok(())
}

trait WriteExt: fmt::Write {
    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> fmt::Result {
        let s = unsafe { std::str::from_utf8_unchecked(bytes) };
        self.write_str(s)
    }
}

impl<W: fmt::Write> WriteExt for W {}
