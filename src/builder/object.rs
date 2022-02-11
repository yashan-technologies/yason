//! Object builder.

use crate::binary::{
    BOOL_SIZE, DATA_TYPE_SIZE, ELEMENT_COUNT_SIZE, KEY_LENGTH_SIZE, KEY_OFFSET_SIZE, MAX_DATA_LENGTH_SIZE,
    NUMBER_LENGTH_SIZE, OBJECT_SIZE,
};
use crate::builder::array::{ArrayRefBuilder, InnerArrayBuilder};
use crate::builder::{BuildResult, BytesWrapper, DEFAULT_SIZE};
use crate::util::cmp_key;
use crate::vec::VecExt;
use crate::yason::{Yason, YasonBuf};
use crate::{BuildError, DataType, Number};
use decimal_rs::MAX_BINARY_SIZE;
use std::ptr;

pub(crate) struct InnerObjectBuilder<B: AsMut<Vec<u8>>> {
    bytes_wrapper: BytesWrapper<B>,
    element_count: u16,
    start_pos: usize,
    key_offset_pos: usize,
    value_count: u16,
    depth: usize,
    bytes_init_len: usize,
    key_sorted: bool,
}

impl<B: AsMut<Vec<u8>>> InnerObjectBuilder<B> {
    #[inline]
    pub(crate) fn try_new(bytes: B, element_count: u16, key_sorted: bool) -> BuildResult<Self> {
        let mut bytes_wrapper = BytesWrapper::new(bytes);
        let bytes = bytes_wrapper.bytes.as_mut();
        let bytes_init_len = bytes.len();

        let size = DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE + KEY_OFFSET_SIZE * (element_count as usize);
        bytes.try_reserve(size)?;

        bytes.push_data_type(DataType::Object); // type
        bytes.skip_size(); // size
        let start_pos = bytes.len();
        bytes.push_u16(element_count); // element-count
        let key_offset_pos = bytes.len();
        bytes.skip_key_offset(element_count as usize); // key-offset
        bytes_wrapper.depth += 1;

        Ok(Self {
            depth: bytes_wrapper.depth,
            bytes_wrapper,
            element_count,
            start_pos,
            key_offset_pos,
            value_count: 0,
            bytes_init_len,
            key_sorted,
        })
    }

    #[inline]
    fn key_sorted(&mut self) -> bool {
        let bytes = self.bytes_wrapper.bytes.as_mut();

        let left = self.start_pos + ELEMENT_COUNT_SIZE;
        let right = left + self.element_count as usize * KEY_OFFSET_SIZE;

        let key_offsets_bytes = bytes[left..right].as_mut_ptr() as *mut u32;
        let key_offsets = unsafe { std::slice::from_raw_parts(key_offsets_bytes, (right - left) / 4) };

        for i in 0..key_offsets.len() - 1 {
            let left_key = Self::read_key_by_offset(bytes, key_offsets[i] as usize, self.start_pos);
            let right_key = Self::read_key_by_offset(bytes, key_offsets[i + 1] as usize, self.start_pos);
            if left_key.len() > right_key.len() {
                return false;
            } else if left_key.len() < right_key.len() {
                continue;
            } else if left_key > right_key {
                return false;
            } else {
                continue;
            }
        }
        true
    }

    #[inline]
    fn finish(&mut self) -> BuildResult<usize> {
        let bytes = self.bytes_wrapper.bytes.as_mut();

        if self.depth != self.bytes_wrapper.depth {
            return Err(BuildError::InnerUncompletedError);
        }

        if self.value_count != self.element_count {
            return Err(BuildError::InconsistentElementCount {
                expected: self.element_count,
                actual: self.value_count,
            });
        }

        let total_size = bytes.len() - self.start_pos;
        bytes.write_total_size(total_size as i32, self.start_pos - OBJECT_SIZE);

        self.bytes_wrapper.depth -= 1;

        debug_assert!(self.key_sorted());

        Ok(self.bytes_init_len)
    }

