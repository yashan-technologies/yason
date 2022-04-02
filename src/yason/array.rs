//! Array manipulation.

use crate::binary::{ARRAY_SIZE, DATA_TYPE_SIZE, ELEMENT_COUNT_SIZE, OBJECT_SIZE, VALUE_ENTRY_SIZE};
use crate::yason::object::Object;
use crate::yason::{LazyValue, Value, Yason, YasonError, YasonResult};
use crate::{DataType, Number};

/// An array in yason binary format.
#[derive(Clone)]
#[repr(transparent)]
pub struct Array<'a>(&'a Yason);

impl<'a> Array<'a> {
    /// Gets an iterator over the values of the array.
    #[inline]
    pub fn iter(&self) -> YasonResult<ArrayIter<'a>> {
        ArrayIter::try_new(self.0)
    }

    #[inline]
    pub(crate) fn lazy_iter(&self) -> YasonResult<LazyArrayIter<'a>> {
        LazyArrayIter::try_new(self.0)
    }

    /// Creates an `Array`.
    ///
    /// # Safety
    ///
    /// Callers should guarantee the `yason` is a valid `Array`.
    #[inline]
    pub const unsafe fn new_unchecked(yason: &'a Yason) -> Self {
        debug_assert!(yason.bytes.len() >= DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE);
        Self(yason)
    }

    /// Returns the number of elements in the array.
    #[inline]
    pub fn len(&self) -> YasonResult<usize> {
        Ok(self.0.read_u16(DATA_TYPE_SIZE + ARRAY_SIZE)? as usize)
    }

    #[inline]
    pub fn yason(&self) -> &Yason {
        self.0
    }

    /// Returns true if the array contains no elements.
    #[inline]
    pub fn is_empty(&self) -> YasonResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Gets the element at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> YasonResult<Value<'a>> {
        self.check_index(index)?;
        self.read_value(index)
    }

    #[inline]
    pub(crate) unsafe fn lazy_get_unchecked(&self, index: usize) -> YasonResult<LazyValue<'a, true>> {
        debug_assert!(index < self.len()?);
        let (data_type, value_entry_pos) = self.read_type_and_value_entry_pos(index)?;
        Ok(LazyValue::new(self.0, data_type, value_entry_pos))
    }

    /// Gets the element's type at the given index.
    #[inline]
    pub fn type_of(&self, index: usize) -> YasonResult<DataType> {
        self.check_index(index)?;
        Ok(unsafe { self.read_type_and_value_entry_pos(index)?.0 })
    }

    /// Returns whether the element's type is the specified type at the given index.
    #[inline]
    pub fn is_type(&self, index: usize, data_type: DataType) -> YasonResult<bool> {
        self.check_index(index)?;
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.is_type(value_entry_pos, data_type as u8)
    }

    /// Returns whether the element is a null value at the given index.
    #[inline]
    pub fn is_null(&self, index: usize) -> YasonResult<bool> {
        self.is_type(index, DataType::Null)
    }

    /// Gets an object if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn object(&self, index: usize) -> YasonResult<Object<'a>> {
        self.check_index(index)?;
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Object)?;
        self.read_object(value_entry_pos)
    }

    /// Gets an array if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn array(&self, index: usize) -> YasonResult<Array<'a>> {
        self.check_index(index)?;
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Array)?;
        self.read_array(value_entry_pos)
    }

    /// Gets a string value if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn string(&self, index: usize) -> YasonResult<&'a str> {
        self.check_index(index)?;
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::String)?;
        self.read_string(value_entry_pos)
    }

    /// Gets a number value if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn number(&self, index: usize) -> YasonResult<Number> {
        self.check_index(index)?;
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Number)?;
        self.read_number(value_entry_pos)
    }

    /// Gets a bool value if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn bool(&self, index: usize) -> YasonResult<bool> {
        self.check_index(index)?;
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Bool)?;
        self.read_bool(value_entry_pos)
    }
}

impl<'a> Array<'a> {
    #[inline]
    unsafe fn read_type_and_value_entry_pos(&self, index: usize) -> YasonResult<(DataType, usize)> {
        debug_assert!(index < self.len()?);
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        let data_type = self.0.read_type(value_entry_pos)?;
        Ok((data_type, value_entry_pos))
    }

