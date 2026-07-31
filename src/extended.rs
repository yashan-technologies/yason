//! YASON extended types

use crate::DataType;
use sqldatetime::{Formatter, OracleDate, Time, Timestamp};

pub const EXTENDED_NAME_PREFIX: u8 = b'$';
pub const NUMBER_EXTENDED_NAME: &str = "$numberDecimal";
pub const TINYINT_EXTENDED_NAME: &str = "$numberByte";
pub const SMALLINT_EXTENDED_NAME: &str = "$numberShort";
pub const INTEGER_EXTENDED_NAME: &str = "$numberInt";
pub const BIGINT_EXTENDED_NAME: &str = "$numberLong";
pub const FLOAT_EXTENDED_NAME: &str = "$numberFloat";
pub const DOUBLE_EXTENDED_NAME: &str = "$numberDouble";
pub const BINARY_EXTENDED_NAME: &str = "$binary";
pub const BINARY_BASE64_NAME: &str = "base64";
pub const BINARY_SUBTYPE_NAME: &str = "subType";
pub const TIMESTAMP_EXTENDED_NAME: &str = "$yashanTimestamp";
pub const ORACLE_TIMESTAMP_EXTENDED_NAME: &str = "$oracleTimestamp";
pub const DATE_EXTENDED_NAME: &str = "$yashanDate";
pub const ORACLE_DATE_EXTENDED_NAME: &str = "$oracleDate";
pub const TIME_EXTENDED_NAME: &str = "$yashanTime";

pub const MAX_SAFE_BIGINT: i64 = 9007199254740991; // 2^53 -1
pub const MIN_SAFE_BIGINT: i64 = -9007199254740991; // -(2^53-1)

// Note that this array should be ordered by types' name
pub const EXTENDED_NAME_TYPES: [(&str, DataType); 13] = [
    (BINARY_EXTENDED_NAME, DataType::Binary),
    (TINYINT_EXTENDED_NAME, DataType::Tinyint),
    (NUMBER_EXTENDED_NAME, DataType::Number),
    (DOUBLE_EXTENDED_NAME, DataType::Double),
    (FLOAT_EXTENDED_NAME, DataType::Float),
    (INTEGER_EXTENDED_NAME, DataType::Integer),
    (BIGINT_EXTENDED_NAME, DataType::Bigint),
    (SMALLINT_EXTENDED_NAME, DataType::Smallint),
    (ORACLE_DATE_EXTENDED_NAME, DataType::Date),
    (ORACLE_TIMESTAMP_EXTENDED_NAME, DataType::Timestamp),
    (DATE_EXTENDED_NAME, DataType::Date),
    (TIME_EXTENDED_NAME, DataType::Time),
    (TIMESTAMP_EXTENDED_NAME, DataType::Timestamp),
];

#[inline]
pub fn timestamp_formatter() -> &'static Formatter {
    Timestamp::iso8601_formatter()
}

#[inline]
pub fn date_formatter() -> &'static Formatter {
    OracleDate::iso8601_formatter()
}

#[inline]
pub fn time_formatter() -> &'static Formatter {
    Time::iso8601_formatter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_name_types() {
        fn assert_name_type(name: &str, expected_type: DataType) {
            if let Ok(index) = EXTENDED_NAME_TYPES.binary_search_by(|entry| entry.0.cmp(name)) {
                assert_eq!(EXTENDED_NAME_TYPES[index].1, expected_type);
            } else {
                panic!("failed to find {name}");
            }
        }

        assert_name_type(NUMBER_EXTENDED_NAME, DataType::Number);
        assert_name_type(TINYINT_EXTENDED_NAME, DataType::Tinyint);
        assert_name_type(SMALLINT_EXTENDED_NAME, DataType::Smallint);
        assert_name_type(INTEGER_EXTENDED_NAME, DataType::Integer);
        assert_name_type(BIGINT_EXTENDED_NAME, DataType::Bigint);
        assert_name_type(FLOAT_EXTENDED_NAME, DataType::Float);
        assert_name_type(DOUBLE_EXTENDED_NAME, DataType::Double);
        assert_name_type(BINARY_EXTENDED_NAME, DataType::Binary);
        assert_name_type(TIMESTAMP_EXTENDED_NAME, DataType::Timestamp);
        assert_name_type(DATE_EXTENDED_NAME, DataType::Date);
        assert_name_type(TIME_EXTENDED_NAME, DataType::Time);
        assert_name_type(ORACLE_DATE_EXTENDED_NAME, DataType::Date);
        assert_name_type(ORACLE_TIMESTAMP_EXTENDED_NAME, DataType::Timestamp);
    }
}
