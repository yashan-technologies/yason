//! YASON extended types

use crate::DataType;

pub const NUMBER_EXTENDED_NAME: &str = "$numberDecimal";
pub const TINYINT_EXTENDED_NAME: &str = "$numberByte";
pub const SMALLINT_EXTENDED_NAME: &str = "$numberShort";
pub const INTEGER_EXTENDED_NAME: &str = "$numberInt";
pub const BIGINT_EXTENDED_NAME: &str = "$numberLong";
pub const FLOAT_EXTENDED_NAME: &str = "$numberFloat";
pub const DOUBLE_EXTENDED_NAME: &str = "$numberDouble";

pub const MAX_SAFE_BIGINT: i64 = 9007199254740991; // 2^53 -1
pub const MIN_SAFE_BIGINT: i64 = -9007199254740991; // -(2^53-1)

// Note that this array should be ordered by types' name
pub const EXTENDED_NAME_TYPES: [(&str, DataType); 7] = [
    (TINYINT_EXTENDED_NAME, DataType::Tinyint),
    (NUMBER_EXTENDED_NAME, DataType::Number),
    (DOUBLE_EXTENDED_NAME, DataType::Double),
    (FLOAT_EXTENDED_NAME, DataType::Float),
    (INTEGER_EXTENDED_NAME, DataType::Integer),
    (BIGINT_EXTENDED_NAME, DataType::Bigint),
    (SMALLINT_EXTENDED_NAME, DataType::Smallint),
];
