//! Data type.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Possible yason types.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum DataType {
    Object = 1,
    Array = 2,
    String = 3,
    Number = 4,
    Bool = 5,
    Null = 6,
    Tinyint = 7,
    Smallint = 8,
    Integer = 9,
    Bigint = 10,
    Float = 11,
    Double = 12,
    Binary = 13,
}

const DATA_TYPE_NAME: [&str; 14] = [
    "invalid", "object", "array", "string", "number", "boolean", "null", "tinyint", "smallint", "integer", "bigint",
    "float", "double", "binary",
];

impl DataType {
    #[inline]
    pub const fn name(self) -> &'static str {
        DATA_TYPE_NAME[self as usize]
    }
}

impl From<DataType> for u8 {
    #[inline]
    fn from(t: DataType) -> Self {
        t as u8
    }
}

impl Display for DataType {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            DataType::Object => write!(f, "Object"),
            DataType::Array => write!(f, "Array"),
            DataType::String => write!(f, "String"),
            DataType::Number => write!(f, "Number"),
            DataType::Bool => write!(f, "Bool"),
            DataType::Null => write!(f, "Null"),
            DataType::Tinyint => write!(f, "Tinyint"),
            DataType::Smallint => write!(f, "Smallint"),
            DataType::Integer => write!(f, "Integer"),
            DataType::Bigint => write!(f, "Bigint"),
            DataType::Float => write!(f, "Float"),
            DataType::Double => write!(f, "Double"),
            DataType::Binary => write!(f, "Binary"),
        }
    }
}

/// Invalid data type.
#[derive(Debug)]
#[repr(transparent)]
pub struct InvalidDataType(u8);

impl TryFrom<u8> for DataType {
    type Error = InvalidDataType;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(DataType::Object),
            2 => Ok(DataType::Array),
            3 => Ok(DataType::String),
            4 => Ok(DataType::Number),
            5 => Ok(DataType::Bool),
            6 => Ok(DataType::Null),
            7 => Ok(DataType::Tinyint),
            8 => Ok(DataType::Smallint),
            9 => Ok(DataType::Integer),
            10 => Ok(DataType::Bigint),
            11 => Ok(DataType::Float),
            12 => Ok(DataType::Double),
            13 => Ok(DataType::Binary),
            v => Err(InvalidDataType(v)),
        }
    }
}

impl Display for InvalidDataType {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "invalid data type yason '{}'", self.0)
    }
}

impl Error for InvalidDataType {}
