//! Json to Yason tests

use std::cmp::Ordering;
use std::str::FromStr;
use yason::{Array, DataType, Number, Object, Value, YasonBuf};

fn assert_scalar_inner(input: &str, expected: &str, expected_type: DataType, extended: bool) {
    let yason = YasonBuf::parse(input, extended).unwrap();
    match expected_type {
        DataType::String => {
            assert_eq!(yason.data_type().unwrap(), DataType::String);
            assert_eq!(yason.string().unwrap(), expected);
        }
        DataType::Number => {
            assert_eq!(yason.data_type().unwrap(), DataType::Number);
            assert_eq!(yason.number().unwrap(), Number::from_str(expected).unwrap());
        }
        DataType::Bool => {
            assert_eq!(yason.data_type().unwrap(), DataType::Bool);
            assert_eq!(yason.bool().unwrap(), bool::from_str(expected).unwrap());
        }
        DataType::Null => {
            assert_eq!(yason.data_type().unwrap(), DataType::Null);
            assert!(yason.is_null().unwrap());
        }
        DataType::Tinyint => {
            assert_eq!(yason.data_type().unwrap(), DataType::Tinyint);
            assert_eq!(yason.tinyint().unwrap(), i8::from_str(expected).unwrap());
        }
        DataType::Smallint => {
            assert_eq!(yason.data_type().unwrap(), DataType::Smallint);
            assert_eq!(yason.smallint().unwrap(), i16::from_str(expected).unwrap());
        }
        DataType::Integer => {
            assert_eq!(yason.data_type().unwrap(), DataType::Integer);
            assert_eq!(yason.integer().unwrap(), i32::from_str(expected).unwrap());
        }
        DataType::Bigint => {
            assert_eq!(yason.data_type().unwrap(), DataType::Bigint);
            assert_eq!(yason.bigint().unwrap(), i64::from_str(expected).unwrap());
        }
        DataType::Float => {
            assert_eq!(yason.data_type().unwrap(), DataType::Float);
            assert_eq!(yason.float().unwrap(), f32::from_str(expected).unwrap());
        }
        DataType::Double => {
            assert_eq!(yason.data_type().unwrap(), DataType::Double);
            assert_eq!(yason.double().unwrap(), f64::from_str(expected).unwrap());
        }
        DataType::Binary => {
            assert_eq!(yason.data_type().unwrap(), DataType::Binary);
            assert_eq!(yason.binary().unwrap(), expected.as_bytes());
        }
        DataType::Object => unreachable!(),
        DataType::Array => unreachable!(),
    }
}

fn assert_scalar(input: &str, expected: &str, expected_type: DataType) {
    assert_scalar_inner(input, expected, expected_type, false);
}

fn assert_scalar_extended(input: &str, expected: &str, expected_type: DataType) {
    assert_scalar_inner(input, expected, expected_type, true);
}

