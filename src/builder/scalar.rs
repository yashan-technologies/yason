//! Scalar builder.

use crate::binary::{BOOL_SIZE, DATA_TYPE_SIZE, MAX_DATA_LENGTH_SIZE, NUMBER_LENGTH_SIZE};
use crate::builder::BuildResult;
use crate::vec::VecExt;
use crate::yason::{Yason, YasonBuf};
use crate::{DataType, Number};
use decimal_rs::MAX_BINARY_SIZE;

/// Builder for encoding a scalar value.
#[derive(Debug)]
pub struct Scalar {}

impl Scalar {
    /// Encodes a string value.
    #[inline]
    pub fn string<T: AsRef<str>>(s: T) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::string_with_vec(s, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a string value into the provided vector.
    #[inline]
    pub fn string_with_vec<T: AsRef<str>>(s: T, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let s = s.as_ref();
        let size = DATA_TYPE_SIZE + MAX_DATA_LENGTH_SIZE + s.len();
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::String);
        bytes.push_string(s)?;
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a number value.
    #[inline]
    pub fn number(value: Number) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::number_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a number value into the provided vector.
    #[inline]
    pub fn number_with_vec(value: Number, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + NUMBER_LENGTH_SIZE + MAX_BINARY_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Number);
        bytes.push_number(value);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a bool value.
    #[inline]
    pub fn bool(value: bool) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::bool_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a bool value into the provided vector.
    #[inline]
    pub fn bool_with_vec(value: bool, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + BOOL_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Bool);
        bytes.push_u8(value as u8);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a null value.
    #[inline]
    pub fn null() -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::null_with_vec(&mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a null value into the provided vector.
    #[inline]
    pub fn null_with_vec(bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        bytes.try_reserve(DATA_TYPE_SIZE)?;
        bytes.push_data_type(DataType::Null);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }
}
