//! Json to Yason

use crate::builder::{ArrBuilder, BuildResult, NumberError, ObjBuilder};
use crate::{
    ArrayBuilder, ArrayRefBuilder, BuildError, Number, ObjectBuilder, ObjectRefBuilder, Scalar, Yason, YasonBuf,
};
use decimal_rs::DecimalParseError;
use serde_json::{Map, Value};
use std::fmt::Write;
use std::str::FromStr;

impl TryFrom<&serde_json::Value> for YasonBuf {
    type Error = BuildError;

    #[inline]
    fn try_from(value: &serde_json::Value) -> Result<Self, Self::Error> {
        let mut buf = String::new();
        match value {
            Value::Null => Scalar::null(),
            Value::Bool(val) => Scalar::bool(*val),
            Value::Number(val) => Scalar::number(number2decimal(val, &mut buf)?),
            Value::String(val) => Scalar::string(val),
            Value::Array(val) => {
                let mut array_builder = ArrayBuilder::try_new(val.len() as u16)?;
                write_array(&mut array_builder, val, &mut buf)?;
                array_builder.finish()
            }
            Value::Object(val) => {
                let mut object_builder = ObjectBuilder::try_new(val.len() as u16, false)?;
                write_object(&mut object_builder, val, &mut buf)?;
                object_builder.finish()
            }
        }
    }
}

impl YasonBuf {
    /// Parses a json string to `YasonBuf`.
    #[inline]
    pub fn parse<T: AsRef<str>>(str: T) -> BuildResult<Self> {
        let json: Value = serde_json::from_str(str.as_ref()).map_err(BuildError::JsonError)?;
        YasonBuf::try_from(&json)
    }
}

impl Yason {
    /// Parses a json string to `Yason`.
    #[inline]
    pub fn parse_to<T: AsRef<str>>(bytes: &mut Vec<u8>, str: T) -> BuildResult<&Yason> {
        let mut buf = String::new();

        let json: Value = serde_json::from_str(str.as_ref()).map_err(BuildError::JsonError)?;
        match &json {
            Value::Null => Scalar::null_with_vec(bytes),
            Value::Bool(val) => Scalar::bool_with_vec(*val, bytes),
            Value::Number(val) => Scalar::number_with_vec(number2decimal(val, &mut buf)?, bytes),
            Value::String(val) => Scalar::string_with_vec(val, bytes),
            Value::Array(array) => {
                let mut builder = ArrayRefBuilder::try_new(bytes, array.len() as u16)?;
                write_array(&mut builder, array, &mut buf)?;
                builder.finish()
            }
            Value::Object(object) => {
                let mut builder = ObjectRefBuilder::try_new(bytes, object.len() as u16, false)?;
                write_object(&mut builder, object, &mut buf)?;
                builder.finish()
            }
        }
    }
}

#[inline]
fn write_array<T: ArrBuilder>(builder: &mut T, array: &[serde_json::Value], buf: &mut String) -> BuildResult<()> {
    for value in array {
        match value {
            Value::Null => {
                builder.push_null()?;
            }
            Value::Bool(val) => {
                builder.push_bool(*val)?;
            }
            Value::Number(val) => {
                builder.push_number(number2decimal(val, buf)?)?;
            }
            Value::String(val) => {
                builder.push_string(val)?;
            }
            Value::Array(val) => {
                let mut array_builder = builder.push_array(val.len() as u16)?;
                write_array(&mut array_builder, val, buf)?;
                array_builder.finish()?;
            }
            Value::Object(val) => {
                let mut object_builder = builder.push_object(val.len() as u16, false)?;
                write_object(&mut object_builder, val, buf)?;
                object_builder.finish()?;
            }
        }
    }
    Ok(())
}

#[inline]
fn write_object<T: ObjBuilder>(
    builder: &mut T,
    object: &Map<String, serde_json::Value>,
    buf: &mut String,
) -> BuildResult<()> {
    for (key, value) in object {
        match value {
            Value::Null => {
                builder.push_null(key)?;
            }
            Value::Bool(val) => {
                builder.push_bool(key, *val)?;
            }
            Value::Number(val) => {
                builder.push_number(key, number2decimal(val, buf)?)?;
            }
            Value::String(val) => {
                builder.push_string(key, val)?;
            }
            Value::Array(val) => {
                let mut array_builder = builder.push_array(key, val.len() as u16)?;
                write_array(&mut array_builder, val, buf)?;
                array_builder.finish()?;
            }
            Value::Object(val) => {
                let mut object_builder = builder.push_object(key, val.len() as u16, false)?;
                write_object(&mut object_builder, val, buf)?;
                object_builder.finish()?;
            }
        }
    }
    Ok(())
}

