//! Query by PathExpression tests

use yason::{DataType, PathExpression, QueriedValue, Value, YasonBuf, YasonError};

fn assert_eq(left: &Value, right: &Value) {
    assert_eq!(left.data_type(), right.data_type());

    match left.data_type() {
        DataType::Object => match (left, right) {
            (Value::Object(l_o), Value::Object(r_o)) => {
                for (l, r) in l_o.iter().unwrap().zip(r_o.iter().unwrap()) {
                    let (k_l, v_l) = l.unwrap();
                    let (k_r, v_r) = r.unwrap();
                    assert_eq!(k_l, k_r);
                    assert_eq(&v_l, &v_r);
                }
            }
            _ => unreachable!(),
        },
        DataType::Array => match (left, right) {
            (Value::Array(l_a), Value::Array(r_a)) => {
                for (l, r) in l_a.iter().unwrap().zip(r_a.iter().unwrap()) {
                    assert_eq(&l.unwrap(), &r.unwrap());
                }
            }
            _ => unreachable!(),
        },
        DataType::String => match (left, right) {
            (Value::String(l), Value::String(r)) => assert_eq!(l, r),
            _ => unreachable!(),
        },
        DataType::Number => match (left, right) {
            (Value::Number(l), Value::Number(r)) => assert_eq!(l, r),
            _ => unreachable!(),
        },
        DataType::Bool => match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => assert_eq!(l, r),
            _ => unreachable!(),
        },
        DataType::Null => {}
    }
}

fn assert_inner(input: &str, path: &str, expected: Option<&str>, with_wrapper: bool, to_yason: bool, error: bool) {
    let yason_buf = YasonBuf::parse(input).unwrap();
    let yason = yason_buf.as_ref();
    let path = str::parse::<PathExpression>(path).unwrap();

    let mut result_buf = vec![];
    let res = if to_yason {
        path.query(yason, with_wrapper, None, Some(&mut result_buf))
    } else {
        path.query(yason, with_wrapper, None, None)
    };

    if with_wrapper {
        let res = res.unwrap();
        if let Some(expected) = expected {
            let e_yason_buf = YasonBuf::parse(expected).unwrap();
            let e_yason = e_yason_buf.as_ref();
            let expected_value = Value::try_from(e_yason).unwrap();

            if to_yason {
                match res {
                    QueriedValue::Yason(yason) => {
                        let res_value = Value::try_from(yason).unwrap();
                        assert_eq(&res_value, &expected_value);
                    }
                    _ => unreachable!(),
                }
            } else {
                match (expected_value, res) {
                    (Value::Array(array), QueriedValue::Values(values)) => {
                        assert_eq!(array.len().unwrap(), values.len());
                        let iter = array.iter().unwrap();
                        for (id, value) in iter.enumerate() {
                            assert_eq(&value.unwrap(), &values[id]);
                        }
                    }
                    _ => unreachable!(),
                }
            }
        } else if to_yason {
            match res {
                QueriedValue::Yason(yason) => assert_eq!(yason.array().unwrap().len().unwrap(), 0),
                _ => unreachable!(),
            }
        } else {
            match res {
                QueriedValue::Values(values) => assert!(values.is_empty()),
                _ => unreachable!(),
            }
        };
    } else if error {
        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), YasonError::MultiValuesWithoutWrapper));
    } else {
        assert!(res.is_ok());
        let res = res.unwrap();
        if let Some(expected) = expected {
            let e_yason_buf = YasonBuf::parse(expected).unwrap();
            let e_yason = e_yason_buf.as_ref();
            let expected_value = Value::try_from(e_yason).unwrap();

            match res {
                QueriedValue::Value(value) => {
                    assert_eq(&value, &expected_value);
                }
                _ => unreachable!(),
            }
        } else {
            assert!(matches!(res, QueriedValue::None))
        }
    }
}

fn assert_query(input: &str, path: &str, expected: Option<&str>) {
    assert_inner(input, path, expected, false, false, false)
}

fn assert_query_error(input: &str, path: &str) {
    assert_inner(input, path, None, false, false, true)
}

