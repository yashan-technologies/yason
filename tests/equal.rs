//! Yason cmp tests

use yason::YasonBuf;

fn assert_equal_inner(left: &str, right: &str, extended: bool, expected: bool) {
    let left = YasonBuf::parse(left, extended).unwrap();
    let right = YasonBuf::parse(right, extended).unwrap();

    let res = left == right;
    assert_eq!(res, expected);
}

fn assert_equal_standard(left: &str, right: &str, expected: bool) {
    assert_equal_inner(left, right, false, expected);
}

fn assert_equal_extended(left: &str, right: &str, expected: bool) {
    assert_equal_inner(left, right, true, expected);
}

fn assert_equal(left: &str, right: &str, expected: bool) {
    assert_equal_standard(left, right, expected);
    assert_equal_extended(left, right, expected);
}

#[test]
fn test_yason_equal() {
    assert_equal(r#"null"#, r#"null"#, true);
    assert_equal(r#"false"#, r#"false"#, true);
    assert_equal(r#"true"#, r#"true"#, true);
    assert_equal(r#"true"#, r#"false"#, false);
    assert_equal(r#"true"#, r#"null"#, false);
    assert_equal(r#"false"#, r#"null"#, false);
    assert_equal(r#""abc""#, r#""abc""#, true);
    assert_equal(r#""abc""#, r#""def""#, false);
    assert_equal(r#"123"#, r#"123"#, true);
    assert_equal(r#"123"#, r#"456"#, false);
    assert_equal(r#"{"key": 123}"#, r#"{"key": 123}"#, true);
    assert_equal(r#"{"key": 123}"#, r#"{"key": 456}"#, false);
    assert_equal(r#"[123]"#, r#"[123]"#, true);
    assert_equal(r#"[123]"#, r#"[456]"#, false);
    assert_equal(
        r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#,
        r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#,
        true,
    );
    assert_equal(
        r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, true, null, {"key1": true, "key2": 789, "key3": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#,
        r#"{"key1": 123, "key2": true, "key3": null, "key4": [456, false, null, {"key1": true, "key2": 789, "key34": {"key6": 123}}, [10, false, null]], "key5": {"key1": true, "key2": 789, "key3": null}}"#,
        false,
    );
    assert_equal_extended(
        r#"{
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
        }"#,
        r#"{
            "tinyint": {"$numberByte": 123},
            "smallint": {"$numberShort": 12345},
            "integer": {"$numberInt": 123456},
            "bigint": {"$numberLong": 123456789},
            "number": {"$numberDecimal": "123.456789"},
            "float": {"$numberFloat": "123.456"},
            "double": {"$numberDouble": "12.3456789"},
            "binary": {"$binary": "aGVsbG8h"},
            "ts": {"$yashanTimestamp": "9999-12-31T23:59:59.999999Z"},
            "date": {"$yashanDate": "9999-12-31T23:59:59"},
            "time": {"$yashanTime": "23:59:59.999999"}
        }"#,
        true,
    );
}
