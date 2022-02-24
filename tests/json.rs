//! Json to Yason tests

use std::cmp::Ordering;
use std::str::FromStr;
use yason::{Array, DataType, Number, Object, Value, YasonBuf};

fn assert_scalar(input: &str, expected: &str, expected_type: DataType) {
    let yason = YasonBuf::parse(input).unwrap();
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
        _ => {}
    }
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
    }
}

#[test]
fn test_array() {
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

    let yason = YasonBuf::parse(input).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason.array().unwrap(), &mut TestValue::Array(expected));
}

#[test]
fn test_object() {
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

    let yason = YasonBuf::parse(input).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason.object().unwrap(), &mut TestValue::Object(expected));
}
