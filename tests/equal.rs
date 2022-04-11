//! Yason cmp tests

use yason::YasonBuf;

fn assert_equal(left: &str, right: &str, expected: bool) {
    let left = YasonBuf::parse(left).unwrap();
    let right = YasonBuf::parse(right).unwrap();

    let res = left == right;
    assert_eq!(res, expected);
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
    )
}