    #[inline]
    fn check_index(&self, index: usize) -> YasonResult<()> {
        let element_count = self.len()?;
        if index >= element_count {
            return Err(YasonError::IndexOutOfBounds {
                len: element_count,
                index,
            });
        }
        Ok(())
    }

    #[inline]
    fn read_value_pos(&self, value_entry_pos: usize) -> YasonResult<usize> {
        let value_offset = self.0.read_u32(value_entry_pos + DATA_TYPE_SIZE)? as usize;
        Ok(value_offset + DATA_TYPE_SIZE + ARRAY_SIZE)
    }

    #[inline]
    fn read_size(&self, value_pos: usize) -> YasonResult<i32> {
        let size_pos = value_pos + DATA_TYPE_SIZE;
        self.0.read_i32(size_pos)
    }

    #[inline]
    pub(crate) fn read_object(&self, value_entry_pos: usize) -> YasonResult<Object<'a>> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        let size = self.read_size(value_pos)? as usize + DATA_TYPE_SIZE + OBJECT_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.0.slice(value_pos, value_pos + size)?) };
        Ok(unsafe { Object::new_unchecked(yason) })
    }

    #[inline]
    pub(crate) fn read_array(&self, value_entry_pos: usize) -> YasonResult<Array<'a>> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        let size = self.read_size(value_pos)? as usize + DATA_TYPE_SIZE + ARRAY_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.0.slice(value_pos, value_pos + size)?) };
        Ok(unsafe { Array::new_unchecked(yason) })
    }

    #[inline]
    pub(crate) fn read_string(&self, value_entry_pos: usize) -> YasonResult<&'a str> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        self.0.read_string(value_pos)
    }

    #[inline]
    pub(crate) fn read_number(&self, value_entry_pos: usize) -> YasonResult<Number> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        self.0.read_number(value_pos)
    }

    #[inline]
    pub(crate) fn read_bool(&self, value_entry_pos: usize) -> YasonResult<bool> {
        // bool can be inlined
        Ok(self.0.read_u32(value_entry_pos + DATA_TYPE_SIZE)? == 1)
    }

    #[inline]
    fn read_value(&self, index: usize) -> YasonResult<Value<'a>> {
        let (data_type, value_entry_pos) = unsafe { self.read_type_and_value_entry_pos(index)? };

        let value = match data_type {
            DataType::Object => Value::Object(self.read_object(value_entry_pos)?),
            DataType::Array => Value::Array(self.read_array(value_entry_pos)?),
            DataType::String => Value::String(self.read_string(value_entry_pos)?),
            DataType::Number => Value::Number(self.read_number(value_entry_pos)?),
            DataType::Bool => Value::Bool(self.read_bool(value_entry_pos)?),
            DataType::Null => Value::Null,
        };
        Ok(value)
    }
}

/// An iterator over the array's elements.
pub struct ArrayIter<'a> {
    array: Array<'a>,
    len: usize,
    index: usize,
}

impl<'a> ArrayIter<'a> {
    #[inline]
    fn try_new(yason: &'a Yason) -> YasonResult<ArrayIter<'a>> {
        let array = Array(yason);
        Ok(Self {
            len: array.len()?,
            array,
            index: 0,
        })
    }
}

impl<'a> Iterator for ArrayIter<'a> {
    type Item = YasonResult<Value<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let value = self.array.read_value(self.index);
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

pub struct LazyArrayIter<'a> {
    array: Array<'a>,
    len: usize,
    index: usize,
}

impl<'a> LazyArrayIter<'a> {
    #[inline]
    fn try_new(yason: &'a Yason) -> YasonResult<LazyArrayIter<'a>> {
        let array = Array(yason);
        Ok(Self {
            len: array.len()?,
            array,
            index: 0,
        })
    }

    #[inline]
    fn next(&mut self) -> YasonResult<LazyValue<'a, true>> {
        let (data_type, value_entry_pos) = unsafe { self.array.read_type_and_value_entry_pos(self.index)? };
        self.index += 1;
        Ok(LazyValue::new(self.array.0, data_type, value_entry_pos))
    }
}

impl<'a> Iterator for LazyArrayIter<'a> {
    type Item = YasonResult<LazyValue<'a, true>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            Some(self.next())
        } else {
            None
        }
    }
}
