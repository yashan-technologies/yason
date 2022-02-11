//! Vec extension.

use crate::binary::{KEY_OFFSET_SIZE, MAX_STRING_SIZE, NUMBER_LENGTH_SIZE, OBJECT_SIZE, VALUE_ENTRY_SIZE};
use crate::builder::BuildResult;
use crate::util::encode_varint;
use crate::{BuildError, DataType, Number};
use decimal_rs::MAX_BINARY_SIZE;
use std::collections::TryReserveError;
use std::mem::size_of;

pub trait VecExt: Sized {
    fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError>;
    fn push_u8(&mut self, val: u8);
    fn push_u16(&mut self, val: u16);
    fn push_i32(&mut self, val: i32);
    fn push_data_type(&mut self, data_type: DataType);
    fn write_data_type_by_pos(&mut self, data_type: DataType, type_pos: usize);
    fn push_str(&mut self, s: &str);
    fn skip_size(&mut self);
    fn skip_key_offset(&mut self, element_count: usize);
    fn skip_value_entry(&mut self, element_count: usize);
    fn write_total_size(&mut self, size: i32, size_pos: usize);
    fn write_offset(&mut self, offset: u32, offset_pos: usize);
    fn push_bytes(&mut self, bytes: &[u8]);
    fn push_data_length(&mut self, length: usize) -> BuildResult<()>;
    fn push_key(&mut self, s: &str);
    fn push_string(&mut self, s: &str) -> BuildResult<()>;
    fn push_number(&mut self, value: Number);
}

impl VecExt for Vec<u8> {
    #[inline]
    fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut vec = Vec::new();
        vec.try_reserve(capacity)?;
        Ok(vec)
    }

    #[inline]
    fn push_u8(&mut self, val: u8) {
        debug_assert!(size_of::<u8>() <= self.capacity() - self.len());
        self.push(val);
    }

    #[inline]
    fn push_u16(&mut self, val: u16) {
        debug_assert!(size_of::<u16>() <= self.capacity() - self.len());
        self.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn push_i32(&mut self, val: i32) {
        debug_assert!(size_of::<i32>() <= self.capacity() - self.len());
        self.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn push_data_type(&mut self, data_type: DataType) {
        self.push_u8(data_type as u8);
    }

    #[inline]
    fn write_data_type_by_pos(&mut self, data_type: DataType, type_pos: usize) {
        debug_assert!(type_pos < self.len());
        self[type_pos] = data_type as u8;
    }

    #[inline]
    fn push_str(&mut self, s: &str) {
        debug_assert!(s.len() <= self.capacity() - self.len());
        self.extend_from_slice(s.as_bytes());
    }

    #[inline]
    fn skip_size(&mut self) {
        let new_len = self.len() + OBJECT_SIZE;
        debug_assert!(new_len <= self.capacity());
        unsafe {
            self.set_len(new_len);
        }
    }

    #[inline]
    fn skip_key_offset(&mut self, element_count: usize) {
        let new_len = self.len() + element_count * KEY_OFFSET_SIZE;
        debug_assert!(new_len <= self.capacity());
        unsafe {
            self.set_len(new_len);
        }
    }

    #[inline]
    fn skip_value_entry(&mut self, element_count: usize) {
        let new_len = self.len() + element_count * VALUE_ENTRY_SIZE;
        debug_assert!(new_len <= self.capacity());
        unsafe {
            self.set_len(new_len);
        }
    }

    #[inline]
    fn write_total_size(&mut self, size: i32, size_pos: usize) {
        debug_assert!(size_pos + OBJECT_SIZE <= self.len());
        let s = &mut self[size_pos..size_pos + OBJECT_SIZE];
        s.copy_from_slice(&size.to_le_bytes());
    }

    #[inline]
    fn write_offset(&mut self, offset: u32, offset_pos: usize) {
        debug_assert!(offset_pos + KEY_OFFSET_SIZE <= self.len());
        let s = &mut self[offset_pos..offset_pos + KEY_OFFSET_SIZE];
        s.copy_from_slice(&offset.to_le_bytes());
    }

    #[inline]
    fn push_bytes(&mut self, bytes: &[u8]) {
        debug_assert!(bytes.len() <= self.capacity() - self.len());
        self.extend_from_slice(bytes)
    }

    #[inline]
    fn push_data_length(&mut self, length: usize) -> BuildResult<()> {
        if length > MAX_STRING_SIZE {
            return Err(BuildError::StringTooLong(length));
        }
        encode_varint(length as u32, self);
        Ok(())
    }

    #[inline]
    fn push_key(&mut self, s: &str) {
        self.push_u16(s.len() as u16);
        self.push_str(s);
    }

    #[inline]
    fn push_string(&mut self, s: &str) -> BuildResult<()> {
        self.push_data_length(s.len())?;
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn push_number(&mut self, value: Number) {
        let length_pos = self.len();
        let value_pos = length_pos + NUMBER_LENGTH_SIZE;
        let new_len = value_pos + MAX_BINARY_SIZE;
        debug_assert!(NUMBER_LENGTH_SIZE + MAX_BINARY_SIZE <= self.capacity() - self.len());
        unsafe {
            self.set_len(new_len);
        }
        let bytes = &mut self[value_pos..value_pos + MAX_BINARY_SIZE];
        // SAFETY: Because we have ensured that the memory is sufficient before encoding.
        let size = value.compact_encode(bytes).expect("failed to encode number");
        self[length_pos] = size as u8;
        unsafe {
            self.set_len(value_pos + size);
        }
    }
}
