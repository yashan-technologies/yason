//! Json to Yason

mod error;
mod parser;
mod read;
mod tokenizer;
mod value;

pub use error::JsonParseError;

use crate::builder::{ArrBuilder, BuildResult, NumberError, ObjBuilder};
use crate::extended::*;
use crate::vec::VecExt;
use crate::{ArrayRefBuilder, BuildError, DataType, Number, ObjectRefBuilder, Scalar, Yason, YasonBuf};
use decimal_rs::DecimalParseError;
use error::Result;
use std::borrow::Cow;
use std::str::FromStr;
use value::{Map, Value};

#[inline]
fn from_str(s: &str) -> Result<Value<'_>> {
    let mut parser = parser::Parser::new(s);
    parser.parse_str()
}

impl YasonBuf {
    /// Parses a json string to `YasonBuf`.
    #[inline]
    pub fn parse<T: AsRef<str>>(str: T, extended: bool) -> BuildResult<Self> {
        let mut bytes = Vec::new();
        Yason::parse_to(&mut bytes, str, extended)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }
}

impl Yason {
    /// Parses a json string to `Yason`.
    #[inline]
    pub fn parse_to<T: AsRef<str>>(bytes: &mut Vec<u8>, str: T, extended: bool) -> BuildResult<&Yason> {
        let json = from_str(str.as_ref()).map_err(BuildError::JsonParseError)?;
        match &json {
            Value::Null => Scalar::null_with_vec(bytes),
            Value::Bool(val) => Scalar::bool_with_vec(*val, bytes),
            Value::Number(val) => {
                let decimal = number2decimal(val)?;
                if extended {
                    if decimal.has_fract() {
                        Scalar::double_with_vec(number2f64(val)?, bytes)
                    } else if let Ok(i) = i64::try_from(decimal) {
                        if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
                            Scalar::tinyint_with_vec(i as i8, bytes)
                        } else if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
                            Scalar::smallint_with_vec(i as i16, bytes)
                        } else if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                            Scalar::integer_with_vec(i as i32, bytes)
                        } else {
                            Scalar::bigint_with_vec(i, bytes)
                        }
                    } else {
                        Scalar::number_with_vec(decimal, bytes)
                    }
                } else {
                    Scalar::number_with_vec(decimal, bytes)
                }
            }
            Value::String(val) => Scalar::string_with_vec(val, bytes),
            Value::Array(array) => {
                let mut builder = ArrayRefBuilder::try_new(bytes, array.len() as u16)?;
                write_array(&mut builder, array, extended)?;
                builder.finish()
            }
            Value::Object(object) => {
                if extended && object.len() == 1 {
                    let init_len = bytes.len();
                    if write_extended_object_as_scalar(bytes, object)? {
                        return Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) });
                    }
                }

                let mut builder = ObjectRefBuilder::try_new(bytes, object.len() as u16, false)?;
                write_object(&mut builder, object, extended)?;
                builder.finish()
            }
        }
    }
}

#[inline]
fn write_array<T: ArrBuilder>(builder: &mut T, array: &[Value], extended: bool) -> BuildResult<()> {
    for value in array {
        match value {
            Value::Null => {
                builder.push_null()?;
            }
            Value::Bool(val) => {
                builder.push_bool(*val)?;
            }
            Value::Number(val) => {
                let decimal = number2decimal(val)?;
                if extended {
                    if decimal.has_fract() {
                        builder.push_double(number2f64(val)?)?;
                    } else if let Ok(i) = i64::try_from(decimal) {
                        if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
                            builder.push_tinyint(i as i8)?;
                        } else if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
                            builder.push_smallint(i as i16)?;
                        } else if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                            builder.push_integer(i as i32)?;
                        } else {
                            builder.push_bigint(i)?;
                        }
                    } else {
                        builder.push_number(decimal)?;
                    }
                } else {
                    builder.push_number(decimal)?;
                }
            }
            Value::String(val) => {
                builder.push_string(val)?;
            }
            Value::Array(val) => {
                let mut array_builder = builder.push_array(val.len() as u16)?;
                write_array(&mut array_builder, val, extended)?;
                array_builder.finish()?;
            }
            Value::Object(val) => {
                if extended && val.len() == 1 && write_extended_object_to_array(builder, val)? {
                    continue;
                }

                let mut object_builder = builder.push_object(val.len() as u16, false)?;
                write_object(&mut object_builder, val, extended)?;
                object_builder.finish()?;
            }
        }
    }
    Ok(())
}

