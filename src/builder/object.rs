//! Object builder.

use crate::binary::*;
use crate::builder::array::{ArrayRefBuilder, InnerArrayBuilder};
use crate::builder::{BuildResult, DEFAULT_SIZE, Depth, MAX_NESTED_DEPTH};
use crate::util::cmp_key;
use crate::vec::VecExt;
use crate::yason::{Yason, YasonBuf};
use crate::{BuildError, DataType, Date, Number, Time, Timestamp};
use decimal_rs::MAX_BINARY_SIZE;
use std::cmp::Ordering;
use std::hint;
use std::ptr;

pub(crate) struct InnerObjectBuilder<'a, B: AsMut<Vec<u8>>> {
    bytes: B,
    element_count: u16,
    start_pos: usize,
    key_offset_pos: usize,
    value_count: u16,
    bytes_init_len: usize,
    key_sorted: bool,
    current_depth: usize,
    total_nested_depth: Depth<'a>,
}

impl<'a, B: AsMut<Vec<u8>> + AsRef<Vec<u8>>> InnerObjectBuilder<'a, B> {
    #[inline]
    pub(crate) fn try_new(
        mut bytes: B,
        element_count: u16,
        key_sorted: bool,
        mut total_depth: Depth<'a>,
    ) -> BuildResult<Self> {
        if total_depth.depth() >= MAX_NESTED_DEPTH {
            return Err(BuildError::NestedTooDeeply);
        }

        let bs = bytes.as_mut();
        let bytes_init_len = bs.len();

        let size = DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE + KEY_OFFSET_SIZE * (element_count as usize);
        bs.try_reserve(size)?;

        bs.push_data_type(DataType::Object); // type
        bs.skip_size(); // size
        let start_pos = bs.len();
        bs.push_u16(element_count); // element-count
        let key_offset_pos = bs.len();
        bs.skip_key_offset(element_count as usize); // key-offset

        total_depth.increase();

        Ok(Self {
            bytes,
            element_count,
            start_pos,
            key_offset_pos,
            value_count: 0,
            bytes_init_len,
            key_sorted,
            current_depth: total_depth.depth(),
            total_nested_depth: total_depth,
        })
    }

    #[inline]
    fn key_sorted(&mut self) -> bool {
        if self.element_count <= 1 {
            return true;
        }

        let bytes = self.bytes.as_ref();
        let mut key_offsets = self.key_offset_iter();
        let mut cur_key_offset = key_offsets.next().unwrap();

        for next_key_offset in key_offsets {
            let cur_key = Self::read_key_by_offset(bytes, cur_key_offset, self.start_pos);
            let next_key = Self::read_key_by_offset(bytes, next_key_offset, self.start_pos);
            if cur_key.len() > next_key.len() || (cur_key.len() == next_key.len() && cur_key > next_key) {
                return false;
            }
            cur_key_offset = next_key_offset;
        }
        true
    }

    #[inline]
    fn finish(&mut self) -> BuildResult<usize> {
        if self.current_depth != self.total_nested_depth.depth() {
            return Err(BuildError::InnerUncompletedError);
        }
        if self.value_count != self.element_count {
            return Err(BuildError::InconsistentElementCount {
                expected: self.element_count,
                actual: self.value_count,
            });
        }

        let bytes = self.bytes.as_mut();
        let total_size = bytes.len() - self.start_pos;
        bytes.write_total_size(total_size as i32, self.start_pos - OBJECT_SIZE);

        self.total_nested_depth.decrease();

        debug_assert!(self.key_sorted());
        Ok(self.bytes_init_len)
    }

