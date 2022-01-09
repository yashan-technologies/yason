//! Data type.

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum DataType {
    Object = 1,
    Array = 2,
    String = 3,
    Number = 4,
    Bool = 5,
    Null = 6,
}

impl From<DataType> for u8 {
    #[inline]
    fn from(t: DataType) -> Self {
        t as u8
    }
}

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
            v => Err(InvalidDataType(v)),
        }
    }
}

impl Display for InvalidDataType {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "invalid data type value '{}'", self.0)
    }
}

impl Error for InvalidDataType {}
