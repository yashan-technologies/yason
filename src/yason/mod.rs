//! Yason manipulation.

mod array;
mod object;

pub use crate::yason::array::{Array, ArrayIter};
pub use crate::yason::object::{KeyIter, Object, ObjectIter, ValueIter};

use crate::binary::{DATA_TYPE_SIZE, NUMBER_LENGTH_SIZE};
use crate::format::LazyFormat;
use crate::util::decode_varint;
use crate::{BuildError, DataType, Number, Scalar};
use std::borrow::Borrow;
use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::mem::size_of;
use std::ops::Deref;

/// Possible errors that can arise during accessing.
#[derive(Debug)]
pub enum YasonError {
    IndexOutOfBounds { len: usize, index: usize },
    UnexpectedType { expected: DataType, actual: DataType },
    InvalidDataType(u8),
    MultiValuesWithoutWrapper,
    TryReserveError(TryReserveError),
    InvalidPathExpression,
}

impl fmt::Display for YasonError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YasonError::IndexOutOfBounds { len, index } => {
                write!(f, "index out of bounds: the len is {} but the index is {}", len, index)
            }
            YasonError::UnexpectedType { expected, actual } => {
                write!(f, "data type mismatch, expect {}, but actual {}", expected, actual)
            }
            YasonError::InvalidDataType(e) => write!(f, "invalid data type value '{}'", e),
            YasonError::MultiValuesWithoutWrapper => {
                write!(f, "multiple values cannot be returned without array wrapper")
            }
            YasonError::TryReserveError(e) => write!(f, "{}", e),
            YasonError::InvalidPathExpression => write!(f, "invalid path expression"),
        }
    }
}

impl From<BuildError> for YasonError {
    #[inline]
    fn from(err: BuildError) -> Self {
        match err {
            BuildError::TryReserveError(e) => YasonError::TryReserveError(e),
            _ => unreachable!(),
        }
    }
}

impl Error for YasonError {}

pub type YasonResult<T> = std::result::Result<T, YasonError>;

/// An owned `Yason` value, backed by a buffer of bytes in yason binary format.
/// This can be created from a Vec<u8>.
#[repr(transparent)]
pub struct YasonBuf {
    bytes: Vec<u8>,
}

impl YasonBuf {
    /// Creates a new `YasonBuf` from `Vec<u8>`.
    ///
    /// # Safety
    ///
    /// Callers should guarantee the `bytes` is a valid `YASON`.
    #[inline]
    pub unsafe fn new_unchecked(bytes: Vec<u8>) -> Self {
        debug_assert!(!bytes.is_empty());
        YasonBuf { bytes }
    }
}

/// A slice of `Yason` value. This can be created from a [`YasonBuf`] or any type the contains
/// valid bytes in yason binary format.
#[repr(transparent)]
pub struct Yason {
    bytes: [u8],
}

impl Deref for YasonBuf {
    type Target = Yason;

    #[inline]
    fn deref(&self) -> &Yason {
        unsafe { Yason::new_unchecked(&self.bytes) }
    }
}

impl Borrow<Yason> for YasonBuf {
    #[inline]
    fn borrow(&self) -> &Yason {
        self.deref()
    }
}

impl ToOwned for Yason {
    type Owned = YasonBuf;

    #[inline]
    fn to_owned(&self) -> YasonBuf {
        self.to_yason_buf()
            .expect("an out-of-memory error occurred when converting a yason")
    }
}

impl AsRef<Yason> for YasonBuf {
    #[inline]
    fn as_ref(&self) -> &Yason {
        self
    }
}

impl Yason {
    /// Creates a new `Yason` from the reference of `[u8]`.
    ///
    /// # Safety
    ///
    /// Callers should guarantee the `bytes` is a valid `YASON`.
    #[inline]
    pub unsafe fn new_unchecked<B: AsRef<[u8]> + ?Sized>(bytes: &B) -> &Yason {
        debug_assert!(!bytes.as_ref().is_empty());
        &*(bytes.as_ref() as *const [u8] as *const Yason)
    }

    #[inline]
    pub fn to_yason_buf(&self) -> YasonResult<YasonBuf> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve(self.bytes.len())
            .map_err(YasonError::TryReserveError)?;
        bytes.extend_from_slice(&self.bytes);