    #[inline]
    fn push_key_value_by<F>(&mut self, key: &str, reserved_size: usize, f: F) -> BuildResult<()>
    where
        F: FnOnce(&mut Vec<u8>) -> BuildResult<()>,
    {
        if self.current_depth != self.total_nested_depth.depth() {
            return Err(BuildError::InnerUncompletedError);
        }

        let bytes = self.bytes.as_mut();
        bytes.try_reserve(reserved_size)?;

        if !self.key_sorted {
            let pos = Self::binary_search(key, bytes, self.start_pos, self.value_count as usize);

            let key_offset = bytes.len() - self.start_pos;
            let offset_pos = self.start_pos + ELEMENT_COUNT_SIZE + pos * KEY_OFFSET_SIZE;

            if pos < self.value_count as usize {
                let count = (self.value_count as usize - pos) * KEY_OFFSET_SIZE;
                let src = bytes[offset_pos..offset_pos + count].as_mut_ptr();
                let dst = unsafe { src.add(KEY_OFFSET_SIZE) };

                unsafe { ptr::copy(src, dst, count) }
            }
            bytes.write_offset(key_offset as u32, offset_pos);
            bytes.push_key(key);
        } else {
            let key_offset = bytes.len() - self.start_pos;
            bytes.write_offset(key_offset as u32, self.key_offset_pos);
            bytes.push_key(key);
        }

        self.key_offset_pos += KEY_OFFSET_SIZE;

        f(bytes)?;

        self.value_count += 1;
        Ok(())
    }

    #[inline]
    fn binary_search(target: &str, bytes: &[u8], start_pos: usize, value_count: usize) -> usize {
        let begin = start_pos + ELEMENT_COUNT_SIZE;
        let end = begin + value_count * KEY_OFFSET_SIZE;

        let key_offsets = bytes[begin..end].as_ptr().cast::<u32>();
        Self::key_offset_binary_search_by(key_offsets, value_count, |key_offset| {
            let key = Self::read_key_by_offset(bytes, key_offset, start_pos);
            cmp_key(key, target)
        })
        .unwrap_or_else(|index| index)
    }

    #[inline]
    fn key_offset_binary_search_by<F>(key_offsets: *const u32, len: usize, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(usize) -> Ordering,
    {
        if len == 0 {
            return Err(0);
        }

        let mut size = len;
        let mut base = 0;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            // SAFETY: `mid` is always less than `len`; the packed YASON offset table may be unaligned.
            let key_offset = unsafe { u32::from_le(key_offsets.add(mid).read_unaligned()) } as usize;
            let cmp = f(key_offset);

            base = hint::select_unpredictable(cmp == Ordering::Greater, base, mid);
            size -= half;
        }

        // SAFETY: `base` is always less than `len`.
        let key_offset = unsafe { u32::from_le(key_offsets.add(base).read_unaligned()) } as usize;
        let cmp = f(key_offset);
        if cmp == Ordering::Equal {
            // SAFETY: `base` is always less than `len`.
            unsafe { hint::assert_unchecked(base < len) };
            Ok(base)
        } else {
            let result = base + (cmp == Ordering::Less) as usize;
            // SAFETY: `result` is at most `len`.
            unsafe { hint::assert_unchecked(result <= len) };
            Err(result)
        }
    }

