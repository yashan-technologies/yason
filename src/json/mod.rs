//! Json to Yason

mod error;
mod parser;
mod read;
mod tokenizer;
mod value;

pub use error::JsonParseError;

use crate::builder::{ArrBuilder, BuildResult, NumberError, ObjBuilder};
use crate::{
    ArrayBuilder, ArrayRefBuilder, BuildError, Number, ObjectBuilder, ObjectRefBuilder, Scalar, Yason, YasonBuf,
};
use decimal_rs::DecimalParseError;
use error::Result;
use std::borrow::Cow;
use value::{Map, Value};

#[inline]
fn from_str(s: &str) -> Result<Value> {
    let mut parser = parser::Parser::new(s);
    parser.parse_str()
}

impl<'a> TryFrom<&Value<'a>> for YasonBuf {
    type Error = BuildError;

    #[inline]
    fn try_from(value: &Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Null => Scalar::null(),
            Value::Bool(val) => Scalar::bool(*val),
            Value::Number(val) => Scalar::number(number2decimal(val)?),
            Value::String(val) => Scalar::string(val),
            Value::Array(val) => {
                let mut array_builder = ArrayBuilder::try_new(val.len() as u16)?;
                write_array(&mut array_builder, val)?;
                array_builder.finish()
            }
            Value::Object(val) => {
                let mut object_builder = ObjectBuilder::try_new(val.len() as u16, false)?;
                write_object(&mut object_builder, val)?;
                object_builder.finish()
            }
        }
    }
}

impl YasonBuf {
    /// Parses a json string to `YasonBuf`.
    #[inline]
    pub fn parse<T: AsRef<str>>(str: T) -> BuildResult<Self> {
        let json = from_str(str.as_ref()).map_err(BuildError::JsonParseError)?;
        YasonBuf::try_from(&json)
    }
}

impl Yason {
    /// Parses a json string to `Yason`.
    #[inline]
    pub fn parse_to<T: AsRef<str>>(bytes: &mut Vec<u8>, str: T) -> BuildResult<&Yason> {
        let json = from_str(str.as_ref()).map_err(BuildError::JsonParseError)?;
        match &json {
            Value::Null => Scalar::null_with_vec(bytes),
            Value::Bool(val) => Scalar::bool_with_vec(*val, bytes),
            Value::Number(val) => Scalar::number_with_vec(number2decimal(val)?, bytes),
            Value::String(val) => Scalar::string_with_vec(val, bytes),
            Value::Array(array) => {
                let mut builder = ArrayRefBuilder::try_new(bytes, array.len() as u16)?;
                write_array(&mut builder, array)?;
                builder.finish()
            }
            Value::Object(object) => {
                let mut builder = ObjectRefBuilder::try_new(bytes, object.len() as u16, false)?;
                write_object(&mut builder, object)?;
                builder.finish()
            }
        }
    }
}

#[inline]
fn write_array<T: ArrBuilder>(builder: &mut T, array: &[Value]) -> BuildResult<()> {
    for value in array {
        match value {
            Value::Null => {
                builder.push_null()?;
            }
            Value::Bool(val) => {
                builder.push_bool(*val)?;
            }
            Value::Number(val) => {
                builder.push_number(number2decimal(val)?)?;
            }
            Value::String(val) => {
                builder.push_string(val)?;
            }
            Value::Array(val) => {
                let mut array_builder = builder.push_array(val.len() as u16)?;
                write_array(&mut array_builder, val)?;
                array_builder.finish()?;
            }
            Value::Object(val) => {
                let mut object_builder = builder.push_object(val.len() as u16, false)?;
                write_object(&mut object_builder, val)?;
                object_builder.finish()?;
            }
        }
    }
    Ok(())
}

#[inline]
fn write_object<T: ObjBuilder>(builder: &mut T, object: &Map<Cow<str>, Value>) -> BuildResult<()> {
    for (key, value) in object {
        match value {
            Value::Null => {
                builder.push_null(key)?;
            }
            Value::Bool(val) => {
                builder.push_bool(key, *val)?;
            }
            Value::Number(val) => {
                builder.push_number(key, number2decimal(val)?)?;
            }
            Value::String(val) => {
                builder.push_string(key, val)?;
            }
            Value::Array(val) => {
                let mut array_builder = builder.push_array(key, val.len() as u16)?;
                write_array(&mut array_builder, val)?;
                array_builder.finish()?;
            }
            Value::Object(val) => {
                let mut object_builder = builder.push_object(key, val.len() as u16, false)?;
                write_object(&mut object_builder, val)?;
                object_builder.finish()?;
            }
        }
    }
    Ok(())
}

