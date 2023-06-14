//! Array builder tests.

use yason::{
    ArrayBuilder, ArrayRefBuilder, BuildError, DataType, Date, Number, Time, Timestamp, Value, Yason, YasonBuf,
};

fn assert_string<T: AsRef<str>>(input: Value, expected: T) {
    if let Value::String(value) = input {
        assert_eq!(value, expected.as_ref());
    } else {
        panic!("type inconsistency");
    };
}

fn assert_binary<T: AsRef<[u8]>>(input: Value, expected: T) {
    if let Value::Binary(value) = input {
        assert_eq!(value, expected.as_ref());
    } else {
        panic!("type inconsistency");
    };
}

fn assert_number(input: Value, expected: Number) {
    if let Value::Number(value) = input {
        assert_eq!(value, expected);
    } else {
        panic!("type inconsistency");
    };
}

fn assert_bool(input: Value, expected: bool) {
    if let Value::Bool(value) = input {
        assert_eq!(value, expected);
    } else {
        panic!("type inconsistency");
    };
}

fn assert_null(input: Value) {
    let res = matches!(input, Value::Null);
    assert!(res);
}

macro_rules! assert_value_eq {
    ($ty: ident, $input: expr, $expected: expr) => {
        if let Value::$ty(value) = $input {
            assert_eq!(value, $expected);
        } else {
            panic!("type inconsistency");
        };
    };
}

fn assert_array(yason: &Yason) {
    let array = yason.array().unwrap();
    assert_eq!(array.len().unwrap(), 16);
    assert!(!array.is_empty().unwrap());
    assert_eq!(array.type_of(0).unwrap(), DataType::Number);
    assert!(array.is_type(1, DataType::String).unwrap());

    assert_number(array.get(0).unwrap(), Number::from(123));
    assert_string(array.get(1).unwrap(), "abc");
    assert_null(array.get(2).unwrap());
    assert!(array.is_null(2).unwrap());
    assert_bool(array.get(3).unwrap(), false);
    assert_value_eq!(Tinyint, array.get(4).unwrap(), 123_i8);
    assert_value_eq!(Smallint, array.get(5).unwrap(), 12345_i16);
    assert_value_eq!(Integer, array.get(6).unwrap(), 1234567_i32);
    assert_value_eq!(Bigint, array.get(7).unwrap(), 12345678_i64);
    assert_value_eq!(Float, array.get(8).unwrap(), 123.456_f32);
    assert_value_eq!(Double, array.get(9).unwrap(), 12.3456789_f64);
    assert_binary(array.get(10).unwrap(), b"abc");
    assert_value_eq!(Timestamp, array.get(11).unwrap(), Timestamp::MAX);
    assert_value_eq!(Date, array.get(12).unwrap(), Date::MAX);
    assert_value_eq!(Time, array.get(13).unwrap(), Time::MAX);
    assert_eq!(array.get(14).unwrap().data_type(), DataType::Array);
    assert_eq!(array.get(15).unwrap().data_type(), DataType::Object);

    assert!(array.bool(0).is_err());

    let value = array.get(16);
    assert!(value.is_err());

    // tests iter
    for (id, value) in array.iter().unwrap().enumerate() {
        let value = value.unwrap();
        if id == 0 {
            assert_number(value, Number::from(123));
        } else if id == 1 {
            assert_string(value, "abc");
        } else if id == 2 {
            assert_null(value);
        } else if id == 3 {
            assert_bool(value, false);
        } else if id == 4 {
            assert_value_eq!(Tinyint, value, 123_i8);
        } else if id == 5 {
            assert_value_eq!(Smallint, value, 12345_i16);
        } else if id == 6 {
            assert_value_eq!(Integer, value, 1234567_i32);
        } else if id == 7 {
            assert_value_eq!(Bigint, value, 12345678_i64);
        } else if id == 8 {
            assert_value_eq!(Float, value, 123.456_f32);
        } else if id == 9 {
            assert_value_eq!(Double, value, 12.3456789_f64);
        } else if id == 10 {
            assert_binary(value, b"abc");
        } else if id == 11 {
            assert_value_eq!(Timestamp, value, Timestamp::MAX);
        } else if id == 12 {
            assert_value_eq!(Date, value, Date::MAX);
        } else if id == 13 {
            assert_value_eq!(Time, value, Time::MAX);
        } else if id == 14 {
            assert_eq!(value.data_type(), DataType::Array);
        } else if id == 15 {
            assert_eq!(value.data_type(), DataType::Object);
        }
    }

    assert_eq!(array.object(15).unwrap().string("key").unwrap().unwrap(), "value");
    assert!(array.array(14).unwrap().bool(0).unwrap());
    assert_eq!(array.time(13).unwrap(), Time::MAX);
    assert_eq!(array.date(12).unwrap(), Date::MAX);
    assert_eq!(array.timestamp(11).unwrap(), Timestamp::MAX);
    assert_eq!(array.binary(10).unwrap(), b"abc");
    assert_eq!(array.double(9).unwrap(), 12.3456789_f64);
    assert_eq!(array.float(8).unwrap(), 123.456_f32);
    assert_eq!(array.bigint(7).unwrap(), 12345678_i64);
    assert_eq!(array.integer(6).unwrap(), 1234567_i32);
    assert_eq!(array.smallint(5).unwrap(), 12345_i16);
    assert_eq!(array.tinyint(4).unwrap(), 123_i8);
    assert!(!array.bool(3).unwrap());
    assert_eq!(array.string(1).unwrap(), "abc");
    assert_eq!(array.number(0).unwrap(), Number::from(123));
}

