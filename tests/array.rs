//! Array builder tests.

use yason::{ArrayBuilder, ArrayRefBuilder, BuildError, DataType, Number, Value, Yason, YasonBuf};

fn assert_string<T: AsRef<str>>(input: Value, expected: T) {
    if let Value::String(value) = input {
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

fn assert_array(yason: &Yason) {
    let array = yason.array().unwrap();
    assert_eq!(array.len().unwrap(), 6);
    assert!(!array.is_empty().unwrap());
    assert_eq!(array.type_of(0).unwrap(), DataType::Number);
    assert!(array.is_type(1, DataType::String).unwrap());

    assert_number(array.get(0).unwrap(), Number::from(123));
    assert_string(array.get(1).unwrap(), "abc");
    assert_null(array.get(2).unwrap());
    assert!(array.is_null(2).unwrap());
    assert_bool(array.get(3).unwrap(), false);
    assert_eq!(array.get(4).unwrap().data_type(), DataType::Array);
    assert_eq!(array.get(5).unwrap().data_type(), DataType::Object);

    assert!(array.bool(0).is_err());

    let value = array.get(10);
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
            assert_eq!(value.data_type(), DataType::Array);
        } else if id == 5 {
            assert_eq!(value.data_type(), DataType::Object);
        }
    }

    assert_eq!(array.object(5).unwrap().string("key").unwrap().unwrap(), "value");
    assert!(array.array(4).unwrap().bool(0).unwrap());
    assert!(!array.bool(3).unwrap());
    assert_eq!(array.string(1).unwrap(), "abc");
    assert_eq!(array.number(0).unwrap(), Number::from(123));
}

fn create_yason() -> YasonBuf {
    // [123, "abc", null, false, [true], {key: value}]
    let mut builder = ArrayBuilder::try_new(6).unwrap();
    builder.push_number(Number::from(123)).unwrap();
    builder.push_string("abc").unwrap();
    builder.push_null().unwrap();
    builder.push_bool(false).unwrap();

    let mut array_builder = builder.push_array(1).unwrap();
    array_builder.push_bool(true).unwrap();
    array_builder.finish().unwrap();

    let mut object_builder = builder.push_object(1, true).unwrap();
    object_builder.push_string("key", "value").unwrap();
    object_builder.finish().unwrap();

    builder.finish().unwrap()
}

fn create_yason_with_vec(bytes: &mut Vec<u8>) -> &Yason {
    // [123, "abc", null, false, [true], {key: value}]
    let mut builder = ArrayRefBuilder::try_new(bytes, 6).unwrap();
    builder.push_number(Number::from(123)).unwrap();
    builder.push_string("abc").unwrap();
    builder.push_null().unwrap();
    builder.push_bool(false).unwrap();

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
