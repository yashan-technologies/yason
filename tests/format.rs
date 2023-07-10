//! Yason format tests

use yason::YasonBuf;

fn assert_fmt(input: &str, expected: &str, pretty: bool, extended_parse: bool, extended_fmt: bool) {
    let yason_buf = YasonBuf::parse(input, extended_parse).unwrap();
    let yason = yason_buf.as_ref();
    assert_eq!(format!("{}", yason.format(pretty, extended_fmt)), expected)
}

fn assert_compact_fmt(input: &str, expected: &str) {
    assert_fmt(input, expected, false, false, false)
}

fn assert_extended_compact_fmt(input: &str, expected: &str) {
    assert_fmt(input, expected, false, true, true)
}

fn assert_pretty_fmt(input: &str, expected: &str) {
    assert_fmt(input, expected, true, false, false)
}

fn assert_extended_pretty_fmt(input: &str, expected: &str) {
    assert_fmt(input, expected, true, true, true)
}

fn assert_scalar_fmt(input: &str, expected: &str) {
    assert_compact_fmt(input, expected);
    assert_pretty_fmt(input, expected);
}

fn assert_extended_scalar_fmt(input: &str, expected: &str, pretty_expected: &str) {
    assert_fmt(input, expected, false, true, true);
    assert_fmt(input, pretty_expected, true, true, true);
}

#[test]
fn test_scalar_bool_and_null_fmt() {
    assert_scalar_fmt("true", "true");
    assert_scalar_fmt("false", "false");
    assert_scalar_fmt("null", "null");
}

#[test]
fn test_scalar_string_fmt() {
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

#[test]
fn test_scalar_number_fmt() {
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

#[test]
fn test_scalar_ext_numeric_fmt() {
    assert_extended_scalar_fmt(r#"{"$numberByte": 127}"#, r#"127"#, r#"127"#);
    assert_extended_scalar_fmt(r#"{"$numberShort": 12345}"#, r#"12345"#, r#"12345"#);
    assert_extended_scalar_fmt(r#"{"$numberInt": 1234567}"#, r#"1234567"#, r#"1234567"#);
    assert_extended_scalar_fmt(
        r#"{"$numberLong": 9007199254740991}"#,
        r#"9007199254740991"#,
        r#"9007199254740991"#,
    );
    assert_extended_scalar_fmt(
        r#"{"$numberLong": 9007199254740992}"#,
        r#"{"$numberLong":"9007199254740992"}"#,
        "{\n  \"$numberLong\" : \"9007199254740992\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberFloat": 123.4567}"#,
        r#"{"$numberFloat":"123.456703"}"#,
        "{\n  \"$numberFloat\" : \"123.456703\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberFloat": 8.9345678e3}"#,
        r#"{"$numberFloat":"8934.56738"}"#,
        "{\n  \"$numberFloat\" : \"8934.56738\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberFloat": "naN"}"#,
        r#"{"$numberFloat":"Nan"}"#,
        "{\n  \"$numberFloat\" : \"Nan\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberFloat": "+inf"}"#,
        r#"{"$numberFloat":"Inf"}"#,
        "{\n  \"$numberFloat\" : \"Inf\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberFloat": "-infinity"}"#,
        r#"{"$numberFloat":"-Inf"}"#,
        "{\n  \"$numberFloat\" : \"-Inf\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberDouble": 12.3456789}"#,
        r#"12.345678899999999"#,
        r#"12.345678899999999"#,
    );
    assert_extended_scalar_fmt(
        r#"{"$numberDouble": "naN"}"#,
        r#"{"$numberDouble":"Nan"}"#,
        "{\n  \"$numberDouble\" : \"Nan\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberDouble": "+inf"}"#,
        r#"{"$numberDouble":"Inf"}"#,
        "{\n  \"$numberDouble\" : \"Inf\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberDouble": "-infinity"}"#,
        r#"{"$numberDouble":"-Inf"}"#,
        "{\n  \"$numberDouble\" : \"-Inf\"\n}",
    );
    assert_extended_scalar_fmt(r#"{"$numberDecimal": 127}"#, r#"127"#, r#"127"#);
    assert_extended_scalar_fmt(
        r#"{"$numberDecimal": 12.3456789}"#,
        r#"{"$numberDecimal":"12.3456789"}"#,
        "{\n  \"$numberDecimal\" : \"12.3456789\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$numberDecimal": 9007199254740991}"#,
        r#"9007199254740991"#,
        r#"9007199254740991"#,
    );
    assert_extended_scalar_fmt(
        r#"{"$numberDecimal": 9007199254740992}"#,
        r#"{"$numberLong":"9007199254740992"}"#,
        "{\n  \"$numberLong\" : \"9007199254740992\"\n}",
    );
}