    #[inline]
    fn key_offset_iter(&self) -> impl Iterator<Item = usize> + '_ {
        let begin = self.start_pos + ELEMENT_COUNT_SIZE;
        let end = begin + self.element_count as usize * KEY_OFFSET_SIZE;
        let bytes = self.bytes.as_ref();
        bytes[begin..end]
            .chunks_exact(KEY_OFFSET_SIZE)
            .map(|key_offset| u32::from_le_bytes(key_offset.try_into().unwrap()) as usize)
    }

    #[inline]
    fn read_key_by_offset(bytes: &[u8], key_offset: usize, start_pos: usize) -> &str {
        let key_index = key_offset + start_pos;
        let key_length_bytes = &bytes[key_index..key_index + KEY_LENGTH_SIZE];
        // SAFETY: The `key_length_bytes` must be valid because the slice operation always takes 2 bytes.
        let key_length = u16::from_le_bytes(key_length_bytes.try_into().unwrap()) as usize;

        let key_bytes = &bytes[key_index + KEY_LENGTH_SIZE..key_index + KEY_LENGTH_SIZE + key_length];
        unsafe { std::str::from_utf8_unchecked(key_bytes) }
    }

    #[inline]
    fn push_object(
        &mut self,
        key: &str,
        element_count: u16,
        key_sorted: bool,
    ) -> BuildResult<InnerObjectBuilder<'_, &mut Vec<u8>>> {
        let size = key.len() + KEY_LENGTH_SIZE;
        self.push_key_value_by(key, size, |_| Ok(()))?;
        let bytes = self.bytes.as_mut();
        InnerObjectBuilder::try_new(bytes, element_count, key_sorted, self.total_nested_depth.borrow_mut())
    }

    #[inline]
    fn push_array(&mut self, key: &str, element_count: u16) -> BuildResult<InnerArrayBuilder<'_, &mut Vec<u8>>> {
        let size = key.len() + KEY_LENGTH_SIZE;
        self.push_key_value_by(key, size, |_| Ok(()))?;
        let bytes = self.bytes.as_mut();
        InnerArrayBuilder::try_new(bytes, element_count, self.total_nested_depth.borrow_mut())
    }

    #[inline]
    fn push_string(&mut self, key: &str, value: &str) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + MAX_DATA_LENGTH_SIZE + value.len();
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::String);
            bytes.push_string(value)?;
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_binary(&mut self, key: &str, value: &[u8]) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + MAX_DATA_LENGTH_SIZE + value.len();
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Binary);
            bytes.push_binary(value)?;
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_number(&mut self, key: &str, value: &Number) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + NUMBER_LENGTH_SIZE + MAX_BINARY_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Number);
            bytes.push_number(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_bool(&mut self, key: &str, value: bool) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + BOOL_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Bool);
            bytes.push_u8(value as u8);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_null(&mut self, key: &str) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Null);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_tinyint(&mut self, key: &str, value: i8) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + TINYINT_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Tinyint);
            bytes.push_i8(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_smallint(&mut self, key: &str, value: i16) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + SMALLINT_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Smallint);
            bytes.push_i16(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_integer(&mut self, key: &str, value: i32) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + INTEGER_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Integer);
            bytes.push_i32(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_bigint(&mut self, key: &str, value: i64) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + BIGINT_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Bigint);
            bytes.push_i64(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_float(&mut self, key: &str, value: f32) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + FLOAT_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Float);
            bytes.push_f32(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_double(&mut self, key: &str, value: f64) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + DOUBLE_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Double);
            bytes.push_f64(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_timestamp(&mut self, key: &str, value: Timestamp) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + TIMESTAMP_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Timestamp);
            bytes.push_timestamp(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_date(&mut self, key: &str, value: Date) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + DATE_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Date);
            bytes.push_date(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }

    #[inline]
    fn push_time(&mut self, key: &str, value: Time) -> BuildResult<()> {
        let size = KEY_LENGTH_SIZE + key.len() + DATA_TYPE_SIZE + TIME_SIZE;
        let f = |bytes: &mut Vec<u8>| {
            bytes.push_data_type(DataType::Time);
            bytes.push_time(value);
            Ok(())
        };
        self.push_key_value_by(key, size, f)
    }
}

/// Builder for encoding an object.
#[repr(transparent)]
pub struct ObjectBuilder<'a>(InnerObjectBuilder<'a, Vec<u8>>);

impl ObjectBuilder<'_> {
    /// Creates `ObjectBuilder` with specified element count.
    /// `key_sorted` indicates whether the object is sorted by key.
    #[inline]
    pub fn try_new(element_count: u16, key_sorted: bool) -> BuildResult<Self> {
        let bytes = <Vec<u8> as VecExt>::try_with_capacity(DEFAULT_SIZE)?;
        let builder = InnerObjectBuilder::try_new(bytes, element_count, key_sorted, Depth::new())?;
        Ok(Self(builder))
    }

    /// Finishes building the object.
    #[inline]
    pub fn finish(mut self) -> BuildResult<YasonBuf> {
        self.0.finish()?;
        Ok(unsafe { YasonBuf::new_unchecked(self.0.bytes) })
    }
}

/// Builder for encoding an object.
#[repr(transparent)]
pub struct ObjectRefBuilder<'a>(pub(crate) InnerObjectBuilder<'a, &'a mut Vec<u8>>);

