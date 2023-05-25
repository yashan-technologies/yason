//! Yason manipulation.

mod array;
mod object;

pub use crate::yason::array::{Array, ArrayIter};
pub use crate::yason::object::{KeyIter, Object, ObjectIter, ValueIter};

use crate::binary::{ARRAY_SIZE, DATA_TYPE_SIZE, NUMBER_LENGTH_SIZE, OBJECT_SIZE};
use crate::format::{CompactFormatter, FormatResult, Formatter, LazyFormat, PrettyFormatter};
use crate::util::{decode_varint, slice_to_array};
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
/// This can be created from a `Vec<u8>`.
#[derive(Debug, Clone)]
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

    #[inline]
    pub fn clone_from_yason(&mut self, yason: &Yason) {
        self.bytes.clear();
        self.bytes.extend_from_slice(yason.as_bytes())
    }
}

/// A slice of `Yason` value. This can be created from a [`YasonBuf`] or any type the contains
/// valid bytes in yason binary format.
#[derive(Debug)]
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
        unsafe { YasonBuf::new_unchecked(self.bytes.to_vec()) }
    }
}

impl AsRef<Yason> for Yason {
    #[inline]
    fn as_ref(&self) -> &Yason {
        self
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
        unsafe { self.object_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn object_unchecked(&self) -> YasonResult<Object> {
        debug_assert!(self.data_type()? == DataType::Object);
        Ok(Object::new_unchecked(self))
    }

    /// If `Yason` is `Array`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn array(&self) -> YasonResult<Array> {
        self.check_type(0, DataType::Array)?;
        unsafe { self.array_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn array_unchecked(&self) -> YasonResult<Array> {
        debug_assert!(self.data_type()? == DataType::Array);
        Ok(Array::new_unchecked(self))
    }

    /// If `Yason` is `String`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn string(&self) -> YasonResult<&str> {
        self.check_type(0, DataType::String)?;
        unsafe { self.string_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn string_unchecked(&self) -> YasonResult<&str> {
        debug_assert!(self.data_type()? == DataType::String);
        self.read_string(0)
    }

    /// If `Yason` is `Binary`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn binary(&self) -> YasonResult<&[u8]> {
        self.check_type(0, DataType::Binary)?;
        unsafe { self.binary_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn binary_unchecked(&self) -> YasonResult<&[u8]> {
        debug_assert!(self.data_type()? == DataType::Binary);
        self.read_binary(0)
    }

    /// If `Yason` is `Number`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn number(&self) -> YasonResult<Number> {
        self.check_type(0, DataType::Number)?;
        unsafe { self.number_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn number_unchecked(&self) -> YasonResult<Number> {
        debug_assert!(self.data_type()? == DataType::Number);
        self.read_number(0)
    }

    /// If `Yason` is `Bool`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn bool(&self) -> YasonResult<bool> {
        self.check_type(0, DataType::Bool)?;
        unsafe { self.bool_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn bool_unchecked(&self) -> YasonResult<bool> {
        debug_assert!(self.data_type()? == DataType::Bool);
        self.read_bool(0)
    }

    /// If `Yason` is `Null`, return true. Returns false otherwise.
    #[inline]
    pub fn is_null(&self) -> YasonResult<bool> {
        self.is_type(0, DataType::Null as u8)
    }

    /// If `Yason` is `Tinyint`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn tinyint(&self) -> YasonResult<i8> {
        self.check_type(0, DataType::Tinyint)?;
        unsafe { self.tinyint_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn tinyint_unchecked(&self) -> YasonResult<i8> {
        debug_assert!(self.data_type()? == DataType::Tinyint);
        self.read_tinyint(0)
    }

    /// If `Yason` is `Smallint`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn smallint(&self) -> YasonResult<i16> {
        self.check_type(0, DataType::Smallint)?;
        unsafe { self.smallint_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn smallint_unchecked(&self) -> YasonResult<i16> {
        debug_assert!(self.data_type()? == DataType::Smallint);
        self.read_smallint(0)
    }

    /// If `Yason` is `Integer`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn integer(&self) -> YasonResult<i32> {
        self.check_type(0, DataType::Integer)?;
        unsafe { self.integer_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn integer_unchecked(&self) -> YasonResult<i32> {
        debug_assert!(self.data_type()? == DataType::Integer);
        self.read_integer(0)
    }

    /// If `Yason` is `Bigint`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn bigint(&self) -> YasonResult<i64> {
        self.check_type(0, DataType::Bigint)?;
        unsafe { self.bigint_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn bigint_unchecked(&self) -> YasonResult<i64> {
        debug_assert!(self.data_type()? == DataType::Bigint);
        self.read_bigint(0)
    }

    /// If `Yason` is `Float`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn float(&self) -> YasonResult<f32> {
        self.check_type(0, DataType::Float)?;
        unsafe { self.float_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn float_unchecked(&self) -> YasonResult<f32> {
        debug_assert!(self.data_type()? == DataType::Float);
        self.read_float(0)
    }

    /// If `Yason` is `Double`, return its value. Returns `YasonError` otherwise.
    #[inline]
    pub fn double(&self) -> YasonResult<f64> {
        self.check_type(0, DataType::Double)?;
        unsafe { self.double_unchecked() }
    }

    #[inline]
    pub(crate) unsafe fn double_unchecked(&self) -> YasonResult<f64> {
        debug_assert!(self.data_type()? == DataType::Double);
        self.read_double(0)
    }

    /// Formats the yason as a compact or pretty string.
    #[inline]
    pub fn format(&self, pretty: bool, extended: bool) -> impl Display + '_ {
        LazyFormat::new(self, pretty, extended)
    }

    /// Formats the yason as a compact or pretty string to a provided buffer.
    #[inline]
    pub fn format_to<W: fmt::Write>(&self, pretty: bool, extended: bool, buf: &mut W) -> FormatResult<()> {
        format_to!(buf, pretty, extended, self, format)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns whether two Yason are equal.
    #[inline]
    pub fn equals<T: AsRef<Yason>>(&self, other: T) -> YasonResult<bool> {
        let other = other.as_ref();
        if self.bytes.len() != other.bytes.len() || self.data_type()? != other.data_type()? {
            return Ok(false);
        }

        let left = LazyValue::try_from(self)?;
        let right = LazyValue::try_from(other)?;
        left.equals(right)
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
    fn read_tinyint(&self, index: usize) -> YasonResult<i8> {
        self.get(index + DATA_TYPE_SIZE).map(|i| i as i8)
    }

    #[inline]
    fn read_smallint(&self, index: usize) -> YasonResult<i16> {
        let index = index + DATA_TYPE_SIZE;
        let end = index + size_of::<i16>();
        let bytes = self.slice(index, end)?;
        Ok(i16::from_le_bytes(slice_to_array(bytes)))
    }

    #[inline]
    fn read_i32(&self, index: usize) -> YasonResult<i32> {
        let end = index + size_of::<i32>();
        let bytes = self.slice(index, end)?;
        Ok(i32::from_le_bytes(slice_to_array(bytes)))
    }

    #[inline]
    fn read_integer(&self, index: usize) -> YasonResult<i32> {
        self.read_i32(index + DATA_TYPE_SIZE)
    }

    #[inline]
    fn read_bigint(&self, index: usize) -> YasonResult<i64> {
        let index = index + DATA_TYPE_SIZE;
        let end = index + size_of::<i64>();
        let bytes = self.slice(index, end)?;
        Ok(i64::from_le_bytes(slice_to_array(bytes)))
    }

    #[inline]
    fn read_float(&self, index: usize) -> YasonResult<f32> {
        let index = index + DATA_TYPE_SIZE;
        let end = index + size_of::<f32>();
        let bytes = self.slice(index, end)?;
        Ok(f32::from_le_bytes(slice_to_array(bytes)))
    }

    #[inline]
    fn read_double(&self, index: usize) -> YasonResult<f64> {
        let index = index + DATA_TYPE_SIZE;
        let end = index + size_of::<f64>();
        let bytes = self.slice(index, end)?;
        Ok(f64::from_le_bytes(slice_to_array(bytes)))
    }

    #[inline]
    fn read_u8(&self, index: usize) -> YasonResult<u8> {
        self.get(index)
    }

    #[inline]
    fn read_u16(&self, index: usize) -> YasonResult<u16> {
        let end = index + size_of::<u16>();
        let bytes = self.slice(index, end)?;
        Ok(u16::from_le_bytes(slice_to_array(bytes)))
    }

    #[inline]
    fn read_u32(&self, index: usize) -> YasonResult<u32> {
        let end = index + size_of::<u32>();
        let bytes = self.slice(index, end)?;
        Ok(u32::from_le_bytes(slice_to_array(bytes)))
    }

    #[inline]
    fn read_object(&self, index: usize) -> YasonResult<Object> {
        let size = self.read_i32(index + DATA_TYPE_SIZE)? as usize + DATA_TYPE_SIZE + OBJECT_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.slice(index, size + index)?) };
        Ok(unsafe { Object::new_unchecked(yason) })
    }

    #[inline]
    fn read_array(&self, index: usize) -> YasonResult<Array> {
        let size = self.read_i32(index + DATA_TYPE_SIZE)? as usize + DATA_TYPE_SIZE + ARRAY_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.slice(index, size + index)?) };
        Ok(unsafe { Array::new_unchecked(yason) })
    }

    #[inline]
    fn read_string(&self, index: usize) -> YasonResult<&str> {
        let index = index + DATA_TYPE_SIZE;
        let (data_length, data_length_len) = decode_varint(&self.bytes, index)?;
        let end = index + data_length_len + data_length as usize;
        let bytes = self.slice(index + data_length_len, end)?;
        let string = unsafe { std::str::from_utf8_unchecked(bytes) };
        Ok(string)
    }

    #[inline]
    fn read_binary(&self, index: usize) -> YasonResult<&[u8]> {
        let index = index + DATA_TYPE_SIZE;
        let (data_length, data_length_len) = decode_varint(&self.bytes, index)?;
        let end = index + data_length_len + data_length as usize;
        let bytes = self.slice(index + data_length_len, end)?;
        Ok(bytes)
    }

    #[inline]
    fn read_number(&self, index: usize) -> YasonResult<Number> {
        let index = index + DATA_TYPE_SIZE;
        let data_length = self.get(index)? as usize;
        let end = index + NUMBER_LENGTH_SIZE + data_length;
        let bytes = self.slice(index + NUMBER_LENGTH_SIZE, end)?;
        Ok(Number::decode(bytes))
    }

    #[inline]
    fn read_bool(&self, index: usize) -> YasonResult<bool> {
        Ok(self.read_u8(index + DATA_TYPE_SIZE)? == 1)
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

impl PartialEq for Yason {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.equals(other).expect("an error occurred when comparing yason")
    }
}

impl PartialEq for YasonBuf {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref()
            .equals(other)
            .expect("an error occurred when comparing yason")
    }
}

/// Possible yason value corresponding to the data type.
#[derive(Clone, Debug)]
pub enum Value<'a> {
    Object(Object<'a>),
    Array(Array<'a>),
    String(&'a str),
    Number(Number),
    Bool(bool),
    Null,
    Tinyint(i8),
    Smallint(i16),
    Integer(i32),
    Bigint(i64),
    Float(f32),
    Double(f64),
    Binary(&'a [u8]),
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
            Value::Tinyint(_) => DataType::Tinyint,
            Value::Smallint(_) => DataType::Smallint,
            Value::Integer(_) => DataType::Integer,
            Value::Bigint(_) => DataType::Bigint,
            Value::Float(_) => DataType::Float,
            Value::Double(_) => DataType::Double,
            Value::Binary(_) => DataType::Binary,
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
            Value::Tinyint(i) => Ok(Scalar::tinyint_with_vec(*i, buf)?),
            Value::Smallint(i) => Ok(Scalar::smallint_with_vec(*i, buf)?),
            Value::Integer(i) => Ok(Scalar::integer_with_vec(*i, buf)?),
            Value::Bigint(i) => Ok(Scalar::bigint_with_vec(*i, buf)?),
            Value::Float(f) => Ok(Scalar::float_with_vec(*f, buf)?),
            Value::Double(f) => Ok(Scalar::double_with_vec(*f, buf)?),
            Value::Binary(bin) => Ok(Scalar::binary_with_vec(bin, buf)?),
        }
    }

    #[inline]
    pub(crate) fn format_to<W: fmt::Write>(&self, pretty: bool, extended: bool, writer: &mut W) -> FormatResult<()> {
        match self {
            Value::Object(object) => object.yason().format_to(pretty, extended, writer),
            Value::Array(array) => array.yason().format_to(pretty, extended, writer),
            Value::String(str) => {
                // string are formatted the same in standard and extended mode
                let mut fmt = CompactFormatter::new(extended);
                fmt.write_string(str, writer)
            }
            Value::Number(number) => {
                format_to!(writer, pretty, extended, number, write_number)
            }
            Value::Bool(bool) => {
                // bool are formatted the same in standard and extended mode
                let mut fmt = CompactFormatter::new(extended);
                fmt.write_bool(*bool, writer)
            }
            Value::Null => {
                // null are formatted the same in standard and extended mode
                let mut fmt = CompactFormatter::new(extended);
                fmt.write_null(writer)
            }
            Value::Tinyint(i) => {
                // tinyint are formatted the same in standard and extended mode
                let mut fmt = CompactFormatter::new(extended);
                fmt.write_tinyint(*i, writer)
            }
            Value::Smallint(i) => {
                // smallint are formatted the same in standard and extended mode
                let mut fmt = CompactFormatter::new(extended);
                fmt.write_smallint(*i, writer)
            }
            Value::Integer(i) => {
                // integer are formatted the same in standard and extended mode
                let mut fmt = CompactFormatter::new(extended);
                fmt.write_integer(*i, writer)
            }
            Value::Bigint(i) => {
                format_to!(writer, pretty, extended, *i, write_bigint)
            }
            Value::Float(f) => {
                format_to!(writer, pretty, extended, *f, write_float)
            }
            Value::Double(f) => {
                // double are formatted the same in standard and extended mode
                let mut fmt = CompactFormatter::new(extended);
                fmt.write_double(*f, writer)
            }
            Value::Binary(bin) => {
                format_to!(writer, pretty, extended, bin, write_binary)
            }
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
            DataType::Tinyint => Ok(Value::Tinyint(unsafe { yason.tinyint_unchecked()? })),
            DataType::Smallint => Ok(Value::Smallint(unsafe { yason.smallint_unchecked()? })),
            DataType::Integer => Ok(Value::Integer(unsafe { yason.integer_unchecked()? })),
            DataType::Bigint => Ok(Value::Bigint(unsafe { yason.bigint_unchecked()? })),
            DataType::Float => Ok(Value::Float(unsafe { yason.float_unchecked()? })),
            DataType::Double => Ok(Value::Double(unsafe { yason.double_unchecked()? })),
            DataType::Binary => Ok(Value::Binary(unsafe { yason.binary_unchecked()? })),
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
                DataType::Tinyint => Value::Tinyint(self.tinyint()?),
                DataType::Smallint => Value::Smallint(self.smallint()?),
                DataType::Integer => Value::Integer(self.integer()?),
                DataType::Bigint => Value::Bigint(self.bigint()?),
                DataType::Float => Value::Float(self.float()?),
                DataType::Double => Value::Double(self.double()?),
                DataType::Binary => Value::Binary(self.binary()?),
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
            self.yason.read_object(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn array(&self) -> YasonResult<Array<'a>> {
        debug_assert!(self.ty == DataType::Array);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_array(self.value_pos)
        } else {
            self.yason.read_array(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn string(&self) -> YasonResult<&'a str> {
        debug_assert!(self.ty == DataType::String);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_string(self.value_pos)
        } else {
            self.yason.read_string(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn binary(&self) -> YasonResult<&'a [u8]> {
        debug_assert!(self.ty == DataType::Binary);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_binary(self.value_pos)
        } else {
            self.yason.read_binary(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn number(&self) -> YasonResult<Number> {
        debug_assert!(self.ty == DataType::Number);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_number(self.value_pos)
        } else {
            self.yason.read_number(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn bool(&self) -> YasonResult<bool> {
        debug_assert!(self.ty == DataType::Bool);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_bool(self.value_pos)
        } else {
            self.yason.read_bool(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn tinyint(&self) -> YasonResult<i8> {
        debug_assert!(self.ty == DataType::Tinyint);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_tinyint(self.value_pos)
        } else {
            self.yason.read_tinyint(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn smallint(&self) -> YasonResult<i16> {
        debug_assert!(self.ty == DataType::Smallint);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_smallint(self.value_pos)
        } else {
            self.yason.read_smallint(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn integer(&self) -> YasonResult<i32> {
        debug_assert!(self.ty == DataType::Integer);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_integer(self.value_pos)
        } else {
            self.yason.read_integer(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn bigint(&self) -> YasonResult<i64> {
        debug_assert!(self.ty == DataType::Bigint);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_bigint(self.value_pos)
        } else {
            self.yason.read_bigint(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn float(&self) -> YasonResult<f32> {
        debug_assert!(self.ty == DataType::Float);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_float(self.value_pos)
        } else {
            self.yason.read_float(self.value_pos)
        }
    }

    #[inline]
    pub unsafe fn double(&self) -> YasonResult<f64> {
        debug_assert!(self.ty == DataType::Double);
        if IN_ARRAY {
            Array::new_unchecked(self.yason).read_double(self.value_pos)
        } else {
            self.yason.read_double(self.value_pos)
        }
    }

    #[inline]
    pub fn equals(&self, other: LazyValue<IN_ARRAY>) -> YasonResult<bool> {
        if self.data_type() != other.data_type() || self.yason.bytes.len() != other.yason.bytes.len() {
            return Ok(false);
        }

        match self.data_type() {
            DataType::Object => unsafe { self.object()?.equals(other.object()?) },
            DataType::Array => unsafe { self.array()?.equals(other.array()?) },
            DataType::String => unsafe { Ok(self.string()?.eq(other.string()?)) },
            DataType::Number => unsafe { Ok(self.number()?.eq(&other.number()?)) },
            DataType::Bool => unsafe { Ok(self.bool()?.eq(&other.bool()?)) },
            DataType::Null => Ok(true),
            DataType::Tinyint => unsafe { Ok(self.tinyint()?.eq(&other.tinyint()?)) },
            DataType::Smallint => unsafe { Ok(self.smallint()?.eq(&other.smallint()?)) },
            DataType::Integer => unsafe { Ok(self.integer()?.eq(&other.integer()?)) },
            DataType::Bigint => unsafe { Ok(self.bigint()?.eq(&other.bigint()?)) },
            DataType::Float => unsafe { Ok(self.float()?.eq(&other.float()?)) },
            DataType::Double => unsafe { Ok(self.double()?.eq(&other.double()?)) },
            DataType::Binary => unsafe { Ok(self.binary()?.eq(other.binary()?)) },
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
