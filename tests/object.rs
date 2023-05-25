//! Object builder tests.

use yason::{BuildError, DataType, Number, ObjectBuilder, ObjectRefBuilder, Value, Yason, YasonBuf};

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

fn assert_object(yason: &Yason) {
    let object = yason.object().unwrap();
    assert_eq!(object.len().unwrap(), 13);
    assert!(!object.is_empty().unwrap());
    assert_eq!(object.type_of("id").unwrap().unwrap(), DataType::Number);
    assert!(object.is_type("name", DataType::String).unwrap().unwrap());
    assert!(object.contains_key("child").unwrap());

    assert_number(object.get("id").unwrap().unwrap(), Number::from(1));
    assert_string(object.get("name").unwrap().unwrap(), "abc");
    assert_bool(object.get("child").unwrap().unwrap(), false);
    assert_null(object.get("phone").unwrap().unwrap());
    assert_value_eq!(Tinyint, object.get("tinyint").unwrap().unwrap(), 123_i8);
    assert_value_eq!(Smallint, object.get("smallint").unwrap().unwrap(), 12345_i16);
    assert_value_eq!(Integer, object.get("integer").unwrap().unwrap(), 1234567_i32);
    assert_value_eq!(Bigint, object.get("bigint").unwrap().unwrap(), 12345678_i64);
    assert_value_eq!(Float, object.get("float").unwrap().unwrap(), 123.456_f32);
    assert_value_eq!(Double, object.get("double").unwrap().unwrap(), 12.3456789_f64);
    assert_eq!(object.get("array").unwrap().unwrap().data_type(), DataType::Array);
    assert_eq!(object.get("object").unwrap().unwrap().data_type(), DataType::Object);
    assert_binary(object.get("binary").unwrap().unwrap(), b"abc");

    assert_eq!(object.number("id").unwrap().unwrap(), Number::from(1));
    assert_eq!(object.string("name").unwrap().unwrap(), "abc");
    assert!(!object.bool("child").unwrap().unwrap());
    assert!(object.is_null("phone").unwrap().unwrap());
    assert_eq!(object.tinyint("tinyint").unwrap().unwrap(), 123_i8);
    assert_eq!(object.smallint("smallint").unwrap().unwrap(), 12345_i16);
    assert_eq!(object.integer("integer").unwrap().unwrap(), 1234567_i32);
    assert_eq!(object.bigint("bigint").unwrap().unwrap(), 12345678_i64);
    assert_eq!(object.float("float").unwrap().unwrap(), 123.456_f32);
    assert_eq!(object.double("double").unwrap().unwrap(), 12.3456789_f64);
    assert_eq!(object.array("array").unwrap().unwrap().len().unwrap(), 1);
    assert_eq!(object.object("object").unwrap().unwrap().len().unwrap(), 1);
    assert_eq!(object.binary("binary").unwrap().unwrap(), b"abc");

    assert!(object.bool("id").is_err());

    let value = object.get("invalid").unwrap();
    assert!(value.is_none());

    // tests iter
    for (id, item) in object.iter().unwrap().enumerate() {
        let (key, value) = item.unwrap();
        if id == 0 {
            assert_eq!(key, "id");
            assert_number(value, Number::from(1));
        } else if id == 1 {
            assert_eq!(key, "name");
            assert_string(value, "abc");
        } else if id == 2 {
            assert_eq!(key, "array");
            assert_eq!(value.data_type(), DataType::Array);
        } else if id == 3 {
            assert_eq!(key, "child");
            assert_bool(value, false);
        } else if id == 4 {
            assert_eq!(key, "float");
            assert_value_eq!(Float, value, 123.456_f32);
        } else if id == 5 {
            assert_eq!(key, "phone");
            assert_null(value);
        } else if id == 6 {
            assert_eq!(key, "bigint");
            assert_value_eq!(Bigint, value, 12345678_i64);
        } else if id == 7 {
            assert_eq!(key, "binary");
            assert_binary(value, b"abc");
        } else if id == 8 {
            assert_eq!(key, "double");
            assert_value_eq!(Double, value, 12.3456789_f64);
        } else if id == 9 {
            assert_eq!(key, "object");
            assert_eq!(value.data_type(), DataType::Object);
        } else if id == 10 {
            assert_eq!(key, "integer");
            assert_value_eq!(Integer, value, 1234567_i32);
        } else if id == 11 {
            assert_eq!(key, "tinyint");
            assert_value_eq!(Tinyint, value, 123_i8);
        } else if id == 12 {
            assert_eq!(key, "smallint");
            assert_value_eq!(Smallint, value, 12345_i16);
        } else {
            panic!();
        }
    }

    // tests key iter
    for (id, key) in object.key_iter().unwrap().enumerate() {
        let key = key.unwrap();
        if id == 0 {
            assert_eq!(key, "id");
        } else if id == 1 {
            assert_eq!(key, "name");
        } else if id == 2 {
            assert_eq!(key, "array");
        } else if id == 3 {
            assert_eq!(key, "child");
        } else if id == 4 {
            assert_eq!(key, "float");
        } else if id == 5 {
            assert_eq!(key, "phone");
        } else if id == 6 {
            assert_eq!(key, "bigint");
        } else if id == 7 {
            assert_eq!(key, "binary");
        } else if id == 8 {
            assert_eq!(key, "double");
        } else if id == 9 {
            assert_eq!(key, "object");
        } else if id == 10 {
            assert_eq!(key, "integer");
        } else if id == 11 {
            assert_eq!(key, "tinyint");
        } else if id == 12 {
            assert_eq!(key, "smallint");
        } else {
            panic!();
        }
    }

    // tests value iter
    for (id, value) in object.value_iter().unwrap().enumerate() {
        let value = value.unwrap();
        if id == 0 {
            assert_number(value, Number::from(1));
        } else if id == 1 {
            assert_string(value, "abc");
        } else if id == 2 {
            assert_eq!(value.data_type(), DataType::Array);
        } else if id == 3 {
            assert_bool(value, false);
        } else if id == 4 {
            assert_value_eq!(Float, value, 123.456_f32);
        } else if id == 5 {
            assert_null(value);
        } else if id == 6 {
            assert_value_eq!(Bigint, value, 12345678_i64);
        } else if id == 7 {
            assert_binary(value, b"abc");
        } else if id == 8 {
            assert_value_eq!(Double, value, 12.3456789_f64);
        } else if id == 9 {
            assert_eq!(value.data_type(), DataType::Object);
        } else if id == 10 {
            assert_value_eq!(Integer, value, 1234567_i32);
        } else if id == 11 {
            assert_value_eq!(Tinyint, value, 123_i8);
        } else if id == 12 {
            assert_value_eq!(Smallint, value, 12345_i16);
        } else {
            panic!();
        }
    }
}