fn assert_query_with_wrapper(input: &str, path: &str, expected: Option<&str>) {
    assert_inner(input, path, expected, true, false, false);
    assert_inner(input, path, expected, true, true, false);
}

#[test]
fn test_query() {
    let input = r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#;

    let path = r#"$"#;
    let expected = r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#;
    assert_query(input, path, Some(expected));

    let path = r#"$.key1"#;
    let expected = r#"123"#;
    assert_query(input, path, Some(expected));

    let path = r#"$.key4[0]"#;
    let expected = r#"456"#;
    assert_query(input, path, Some(expected));

    let path = r#"$.key4[10]"#;
    assert_query(input, path, None);

    let path = r#"$.key4[last]"#;
    let expected = r#"[10, false, null]"#;
    assert_query(input, path, Some(expected));

    let path = r#"$.key4[last - 3]"#;
    let expected = r#"false"#;
    assert_query(input, path, Some(expected));

    let path = r#"$.key4[last - 50]"#;
    assert_query(input, path, None);

    let path = r#"$.key4[1 to 1]"#;
    let expected = r#"false"#;
    assert_query(input, path, Some(expected));

    let path = r#"$..key6"#;
    let expected = r#"123"#;
    assert_query(input, path, Some(expected));

    let path = r#"$..key8"#;
    assert_query(input, path, None);

    let path = r#"$..key3.key6"#;
    let expected = r#"123"#;
    assert_query(input, path, Some(expected));

    let path = "$.key4[last - 20, last - 10]";
    assert_query(input, path, None);

    let path = "$[1]";
    assert_query(input, path, None);
}

#[test]
fn test_query_error() {
    let input = r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#;

    let path = "$.*";
    assert_query_error(input, path);

    let path = "$.key4[0, 1]";
    assert_query_error(input, path);

    let path = "$.key4[*]";
    assert_query_error(input, path);

    let path = "$.key4[0 to 5]";
    assert_query_error(input, path);

    let path = "$..key1";
    assert_query_error(input, path);

    let path = "$.key1.size()";
    assert_query_error(input, path);

    let path = "$.key1.count()";
    assert_query_error(input, path);

    let path = "$.key1.type()";
    assert_query_error(input, path);
}

#[test]
fn test_query_with_wrapper() {
    let input = r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#;

    let path = "$";
    let expected = r#"[{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$.key1"#;
    let expected = r#"[123]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$.key4[0]"#;
    let expected = r#"[456]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$[0].key4[0]"#;
    let expected = r#"[456]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$[0].key4[0][0][0]"#;
    let expected = r#"[456]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$[0].key4.key1"#;
    let expected = r#"[true]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$[0][0]"#;
    let expected = r#"[{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$[0].key5[0].key1"#;
    let expected = r#"[true]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$.key4[10]"#;
    assert_query_with_wrapper(input, path, None);

    let path = r#"$.key4[last - 10]"#;
    assert_query_with_wrapper(input, path, None);

    let path = r#"$.key4[last]"#;
    let expected = r#"[[10, false, null]]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$.key4[last - 3]"#;
    let expected = r#"[false]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$.key4[last - 50]"#;
    assert_query_with_wrapper(input, path, None);

    let path = r#"$.key4[1 to 1]"#;
    let expected = r#"[false]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$..key6"#;
    let expected = r#"[123]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = r#"$..key8"#;
    assert_query_with_wrapper(input, path, None);

    let path = r#"$..key3.key6"#;
    let expected = r#"[123]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key4[3].*";
    let expected = r#"[true, 789, {"key6": 123}]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key4[1, 0, 2]";
    let expected = r#"[false, 456, null]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key2..key6";
    assert_query_with_wrapper(input, path, None);

    let path = "$[*].key2";
    let expected = "[true]";
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$[1].key2";
    assert_query_with_wrapper(input, path, None);

    let path = "$.key4.key2";
    let expected = "[789]";
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key4[1, last, 0, 6 to 2]";
    let expected = r#"[false, [10, false, null], 456, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key4[last - 20, last - 10, 2 to 4, 0]";
    let expected = r#"[null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null], 456]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key4[last - 20, last - 10]";
    assert_query_with_wrapper(input, path, None);

    let path = "$.key4[last - 20, last - 10, 2 to 4, 0].size()";
    let expected = r#"[1, 1, 3, 1]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key4[last - 20, last - 10, 2 to 4, 0].count()";
    let expected = r#"[4]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let path = "$.key4[last - 20, last - 10, 2 to 4, 0].type()";
    let expected = r#"[6, 1, 2, 4]"#;
    assert_query_with_wrapper(input, path, Some(expected));

    let input = r#"[{"key": [{"key": [{"key": [{"key": 123}]}]}]}]"#;
    let path = r#"$.key.key.key.key"#;
    let expected = r#"[123]"#;
    assert_query_with_wrapper(input, path, Some(expected));
}