#[inline]
fn number2decimal(val: value::Number) -> BuildResult<Number> {
    val.parse().map_or_else(
        |e| match e {
            DecimalParseError::Underflow => Ok(Number::ZERO),
            DecimalParseError::Overflow => Err(BuildError::NumberError(NumberError::Overflow)),
            _ => unreachable!("internal error: entered unreachable parsing error"),
        },
        Ok,
    )
}

#[cfg(test)]
mod test {
    use super::error::ErrorCode;
    use super::*;
    use decimal_rs::Decimal;

    #[test]
    fn test_null() {
        let v: Value = from_str(r#"null"#).unwrap();
        assert!(v.is_null());

        let v = from_str(r#"nual"#);
        assert_eq!(ErrorCode::ExpectedSomeIdent, v.err().unwrap().errcode());
    }

    #[test]
    fn test_bool() {
        let v: Value = from_str(r#"true"#).unwrap();
        assert!(v.as_bool().unwrap());
        let v: Value = from_str(r#"false"#).unwrap();
        assert!(!v.as_bool().unwrap());

        let v = from_str(r#"trues"#);
        assert_eq!(ErrorCode::TrailingCharacters, v.err().unwrap().errcode());
        let v = from_str(r#"falses"#);
        assert_eq!(ErrorCode::TrailingCharacters, v.err().unwrap().errcode());
    }

    #[test]
    fn test_parse_array() {
        let v = from_str(r#"[123, "1234", null, true, false]"#).unwrap();
        let array = v.as_array().unwrap();

        assert_eq!("123", array.get(0).unwrap().as_number().unwrap());
        assert_eq!("1234", array.get(1).unwrap().as_str().unwrap());
        assert!(array.get(2).unwrap().is_null());
        assert!(array.get(3).unwrap().as_bool().unwrap());
        assert!(!array.get(4).unwrap().as_bool().unwrap());

        let v = from_str(r#"["中国人", "Би Хятадын байна"]"#).unwrap();
        let array = v.as_array().unwrap();

        assert_eq!("中国人", array.get(0).unwrap().as_str().unwrap());
        assert_eq!("Би Хятадын байна", array.get(1).unwrap().as_str().unwrap());

        let v = from_str(r#"[123, "1234", null, true, false,]"#);
        assert_eq!(ErrorCode::ExpectedSomeValue, v.err().unwrap().errcode());

        let v = from_str(r#"[123, "1234", null, true, ,false]"#);
        assert_eq!(ErrorCode::ExpectedSomeValue, v.err().unwrap().errcode());

        let v = from_str(r#"[123, "1234", null, true, false,,]"#);
        assert_eq!(ErrorCode::ExpectedSomeValue, v.err().unwrap().errcode());

        let v = from_str(r#"[,,]"#);
        assert_eq!(ErrorCode::ExpectedSomeValue, v.err().unwrap().errcode());
    }

    #[test]
    fn test_parse_object() {
        let v = from_str(r#"{"a": 123, "b": "abc", "c": null, "d": true, "e": []}"#).unwrap();
        let object = v.as_object().unwrap();

        assert_eq!("123", object.get("a").unwrap().as_number().unwrap());
        assert_eq!("abc", object.get("b").unwrap().as_str().unwrap());
        assert!(object.get("c").unwrap().is_null());
        assert!(object.get("d").unwrap().as_bool().unwrap());
        assert_eq!(Vec::<Value>::new(), *object.get("e").unwrap().as_array().unwrap());

        let v = from_str(r#"{"a": 123, "b": "abc", "c": null, "d": true, "e": [],}"#);
        assert_eq!(ErrorCode::KeyMustBeAString, v.err().unwrap().errcode());

        let v = from_str(r#"{"a": 123, "b": "abc", "c": null, "d": true,, "e": []}"#);
        assert_eq!(ErrorCode::KeyMustBeAString, v.err().unwrap().errcode());
    }

    #[test]
    fn test_parse_number() {
        let number: Value = from_str(r#"1234"#).unwrap();
        assert_eq!("1234".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"1234   "#).unwrap();
        assert_eq!("1234".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"-1234"#).unwrap();
        assert_eq!("-1234".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"-1234.1"#).unwrap();
        assert_eq!("-1234.1".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"-1234.1e11"#).unwrap();
        assert_eq!("-1234.1e11".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"-1234.1e+11"#).unwrap();
        assert_eq!("-1234.1e+11".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"-1234.1e-11"#).unwrap();
        assert_eq!("-1234.1e-11".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"1e23"#).unwrap();
        assert_eq!("1e23".to_string(), number.as_number().unwrap());

        let number: Value = from_str(r#"0"#).unwrap();
        assert_eq!("0".to_string(), number.as_number().unwrap());

        let number = from_str(r#"1234-"#);
        assert_eq!(ErrorCode::TrailingCharacters, number.err().unwrap().errcode());

        let number = from_str(r#"+1234"#);
        assert_eq!(ErrorCode::ExpectedSomeValue, number.err().unwrap().errcode());

        let number = from_str(r#"1234."#);
        assert_eq!(ErrorCode::EofWhileParsingValue, number.err().unwrap().errcode());

        let number = from_str(r#"1234e"#);
        assert_eq!(ErrorCode::EofWhileParsingValue, number.err().unwrap().errcode());

        let number = from_str(r#"1234.e11"#);
        assert_eq!(ErrorCode::InvalidNumber, number.err().unwrap().errcode());

        let number = from_str(r#"1234.1e"#);
        assert_eq!(ErrorCode::EofWhileParsingValue, number.err().unwrap().errcode());

        let number = from_str(r#"01234"#);
        assert_eq!(ErrorCode::InvalidNumber, number.err().unwrap().errcode());

        let number = from_str(r#"-01234"#);
        assert_eq!(ErrorCode::InvalidNumber, number.err().unwrap().errcode());

        let number = from_str(r#"-012.34"#);
        assert_eq!(ErrorCode::InvalidNumber, number.err().unwrap().errcode());

        let number = from_str(r#"-.1234"#);
        assert_eq!(ErrorCode::InvalidNumber, number.err().unwrap().errcode());

        let number = from_str(r#"-000000001"#);
        assert_eq!(ErrorCode::InvalidNumber, number.err().unwrap().errcode());
    }

    #[test]
    fn test_parse_string() {
        let str: Value = from_str(r#""abcd""#).unwrap();
        assert_eq!("abcd", str.as_str().unwrap());

        let str: Value = from_str(r#""abc\uD800\uDC00\uD800\uDC00\b\f\n\n\t\\\/""#).unwrap();
        assert_eq!("abc𐀀𐀀\u{8}\u{c}\n\n\t\\/", str.as_str().unwrap());
    }

    #[test]
    fn test_parse_utf8_string() {
        let str: Value = from_str(r#""中国人""#).unwrap();
        assert_eq!("中国人", str.as_str().unwrap());

        let str: Value = from_str(r#""Би Хятадын байна""#).unwrap();
        assert_eq!("Би Хятадын байна", str.as_str().unwrap());

        let str: Value = from_str(r#""ちゅうごくじん""#).unwrap();
        assert_eq!("ちゅうごくじん", str.as_str().unwrap());

        let str: Value = from_str(r#""??????""#).unwrap();
        assert_eq!("??????", str.as_str().unwrap());
    }

    #[test]
    fn test_number2decimal() {
        fn assert_number(input: &str, output: &str) {
            let number = from_str(input).unwrap().as_number().unwrap();
            let decimal = number2decimal(number).unwrap();
            assert_eq!(decimal, output.parse::<Decimal>().unwrap());
        }

        fn assert_number_invalid(s: &str) {
            let number = from_str(s);
            assert!(number.is_err());
        }

        fn assert_number_overflow(s: &str) {
            let number = from_str(s).unwrap().as_number().unwrap();
            let decimal = number2decimal(number);
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
