//! Array builder.

use crate::binary::{
    ARRAY_SIZE, DATA_TYPE_SIZE, ELEMENT_COUNT_SIZE, MAX_DATA_LENGTH_SIZE, NUMBER_LENGTH_SIZE, VALUE_ENTRY_SIZE,
};
use crate::builder::object::InnerObjectBuilder;
use crate::builder::{BuildResult, Depth, DEFAULT_SIZE, MAX_NESTED_DEPTH};
use crate::vec::VecExt;
use crate::yason::{Yason, YasonBuf};
use crate::{BuildError, DataType, Number, ObjectRefBuilder};
use decimal_rs::MAX_BINARY_SIZE;

pub(crate) struct InnerArrayBuilder<'a, B: AsMut<Vec<u8>>> {
    bytes: B,
    element_count: u16,
    start_pos: usize,
    value_entry_pos: usize,
    value_count: u16,
    bytes_init_len: usize,
    current_depth: usize,
    total_nested_depth: Depth<'a>,
}

impl<'a, B: AsMut<Vec<u8>>> InnerArrayBuilder<'a, B> {
    #[inline]
    pub(crate) fn try_new(mut bytes: B, element_count: u16, mut total_depth: Depth<'a>) -> BuildResult<Self> {
        if total_depth.depth() >= MAX_NESTED_DEPTH {
            return Err(BuildError::NestedTooDeeply);
        }

        let bs = bytes.as_mut();
        let bytes_init_len = bs.len();

        let size = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + VALUE_ENTRY_SIZE * element_count as usize;
        bs.try_reserve(size)?;

        bs.push_data_type(DataType::Array); // type
        bs.skip_size(); // size
        let start_pos = bs.len();
        bs.push_u16(element_count); // element-count
        let value_entry_pos = bs.len();
        bs.skip_value_entry(element_count as usize); // value-entry

        total_depth.increase();

        Ok(Self {
            bytes,
            element_count,
            start_pos,
            value_entry_pos,
            value_count: 0,
            bytes_init_len,
            current_depth: total_depth.depth(),
            total_nested_depth: total_depth,
        })
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
        bytes.write_total_size(total_size as i32, self.start_pos - ARRAY_SIZE);

        self.total_nested_depth.decrease();

        Ok(self.bytes_init_len)
    }

    #[inline]
    fn push_value<F>(&mut self, data_type: DataType, f: F) -> BuildResult<()>
    where
        F: FnOnce(&mut Vec<u8>, u32, usize) -> BuildResult<()>,
    {
        if self.current_depth != self.total_nested_depth.depth() {
            return Err(BuildError::InnerUncompletedError);
        }

        let bytes = self.bytes.as_mut();
        bytes.write_data_type_by_pos(data_type, self.value_entry_pos);
        let offset = bytes.len() - self.start_pos;

        f(bytes, offset as u32, self.value_entry_pos)?;

        self.value_entry_pos += VALUE_ENTRY_SIZE;
        self.value_count += 1;
        Ok(())
    }

    #[inline]
    fn push_object(&mut self, element_count: u16, key_sorted: bool) -> BuildResult<InnerObjectBuilder<&mut Vec<u8>>> {
        let f = |bytes: &mut Vec<u8>, offset: u32, value_entry_pos: usize| {
            bytes.write_offset(offset, value_entry_pos + DATA_TYPE_SIZE);
            Ok(())
        };
        self.push_value(DataType::Object, f)?;

        let bytes = self.bytes.as_mut();
        InnerObjectBuilder::try_new(bytes, element_count, key_sorted, self.total_nested_depth.borrow_mut())
    }

    #[inline]
    fn push_array(&mut self, element_count: u16) -> BuildResult<InnerArrayBuilder<&mut Vec<u8>>> {
        let f = |bytes: &mut Vec<u8>, offset: u32, value_entry_pos: usize| {
            bytes.write_offset(offset, value_entry_pos + DATA_TYPE_SIZE);
            Ok(())
        };
        self.push_value(DataType::Array, f)?;

        let bytes = self.bytes.as_mut();
        InnerArrayBuilder::try_new(bytes, element_count, self.total_nested_depth.borrow_mut())
    }

    #[inline]
    fn push_string(&mut self, value: &str) -> BuildResult<()> {
        let size = MAX_DATA_LENGTH_SIZE + value.len();
        let f = |bytes: &mut Vec<u8>, offset: u32, value_entry_pos: usize| {
            bytes.write_offset(offset, value_entry_pos + DATA_TYPE_SIZE);
            bytes.try_reserve(size)?;
            bytes.push_string(value)?;
            Ok(())
        };
        self.push_value(DataType::String, f)
    }

    #[inline]
    fn push_number(&mut self, value: &Number) -> BuildResult<()> {
        let size = MAX_BINARY_SIZE + NUMBER_LENGTH_SIZE;
        let f = |bytes: &mut Vec<u8>, offset: u32, value_entry_pos: usize| {
            bytes.write_offset(offset, value_entry_pos + DATA_TYPE_SIZE);
            bytes.try_reserve(size)?;
            bytes.push_number(value);
            Ok(())
        };
        self.push_value(DataType::Number, f)
    }

    #[inline]
    fn push_bool(&mut self, value: bool) -> BuildResult<()> {
        // bool can be inlined
        let f = |bytes: &mut Vec<u8>, _offset: u32, value_entry_pos: usize| {
            bytes.write_offset(value as u32, value_entry_pos + DATA_TYPE_SIZE);
            Ok(())
        };
        self.push_value(DataType::Bool, f)
    }

