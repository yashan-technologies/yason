//! Json to Yason tests

use std::cmp::Ordering;
use std::str::FromStr;
use yason::{Array, DataType, Date, Number, Object, Time, Timestamp, Value, YasonBuf};

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
        DataType::Timestamp => {
            assert_eq!(yason.data_type().unwrap(), DataType::Timestamp);
            assert_eq!(
                yason.timestamp().unwrap(),
                Timestamp::iso8601_formatter()
                    .parse::<&str, Timestamp>(expected)
                    .unwrap()
            );
        }
        DataType::Date => {
            assert_eq!(yason.data_type().unwrap(), DataType::Date);
            assert_eq!(
                yason.date().unwrap(),
                Date::iso8601_formatter().parse::<&str, Date>(expected).unwrap()
            );
        }
        DataType::Time => {
            assert_eq!(yason.data_type().unwrap(), DataType::Time);
            assert_eq!(
                yason.time().unwrap(),
                Time::iso8601_formatter().parse::<&str, Time>(expected).unwrap()
            );
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
    assert_scalar_extended(
        r#"{"$yashanTimestamp": "2023-05-25T16:50:20.123"}"#,
        "2023-05-25T16:50:20.123000",
        DataType::Timestamp,
    );
    assert_scalar_extended(
        r#"{"$yashanDate": "2023-05-25T16:50:20"}"#,
        "2023-05-25T16:50:20",
        DataType::Date,
    );
    assert_scalar_extended(r#"{"$yashanTime": "16:50:20"}"#, "16:50:20", DataType::Time);
}

enum TestValue {
    Object(Vec<(String, TestValue)>),
    Array(Vec<TestValue>),
    Number(Number),
    String(String),
    Bool(bool),
    Null,
    Tinyint(i8),
    Smallint(i16),
    Integer(i32),
    Bigint(i64),
    Float(f32),
    Double(f64),
    Binary(Vec<u8>),
    Timestamp(Timestamp),
    Date(Date),
    Time(Time),
}

impl TestValue {
    fn object(&mut self) -> &mut [(String, TestValue)] {
        match self {
            TestValue::Object(object) => object.as_mut(),
            _ => panic!(),
        }
    }

    fn array(&mut self) -> &mut [TestValue] {
        match self {
            TestValue::Array(array) => array.as_mut(),
            _ => panic!(),
        }
    }

    fn number(&self) -> Number {
        match self {
            TestValue::Number(n) => *n,
            _ => panic!(),
        }
    }

    fn string(&self) -> &str {
        match self {
            TestValue::String(str) => str,
            _ => panic!(),
        }
    }

    fn bool(&self) -> bool {
        match self {
            TestValue::Bool(b) => *b,
            _ => panic!(),
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, TestValue::Null)
    }

    fn tinyint(&self) -> i8 {
        match self {
            TestValue::Tinyint(i) => *i,
            _ => panic!(),
        }
    }

    fn smallint(&self) -> i16 {
        match self {
            TestValue::Smallint(i) => *i,
            _ => panic!(),
        }
    }

    fn integer(&self) -> i32 {
        match self {
            TestValue::Integer(i) => *i,
            _ => panic!(),
        }
    }

    fn bigint(&self) -> i64 {
        match self {
            TestValue::Bigint(i) => *i,
            _ => panic!(),
        }
    }

    fn float(&self) -> f32 {
        match self {
            TestValue::Float(f) => *f,
            _ => panic!(),
        }
    }

    fn double(&self) -> f64 {
        match self {
            TestValue::Double(f) => *f,
            _ => panic!(),
        }
    }

    fn binary(&self) -> &[u8] {
        match self {
            TestValue::Binary(bin) => bin,
            _ => panic!(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            TestValue::Timestamp(t) => *t,
            _ => panic!(),
        }
    }

    fn date(&self) -> Date {
        match self {
            TestValue::Date(t) => *t,
            _ => panic!(),
        }
    }

    fn time(&self) -> Time {
        match self {
            TestValue::Time(t) => *t,
            _ => panic!(),
        }
    }

    fn data_type(&self) -> DataType {
        match self {
            TestValue::Object(_) => DataType::Object,
            TestValue::Array(_) => DataType::Array,
            TestValue::Number(_) => DataType::Number,
            TestValue::String(_) => DataType::String,
            TestValue::Bool(_) => DataType::Bool,
            TestValue::Null => DataType::Null,
            TestValue::Tinyint(_) => DataType::Tinyint,
            TestValue::Smallint(_) => DataType::Smallint,
            TestValue::Integer(_) => DataType::Integer,
            TestValue::Bigint(_) => DataType::Bigint,
            TestValue::Float(_) => DataType::Float,
            TestValue::Double(_) => DataType::Double,
            TestValue::Binary(_) => DataType::Binary,
            TestValue::Timestamp(_) => DataType::Timestamp,
            TestValue::Date(_) => DataType::Date,
            TestValue::Time(_) => DataType::Time,
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
        Value::String(val) => assert_eq!(val, expected.string()),
        Value::Number(val) => assert_eq!(val, expected.number()),
        Value::Bool(val) => assert_eq!(val, expected.bool()),
        Value::Null => assert!(expected.is_null()),
        Value::Tinyint(i) => assert_eq!(i, expected.tinyint()),
        Value::Smallint(i) => assert_eq!(i, expected.smallint()),
        Value::Integer(i) => assert_eq!(i, expected.integer()),
        Value::Bigint(i) => assert_eq!(i, expected.bigint()),
        Value::Float(f) => assert_eq!(f, expected.float()),
        Value::Double(f) => assert_eq!(f, expected.double()),
        Value::Binary(bin) => assert_eq!(bin, expected.binary()),
        Value::Timestamp(t) => assert_eq!(t, expected.timestamp()),
        Value::Date(t) => assert_eq!(t, expected.date()),
        Value::Time(t) => assert_eq!(t, expected.time()),
    }
}

#[test]
fn test_array() {
    let input = r#"[]"#;
    let expected = vec![];
    let yason = YasonBuf::parse(input, false).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason.array().unwrap(), &mut TestValue::Array(expected));

    let input = r#"["John Doe", 43, true, null,[2345678], {"key": true}]"#;
    let expected = vec![
        TestValue::String("John Doe".to_string()),
        TestValue::Number("43".parse().unwrap()),
        TestValue::Bool(true),
        TestValue::Null,
        TestValue::Array(vec![TestValue::Number("2345678".parse().unwrap())]),
        TestValue::Object(vec![("key".to_string(), TestValue::Bool(true))]),
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
    let expected = vec![("key".to_string(), TestValue::Number("123".parse().unwrap()))];
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
        ("name".to_string(), TestValue::String("John Doe".to_string())),
        ("age".to_string(), TestValue::Number("43".parse().unwrap())),
        ("bool".to_string(), TestValue::Bool(true)),
        ("null".to_string(), TestValue::Null),
        (
            "phone".to_string(),
            TestValue::Array(vec![TestValue::Number("2345678".parse().unwrap())]),
        ),
        (
            "object".to_string(),
            TestValue::Object(vec![("key".to_string(), TestValue::Bool(true))]),
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
        {"$binary": "aGVsbG8h"},
        {"$yashanTimestamp": "9999-12-31T23:59:59.999999"},
        {"$yashanDate": "9999-12-31T23:59:59"},
        {"$yashanTime": "23:59:59.999999"}
    ]"#;
    let expected = vec![
        TestValue::Tinyint(123),
        TestValue::Smallint(12345),
        TestValue::Integer(123456),
        TestValue::Bigint(123456789),
        TestValue::Number("123.456789".parse().unwrap()),
        TestValue::Float("123.456".parse().unwrap()),
        TestValue::Double("12.3456789".parse().unwrap()),
        TestValue::Binary(Vec::from(b"hello!".as_slice())),
        TestValue::Timestamp(Timestamp::MAX),
        TestValue::Date(Date::MAX),
        TestValue::Time(Time::MAX),
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
        "binary": {"$binary": "aGVsbG8h"},
        "ts": {"$yashanTimestamp": "9999-12-31T23:59:59.999999"},
        "date": {"$yashanDate": "9999-12-31T23:59:59"},
        "time": {"$yashanTime": "23:59:59.999999"}
    }"#;

    let expected = vec![
        ("tinyint".to_string(), TestValue::Tinyint(123)),
        ("smallint".to_string(), TestValue::Smallint(12345)),
        ("integer".to_string(), TestValue::Integer(123456)),
        ("bigint".to_string(), TestValue::Bigint(123456789)),
        ("number".to_string(), TestValue::Number("123.456789".parse().unwrap())),
        ("float".to_string(), TestValue::Float("123.456".parse().unwrap())),
        ("double".to_string(), TestValue::Double("12.3456789".parse().unwrap())),
        ("binary".to_string(), TestValue::Binary(Vec::from(b"hello!".as_slice()))),
        ("ts".to_string(), TestValue::Timestamp(Timestamp::MAX)),
        ("date".to_string(), TestValue::Date(Date::MAX)),
        ("time".to_string(), TestValue::Time(Time::MAX)),
    ];

    let yason = YasonBuf::parse(input, true).unwrap();
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason.object().unwrap(), &mut TestValue::Object(expected));
}