#[inline]
fn write_object<T: ObjBuilder>(builder: &mut T, object: &Map<Cow<str>, Value>, extended: bool) -> BuildResult<()> {
    for (key, value) in object {
        match value {
            Value::Null => {
                builder.push_null(key)?;
            }
            Value::Bool(val) => {
                builder.push_bool(key, *val)?;
            }
            Value::Number(val) => {
                let decimal = number2decimal(val)?;
                if extended {
                    if decimal.has_fract() {
                        builder.push_double(key, number2f64(val)?)?;
                    } else if let Ok(i) = i64::try_from(decimal) {
                        if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
                            builder.push_tinyint(key, i as i8)?;
                        } else if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
                            builder.push_smallint(key, i as i16)?;
                        } else if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                            builder.push_integer(key, i as i32)?;
                        } else {
                            builder.push_bigint(key, i)?;
                        }
                    } else {
                        builder.push_number(key, decimal)?;
                    }
                } else {
                    builder.push_number(key, decimal)?;
                }
            }
            Value::String(val) => {
                builder.push_string(key, val)?;
            }
            Value::Array(val) => {
                let mut array_builder = builder.push_array(key, val.len() as u16)?;
                write_array(&mut array_builder, val, extended)?;
                array_builder.finish()?;
            }
            Value::Object(val) => {
                if extended && val.len() == 1 && write_extended_object_to_object(builder, key, val)? {
                    continue;
                }

                let mut object_builder = builder.push_object(key, val.len() as u16, false)?;
                write_object(&mut object_builder, val, extended)?;
                object_builder.finish()?;
            }
        }
    }
    Ok(())
}

fn write_extended_object_as_scalar(bytes: &mut Vec<u8>, object: &Map<Cow<str>, Value>) -> BuildResult<bool> {
    debug_assert_eq!(object.len(), 1);

    macro_rules! create_numeric_scalar {
        ($value: expr, $bytes: expr, $ty: ty, $method: ident) => {
            match $value {
                Value::String(str) => {
                    if let Ok(val) = <$ty>::from_str(str) {
                        let _ = Scalar::$method(val, $bytes)?;
                        return Ok(true);
                    }
                }
                Value::Number(num) => {
                    if let Ok(val) = <$ty>::from_str(num) {
                        let _ = Scalar::$method(val, $bytes)?;
                        return Ok(true);
                    }
                }
                _ => {} // ignore other types
            }
        };
    }

    macro_rules! create_temporal_scalar {
        ($value: expr, $fmt: ident, $method: ident) => {
            if let Value::String(str) = $value {
                if let Ok(t) = $fmt().parse(str) {
                    let _ = Scalar::$method(t, bytes)?;
                    return Ok(true);
                }
            }
        };
    }

    // SAFETY: object has one entry.
    let (key, value) = object.iter().next().unwrap();
    let key_str = key.as_ref();
    if key_str.as_bytes().first() != Some(&EXTENDED_NAME_PREFIX) {
        return Ok(false);
    }
    if let Ok(index) = EXTENDED_NAME_TYPES.binary_search_by(|entry| entry.0.cmp(key_str)) {
        let data_type = EXTENDED_NAME_TYPES[index].1;
        match data_type {
            DataType::Object => unreachable!(),
            DataType::Array => unreachable!(),
            DataType::String => unreachable!(),
            DataType::Bool => unreachable!(),
            DataType::Null => unreachable!(),
            DataType::Number => {
                create_numeric_scalar!(value, bytes, Number, number_with_vec);
            }
            DataType::Tinyint => {
                create_numeric_scalar!(value, bytes, i8, tinyint_with_vec);
            }
            DataType::Smallint => {
                create_numeric_scalar!(value, bytes, i16, smallint_with_vec);
            }
            DataType::Integer => {
                create_numeric_scalar!(value, bytes, i32, integer_with_vec);
            }
            DataType::Bigint => {
                create_numeric_scalar!(value, bytes, i64, bigint_with_vec);
            }
            DataType::Float => {
                create_numeric_scalar!(value, bytes, f32, float_with_vec);
            }
            DataType::Double => {
                create_numeric_scalar!(value, bytes, f64, double_with_vec);
            }
            DataType::Binary => {
                let ret = decode_binary(value, |bin| {
                    let _ = Scalar::binary_with_vec(bin, bytes)?;
                    Ok(())
                })?;
                return Ok(ret);
            }
            DataType::Timestamp => {
                create_temporal_scalar!(value, timestamp_formatter, timestamp_with_vec)
            }
            DataType::Date => {
                create_temporal_scalar!(value, date_formatter, date_with_vec)
            }
            DataType::Time => {
                create_temporal_scalar!(value, time_formatter, time_with_vec)
            }
        }
    }

    Ok(false)
}

