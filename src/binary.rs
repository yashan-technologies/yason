//! Yason binary format.

use crate::DataType;
use std::mem::size_of;

pub const DATA_TYPE_SIZE: usize = size_of::<DataType>();
pub const OBJECT_SIZE: usize = size_of::<i32>();
pub const ARRAY_SIZE: usize = OBJECT_SIZE;
pub const BOOL_SIZE: usize = size_of::<u8>();
pub const ELEMENT_COUNT_SIZE: usize = size_of::<u16>();
pub const KEY_OFFSET_SIZE: usize = size_of::<u32>();
pub const VALUE_ENTRY_SIZE: usize = DATA_TYPE_SIZE + size_of::<u32>();
pub const KEY_LENGTH_SIZE: usize = size_of::<u16>();
pub const MAX_DATA_LENGTH_SIZE: usize = size_of::<u32>();
pub const MAX_STRING_SIZE: usize = 268435455; // 2^28 - 1
pub const NUMBER_LENGTH_SIZE: usize = size_of::<u8>();