    #[inline]
    fn push_key_value_by<F>(&mut self, key: &str, reserved_size: usize, f: F) -> BuildResult<()>
    where
        F: FnOnce(&mut Vec<u8>) -> BuildResult<()>,
    {
        if self.depth != self.bytes_wrapper.depth {
            return Err(BuildError::InnerUncompletedError);
        }

        let bytes = self.bytes_wrapper.bytes.as_mut();
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
    fn binary_search(target: &str, bytes: &mut Vec<u8>, start_pos: usize, value_count: usize) -> usize {
        let begin = start_pos + ELEMENT_COUNT_SIZE;
        let end = begin + value_count * KEY_OFFSET_SIZE;

        let key_offsets_bytes = bytes[begin..end].as_ptr() as *mut u32;
        let key_offsets = unsafe { std::slice::from_raw_parts(key_offsets_bytes, (end - begin) / KEY_OFFSET_SIZE) };

        let found = key_offsets.binary_search_by(|key_offset| {
            let key = Self::read_key_by_offset(bytes, *key_offset as usize, start_pos);
            cmp_key(key, target)
        });

        match found {
            Ok(v) => v,
            Err(v) => v,
        }
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
    ) -> BuildResult<InnerObjectBuilder<&mut Vec<u8>>> {
        let size = key.len() + KEY_LENGTH_SIZE;
        self.push_key_value_by(key, size, |_| Ok(()))?;
        let bytes = self.bytes_wrapper.bytes.as_mut();
        InnerObjectBuilder::try_new(bytes, element_count, key_sorted)
    }

    #[inline]
    fn push_array(&mut self, key: &str, element_count: u16) -> BuildResult<InnerArrayBuilder<&mut Vec<u8>>> {
        let size = key.len() + KEY_LENGTH_SIZE;
        self.push_key_value_by(key, size, |_| Ok(()))?;
        let bytes = self.bytes_wrapper.bytes.as_mut();
        InnerArrayBuilder::try_new(bytes, element_count)
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
    fn push_number(&mut self, key: &str, value: Number) -> BuildResult<()> {
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
}

/// Builder for encoding an object.
#[repr(transparent)]
pub struct ObjectBuilder(InnerObjectBuilder<Vec<u8>>);

impl ObjectBuilder {
    /// Creates `ObjectBuilder` with specified element count.
    /// `key_sorted` indicates whether the object is sorted by key.
    #[inline]
    pub fn try_new(element_count: u16, key_sorted: bool) -> BuildResult<Self> {
        let bytes = Vec::try_with_capacity(DEFAULT_SIZE)?;
        let builder = InnerObjectBuilder::try_new(bytes, element_count, key_sorted)?;
        Ok(Self(builder))
    }

    /// Finishes building the object.
    #[inline]
    pub fn finish(mut self) -> BuildResult<YasonBuf> {
        self.0.finish()?;
        Ok(unsafe { YasonBuf::new_unchecked(self.0.bytes_wrapper.bytes) })
    }

    /// Pushes an embedded object with specified element count and a flag which indicates whether the embedded object is sorted by key.
    #[inline]
    pub fn push_object<Key: AsRef<str>>(
        &mut self,
        key: Key,
        element_count: u16,
        key_sorted: bool,
    ) -> BuildResult<ObjectRefBuilder> {
        let key = key.as_ref();
        let obj_builder = self.0.push_object(key, element_count, key_sorted)?;
        Ok(ObjectRefBuilder(obj_builder))
    }

    /// Pushes an embedded array with specified element count.
    #[inline]
    pub fn push_array<Key: AsRef<str>>(&mut self, key: Key, element_count: u16) -> BuildResult<ArrayRefBuilder> {
        let key = key.as_ref();
        let array_builder = self.0.push_array(key, element_count)?;
        Ok(ArrayRefBuilder(array_builder))
    }

    /// Pushes a string value.
    #[inline]
    pub fn push_string<Key: AsRef<str>, Val: AsRef<str>>(&mut self, key: Key, value: Val) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        let value = value.as_ref();
        self.0.push_string(key, value)?;
        Ok(self)
    }

    /// Pushes a number value.
    #[inline]
    pub fn push_number<Key: AsRef<str>>(&mut self, key: Key, value: Number) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        self.0.push_number(key, value)?;
        Ok(self)
    }

    /// Pushes a bool value.
    #[inline]
    pub fn push_bool<Key: AsRef<str>>(&mut self, key: Key, value: bool) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        self.0.push_bool(key, value)?;
        Ok(self)
    }

    /// Pushes a null value.
    #[inline]
    pub fn push_null<Key: AsRef<str>>(&mut self, key: Key) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        self.0.push_null(key)?;
        Ok(self)
    }
}

/// Builder for encoding an object.
#[repr(transparent)]
pub struct ObjectRefBuilder<'a>(pub(crate) InnerObjectBuilder<&'a mut Vec<u8>>);

impl<'a> ObjectRefBuilder<'a> {
    /// Creates `ObjectRefBuilder` with specified element count.
    /// `key_sorted` indicates whether the object is sorted by key.
    #[inline]
    pub fn try_new(bytes: &'a mut Vec<u8>, element_count: u16, key_sorted: bool) -> BuildResult<Self> {
        let obj_builder = InnerObjectBuilder::try_new(bytes, element_count, key_sorted)?;
        Ok(Self(obj_builder))
    }

    /// Finishes building the object.
    #[inline]
    pub fn finish(mut self) -> BuildResult<&'a Yason> {
        let bytes_init_len = self.0.finish()?;
        let bytes = self.0.bytes_wrapper.bytes;
        Ok(unsafe { Yason::new_unchecked(&bytes[bytes_init_len..]) })
    }

    /// Pushes an embedded object with specified element count and a flag which indicates whether the embedded object is sorted by key.
    #[inline]
    pub fn push_object<Key: AsRef<str>>(
        &mut self,
        key: Key,
        element_count: u16,
        key_sorted: bool,
    ) -> BuildResult<ObjectRefBuilder> {
        let key = key.as_ref();
        let obj_builder = self.0.push_object(key, element_count, key_sorted)?;
        Ok(ObjectRefBuilder(obj_builder))
    }

    /// Pushes an embedded array with specified element count.
    #[inline]
    pub fn push_array<Key: AsRef<str>>(&mut self, key: Key, element_count: u16) -> BuildResult<ArrayRefBuilder> {
        let key = key.as_ref();
        let array_builder = self.0.push_array(key, element_count)?;
        Ok(ArrayRefBuilder(array_builder))
    }

    /// Pushes a string value.
    #[inline]
    pub fn push_string<Key: AsRef<str>, Val: AsRef<str>>(&mut self, key: Key, value: Val) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        let value = value.as_ref();
        self.0.push_string(key, value)?;
        Ok(self)
    }

    /// Pushes a number value.
    #[inline]
    pub fn push_number<Key: AsRef<str>>(&mut self, key: Key, value: Number) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        self.0.push_number(key, value)?;
        Ok(self)
    }

    /// Pushes a bool value.
    #[inline]
    pub fn push_bool<Key: AsRef<str>>(&mut self, key: Key, value: bool) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        self.0.push_bool(key, value)?;
        Ok(self)
    }

    /// Pushes a null value.
    #[inline]
    pub fn push_null<Key: AsRef<str>>(&mut self, key: Key) -> BuildResult<&mut Self> {
        let key = key.as_ref();
        self.0.push_null(key)?;
        Ok(self)
    }
}