fn write_extended_object_to_object<T: ObjBuilder>(
    builder: &mut T,
    key: &str,
    object: &Map<Cow<str>, Value>,
) -> BuildResult<bool> {
    debug_assert_eq!(object.len(), 1);

    macro_rules! push_extended_numeric {
        ($key: expr, $value: expr, $builder: expr, $ty: ident, $method: ident) => {
            match $value {
                Value::String(str) => {
                    if let Ok(val) = $ty::from_str(str) {
                        $builder.$method($key, val)?;
                        return Ok(true);
                    }
                }
                Value::Number(num) => {
                    if let Ok(val) = $ty::from_str(num) {
                        $builder.$method($key, val)?;
                        return Ok(true);
                    }
                }
                _ => {} // ignore other types
            }
        };
    }

    macro_rules! push_extended_temporal {
        ($key: expr, $value: expr, $builder: expr, $fmt: ident, $method: ident) => {
            if let Value::String(str) = $value {
                if let Ok(t) = $fmt().parse(str) {
                    $builder.$method($key, t)?;
                    return Ok(true);
                }
            }
        };
    }

    // SAFETY: object has one entry.
    let (obj_key, value) = object.iter().next().unwrap();
    let key_str = obj_key.as_ref();
    if key_str.as_bytes().first() != Some(&EXTENDED_NAME_PREFIX) {
        return Ok(false);
    }
    if let Ok(index) = EXTENDED_NAME_TYPES.binary_search_by(|entry| entry.0.cmp(key_str)) {
        let data_type = EXTENDED_NAME_TYPES[index].1;
        match data_type {
            DataType::Object => unreachable!(),
            DataType::Array => unreachable!(),
            DataType::String => unreachable!(),
            DataType::Bool => unreachable!(),
            DataType::Null => unreachable!(),
            DataType::Number => {
                push_extended_numeric!(key, value, builder, Number, push_number);
            }
            DataType::Tinyint => {
                push_extended_numeric!(key, value, builder, i8, push_tinyint);
            }
            DataType::Smallint => {
                push_extended_numeric!(key, value, builder, i16, push_smallint);
            }
            DataType::Integer => {
                push_extended_numeric!(key, value, builder, i32, push_integer);
            }
            DataType::Bigint => {
                push_extended_numeric!(key, value, builder, i64, push_bigint);
            }
            DataType::Float => {
                push_extended_numeric!(key, value, builder, f32, push_float);
            }
            DataType::Double => {
                push_extended_numeric!(key, value, builder, f64, push_double);
            }
            DataType::Binary => {
                let ret = decode_binary(value, |bin| {
                    builder.push_binary(key, bin)?;
                    Ok(())
                })?;
                return Ok(ret);
            }
            DataType::Timestamp => {
                push_extended_temporal!(key, value, builder, timestamp_formatter, push_timestamp)
            }
            DataType::Date => {
                push_extended_temporal!(key, value, builder, date_formatter, push_date)
            }
            DataType::Time => {
                push_extended_temporal!(key, value, builder, time_formatter, push_time)
            }
        }
    }

    Ok(false)
}