impl<'a> ObjectRefBuilder<'a> {
    /// Creates `ObjectRefBuilder` with specified element count.
    /// `key_sorted` indicates whether the object is sorted by key.
    #[inline]
    pub fn try_new(bytes: &'a mut Vec<u8>, element_count: u16, key_sorted: bool) -> BuildResult<Self> {
        let obj_builder = InnerObjectBuilder::try_new(bytes, element_count, key_sorted, Depth::new())?;
        Ok(Self(obj_builder))
    }

    /// Finishes building the object.
    #[inline]
    pub fn finish(mut self) -> BuildResult<&'a Yason> {
        let bytes_init_len = self.0.finish()?;
        let bytes = self.0.bytes;
        Ok(unsafe { Yason::new_unchecked(&bytes[bytes_init_len..]) })
    }
}

pub trait ObjBuilder {
    /// Pushes an embedded object with specified element count and a flag which indicates whether the embedded object is sorted by key.
    fn push_object<Key: AsRef<str>>(
        &mut self,
        key: Key,
        element_count: u16,
        key_sorted: bool,
    ) -> BuildResult<ObjectRefBuilder<'_>>;

    /// Pushes an embedded array with specified element count.
    fn push_array<Key: AsRef<str>>(&mut self, key: Key, element_count: u16) -> BuildResult<ArrayRefBuilder<'_>>;

    /// Pushes a string value.
    fn push_string<Key: AsRef<str>, Val: AsRef<str>>(&mut self, key: Key, value: Val) -> BuildResult<&mut Self>;

    /// Pushes a binary value.
    fn push_binary<Key: AsRef<str>, Val: AsRef<[u8]>>(&mut self, key: Key, value: Val) -> BuildResult<&mut Self>;

    /// Pushes a number value.
    fn push_number<Key: AsRef<str>, Num: AsRef<Number>>(&mut self, key: Key, value: Num) -> BuildResult<&mut Self>;

    /// Pushes a bool value.
    fn push_bool<Key: AsRef<str>>(&mut self, key: Key, value: bool) -> BuildResult<&mut Self>;

    /// Pushes a null value.
    fn push_null<Key: AsRef<str>>(&mut self, key: Key) -> BuildResult<&mut Self>;

    /// Pushes a tinyint value.
    fn push_tinyint<Key: AsRef<str>>(&mut self, key: Key, value: i8) -> BuildResult<&mut Self>;

    /// Pushes a smallint value.
    fn push_smallint<Key: AsRef<str>>(&mut self, key: Key, value: i16) -> BuildResult<&mut Self>;

    /// Pushes a integer value.
    fn push_integer<Key: AsRef<str>>(&mut self, key: Key, value: i32) -> BuildResult<&mut Self>;

    /// Pushes a bigint value.
    fn push_bigint<Key: AsRef<str>>(&mut self, key: Key, value: i64) -> BuildResult<&mut Self>;

    /// Pushes a float value.
    fn push_float<Key: AsRef<str>>(&mut self, key: Key, value: f32) -> BuildResult<&mut Self>;

    /// Pushes a double value.
    fn push_double<Key: AsRef<str>>(&mut self, key: Key, value: f64) -> BuildResult<&mut Self>;

    /// Pushes a timestamp value.
    fn push_timestamp<Key: AsRef<str>>(&mut self, key: Key, value: Timestamp) -> BuildResult<&mut Self>;

    /// Pushes a date value.
    fn push_date<Key: AsRef<str>>(&mut self, key: Key, value: Date) -> BuildResult<&mut Self>;

    /// Pushes a time value.
    fn push_time<Key: AsRef<str>>(&mut self, key: Key, value: Time) -> BuildResult<&mut Self>;
}