#[test]
fn test_scalar_binary_fmt() {
    assert_extended_scalar_fmt(r#"{"$binary": ""}"#, r#"{"$binary":""}"#, "{\n  \"$binary\" : \"\"\n}");
    assert_extended_scalar_fmt(
        r#"{"$binary": "abc"}"#,
        r#"{"$binary":"abc="}"#,
        "{\n  \"$binary\" : \"abc=\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$binary": "测试"}"#,
        r#"{"$binary":"测试"}"#,
        "{\n  \"$binary\" : \"测试\"\n}",
    );
    assert_fmt(
        r#"{"$binary": "SmF2YVNjcmlwdA=="}"#,
        "\"4A617661536372697074\"",
        false,
        true,
        false,
    );

    // length = 4k + 1, invalid input
    assert_fmt(r#"{"$binary": " "}"#, r#"{"$binary":" "}"#, false, true, false);
    // length = 4k + 1, valid input
    assert_fmt(r#"{"$binary": "X"}"#, r#""""#, false, true, false);
}

#[test]
fn test_scalar_timestamp_fmt() {
    assert_extended_scalar_fmt(
        r#"{"$yashanTimestamp": "2023-05-25T16:50:20.123"}"#,
        r#"{"$yashanTimestamp":"2023-05-25T16:50:20.123000"}"#,
        "{\n  \"$yashanTimestamp\" : \"2023-05-25T16:50:20.123000\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$yashanTimestamp": "2023-05-25T16:50:20"}"#,
        r#"{"$yashanTimestamp":"2023-05-25T16:50:20"}"#,
        "{\n  \"$yashanTimestamp\" : \"2023-05-25T16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$yashanTimestamp": "2023-05-25T16:50:20.123Z"}"#,
        r#"{"$yashanTimestamp":"2023-05-25T16:50:20.123000"}"#,
        "{\n  \"$yashanTimestamp\" : \"2023-05-25T16:50:20.123000\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$yashanTimestamp": "2023-05-25T16:50:20Z"}"#,
        r#"{"$yashanTimestamp":"2023-05-25T16:50:20"}"#,
        "{\n  \"$yashanTimestamp\" : \"2023-05-25T16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleTimestamp": "2023-05-25T16:50:20.123"}"#,
        r#"{"$yashanTimestamp":"2023-05-25T16:50:20.123000"}"#,
        "{\n  \"$yashanTimestamp\" : \"2023-05-25T16:50:20.123000\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleTimestamp": "9999-12-31T23:59:59.999999"}"#,
        r#"{"$yashanTimestamp":"9999-12-31T23:59:59.999999"}"#,
        "{\n  \"$yashanTimestamp\" : \"9999-12-31T23:59:59.999999\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleTimestamp": "9999-12-31T23:59:59.999999999"}"#,
        // fail to parse as extended because overflow
        r#"{"$oracleTimestamp":"9999-12-31T23:59:59.999999999"}"#,
        "{\n  \"$oracleTimestamp\" : \"9999-12-31T23:59:59.999999999\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleTimestamp": "9999-12-31T23:59:59.9999999999"}"#,
        // fail to parse as extended because too long
        r#"{"$oracleTimestamp":"9999-12-31T23:59:59.9999999999"}"#,
        "{\n  \"$oracleTimestamp\" : \"9999-12-31T23:59:59.9999999999\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleTimestamp": "9999-12-30T23:59:59.999999"}"#,
        r#"{"$yashanTimestamp":"9999-12-30T23:59:59.999999"}"#,
        "{\n  \"$yashanTimestamp\" : \"9999-12-30T23:59:59.999999\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleTimestamp": "9999-12-30T23:59:59.999999999"}"#,
        // round up
        r#"{"$yashanTimestamp":"9999-12-31T00:00:00"}"#,
        "{\n  \"$yashanTimestamp\" : \"9999-12-31T00:00:00\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleTimestamp": "9999-12-30T23:59:59.9999999999"}"#,
        // fail to parse as extended because too long
        r#"{"$oracleTimestamp":"9999-12-30T23:59:59.9999999999"}"#,
        "{\n  \"$oracleTimestamp\" : \"9999-12-30T23:59:59.9999999999\"\n}",
    );
}