#[test]
fn test_exists_error() {
    fn assert(input: &str, path: &str) {
        let yason_buf = YasonBuf::parse(input).unwrap();
        let yason = yason_buf.as_ref();
        let path = str::parse::<PathExpression>(path).unwrap();

        let res = path.exists(yason);
        assert!(res.is_err());
    }

    let path = r#"$[0].type()"#;
    assert(r#"[1, 2]"#, path);

    let path = r#"$[0].size()"#;
    assert(r#"[1, 2]"#, path);

    let path = r#"$[0].count()"#;
    assert(r#"[1, 2]"#, path);
}

#[test]
fn test_exists() {
    fn assert(input: &str, path: &str, expected: bool) {
        let yason_buf = YasonBuf::parse(input).unwrap();
        let yason = yason_buf.as_ref();
        let path = str::parse::<PathExpression>(path).unwrap();

        let res = path.exists(yason).unwrap();
        assert_eq!(res, expected);
    }

    let input = r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#;

    let path = "$";
    assert(input, path, true);

    let path = "$.*";
    assert(input, path, true);

    let path = r#"$.key1"#;
    assert(input, path, true);

    let path = r#"$.key4[*]"#;
    assert(input, path, true);

    let path = r#"$.key4[0]"#;
    assert(input, path, true);

    let path = r#"$.key4[10]"#;
    assert(input, path, false);

    let path = r#"$.key4[last]"#;
    assert(input, path, true);

    let path = r#"$.key4[last - 1]"#;
    assert(input, path, true);

    let path = r#"$.key4[last - 100]"#;
    assert(input, path, false);

    let path = r#"$.key4[1 to 4]"#;
    assert(input, path, true);

    let path = r#"$.key4[50 to 100]"#;
    assert(input, path, false);

    let path = r#"$.key4[1, 0]"#;
    assert(input, path, true);

    let path = r#"$.key4[last - 100, 0]"#;
    assert(input, path, true);

    let path = "$.key4[1, last, 0, 6 to 2]";
    assert(input, path, true);

    let path = "$.key4[last - 20, last - 10, 2 to 4, 0]";
    assert(input, path, true);

    let path = "$.key4[last - 20, last - 10]";
    assert(input, path, false);

    let path = r#"$..key1"#;
    assert(input, path, true);

    let path = r#"$..key6"#;
    assert(input, path, true);

    let path = r#"$..key8"#;
    assert(input, path, false);

    let path = r#"$[0].key4[0]"#;
    assert(input, path, true);

    let path = r#"$[0].key4[0][0][0]"#;
    assert(input, path, true);

    let path = r#"$[0].key4.key1"#;
    assert(input, path, true);

    let path = r#"$[0][0]"#;
    assert(input, path, true);

    let path = r#"$[0].key5[0].key1"#;
    assert(input, path, true);

    let path = r#"$..key3.key6"#;
    assert(input, path, true);

    let path = "$.key4[3].*";
    assert(input, path, true);

    let path = "$.key4[1, 0, 2]";
    assert(input, path, true);

    let path = "$.key2..key6";
    assert(input, path, false);

    let path = "$[*].key2";
    assert(input, path, true);

    let path = "$[1].key2";
    assert(input, path, false);

    let path = "$.key4.key2";
    assert(input, path, true);

    let input = r#"[{"key": [{"key": [{"key": [{"key": 123}]}]}]}]"#;
    let path = r#"$.key.key.key.key"#;
    assert(input, path, true);
}
