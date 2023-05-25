//! Scalar builder.

use crate::binary::{
    BIGINT_SIZE, BOOL_SIZE, DATA_TYPE_SIZE, DOUBLE_SIZE, FLOAT_SIZE, INTEGER_SIZE, MAX_DATA_LENGTH_SIZE,
    NUMBER_LENGTH_SIZE, SMALLINT_SIZE, TINYINT_SIZE,
};
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

    /// Encodes a binary value.
    #[inline]
    pub fn binary<T: AsRef<[u8]>>(s: T) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::binary_with_vec(s, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a binary value into the provided vector.
    #[inline]
    pub fn binary_with_vec<T: AsRef<[u8]>>(s: T, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let s = s.as_ref();
        let size = DATA_TYPE_SIZE + MAX_DATA_LENGTH_SIZE + s.len();
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Binary);
        bytes.push_binary(s)?;
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a number value.
    #[inline]
    pub fn number<Num: AsRef<Number>>(value: Num) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::number_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a number value into the provided vector.
    #[inline]
    pub fn number_with_vec<Num: AsRef<Number>>(value: Num, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + NUMBER_LENGTH_SIZE + MAX_BINARY_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Number);
        bytes.push_number(value.as_ref());
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

    /// Encodes a tinyint value.
    #[inline]
    pub fn tinyint(value: i8) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::tinyint_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a tinyint value into the provided vector.
    #[inline]
    pub fn tinyint_with_vec(value: i8, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + TINYINT_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Tinyint);
        bytes.push_i8(value);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a smallint value.
    #[inline]
    pub fn smallint(value: i16) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::smallint_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a smallint value into the provided vector.
    #[inline]
    pub fn smallint_with_vec(value: i16, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + SMALLINT_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Smallint);
        bytes.push_i16(value);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a integer value.
    #[inline]
    pub fn integer(value: i32) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::integer_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a integer value into the provided vector.
    #[inline]
    pub fn integer_with_vec(value: i32, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + INTEGER_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Integer);
        bytes.push_i32(value);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a bigint value.
    #[inline]
    pub fn bigint(value: i64) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::bigint_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a bigint value into the provided vector.
    #[inline]
    pub fn bigint_with_vec(value: i64, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + BIGINT_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Bigint);
        bytes.push_i64(value);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a float value.
    #[inline]
    pub fn float(value: f32) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::float_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a float value into the provided vector.
    #[inline]
    pub fn float_with_vec(value: f32, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + FLOAT_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Float);
        bytes.push_f32(value);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }

    /// Encodes a double value.
    #[inline]
    pub fn double(value: f64) -> BuildResult<YasonBuf> {
        let mut bytes = Vec::new();
        Scalar::double_with_vec(value, &mut bytes)?;
        Ok(unsafe { YasonBuf::new_unchecked(bytes) })
    }

    /// Encodes a double value into the provided vector.
    #[inline]
    pub fn double_with_vec(value: f64, bytes: &mut Vec<u8>) -> BuildResult<&Yason> {
        let init_len = bytes.len();
        let size = DATA_TYPE_SIZE + DOUBLE_SIZE;
        bytes.try_reserve(size)?;
        bytes.push_data_type(DataType::Double);
        bytes.push_f64(value);
        Ok(unsafe { Yason::new_unchecked(&bytes[init_len..]) })
    }
}