    #[inline]
    fn push_null(&mut self) -> BuildResult<()> {
        // null can be inlined
        self.push_value(DataType::Null, |_, _, _| Ok(()))
    }

    #[inline]
    unsafe fn push_object_or_array(&mut self, yason: &Yason, data_type: DataType) -> BuildResult<()> {
        let value = yason.as_bytes();
        let size = value.len();
        let f = |bytes: &mut Vec<u8>, offset: u32, value_entry_pos: usize| {
            bytes.write_offset(offset, value_entry_pos + DATA_TYPE_SIZE);
            bytes.try_reserve(size)?;
            bytes.extend_from_slice(value);
            Ok(())
        };
        self.push_value(data_type, f)
    }
}

/// Builder for encoding an array.
#[repr(transparent)]
pub struct ArrayBuilder<'a>(InnerArrayBuilder<'a, Vec<u8>>);

impl ArrayBuilder<'_> {
    /// Creates `ArrayBuilder` with specified element count.
    #[inline]
    pub fn try_new(element_count: u16) -> BuildResult<Self> {
        let bytes = Vec::try_with_capacity(DEFAULT_SIZE)?;
        let builder = InnerArrayBuilder::try_new(bytes, element_count, Depth::new())?;
        Ok(Self(builder))
    }

    /// Finishes building the array.
    #[inline]
    pub fn finish(mut self) -> BuildResult<YasonBuf> {
        self.0.finish()?;
        Ok(unsafe { YasonBuf::new_unchecked(self.0.bytes) })
    }
}

/// Builder for encoding an array.
#[repr(transparent)]
pub struct ArrayRefBuilder<'a>(pub(crate) InnerArrayBuilder<'a, &'a mut Vec<u8>>);

impl<'a> ArrayRefBuilder<'a> {
    /// Creates `ArrayRefBuilder` with specified element count.
    #[inline]
    pub fn try_new(bytes: &'a mut Vec<u8>, element_count: u16) -> BuildResult<Self> {
        let array_builder = InnerArrayBuilder::try_new(bytes, element_count, Depth::new())?;
        Ok(Self(array_builder))
    }

    /// Finishes building the array.
    #[inline]
    pub fn finish(mut self) -> BuildResult<&'a Yason> {
        let bytes_init_len = self.0.finish()?;
        let bytes = self.0.bytes;
        Ok(unsafe { Yason::new_unchecked(&bytes[bytes_init_len..]) })
    }

    #[inline]
    pub(crate) unsafe fn push_object_or_array(&mut self, yason: &Yason, data_type: DataType) -> BuildResult<&mut Self> {
        debug_assert!(matches!(yason.data_type().unwrap(), DataType::Object | DataType::Array));
        debug_assert!(yason.data_type().unwrap() == data_type);
        self.0.push_object_or_array(yason, data_type)?;
        Ok(self)
    }
}

pub trait ArrBuilder {
    /// Pushes an embedded object with specified element count and a flag which indicates whether the embedded object is sorted by key.
    fn push_object(&mut self, element_count: u16, key_sorted: bool) -> BuildResult<ObjectRefBuilder>;

    /// Pushes an embedded array with specified element count.
    fn push_array(&mut self, element_count: u16) -> BuildResult<ArrayRefBuilder>;

    /// Pushes a string value.
    fn push_string<Val: AsRef<str>>(&mut self, value: Val) -> BuildResult<&mut Self>;

    /// Pushes a number value.
    fn push_number<Num: AsRef<Number>>(&mut self, value: Num) -> BuildResult<&mut Self>;

    /// Pushes a bool value.
    fn push_bool(&mut self, value: bool) -> BuildResult<&mut Self>;

    /// Pushes a null value.
    fn push_null(&mut self) -> BuildResult<&mut Self>;
}

macro_rules! impl_push_methods {
    ($v: vis,) => {
        /// Pushes an embedded object with specified element count and a flag which indicates whether the embedded object is sorted by key.
        #[inline]
        $v fn push_object(&mut self, element_count: u16, key_sorted: bool) -> BuildResult<ObjectRefBuilder> {
            let obj_builder = self.0.push_object(element_count, key_sorted)?;
            Ok(ObjectRefBuilder(obj_builder))
        }

        /// Pushes an embedded array with specified element count.
        #[inline]
        $v fn push_array(&mut self, element_count: u16) -> BuildResult<ArrayRefBuilder> {
            let array_builder = self.0.push_array(element_count)?;
            Ok(ArrayRefBuilder(array_builder))
        }

        /// Pushes a string value.
        #[inline]
        $v fn push_string<Val: AsRef<str>>(&mut self, value: Val) -> BuildResult<&mut Self> {
            let value = value.as_ref();
            self.0.push_string(value)?;
            Ok(self)
        }

        /// Pushes a number value.
        #[inline]
        $v fn push_number<Num: AsRef<Number>>(&mut self, value: Num) -> BuildResult<&mut Self> {
            self.0.push_number(value.as_ref())?;
            Ok(self)
        }

        /// Pushes a bool value.
        #[inline]
        $v fn push_bool(&mut self, value: bool) -> BuildResult<&mut Self> {
            self.0.push_bool(value)?;
            Ok(self)
        }

        /// Pushes a null value.
        #[inline]
        $v fn push_null(&mut self) -> BuildResult<&mut Self> {
            self.0.push_null()?;
            Ok(self)
        }
    };
}

macro_rules! impl_builder {
    ($builder: ty) => {
        impl $builder {
            impl_push_methods!(pub,);
        }

        impl ArrBuilder for $builder {
            impl_push_methods!(,);
        }
    };
}

impl_builder!(ArrayBuilder<'_>);
impl_builder!(ArrayRefBuilder<'_>);