fn write_extended_object_to_array<T: ArrBuilder>(builder: &mut T, object: &Map<Cow<str>, Value>) -> BuildResult<bool> {
    debug_assert_eq!(object.len(), 1);

    macro_rules! push_extended_numeric {
        ($value: expr, $builder: expr, $ty: ident, $method: ident) => {
            match $value {
                Value::String(str) => {
                    if let Ok(val) = $ty::from_str(str) {
                        $builder.$method(val)?;
                        return Ok(true);
                    }
                }
                Value::Number(num) => {
                    if let Ok(val) = $ty::from_str(num) {
                        $builder.$method(val)?;
                        return Ok(true);
                    }
                }
                _ => {} // ignore other types
            }
        };
    }

    macro_rules! push_extended_temporal {
        ($value: expr, $builder: expr, $fmt: ident, $method: ident) => {
            if let Value::String(str) = $value {
                if let Ok(t) = $fmt().parse(str) {
                    $builder.$method(t)?;
                    return Ok(true);
                }
            }
        };
    }

    // SAFETY: object has one entry.
    let (obj_key, value) = object.iter().next().unwrap();
    let key_str = obj_key.as_ref();
    if key_str.as_bytes().first() != Some(&EXTENDED_NAME_PREFIX) {
        return Ok(false);
    }
    if let Ok(index) = EXTENDED_NAME_TYPES.binary_search_by(|entry| entry.0.cmp(key_str)) {
        let data_type = EXTENDED_NAME_TYPES[index].1;
        match data_type {
            DataType::Object => unreachable!(),
            DataType::Array => unreachable!(),
            DataType::String => unreachable!(),
            DataType::Bool => unreachable!(),
            DataType::Null => unreachable!(),
            DataType::Number => {
                push_extended_numeric!(value, builder, Number, push_number);
            }
            DataType::Tinyint => {
                push_extended_numeric!(value, builder, i8, push_tinyint);
            }
            DataType::Smallint => {
                push_extended_numeric!(value, builder, i16, push_smallint);
            }
            DataType::Integer => {
                push_extended_numeric!(value, builder, i32, push_integer);
            }
            DataType::Bigint => {
                push_extended_numeric!(value, builder, i64, push_bigint);
            }
            DataType::Float => {
                push_extended_numeric!(value, builder, f32, push_float);
            }
            DataType::Double => {
                push_extended_numeric!(value, builder, f64, push_double);
            }
            DataType::Binary => {
                let ret = decode_binary(value, |bin| {
                    builder.push_binary(bin)?;
                    Ok(())
                })?;
                return Ok(ret);
            }
            DataType::Timestamp => {
                push_extended_temporal!(value, builder, timestamp_formatter, push_timestamp)
            }
            DataType::Date => {
                push_extended_temporal!(value, builder, date_formatter, push_date)
            }
            DataType::Time => {
                push_extended_temporal!(value, builder, time_formatter, push_time)
            }
        }
    }

    Ok(false)
}

#[inline]
fn decode_binary<F>(value: &Value, f: F) -> BuildResult<bool>
where
    F: for<'a> FnOnce(&'a [u8]) -> BuildResult<()>,
{
    match value {
        Value::String(str) => {
            if decode_base64(str, f)? {
                return Ok(true);
            }
        }
        Value::Object(obj) if obj.len() == 2 && obj_has_valid_subtype(obj) => {
            if let Some(Value::String(str)) = obj.get(BINARY_BASE64_NAME)
                && decode_base64(str, f)?
            {
                return Ok(true);
            }
        }
        _ => (), // ignore other types
    }

    Ok(false)
}

#[inline]
fn decode_base64<F>(value: &str, f: F) -> BuildResult<bool>
where
    F: FnOnce(&[u8]) -> BuildResult<()>,
{
    use crate::base64::*;

    let decoded_len = decoded_len_estimate(value.len());
    let mut buf = <Vec<u8> as VecExt>::try_with_capacity(decoded_len)?;
    unsafe { buf.set_len(decoded_len) };
    if let Ok(len) = decode(value.as_bytes(), &mut buf[..]) {
        f(&buf[..len])?;
        return Ok(true);
    }

    Ok(false)
}

#[inline]
fn obj_has_valid_subtype(obj: &Map<Cow<str>, Value>) -> bool {
    if let Some(Value::Number(str)) = obj.get(BINARY_SUBTYPE_NAME)
        && let Ok(n) = number2decimal(str)
        && !n.has_fract()
        && u8::try_from(n).is_ok()
    {
        return true;
    }

    false
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

#[inline]
fn number2f64(val: value::Number) -> BuildResult<f64> {
    val.parse().map_err(|_| BuildError::NumberError(NumberError::Invalid))
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

        assert_eq!("123", array.first().unwrap().as_number().unwrap());
        assert_eq!("1234", array.get(1).unwrap().as_str().unwrap());
        assert!(array.get(2).unwrap().is_null());
        assert!(array.get(3).unwrap().as_bool().unwrap());
        assert!(!array.get(4).unwrap().as_bool().unwrap());

        let v = from_str(r#"["中国人", "Би Хятадын байна"]"#).unwrap();
        let array = v.as_array().unwrap();

        assert_eq!("中国人", array.first().unwrap().as_str().unwrap());
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