fn create_yason() -> YasonBuf {
    let mut builder = ArrayBuilder::try_new(16).unwrap();
    builder.push_number(Number::from(123)).unwrap();
    builder.push_string("abc").unwrap();
    builder.push_null().unwrap();
    builder.push_bool(false).unwrap();
    builder.push_tinyint(123_i8).unwrap();
    builder.push_smallint(12345_i16).unwrap();
    builder.push_integer(1234567_i32).unwrap();
    builder.push_bigint(12345678_i64).unwrap();
    builder.push_float(123.456_f32).unwrap();
    builder.push_double(12.3456789_f64).unwrap();
    builder.push_binary(b"abc").unwrap();
    builder.push_timestamp(Timestamp::MAX).unwrap();
    builder.push_date(Date::MAX).unwrap();
    builder.push_time(Time::MAX).unwrap();

    let mut array_builder = builder.push_array(1).unwrap();
    array_builder.push_bool(true).unwrap();
    array_builder.finish().unwrap();

    let mut object_builder = builder.push_object(1, true).unwrap();
    object_builder.push_string("key", "value").unwrap();
    object_builder.finish().unwrap();

    builder.finish().unwrap()
}

fn create_yason_with_vec(bytes: &mut Vec<u8>) -> &Yason {
    let mut builder = ArrayRefBuilder::try_new(bytes, 16).unwrap();
    builder.push_number(Number::from(123)).unwrap();
    builder.push_string("abc").unwrap();
    builder.push_null().unwrap();
    builder.push_bool(false).unwrap();
    builder.push_tinyint(123_i8).unwrap();
    builder.push_smallint(12345_i16).unwrap();
    builder.push_integer(1234567_i32).unwrap();
    builder.push_bigint(12345678_i64).unwrap();
    builder.push_float(123.456_f32).unwrap();
    builder.push_double(12.3456789_f64).unwrap();
    builder.push_binary(b"abc").unwrap();
    builder.push_timestamp(Timestamp::MAX).unwrap();
    builder.push_date(Date::MAX).unwrap();
    builder.push_time(Time::MAX).unwrap();

    let mut array_builder = builder.push_array(1).unwrap();
    array_builder.push_bool(true).unwrap();
    array_builder.finish().unwrap();

    let mut object_builder = builder.push_object(1, true).unwrap();
    object_builder.push_string("key", "value").unwrap();
    object_builder.finish().unwrap();

    builder.finish().unwrap()
}

#[test]
fn test_array() {
    let yason = create_yason();
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason.as_ref())
}

#[test]
fn test_array_with_vec() {
    let mut bytes = Vec::with_capacity(128);
    let yason = create_yason_with_vec(&mut bytes);
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason);
}

#[test]
fn test_array_with_used_vec() {
    let mut bytes = Vec::with_capacity(128);
    bytes.push(1u8);
    bytes.push(1u8);
    bytes.push(1u8);
    bytes.push(1u8);

    let yason = create_yason_with_vec(&mut bytes);
    assert_eq!(yason.data_type().unwrap(), DataType::Array);
    assert_array(yason);
}

#[test]
fn test_create_array_error() {
    let mut builder = ArrayBuilder::try_new(3).unwrap();
    builder.push_bool(true).unwrap();
    let res = builder.finish();
    assert!(res.is_err());

    let mut builder = ArrayBuilder::try_new(3).unwrap();
    let _ = builder.push_array(1).unwrap();
    let res = builder.finish();
    assert!(res.is_err());
}

#[test]
fn test_array_finish_error() {
    let mut builder = ArrayBuilder::try_new(1).unwrap();
    let _ = builder.push_array(1).unwrap();
    let res = builder.finish();
    assert!(matches!(res.err(), Some(BuildError::InnerUncompletedError)));

    let mut builder = ArrayBuilder::try_new(1).unwrap();
    let _ = builder.push_array(1).unwrap();
    let res = builder.push_null();
    assert!(matches!(res.err(), Some(BuildError::InnerUncompletedError)));
}

#[test]
fn test_array_nested_depth() {
    fn assert_nested_depth(expect_depth: usize, err: Option<BuildError>) {
        fn inner(
            builder: Result<&mut ArrayRefBuilder, BuildError>,
            cur_depth: usize,
            total_depth: usize,
        ) -> Option<BuildError> {
            if cur_depth < total_depth {
                let nested_builder = builder.unwrap().push_array(1);
                return if cur_depth < 100 {
                    inner(Ok(&mut nested_builder.unwrap()), cur_depth + 1, total_depth)
                } else {
                    nested_builder.err()
                };
            }
            None
        }

        let mut bytes = vec![];
        let mut builder = ArrayRefBuilder::try_new(&mut bytes, 1).unwrap();
        let res = inner(Ok(&mut builder), 1, expect_depth);

        if let Some(e) = err {
            assert!(matches!(e, BuildError::NestedTooDeeply));
        } else {
            assert!(res.is_none());
        }
    }

    assert_nested_depth(98, None);
    assert_nested_depth(99, None);
    assert_nested_depth(100, None);
    assert_nested_depth(101, Some(BuildError::NestedTooDeeply));
    assert_nested_depth(102, Some(BuildError::NestedTooDeeply));
}