#[test]
fn test_scalar_date_fmt() {
    assert_extended_scalar_fmt(
        r#"{"$yashanDate": "2023-05-25T16:50:20"}"#,
        r#"{"$yashanDate":"2023-05-25T16:50:20"}"#,
        "{\n  \"$yashanDate\" : \"2023-05-25T16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$yashanDate": "2023-05-25T16:50:20.123"}"#,
        r#"{"$yashanDate":"2023-05-25T16:50:20"}"#,
        "{\n  \"$yashanDate\" : \"2023-05-25T16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$yashanDate": "2023-05-25T16:50:20Z"}"#,
        r#"{"$yashanDate":"2023-05-25T16:50:20"}"#,
        "{\n  \"$yashanDate\" : \"2023-05-25T16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$yashanDate": "2023-05-25T16:50:20.123Z"}"#,
        r#"{"$yashanDate":"2023-05-25T16:50:20"}"#,
        "{\n  \"$yashanDate\" : \"2023-05-25T16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleDate": "2023-05-25T16:50:20Z"}"#,
        r#"{"$yashanDate":"2023-05-25T16:50:20"}"#,
        "{\n  \"$yashanDate\" : \"2023-05-25T16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleDate": "9999-12-31T23:59:59.999999"}"#,
        r#"{"$yashanDate":"9999-12-31T23:59:59"}"#,
        "{\n  \"$yashanDate\" : \"9999-12-31T23:59:59\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleDate": "9999-12-31T23:59:59.999999999"}"#,
        // When below 9 digits, truncate
        r#"{"$yashanDate":"9999-12-31T23:59:59"}"#,
        "{\n  \"$yashanDate\" : \"9999-12-31T23:59:59\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$oracleDate": "9999-12-31T23:59:59.9999999999"}"#,
        // fail to parse as extended because too long
        r#"{"$oracleDate":"9999-12-31T23:59:59.9999999999"}"#,
        "{\n  \"$oracleDate\" : \"9999-12-31T23:59:59.9999999999\"\n}",
    );
}