#[test]
fn test_scalar() {
    // string
    assert_scalar(r#""string""#, "string", DataType::String);
    assert_scalar(r#""Nan""#, "Nan", DataType::String);
    assert_scalar(r#""string\tstring""#, "string\tstring", DataType::String);
    assert_scalar(r#""string\\string""#, "string\\string", DataType::String);
    assert_scalar(r#""string\nstring""#, "string\nstring", DataType::String);
    assert_scalar(r#""string\"string""#, "string\"string", DataType::String);
    assert_scalar(r#""string\rstring""#, "string\rstring", DataType::String);

    // number
    assert_scalar("123", "123", DataType::Number);
    assert_scalar("123e2", "123e2", DataType::Number);
    assert_scalar("123.123", "123.123", DataType::Number);
    assert_scalar(
        "222222222222222222222222222222222222222222",
        "222222222222222222222222222222222222222200",
        DataType::Number,
    );
    assert_scalar(
        "555555555555555555555555555555555555555555",
        "555555555555555555555555555555555555555600",
        DataType::Number,
    );
    assert_scalar("1e-140", "0", DataType::Number);

    // bool
    assert_scalar("true", "true", DataType::Bool);
    assert_scalar("false", "false", DataType::Bool);

    // null
    assert_scalar("null", "null", DataType::Null);

    assert_scalar_extended("123", "123", DataType::Tinyint);
    assert_scalar_extended("12345", "12345", DataType::Smallint);
    assert_scalar_extended("1234567", "1234567", DataType::Integer);
    assert_scalar_extended("12345678900", "12345678900", DataType::Bigint);
    assert_scalar_extended(r#"{"$numberDecimal": "12.34567"}"#, "12.34567", DataType::Number);
    assert_scalar_extended("123.123", "123.123", DataType::Double);
    assert_scalar_extended(r#"{"$numberDouble": "inf"}"#, "Inf", DataType::Double);
    assert_scalar_extended(r#"{"$numberDouble": "-infinity"}"#, "-infinity", DataType::Double);
    assert_scalar_extended(r#"{"$numberFloat": "inf"}"#, "Inf", DataType::Float);
    assert_scalar_extended(r#"{"$numberFloat": "-infinity"}"#, "-infinity", DataType::Float);

    assert_scalar_extended(r#"{"$binary": "aGVsbG8h"}"#, "hello!", DataType::Binary);
    assert_scalar_extended(
        r#"{"$binary": {"base64": "SmF2YVNjcmlwdA==", "subType": 0}}"#,
        "JavaScript",
        DataType::Binary,
    );
}

enum TestValue {
    Scalar((DataType, String)),
    Object(Vec<(String, TestValue)>),
    Array(Vec<TestValue>),
}

impl TestValue {
    fn scalar(&self) -> &str {
        match self {
            TestValue::Scalar((_, str)) => str.as_str(),
            _ => unreachable!(),
        }
    }

    fn object(&mut self) -> &mut [(String, TestValue)] {
        match self {
            TestValue::Object(object) => object.as_mut(),
            _ => unreachable!(),
        }
    }

    fn array(&mut self) -> &mut [TestValue] {
        match self {
            TestValue::Array(array) => array.as_mut(),
            _ => unreachable!(),
        }
    }

    fn data_type(&self) -> DataType {
        match self {
            TestValue::Scalar((ty, _)) => *ty,
            TestValue::Object(_) => DataType::Object,
            TestValue::Array(_) => DataType::Array,
        }
    }
}

fn assert_object(object: Object, expected: &mut TestValue) {
    let expected = expected.object();
    assert_eq!(object.len().unwrap(), expected.len());

    expected.sort_by(|a, b| match a.0.len().cmp(&b.0.len()) {
        Ordering::Equal => a.0.cmp(&b.0),
        Ordering::Greater => Ordering::Greater,
        Ordering::Less => Ordering::Less,
    });

    for (id, item) in object.iter().unwrap().enumerate() {
        let (key, value) = item.unwrap();
        assert_eq!(key, expected[id].0.as_str());
        assert_value(value, &mut expected[id].1);
    }
}

fn assert_array(array: Array, expected: &mut TestValue) {
    let expected = expected.array();
    assert_eq!(array.len().unwrap(), expected.len());

    for (id, value) in array.iter().unwrap().enumerate() {
        assert_value(value.unwrap(), &mut expected[id]);
    }
}

fn assert_value(value: Value, expected: &mut TestValue) {
    assert_eq!(value.data_type(), expected.data_type());
    match value {
        Value::Object(obj) => assert_object(obj, expected),
        Value::Array(arr) => assert_array(arr, expected),
        Value::String(val) => assert_eq!(val, expected.scalar()),
        Value::Number(val) => assert_eq!(val, Number::from_str(expected.scalar()).unwrap()),
        Value::Bool(val) => assert_eq!(val, bool::from_str(expected.scalar()).unwrap()),
        Value::Null => assert_eq!("null", expected.scalar()),
        Value::Tinyint(i) => assert_eq!(i, i8::from_str(expected.scalar()).unwrap()),
        Value::Smallint(i) => assert_eq!(i, i16::from_str(expected.scalar()).unwrap()),
        Value::Integer(i) => assert_eq!(i, i32::from_str(expected.scalar()).unwrap()),
        Value::Bigint(i) => assert_eq!(i, i64::from_str(expected.scalar()).unwrap()),
        Value::Float(f) => assert_eq!(f, f32::from_str(expected.scalar()).unwrap()),
        Value::Double(f) => assert_eq!(f, f64::from_str(expected.scalar()).unwrap()),
        Value::Binary(bin) => assert_eq!(bin, expected.scalar().as_bytes()),
    }
}

#[test]
fn test_array() {
    let input = r#"[]"#;
    let expected = vec![];
    let yason = YasonBuf::parse(input, false).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason.array().unwrap(), &mut TestValue::Array(expected));

    let input = r#"["John Doe", 43, true, null, [2345678], {"key": true}]"#;
    let expected = vec![
        TestValue::Scalar((DataType::String, "John Doe".to_string())),
        TestValue::Scalar((DataType::Number, "43".to_string())),
        TestValue::Scalar((DataType::Bool, "true".to_string())),
        TestValue::Scalar((DataType::Null, "null".to_string())),
        TestValue::Array(vec![TestValue::Scalar((DataType::Number, "2345678".to_string()))]),
        TestValue::Object(vec![(
            "key".to_string(),
            TestValue::Scalar((DataType::Bool, "true".to_string())),
        )]),
    ];

    let yason = YasonBuf::parse(input, false).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason.array().unwrap(), &mut TestValue::Array(expected));
}

#[test]
fn test_object() {
    let input = r#"{}"#;
    let expected = vec![];
    let yason = YasonBuf::parse(input, false).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason.object().unwrap(), &mut TestValue::Object(expected));

    let input = r#"{"key": 123}"#;
    let expected = vec![(
        "key".to_string(),
        TestValue::Scalar((DataType::Number, "123".to_string())),
    )];
    let yason = YasonBuf::parse(input, false).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason.object().unwrap(), &mut TestValue::Object(expected));

    let input = r#"{
        "name": "John Doe",
        "age": 43,
        "bool": true,
        "null": null,
        "phone": [2345678],
        "object": {"key": true}
    }"#;
    let expected = vec![
        (
            "name".to_string(),
            TestValue::Scalar((DataType::String, "John Doe".to_string())),
        ),
        (
            "age".to_string(),
            TestValue::Scalar((DataType::Number, "43".to_string())),
        ),
        (
            "bool".to_string(),
            TestValue::Scalar((DataType::Bool, "true".to_string())),
        ),
        (
            "null".to_string(),
            TestValue::Scalar((DataType::Null, "null".to_string())),
        ),
        (
            "phone".to_string(),
            TestValue::Array(vec![TestValue::Scalar((DataType::Number, "2345678".to_string()))]),
        ),
        (
            "object".to_string(),
            TestValue::Object(vec![(
                "key".to_string(),
                TestValue::Scalar((DataType::Bool, "true".to_string())),
            )]),
        ),
    ];

    let yason = YasonBuf::parse(input, false).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason.object().unwrap(), &mut TestValue::Object(expected));
}

#[test]
fn test_extended_array() {
    let input = r#"[
        {"$numberByte": 123},
        {"$numberShort": 12345},
        {"$numberInt": 123456},
        {"$numberLong": 123456789},
        {"$numberDecimal": "123.456789"},
        {"$numberFloat": "123.456"},
        {"$numberDouble": "12.3456789"},
        {"$binary": "aGVsbG8h"}
    ]"#;
    let expected = vec![
        TestValue::Scalar((DataType::Tinyint, "123".to_string())),
        TestValue::Scalar((DataType::Smallint, "12345".to_string())),
        TestValue::Scalar((DataType::Integer, "123456".to_string())),
        TestValue::Scalar((DataType::Bigint, "123456789".to_string())),
        TestValue::Scalar((DataType::Number, "123.456789".to_string())),
        TestValue::Scalar((DataType::Float, "123.456".to_string())),
        TestValue::Scalar((DataType::Double, "12.3456789".to_string())),
        TestValue::Scalar((DataType::Binary, "hello!".to_string())),
    ];

    let yason = YasonBuf::parse(input, true).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason.array().unwrap(), &mut TestValue::Array(expected));
}

#[test]
fn test_extended_object() {
    let input = r#"{
        "tinyint": {"$numberByte": 123},
        "smallint": {"$numberShort": 12345},
        "integer": {"$numberInt": 123456},
        "bigint": {"$numberLong": 123456789},
        "number": {"$numberDecimal": "123.456789"},
        "float": {"$numberFloat": "123.456"},
        "double": {"$numberDouble": "12.3456789"},
        "binary": {"$binary": "aGVsbG8h"}
    }"#;
    let expected = vec![
        (
            "tinyint".to_string(),
            TestValue::Scalar((DataType::Tinyint, "123".to_string())),
        ),
        (
            "smallint".to_string(),
            TestValue::Scalar((DataType::Smallint, "12345".to_string())),
        ),
        (
            "integer".to_string(),
            TestValue::Scalar((DataType::Integer, "123456".to_string())),
        ),
        (
            "bigint".to_string(),
            TestValue::Scalar((DataType::Bigint, "123456789".to_string())),
        ),
        (
            "number".to_string(),
            TestValue::Scalar((DataType::Number, "123.456789".to_string())),
        ),
        (
            "float".to_string(),
            TestValue::Scalar((DataType::Float, "123.456".to_string())),
        ),
        (
            "double".to_string(),
            TestValue::Scalar((DataType::Double, "12.3456789".to_string())),
        ),
        (
            "binary".to_string(),
            TestValue::Scalar((DataType::Binary, "hello!".to_string())),
        ),
    ];

    let yason = YasonBuf::parse(input, true).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason.object().unwrap(), &mut TestValue::Object(expected));
}
