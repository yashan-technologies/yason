//! Scalar tests.

use std::str::FromStr;
use yason::{DataType, Number, Scalar};

#[test]
fn test_string() {
    let yason = Scalar::string("abc").unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::String);
    let string = yason.string().unwrap();
    assert_eq!(string, "abc");

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::string_with_vec("abc", &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::String);
    let string = yason.string().unwrap();
    assert_eq!(string, "abc");

    // test from used vec
    let yason = Scalar::string_with_vec("abc", &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::String);
    let string = yason.string().unwrap();
    assert_eq!(string, "abc");
}

#[test]
fn test_number() {
    let number = Number::from_str("123.123").unwrap();
    let yason = Scalar::number(number).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Number);
    let number = yason.number().unwrap();
    assert_eq!(number, Number::from_str("123.123").unwrap());

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::number_with_vec(number, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Number);
    let number = yason.number().unwrap();
    assert_eq!(number, Number::from_str("123.123").unwrap());

    // test from used vec
    let yason = Scalar::number_with_vec(number, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Number);
    let number = yason.number().unwrap();
    assert_eq!(number, Number::from_str("123.123").unwrap());
}

#[test]
fn test_bool() {
    let yason = Scalar::bool(false).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Bool);
    let value = yason.bool().unwrap();
    assert!(!value);

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::bool_with_vec(true, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Bool);
    let value = yason.bool().unwrap();
    assert!(value);

    // test from used vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::bool_with_vec(true, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Bool);
    let value = yason.bool().unwrap();
    assert!(value);
}

#[test]
fn test_null() {
    let yason = Scalar::null().unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Null);
    assert!(yason.is_null().unwrap());

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::null_with_vec(&mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Null);
    assert!(yason.is_null().unwrap());

    // test from used vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::null_with_vec(&mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Null);
    assert!(yason.is_null().unwrap());
}

#[test]
fn test_tinyint() {
    let i = 123_i8;
    let yason = Scalar::tinyint(i).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Tinyint);
    let n = yason.tinyint().unwrap();
    assert_eq!(n, i);

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::tinyint_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Tinyint);
    let n = yason.tinyint().unwrap();
    assert_eq!(n, i);

    // test from used vec
    let yason = Scalar::tinyint_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Tinyint);
    let n = yason.tinyint().unwrap();
    assert_eq!(n, i);
}

#[test]
fn test_smallint() {
    let i = 12345_i16;
    let yason = Scalar::smallint(i).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Smallint);
    let n = yason.smallint().unwrap();
    assert_eq!(n, i);

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::smallint_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Smallint);
    let n = yason.smallint().unwrap();
    assert_eq!(n, i);

    // test from used vec
    let yason = Scalar::smallint_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Smallint);
    let n = yason.smallint().unwrap();
    assert_eq!(n, i);
}

#[test]
fn test_integer() {
    let i = 123456_i32;
    let yason = Scalar::integer(i).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Integer);
    let n = yason.integer().unwrap();
    assert_eq!(n, i);

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::integer_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Integer);
    let n = yason.integer().unwrap();
    assert_eq!(n, i);

    // test from used vec
    let yason = Scalar::integer_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Integer);
    let n = yason.integer().unwrap();
    assert_eq!(n, i);
}

#[test]
fn test_bigint() {
    let i = 1234567_i64;
    let yason = Scalar::bigint(i).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Bigint);
    let n = yason.bigint().unwrap();
    assert_eq!(n, i);

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::bigint_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Bigint);
    let n = yason.bigint().unwrap();
    assert_eq!(n, i);

    // test from used vec
    let yason = Scalar::bigint_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Bigint);
    let n = yason.bigint().unwrap();
    assert_eq!(n, i);
}

#[test]
fn test_float() {
    let i = 123.456_f32;
    let yason = Scalar::float(i).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Float);
    let n = yason.float().unwrap();
    assert_eq!(n, i);

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::float_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Float);
    let n = yason.float().unwrap();
    assert_eq!(n, i);

    // test from used vec
    let yason = Scalar::float_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Float);
    let n = yason.float().unwrap();
    assert_eq!(n, i);
}

#[test]
fn test_double() {
    let i = 12.3456789_f64;
    let yason = Scalar::double(i).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Double);
    let n = yason.double().unwrap();
    assert_eq!(n, i);

    // test from vec
    let mut bytes: Vec<u8> = Vec::with_capacity(128);
    let yason = Scalar::double_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Double);
    let n = yason.double().unwrap();
    assert_eq!(n, i);

    // test from used vec
    let yason = Scalar::double_with_vec(i, &mut bytes).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Double);
    let n = yason.double().unwrap();
    assert_eq!(n, i);
}
