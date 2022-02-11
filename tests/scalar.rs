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
    let yason = Scalar::bool(true).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Bool);
    let value = yason.bool().unwrap();
    assert!(value);

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