#[inline]
fn number2decimal(val: &serde_json::Number, buf: &mut String) -> BuildResult<Number> {
    buf.clear();
    buf.try_reserve(256)?;
    write!(buf, "{}", val).map_err(|_| BuildError::NumberError(NumberError::FormatError))?;
    Number::from_str(buf.as_str()).map_or_else(
        |e| match e {
            DecimalParseError::Underflow => Ok(Number::ZERO),
            DecimalParseError::Overflow => Err(BuildError::NumberError(NumberError::Overflow)),
            _ => unreachable!("internal error: entered unreachable parsing error"),
        },
        Ok,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use decimal_rs::Decimal;
    use serde_json::Error;
    use std::str::FromStr;

    #[test]
    fn test_number2decimal() {
        fn assert_number(input: &str, output: &str) {
            let number: serde_json::Number = serde_json::from_str(input).unwrap();
            let mut buf = String::with_capacity(256);
            let decimal = number2decimal(&number, &mut buf).unwrap();
            assert_eq!(decimal, Decimal::from_str(output).unwrap());
        }

        fn assert_number_invalid(s: &str) {
            let number: Result<serde_json::Number, Error> = serde_json::from_str(s);
            assert!(number.is_err());
        }

        fn assert_number_overflow(s: &str) {
            let number: serde_json::Number = serde_json::from_str(s).unwrap();
            let mut buf = String::with_capacity(256);
            let decimal = number2decimal(&number, &mut buf);
            match decimal {
                Err(BuildError::NumberError(NumberError::Overflow)) => {}
                _ => panic!("expected numeric overflow"),
            };
        }

        assert_number_invalid("Nan");
        assert_number_invalid("Inf");
        assert_number_invalid("-Inf");
        assert_number_invalid("123abc");
        assert_number_invalid("");
        assert_number_invalid("   ");

        assert_number_overflow("1e126");
        assert_number_overflow("1e150");

        assert_number("-123", "-123");
        assert_number("0", "0");
        assert_number("123", "123");

        assert_number("9007199254740991", "9007199254740991"); // 2^53-1
        assert_number("-9007199254740991", "-9007199254740991"); // -2^53+1
        assert_number("9007199254740993", "9007199254740993"); // 2^53+1
        assert_number("-9007199254740993", "-9007199254740993"); // -2^53-1
        assert_number("18446744073709551616", "18446744073709551616"); // 2^64

        assert_number("1e125", "1e125");
        assert_number("1e-130", "1e-130");
        assert_number("1e-131", "0");
        assert_number("1e-150", "0");

        assert_number(
            "222222222222222222222222222222222222222222",
            "222222222222222222222222222222222222222200",
        ); // precision 42 only integral
        assert_number(
            "555555555555555555555555555555555555555555",
            "555555555555555555555555555555555555555600",
        ); // precision 42 only integral

        assert_number(
            "0.222222222222222222222222222222222222222222",
            "0.2222222222222222222222222222222222222222",
        ); // precision 42 only fractional
        assert_number(
            "0.555555555555555555555555555555555555555555",
            "0.5555555555555555555555555555555555555556",
        ); // precision 42 only fractional

        assert_number(
            "0.000000222222222222222222222222222222222222222222",
            "0.0000002222222222222222222222222222222222222222",
        ); // precision 42 only fractional
        assert_number(
            "0.000000555555555555555555555555555555555555555555",
            "0.0000005555555555555555555555555555555555555556",
        ); // precision 42 only fractional

        assert_number(
            "222222222222222222222222.222222222222222222222e50",
            "22222222222222222222222222222222222222e36",
        ); // precision 45
        assert_number(
            "555555555555555555555555.555555555555555555555e50",
            "55555555555555555555555555555555555556e36",
        ); // precision 45
    }
}