        Ok(YasonBuf { bytes })
    }

    #[inline]
    pub fn data_type(&self) -> YasonResult<DataType> {
        let data_type = self.get(0)?;
        DataType::try_from(data_type).map_err(|_| YasonError::InvalidDataType(data_type))
    }

    /// If `Yason` is `Object`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn object(&self) -> YasonResult<Object> {
        self.check_type(0, DataType::Object)?;
        Ok(unsafe { Object::new_unchecked(self) })
    }

    /// If `Yason` is `Array`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn array(&self) -> YasonResult<Array> {
        self.check_type(0, DataType::Array)?;
        Ok(unsafe { Array::new_unchecked(self) })
    }

    /// If `Yason` is `String`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn string(&self) -> YasonResult<&str> {
        self.check_type(0, DataType::String)?;
        self.read_string(DATA_TYPE_SIZE)
    }

    #[inline]
    pub(crate) unsafe fn string_unchecked(&self) -> YasonResult<&str> {
        debug_assert!(self.data_type()? == DataType::String);
        self.read_string(DATA_TYPE_SIZE)
    }

    /// If `Yason` is `Number`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn number(&self) -> YasonResult<Number> {
        self.check_type(0, DataType::Number)?;
        self.read_number(DATA_TYPE_SIZE)
    }

    #[inline]
    pub(crate) unsafe fn number_unchecked(&self) -> YasonResult<Number> {
        debug_assert!(self.data_type()? == DataType::Number);
        self.read_number(DATA_TYPE_SIZE)
    }

    /// If `Yason` is `Bool`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn bool(&self) -> YasonResult<bool> {
        self.check_type(0, DataType::Bool)?;
        self.read_bool(DATA_TYPE_SIZE)
    }

    #[inline]
    pub(crate) unsafe fn bool_unchecked(&self) -> YasonResult<bool> {
        debug_assert!(self.data_type()? == DataType::Bool);
        self.read_bool(DATA_TYPE_SIZE)
    }

    /// If `Yason` is `Null`, return true. Returns false otherwise.
    #[inline]
    pub fn is_null(&self) -> YasonResult<bool> {
        self.is_type(0, DataType::Null as u8)
    }

    /// Formats the yason as a compact or pretty string.
    #[inline]
    pub fn format(&self, pretty: bool) -> impl Display + '_ {
        LazyFormat::new(self, pretty)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Yason {
    #[inline]
    fn get(&self, index: usize) -> YasonResult<u8> {
        self.bytes.get(index).map_or_else(
            || {
                Err(YasonError::IndexOutOfBounds {
                    len: self.bytes.len(),
                    index,
                })
            },
            |v| Ok(*v),
        )
    }

    #[allow(clippy::unnecessary_lazy_evaluations)]
    #[inline]
    fn slice(&self, from: usize, to: usize) -> YasonResult<&[u8]> {
        self.bytes.get(from..to).ok_or_else(|| YasonError::IndexOutOfBounds {
            len: self.bytes.len(),
            index: to,
        })
    }

    #[inline]
    fn read_type(&self, index: usize) -> YasonResult<DataType> {
        let data_type = self.get(index)?;
        DataType::try_from(data_type).map_err(|_| YasonError::InvalidDataType(data_type))
    }

    #[inline]
    fn is_type(&self, index: usize, data_type: u8) -> YasonResult<bool> {
        Ok(self.get(index)? == data_type)
    }

    #[inline]
    fn read_i32(&self, index: usize) -> YasonResult<i32> {
        let end = index + size_of::<i32>();
        let bytes = self.slice(index, end)?;
        // SAFETY: The `bytes` must be valid because the `slice()` always takes 4 bytes.
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }

    #[inline]
    fn read_u8(&self, index: usize) -> YasonResult<u8> {
        self.get(index)
    }

    #[inline]
    fn read_u16(&self, index: usize) -> YasonResult<u16> {
        let end = index + size_of::<u16>();
        let bytes = self.slice(index, end)?;
        // SAFETY: The `bytes` must be valid because the `slice()` always takes 2 bytes.
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    #[inline]
    fn read_u32(&self, index: usize) -> YasonResult<u32> {
        let end = index + size_of::<u32>();
        let bytes = self.slice(index, end)?;
        // SAFETY: The `bytes` must be valid because the `slice()` always takes 4 bytes.
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    #[inline]
    fn read_string(&self, pos: usize) -> YasonResult<&str> {
        let (data_length, data_length_len) = decode_varint(&self.bytes, pos)?;
        let end = pos + data_length_len + data_length as usize;
        let bytes = self.slice(pos + data_length_len, end)?;
        let string = unsafe { std::str::from_utf8_unchecked(bytes) };
        Ok(string)
    }

    #[inline]
    fn read_number(&self, index: usize) -> YasonResult<Number> {
        let data_length = self.get(index)? as usize;
        let end = index + NUMBER_LENGTH_SIZE + data_length;
        let bytes = self.slice(index + NUMBER_LENGTH_SIZE, end)?;
        Ok(Number::decode(bytes))
    }

    #[inline]
    fn read_bool(&self, index: usize) -> YasonResult<bool> {
        Ok(self.read_u8(index)? == 1)
    }

    #[inline]
    fn check_type(&self, index: usize, expected: DataType) -> YasonResult<()> {
        if !self.is_type(index, expected as u8)? {
            return Err(YasonError::UnexpectedType {
                expected,
                actual: self.read_type(index)?,
            });
        }

        Ok(())
    }
}

/// Possible yason value corresponding to the data type.
#[derive(Clone)]
pub enum Value<'a> {
    Object(Object<'a>),
    Array(Array<'a>),
    String(&'a str),
    Number(Number),
    Bool(bool),
    Null,
}

impl<'a> Value<'a> {
    #[inline]
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Object(_) => DataType::Object,
            Value::Array(_) => DataType::Array,
            Value::String(_) => DataType::String,
            Value::Number(_) => DataType::Number,
            Value::Bool(_) => DataType::Bool,
            Value::Null => DataType::Null,
        }
    }

    #[inline]
    pub fn try_to_yason(&self, buf: &'a mut Vec<u8>) -> YasonResult<&Yason> {
        match self {
            Value::Object(object) => Ok(object.yason()),
            Value::Array(array) => Ok(array.yason()),
            Value::String(str) => Ok(Scalar::string_with_vec(str, buf)?),
            Value::Number(num) => Ok(Scalar::number_with_vec(num, buf)?),
            Value::Bool(bool) => Ok(Scalar::bool_with_vec(*bool, buf)?),
            Value::Null => Ok(Scalar::null_with_vec(buf)?),
        }
    }
}

impl<'a> TryFrom<&'a Yason> for Value<'a> {
    type Error = YasonError;

    #[inline]
    fn try_from(yason: &'a Yason) -> Result<Self, Self::Error> {
        match yason.data_type()? {
            DataType::Object => Ok(Value::Object(unsafe { Object::new_unchecked(yason) })),
            DataType::Array => Ok(Value::Array(unsafe { Array::new_unchecked(yason) })),
            DataType::String => Ok(Value::String(unsafe { yason.string_unchecked()? })),
            DataType::Number => Ok(Value::Number(unsafe { yason.number_unchecked()? })),
            DataType::Bool => Ok(Value::Bool(unsafe { yason.bool_unchecked()? })),
            DataType::Null => Ok(Value::Null),
        }
    }
}

/// IN_ARRAY: this parameter indicates whether this value is in an array and affects how data is
/// read from value_pos (related to `Yason binary format`).
/// Note:
///   1. IN_ARRAY of a LazyValue generated from the outermost Array is still false.
///   2. IN_ARRAY is true only if this LazyValue is generated from an Array's Iter.
pub struct LazyValue<'a, const IN_ARRAY: bool> {
    yason: &'a Yason,
    ty: DataType,
    value_pos: usize,
}

