//! Path Expression.

use crate::path::parse::{FuncStep, PathParser, Step};
use std::fmt;
use std::str::FromStr;

use crate::yason::YasonResult;
use crate::{ArrayRefBuilder, DataType, Number, Value, Yason, YasonError};

use crate::format::{CompactFormatter, FormatResult, Formatter, PrettyFormatter};
use crate::path::query::Selector;
pub use parse::PathParseError;

mod parse;
mod query;

/// This type represents result returned by a path expression.
pub enum QueriedValue<'a, 'b> {
    /// Result returned when no WITH WRAPPER is specified and there is not query result.
    None,

    /// Result returned when no WITH WRAPPER is specified and there is only one query result.  
    Value(Value<'a>),

    /// Result returned when WITH WRAPPER is specified but the user does not provide the query buffer and result buffer.  
    Values(Vec<Value<'a>>),

    /// Result returned when WITH WRAPPER is specified and the user provide the query buffer but no result buffer.  
    ValuesRef(&'b mut Vec<Value<'a>>),

    /// Result returned when the user provides a result buffer, whether or not WITH WRAPPER is specified and the query buffer is provided.  
    Yason(&'b Yason),
}

impl<'a, 'b> QueriedValue<'a, 'b> {
    /// Formats the value as a compact or pretty string.
    #[inline]
    pub fn format_to<W: fmt::Write>(&self, pretty: bool, writer: &mut W) -> FormatResult<()> {
        match self {
            QueriedValue::None => Ok(()),
            QueriedValue::Value(value) => value.format_to(pretty, writer),
            QueriedValue::Values(values) => values_format_to(values, pretty, writer),
            QueriedValue::ValuesRef(values) => values_format_to(values, pretty, writer),
            QueriedValue::Yason(yason) => yason.format_to(pretty, writer),
        }
    }
}

enum QueryBuf<'a, 'b> {
    Owned(Vec<Value<'a>>),
    Borrowed(&'b mut Vec<Value<'a>>),
}

impl<'a> AsMut<Vec<Value<'a>>> for QueryBuf<'a, '_> {
    #[inline]
    fn as_mut(&mut self) -> &mut Vec<Value<'a>> {
        match self {
            QueryBuf::Owned(buf) => buf,
            QueryBuf::Borrowed(buf) => *buf,
        }
    }
}

impl<'a> AsRef<[Value<'a>]> for QueryBuf<'a, '_> {
    #[inline]
    fn as_ref(&self) -> &[Value<'a>] {
        match self {
            QueryBuf::Owned(buf) => buf,
            QueryBuf::Borrowed(buf) => *buf,
        }
    }
}

/// This type represents a path expression.
#[derive(Debug)]
#[repr(transparent)]
pub struct PathExpression(Vec<Step>);

impl PathExpression {
    #[inline]
    fn new(steps: Vec<Step>) -> Self {
        Self(steps)
    }
}

impl PathExpression {
    #[inline]
    fn steps(&self) -> &[Step] {
        &self.0
    }

    /// Returns whether an item method exists in path expression.
    #[inline]
    pub fn has_method(&self) -> bool {
        let len = self.0.len();
        if len <= 1 {
            return false;
        }
        matches!(self.steps()[len - 1], Step::Func(_))
    }

    #[inline]
    fn has_method_count(&self) -> bool {
        let len = self.0.len();
        if len <= 1 {
            return false;
        }
        matches!(self.0[len - 1], Step::Func(FuncStep::Count))
    }

    /// Selects and returns one or more values according to the path expression.
    #[inline]
    pub fn query<'a, 'b>(
        &self,
        yason: &'a Yason,
        with_wrapper: bool,
        query_buf: Option<&'b mut Vec<Value<'a>>>,
        result_buf: Option<&'b mut Vec<u8>>,
    ) -> YasonResult<QueriedValue<'a, 'b>> {
        if self.has_method() && !with_wrapper {
            return Err(YasonError::MultiValuesWithoutWrapper);
        }

        let mut query_buf = match query_buf {
            None => QueryBuf::Owned(vec![]),
            Some(buf) => {
                buf.clear();
                QueryBuf::Borrowed(buf)
            }
        };

        let mut selector = Selector::new(self.steps(), with_wrapper, query_buf.as_mut(), false);
        selector.query(yason, 1)?;

        if !with_wrapper {
            debug_assert!(query_buf.as_ref().len() <= 1);
            return match query_buf.as_mut().pop() {
                None => Ok(QueriedValue::None),
                Some(val) => Ok(QueriedValue::Value(val)),
            };
        }

        if self.has_method_count() {
            let count = query_buf.as_ref().len();
            let val = Value::Number(Number::from(count));
            query_buf.as_mut().clear();
            push_value(query_buf.as_mut(), val)?;
        }

        if query_buf.as_ref().is_empty() {
            return Ok(QueriedValue::None);
        }

        match result_buf {
            None => match query_buf {
                QueryBuf::Owned(buf) => Ok(QueriedValue::Values(buf)),
                QueryBuf::Borrowed(buf) => Ok(QueriedValue::ValuesRef(buf)),
            },
            Some(bytes) => {
                bytes.clear();
                let yason = values_to_yason(query_buf.as_ref(), bytes)?;
                Ok(QueriedValue::Yason(yason))
            }
        }
    }

    /// Returns true if the data it targets matches one or more values. If no values are matched then it returns false.
    #[inline]
    pub fn exists(&self, yason: &Yason) -> YasonResult<bool> {
        if self.has_method() {
            return Err(YasonError::InvalidPathExpression);
        }

        let mut query_buf = Vec::new();
        let mut selector = Selector::new(self.steps(), true, &mut query_buf, true);
        selector.query(yason, 1)
    }
}

impl FromStr for PathExpression {
    type Err = PathParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parser = PathParser::new(s.as_bytes());
        parser.parse()
    }
}

#[inline]
fn push_value<'a>(buf: &mut Vec<Value<'a>>, value: Value<'a>) -> YasonResult<()> {
    buf.try_reserve(std::mem::size_of::<Value>())
        .map_err(YasonError::TryReserveError)?;
    buf.push(value);
    Ok(())
}

#[inline]
fn values_to_yason<'a>(values: &[Value], bytes: &'a mut Vec<u8>) -> YasonResult<&'a Yason> {
    let mut builder = ArrayRefBuilder::try_new(bytes, values.len() as u16)?;
    for value in values {
        match value {
            Value::Object(object) => unsafe { builder.push_object_or_array(object.yason(), DataType::Object)? },
            Value::Array(array) => unsafe { builder.push_object_or_array(array.yason(), DataType::Array)? },
            Value::String(str) => builder.push_string(str)?,
            Value::Number(number) => builder.push_number(number)?,
            Value::Bool(bool) => builder.push_bool(*bool)?,
            Value::Null => builder.push_null()?,
        };
    }

    Ok(builder.finish()?)
}

#[inline]
fn values_format_to<W: fmt::Write>(values: &[Value], pretty: bool, writer: &mut W) -> FormatResult<()> {
    if values.is_empty() {
        return Ok(());
    }

    if pretty {
        let mut fmt = PrettyFormatter::new();
        unsafe { fmt.write_values(values, writer) }
    } else {
        let mut fmt = CompactFormatter::new();
        unsafe { fmt.write_values(values, writer) }
    }
}