macro_rules! impl_push_methods {
    ($v: vis,) => {
        /// Pushes an embedded object with specified element count and a flag which indicates whether the embedded object is sorted by key.
        #[inline]
        $v fn push_object<Key: AsRef<str>>(
            &mut self,
            key: Key,
            element_count: u16,
            key_sorted: bool,
        ) -> BuildResult<ObjectRefBuilder<'_>> {
            let key = key.as_ref();
            let obj_builder = self.0.push_object(key, element_count, key_sorted)?;
            Ok(ObjectRefBuilder(obj_builder))
        }

        /// Pushes an embedded array with specified element count.
        #[inline]
        $v fn push_array<Key: AsRef<str>>(&mut self, key: Key, element_count: u16) -> BuildResult<ArrayRefBuilder<'_>> {
            let key = key.as_ref();
            let array_builder = self.0.push_array(key, element_count)?;
            Ok(ArrayRefBuilder(array_builder))
        }

        /// Pushes a string value.
        #[inline]
        $v fn push_string<Key: AsRef<str>, Val: AsRef<str>>(
            &mut self,
            key: Key,
            value: Val,
        ) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            let value = value.as_ref();
            self.0.push_string(key, value)?;
            Ok(self)
        }

        /// Pushes a binary value.
        #[inline]
        $v fn push_binary<Key: AsRef<str>, Val: AsRef<[u8]>>(
            &mut self,
            key: Key,
            value: Val,
        ) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            let value = value.as_ref();
            self.0.push_binary(key, value)?;
            Ok(self)
        }

        /// Pushes a number value.
        #[inline]
        $v fn push_number<Key: AsRef<str>, Num: AsRef<Number>>(&mut self, key: Key, value: Num) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_number(key, value.as_ref())?;
            Ok(self)
        }

        /// Pushes a bool value.
        #[inline]
        $v fn push_bool<Key: AsRef<str>>(&mut self, key: Key, value: bool) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_bool(key, value)?;
            Ok(self)
        }

        /// Pushes a null value.
        #[inline]
        $v fn push_null<Key: AsRef<str>>(&mut self, key: Key) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_null(key)?;
            Ok(self)
        }

        /// Pushes a tinyint value.
        #[inline]
        $v fn push_tinyint<Key: AsRef<str>>(&mut self, key: Key, value: i8) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_tinyint(key, value)?;
            Ok(self)
        }

        /// Pushes a smallint value.
        #[inline]
        $v fn push_smallint<Key: AsRef<str>>(&mut self, key: Key, value: i16) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_smallint(key, value)?;
            Ok(self)
        }

        /// Pushes a integer value.
        #[inline]
        $v fn push_integer<Key: AsRef<str>>(&mut self, key: Key, value: i32) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_integer(key, value)?;
            Ok(self)
        }

        /// Pushes a bigint value.
        #[inline]
        $v fn push_bigint<Key: AsRef<str>>(&mut self, key: Key, value: i64) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_bigint(key, value)?;
            Ok(self)
        }

        /// Pushes a float value.
        #[inline]
        $v fn push_float<Key: AsRef<str>>(&mut self, key: Key, value: f32) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_float(key, value)?;
            Ok(self)
        }

        /// Pushes a double value.
        #[inline]
        $v fn push_double<Key: AsRef<str>>(&mut self, key: Key, value: f64) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_double(key, value)?;
            Ok(self)
        }

        /// Pushes a timestamp value.
        #[inline]
        $v fn push_timestamp<Key: AsRef<str>>(&mut self, key: Key, value: Timestamp) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_timestamp(key, value)?;
            Ok(self)
        }

        /// Pushes a date value.
        #[inline]
        $v fn push_date<Key: AsRef<str>>(&mut self, key: Key, value: Date) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_date(key, value)?;
            Ok(self)
        }

        /// Pushes a time value.
        #[inline]
        $v fn push_time<Key: AsRef<str>>(&mut self, key: Key, value: Time) -> BuildResult<&mut Self> {
            let key = key.as_ref();
            self.0.push_time(key, value)?;
            Ok(self)
        }
    };
}

macro_rules! impl_builder {
    ($builder: ty) => {
        impl $builder {
            impl_push_methods!(pub,);
        }

        impl ObjBuilder for $builder {
            impl_push_methods!(,);
        }
    };
}

impl_builder!(ObjectBuilder<'_>);
impl_builder!(ObjectRefBuilder<'_>);