impl<'a, const IN_ARRAY: bool> LazyValue<'a, IN_ARRAY> {
    #[inline]
    const fn new(yason: &'a Yason, ty: DataType, value_pos: usize) -> Self {
        Self { yason, ty, value_pos }
    }

    #[inline]
    pub const fn data_type(&self) -> DataType {
        self.ty
    }

    #[inline]
    pub fn value(&self) -> YasonResult<Value<'a>> {
        let res = unsafe {
            match self.ty {
                DataType::Object => Value::Object(self.object()?),
                DataType::Array => Value::Array(self.array()?),
                DataType::String => Value::String(self.string()?),
                DataType::Number => Value::Number(self.number()?),
                DataType::Bool => Value::Bool(self.bool()?),
                DataType::Null => Value::Null,
            }
        };

        Ok(res)
    }

    #[inline]
    pub unsafe fn object(&self) -> YasonResult<Object<'a>> {
        debug_assert!(self.ty == DataType::Object);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_object(self.value_pos)
        } else {
            Object::new_unchecked(self.yason).read_object(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn array(&self) -> YasonResult<Array<'a>> {
        debug_assert!(self.ty == DataType::Array);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_array(self.value_pos)
        } else {
            Object::new_unchecked(self.yason).read_array(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn string(&self) -> YasonResult<&'a str> {
        debug_assert!(self.ty == DataType::String);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_string(self.value_pos)
        } else {
            self.yason.read_string(self.value_pos + DATA_TYPE_SIZE)
        }
    }

    #[inline]
    pub unsafe fn number(&self) -> YasonResult<Number> {
        debug_assert!(self.ty == DataType::Number);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_number(self.value_pos)
        } else {
            self.yason.read_number(self.value_pos + DATA_TYPE_SIZE)
        }
    }

    #[inline]
    pub unsafe fn bool(&self) -> YasonResult<bool> {
        debug_assert!(self.ty == DataType::Bool);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_bool(self.value_pos)
        } else {
            self.yason.read_bool(self.value_pos + DATA_TYPE_SIZE)
        }
    }
}

impl<'a> TryFrom<&'a Yason> for LazyValue<'a, false> {
    type Error = YasonError;

    #[inline]
    fn try_from(yason: &'a Yason) -> Result<Self, Self::Error> {
        let data_type = yason.data_type()?;
        Ok(Self {
            yason,
            ty: data_type,
            value_pos: 0,
        })
    }
}