fn create_yason() -> YasonBuf {
    let mut builder = ObjectBuilder::try_new(13, false).unwrap();
    builder.push_number("id", Number::from(1)).unwrap();
    builder.push_string("name", "abc").unwrap();
    builder.push_bool("child", false).unwrap();
    builder.push_null("phone").unwrap();
    builder.push_tinyint("tinyint", 123_i8).unwrap();
    builder.push_smallint("smallint", 12345_i16).unwrap();
    builder.push_integer("integer", 1234567_i32).unwrap();
    builder.push_bigint("bigint", 12345678_i64).unwrap();
    builder.push_float("float", 123.456_f32).unwrap();
    builder.push_double("double", 12.3456789_f64).unwrap();
    builder.push_binary("binary", b"abc").unwrap();

    let mut array_builder = builder.push_array("array", 1).unwrap();
    array_builder.push_bool(true).unwrap();
    array_builder.finish().unwrap();

    let mut object_builder = builder.push_object("object", 1, true).unwrap();
    object_builder.push_bool("key", true).unwrap();
    object_builder.finish().unwrap();

    builder.finish().unwrap()
}

fn create_yason_with_vec(bytes: &mut Vec<u8>) -> &Yason {
    let mut builder = ObjectRefBuilder::try_new(bytes, 13, false).unwrap();
    builder.push_number("id", Number::from(1)).unwrap();
    builder.push_string("name", "abc").unwrap();
    builder.push_bool("child", false).unwrap();
    builder.push_null("phone").unwrap();
    builder.push_tinyint("tinyint", 123_i8).unwrap();
    builder.push_smallint("smallint", 12345_i16).unwrap();
    builder.push_integer("integer", 1234567_i32).unwrap();
    builder.push_bigint("bigint", 12345678_i64).unwrap();
    builder.push_float("float", 123.456_f32).unwrap();
    builder.push_double("double", 12.3456789_f64).unwrap();
    builder.push_binary("binary", b"abc").unwrap();

    let mut array_builder = builder.push_array("array", 1).unwrap();
    array_builder.push_bool(true).unwrap();
    array_builder.finish().unwrap();

    let mut object_builder = builder.push_object("object", 1, true).unwrap();
    object_builder.push_bool("key", true).unwrap();
    object_builder.finish().unwrap();

    builder.finish().unwrap()
}

#[test]
fn test_object() {
    let yason = create_yason();
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason.as_ref());
}

#[test]
fn test_object_from_vec() {
    let mut bytes = Vec::with_capacity(128);
    let yason = create_yason_with_vec(&mut bytes);
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason)
}

#[test]
fn test_object_from_used_vec() {
    let mut bytes = Vec::with_capacity(128);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let yason = create_yason_with_vec(&mut bytes);
    assert_eq!(yason.data_type().unwrap(), DataType::Object);
    assert_object(yason)
}

#[test]
fn test_create_object_error() {
    let mut builder = ObjectBuilder::try_new(3, true).unwrap();
    builder.push_bool("key", true).unwrap();
    let res = builder.finish();
    assert!(res.is_err());

    let mut builder = ObjectBuilder::try_new(3, true).unwrap();
    let _ = builder.push_object("key", 3, true).unwrap();
    let res = builder.finish();
    assert!(res.is_err());
}

#[test]
fn test_object_finish_error() {
    let mut builder_level0 = ObjectBuilder::try_new(1, true).unwrap();
    let _ = builder_level0.push_object("key", 1, true).unwrap();
    let res = builder_level0.finish();
    assert!(matches!(res.err(), Some(BuildError::InnerUncompletedError)));

    let mut builder_level0 = ObjectBuilder::try_new(1, true).unwrap();
    let _ = builder_level0.push_object("key", 1, true).unwrap();
    let res = builder_level0.push_null("key");
    assert!(matches!(res.err(), Some(BuildError::InnerUncompletedError)));
}

#[test]
fn test_object_nested_depth() {
    fn assert_nested_depth(total_depth: usize, err: Option<BuildError>) {
        fn inner(
            builder: Result<&mut ObjectRefBuilder, BuildError>,
            cur_depth: usize,
            total_depth: usize,
        ) -> Option<BuildError> {
            if cur_depth < total_depth {
                let nested_builder = builder.unwrap().push_object("key", 1, true);
                return if cur_depth < 100 {
                    inner(Ok(&mut nested_builder.unwrap()), cur_depth + 1, total_depth)
                } else {
                    nested_builder.err()
                };
            }
            None
        }

        let mut bytes = vec![];
        let mut builder = ObjectRefBuilder::try_new(&mut bytes, 1, true).unwrap();
        let res = inner(Ok(&mut builder), 1, total_depth);

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
