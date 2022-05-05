//! Yason format tests

use yason::YasonBuf;

fn assert_fmt(input: &str, expected: &str, pretty: bool) {
    let yason_buf = YasonBuf::parse(input).unwrap();
    let yason = yason_buf.as_ref();
    assert_eq!(format!("{}", yason.format(pretty)), expected)
}

fn assert_compact_fmt(input: &str, expected: &str) {
    assert_fmt(input, expected, false)
}

fn assert_pretty_fmt(input: &str, expected: &str) {
    assert_fmt(input, expected, true)
}

fn assert_scalar_fmt(input: &str, expected: &str) {
    assert_compact_fmt(input, expected);
    assert_pretty_fmt(input, expected);
}

#[test]
fn test_scalar_fmt() {
    // bool && null
    {
        assert_scalar_fmt("true", "true");
        assert_scalar_fmt("false", "false");
        assert_scalar_fmt("null", "null");
    }

    // string
    {
        assert_scalar_fmt(r#""""#, r#""""#);
        assert_scalar_fmt(r#""abc""#, r#""abc""#);
        assert_scalar_fmt(r#""测试""#, r#""测试""#);
        assert_scalar_fmt(r#""ab\bc""#, r#""ab\bc""#);
        assert_scalar_fmt(r#""ab\fc""#, r#""ab\fc""#);
        assert_scalar_fmt(r#""ab\nc""#, r#""ab\nc""#);
        assert_scalar_fmt(r#""ab\rc""#, r#""ab\rc""#);
        assert_scalar_fmt(r#""ab\tc""#, r#""ab\tc""#);
        assert_scalar_fmt(r#""ab\"c""#, r#""ab\"c""#);
        assert_scalar_fmt(r#""ab\\c""#, r#""ab\\c""#);
        assert_scalar_fmt(r#""ab\/tc""#, r#""ab/tc""#);

        assert_scalar_fmt(r#""\u0000""#, r#""\u0000""#);
        assert_scalar_fmt(r#""\u0001""#, r#""\u0001""#);
        assert_scalar_fmt(r#""\u0002""#, r#""\u0002""#);
        assert_scalar_fmt(r#""\u0003""#, r#""\u0003""#);
        assert_scalar_fmt(r#""\u0004""#, r#""\u0004""#);
        assert_scalar_fmt(r#""\u0005""#, r#""\u0005""#);
        assert_scalar_fmt(r#""\u0006""#, r#""\u0006""#);
        assert_scalar_fmt(r#""\u0007""#, r#""\u0007""#);
        assert_scalar_fmt(r#""\u0008""#, r#""\b""#);
        assert_scalar_fmt(r#""\u0009""#, r#""\t""#);
        assert_scalar_fmt(r#""\u000A""#, r#""\n""#);
        assert_scalar_fmt(r#""\u000B""#, r#""\u000B""#);
        assert_scalar_fmt(r#""\u000C""#, r#""\f""#);
        assert_scalar_fmt(r#""\u000D""#, r#""\r""#);
        assert_scalar_fmt(r#""\u000E""#, r#""\u000E""#);
        assert_scalar_fmt(r#""\u000F""#, r#""\u000F""#);
        assert_scalar_fmt(r#""\u0010""#, r#""\u0010""#);
        assert_scalar_fmt(r#""\u0011""#, r#""\u0011""#);
        assert_scalar_fmt(r#""\u0012""#, r#""\u0012""#);
        assert_scalar_fmt(r#""\u0013""#, r#""\u0013""#);
        assert_scalar_fmt(r#""\u0014""#, r#""\u0014""#);
        assert_scalar_fmt(r#""\u0015""#, r#""\u0015""#);
        assert_scalar_fmt(r#""\u0016""#, r#""\u0016""#);
        assert_scalar_fmt(r#""\u0017""#, r#""\u0017""#);
        assert_scalar_fmt(r#""\u0018""#, r#""\u0018""#);
        assert_scalar_fmt(r#""\u0019""#, r#""\u0019""#);
        assert_scalar_fmt(r#""\u001A""#, r#""\u001A""#);
        assert_scalar_fmt(r#""\u001B""#, r#""\u001B""#);
        assert_scalar_fmt(r#""\u001C""#, r#""\u001C""#);
        assert_scalar_fmt(r#""\u001D""#, r#""\u001D""#);
        assert_scalar_fmt(r#""\u001E""#, r#""\u001E""#);
        assert_scalar_fmt(r#""\u001F""#, r#""\u001F""#);

        assert_scalar_fmt(r#""\u0022""#, r#""\"""#);
        assert_scalar_fmt(r#""\u002F""#, r#""/""#);
        assert_scalar_fmt(r#""\u005c""#, r#""\\""#);
        assert_scalar_fmt(r#""\u007F""#, r#""\u007F""#);
        assert_scalar_fmt(r#""\u007f""#, r#""\u007F""#);
    }

    // number
    {
        assert_scalar_fmt("123", "123");
        assert_scalar_fmt("12340", "12340");
        assert_scalar_fmt("123.123", "123.123");
        assert_scalar_fmt("-123", "-123");
        assert_scalar_fmt("-12300000", "-12300000");
        assert_scalar_fmt("1234567890.123456789", "1234567890.123456789");
        assert_scalar_fmt("12300e35", "1230000000000000000000000000000000000000");
        assert_scalar_fmt("12300e36", "1.23E+40");
        assert_scalar_fmt("123e37", "1230000000000000000000000000000000000000");
        assert_scalar_fmt("123e38", "1.23E+40");
        assert_scalar_fmt("-12300e35", "-1230000000000000000000000000000000000000");
        assert_scalar_fmt("-12300e36", "-1.23E+40");
        assert_scalar_fmt("-123e37", "-1230000000000000000000000000000000000000");
        assert_scalar_fmt("-123e38", "-1.23E+40");
        assert_scalar_fmt("123e-41", "1.23E-39");
        assert_scalar_fmt("123e-40", "0.0000000000000000000000000000000000000123");
        assert_scalar_fmt("12300e-43", "1.23E-39");
        assert_scalar_fmt("12300e-42", "0.0000000000000000000000000000000000000123");
        assert_scalar_fmt("-123e-41", "-1.23E-39");
        assert_scalar_fmt("-123e-40", "-0.0000000000000000000000000000000000000123");
        assert_scalar_fmt("-12300e-43", "-1.23E-39");
        assert_scalar_fmt("-12300e-42", "-0.0000000000000000000000000000000000000123");
        assert_scalar_fmt(
            "1234567890123456789012345678901234567800e-42",
            "0.0012345678901234567890123456789012345678",
        );
        assert_scalar_fmt(
            "1234567890123456789012345678901234567800e-43",
            "1.2345678901234567890123456789012345678E-4",
        );
        assert_scalar_fmt(
            "12345678901234567.890123456789012345678e23",
            "1234567890123456789012345678901234567800",
        );
        assert_scalar_fmt(
            "12345678901234567.890123456789012345678e24",
            "1.2345678901234567890123456789012345678E+40",
        );
        assert_scalar_fmt(
            "12345678901234567.890123456789012345678e-19",
            "0.0012345678901234567890123456789012345678",
        );
        assert_scalar_fmt(
            "12345678901234567.890123456789012345678e-21",
            "1.2345678901234567890123456789012345678E-5",
        );
        assert_scalar_fmt(
            "0.00000000012345678901234567890123456789012345678e-1",
            "1.2345678901234567890123456789012345678E-11",
        );
        assert_scalar_fmt(
            "0.00000000012345678901234567890123456789012345678e6",
            "1.2345678901234567890123456789012345678E-4",
        );
        assert_scalar_fmt(
            "0.00000000012345678901234567890123456789012345678e7",
            "0.0012345678901234567890123456789012345678",
        );
        assert_scalar_fmt(
            "0.00000000012345678901234567890123456789012345678e47",
            "12345678901234567890123456789012345678",
        );
        assert_scalar_fmt(
            "0.00000000012345678901234567890123456789012345678e49",
            "1234567890123456789012345678901234567800",
        );
        assert_scalar_fmt(
            "0.00000000012345678901234567890123456789012345678e50",
            "1.2345678901234567890123456789012345678E+40",
        );
    }
}

#[test]
fn test_compact_fmt() {
    // object
    {
        assert_compact_fmt(r#"{}"#, r#"{}"#);
        assert_compact_fmt(
            r#"{"key1": 123, "key2": "string", "key3": true, "key4": null}"#,
            r#"{"key1":123,"key2":"string","key3":true,"key4":null}"#,
        );
        assert_compact_fmt(r#"{"key1": {"key1": 123}}"#, r#"{"key1":{"key1":123}}"#);
        assert_compact_fmt(
            r#"{"key1": [123, false, null, "string"]}"#,
            r#"{"key1":[123,false,null,"string"]}"#,
        );
        assert_compact_fmt(
            r#"{"key1": false, "key2": "abc", "key3": 456, "key4": null, "key5": {"key1": 789}, "key6": ["asd"]}"#,
            r#"{"key1":false,"key2":"abc","key3":456,"key4":null,"key5":{"key1":789},"key6":["asd"]}"#,
        );
    }

    // array
    {
        assert_compact_fmt(r#"[]"#, r#"[]"#);
        assert_compact_fmt(r#"[123, "string", false, null]"#, r#"[123,"string",false,null]"#);
        assert_compact_fmt(
            r#"[{"key1": "abc", "key2": true}, {"key": "string"}]"#,
            r#"[{"key1":"abc","key2":true},{"key":"string"}]"#,
        );
        assert_compact_fmt(r#"[[123, true], [null, "dsf"]]"#, r#"[[123,true],[null,"dsf"]]"#);
        assert_compact_fmt(
            r#"[789, null, "rty", false, [901, true, null, "ghh"], {"key1": true, "key2": 1e23}]"#,
            r#"[789,null,"rty",false,[901,true,null,"ghh"],{"key1":true,"key2":100000000000000000000000}]"#,
        );
    }
}

#[test]
fn test_pretty_fmt() {
    // object
    {
        assert_pretty_fmt(r#"{}"#, "{\n}");
        assert_pretty_fmt(
            r#"{"key1": 123, "key2": "string", "key3": true, "key4": null}"#,
            "{\n  \"key1\" : 123,\n  \"key2\" : \"string\",\n  \"key3\" : true,\n  \"key4\" : null\n}",
        );

        assert_pretty_fmt(
            r#"{"key1": {"key1": 123}}"#,
            "{\n  \"key1\" : \n  {\n    \"key1\" : 123\n  }\n}",
        );
        assert_pretty_fmt(
            r#"{"key1": [123, false, null, "string"]}"#,
            "{\n  \"key1\" : \n  [\n    123,\n    false,\n    null,\n    \"string\"\n  ]\n}",
        );
        assert_pretty_fmt(
            r#"{"key1": false, "key2": "abc", "key3": 456, "key4": null, "key5": {"key1": 789}, "key6": ["asd"]}"#,
            "{\n  \"key1\" : false,\n  \"key2\" : \"abc\",\n  \"key3\" : 456,\n  \"key4\" : null,\n  \"key5\" : \n  {\n    \"key1\" : 789\n  },\n  \"key6\" : \n  [\n    \"asd\"\n  ]\n}",
        );
    }

    // array
    {
        assert_pretty_fmt(r#"[]"#, "[\n]");
        assert_pretty_fmt(
            r#"[123, "string", false, null]"#,
            "[\n  123,\n  \"string\",\n  false,\n  null\n]",
        );
        assert_pretty_fmt(
            r#"[{"key1": "abc", "key2": true}, {"key": "string"}]"#,
            "[\n  {\n    \"key1\" : \"abc\",\n    \"key2\" : true\n  },\n  {\n    \"key\" : \"string\"\n  }\n]",
        );
        assert_pretty_fmt(
            r#"[[123, true], [null, "dsf"]]"#,
            "[\n  [\n    123,\n    true\n  ],\n  [\n    null,\n    \"dsf\"\n  ]\n]",
        );
        assert_pretty_fmt(
            r#"[789, null, "rty", false, [901, true, null, "ghh"], {"key1": true, "key2": 1e23}]"#,
            "[\n  789,\n  null,\n  \"rty\",\n  false,\n  [\n    901,\n    true,\n    null,\n    \"ghh\"\n  ],\n  {\n    \"key1\" : true,\n    \"key2\" : 100000000000000000000000\n  }\n]",
        );
    }
}