#[test]
fn test_scalar_time_fmt() {
    assert_extended_scalar_fmt(
        r#"{"$yashanTime": "16:50:20"}"#,
        r#"{"$yashanTime":"16:50:20"}"#,
        "{\n  \"$yashanTime\" : \"16:50:20\"\n}",
    );
    assert_extended_scalar_fmt(
        r#"{"$yashanTime": "16:50:20.123"}"#,
        r#"{"$yashanTime":"16:50:20.123000"}"#,
        "{\n  \"$yashanTime\" : \"16:50:20.123000\"\n}",
    );
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
        assert_extended_compact_fmt(
            r#"{"$numberByte": "123", "$numberShort": "12345", "$numberInt": 1234567, "$numberLong": 1234567890, "$numberDecimal": "12.3456789", "$numberFloat": 123.456, "$numberDouble": 12.3456789}"#,
            r#"{"$numberInt":1234567,"$numberByte":"123","$numberLong":1234567890,"$numberFloat":123.456,"$numberShort":"12345","$numberDouble":12.345678899999999,"$numberDecimal":"12.3456789"}"#,
        );
        assert_extended_compact_fmt(
            r#"{ "tinyint": {"$numberByte": "123"}, "smallint": {"$numberShort": "12345"}, "integer":{"$numberInt": 1234567}, "bigint": {"$numberLong": 1234567890}, "number": {"$numberDecimal": "12.3456789"}, "float":{"$numberFloat": 123.456}, "double": {"$numberDouble": 12.3456789}}"#,
            r#"{"float":{"$numberFloat":"123.456001"},"bigint":1234567890,"double":12.345678899999999,"number":{"$numberDecimal":"12.3456789"},"integer":1234567,"tinyint":123,"smallint":12345}"#,
        );
        assert_extended_compact_fmt(
            r#"{ "bin2": {"$binary": "aGVsbG8h"}, "bin1": {"$binary": { "base64": "SmF2YVNjcmlwdA==", "subType": 0 }}}"#,
            r#"{"bin1":{"$binary":"SmF2YVNjcmlwdA=="},"bin2":{"$binary":"aGVsbG8h"}}"#,
        );
        assert_extended_compact_fmt(
            r#"{"ts": {"$yashanTimestamp": "2023-05-25T16:50:20.123Z"}, "date": {"$yashanDate": "2023-05-25"}, "time": {"$yashanTime": "16:50:20.123"}}"#,
            r#"{"ts":{"$yashanTimestamp":"2023-05-25T16:50:20.123000"},"date":{"$yashanDate":"2023-05-25T00:00:00"},"time":{"$yashanTime":"16:50:20.123000"}}"#,
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
        assert_extended_compact_fmt(
            r#"[{"$numberByte": "123"}, {"$numberShort": "12345"}, {"$numberInt": 1234567}, {"$numberLong": 1234567890}, {"$numberDecimal": "12.3456789"}, {"$numberFloat": 123.456}, {"$numberDouble": 12.3456789}]"#,
            r#"[123,12345,1234567,1234567890,{"$numberDecimal":"12.3456789"},{"$numberFloat":"123.456001"},12.345678899999999]"#,
        );
        assert_extended_compact_fmt(
            r#"[{"$binary": "aGVsbG8h"}, {"$binary": { "base64": "SmF2YVNjcmlwdA==", "subType": 0 }}]"#,
            r#"[{"$binary":"aGVsbG8h"},{"$binary":"SmF2YVNjcmlwdA=="}]"#,
        );
        assert_extended_compact_fmt(
            r#"[{"$yashanTimestamp": "2023-05-25T16:50:20.123Z"}, {"$yashanDate": "2023-05-25"}, {"$yashanTime": "16:50:20.123"}]"#,
            r#"[{"$yashanTimestamp":"2023-05-25T16:50:20.123000"},{"$yashanDate":"2023-05-25T00:00:00"},{"$yashanTime":"16:50:20.123000"}]"#,
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
        assert_extended_pretty_fmt(
            r#"{"$numberByte": "123", "$numberShort": "12345", "$numberInt": 1234567, "$numberLong": 1234567890, "$numberDecimal": "12.3456789", "$numberFloat": 123.456, "$numberDouble": 12.3456789}"#,
            "{\n  \"$numberInt\" : 1234567,\n  \"$numberByte\" : \"123\",\n  \"$numberLong\" : 1234567890,\n  \"$numberFloat\" : 123.456,\n  \"$numberShort\" : \"12345\",\n  \"$numberDouble\" : 12.345678899999999,\n  \"$numberDecimal\" : \"12.3456789\"\n}",
        );
        assert_extended_pretty_fmt(
            r#"{ "tinyint": {"$numberByte": "123"}, "smallint": {"$numberShort": "12345"}, "integer":{"$numberInt": 1234567}, "bigint": {"$numberLong": 1234567890}, "number": {"$numberDecimal": "12.3456789"}, "float":{"$numberFloat": 123.456}, "double": {"$numberDouble": 12.3456789}}"#,
            "{\n  \"float\" : {\n    \"$numberFloat\" : \"123.456001\"\n  },\n  \"bigint\" : 1234567890,\n  \"double\" : 12.345678899999999,\n  \"number\" : {\n    \"$numberDecimal\" : \"12.3456789\"\n  },\n  \"integer\" : 1234567,\n  \"tinyint\" : 123,\n  \"smallint\" : 12345\n}",
        );
        assert_extended_pretty_fmt(
            r#"{ "bin2": {"$binary": "aGVsbG8h"}, "bin1": {"$binary": { "base64": "SmF2YVNjcmlwdA==", "subType": 0 }}}"#,
            "{\n  \"bin1\" : {\n    \"$binary\" : \"SmF2YVNjcmlwdA==\"\n  },\n  \"bin2\" : {\n    \"$binary\" : \"aGVsbG8h\"\n  }\n}",
        );
        assert_extended_pretty_fmt(
            r#"{"ts": {"$yashanTimestamp": "2023-05-25T16:50:20.123Z"}, "date": {"$yashanDate": "2023-05-25"}, "time": {"$yashanTime": "16:50:20.123"}}"#,
            "{\n  \"ts\" : {\n    \"$yashanTimestamp\" : \"2023-05-25T16:50:20.123000\"\n  },\n  \"date\" : {\n    \"$yashanDate\" : \"2023-05-25T00:00:00\"\n  },\n  \"time\" : {\n    \"$yashanTime\" : \"16:50:20.123000\"\n  }\n}",
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
        assert_extended_pretty_fmt(
            r#"[{"$numberByte": "123"}, {"$numberShort": "12345"}, {"$numberInt": 1234567}, {"$numberLong": 1234567890}, {"$numberDecimal": "12.3456789"}, {"$numberFloat": 123.456}, {"$numberDouble": 12.3456789}]"#,
            "[\n  123,\n  12345,\n  1234567,\n  1234567890,\n  {\n    \"$numberDecimal\" : \"12.3456789\"\n  },\n  {\n    \"$numberFloat\" : \"123.456001\"\n  },\n  12.345678899999999\n]",
        );
        assert_extended_pretty_fmt(
            r#"[{"$binary": "aGVsbG8h"}, {"$binary": { "base64": "SmF2YVNjcmlwdA==", "subType": 0 }}]"#,
            "[\n  {\n    \"$binary\" : \"aGVsbG8h\"\n  },\n  {\n    \"$binary\" : \"SmF2YVNjcmlwdA==\"\n  }\n]",
        );
        assert_extended_pretty_fmt(
            r#"[{"$yashanTimestamp": "2023-05-25T16:50:20.123Z"}, {"$yashanDate": "2023-05-25"}, {"$yashanTime": "16:50:20.123"}]"#,
            "[\n  {\n    \"$yashanTimestamp\" : \"2023-05-25T16:50:20.123000\"\n  },\n  {\n    \"$yashanDate\" : \"2023-05-25T00:00:00\"\n  },\n  {\n    \"$yashanTime\" : \"16:50:20.123000\"\n  }\n]",
        );
    }
}
