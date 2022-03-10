//! Array manipulation.

use crate::binary::{ARRAY_SIZE, DATA_TYPE_SIZE, ELEMENT_COUNT_SIZE, OBJECT_SIZE, VALUE_ENTRY_SIZE};
use crate::yason::object::Object;
use crate::yason::{Value, Yason, YasonError, YasonResult};
use crate::{DataType, Number};

/// An array in yason binary format.
pub struct Array<'a>(&'a Yason);

impl<'a> Array<'a> {
    /// Gets an iterator over the values of the array.
    #[inline]
    pub fn iter(&self) -> YasonResult<ArrayIter<'a>> {
        ArrayIter::try_new(self.0)
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

    /// Returns true if the array contains no elements.
    #[inline]
    pub fn is_empty(&self) -> YasonResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Gets the element at the given index.
    #[inline]
    pub fn get(&self, index: usize) -> YasonResult<Value<'a>> {
        let element_count = self.len()?;
        if index >= element_count {
            return Err(YasonError::IndexOutOfBounds {
                len: element_count,
                index,
            });
        }
        self.read_value(index)
    }

    /// Gets the element's type at the given index.
    #[inline]
    pub fn type_of(&self, index: usize) -> YasonResult<DataType> {
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.read_type(value_entry_pos)
    }

    /// Returns whether the element's type is the specified type at the given index.
    #[inline]
    pub fn is_type(&self, index: usize, data_type: DataType) -> YasonResult<bool> {
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
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Object)?;
        self.read_object(value_entry_pos)
    }

    /// Gets an array if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn array(&self, index: usize) -> YasonResult<Array<'a>> {
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Array)?;
        self.read_array(value_entry_pos)
    }

    /// Gets a string value if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn string(&self, index: usize) -> YasonResult<&'a str> {
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::String)?;
        self.read_string(value_entry_pos)
    }

    /// Gets a number value if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn number(&self, index: usize) -> YasonResult<Number> {
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Number)?;
        self.read_number(value_entry_pos)
    }

    /// Gets a bool value if the element at the given index has the correct type, returns `YasonError` otherwise.
    #[inline]
    pub fn bool(&self, index: usize) -> YasonResult<bool> {
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        self.0.check_type(value_entry_pos, DataType::Bool)?;
        self.read_bool(value_entry_pos)
    }
}

impl<'a> Array<'a> {
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
    fn read_object(&self, value_entry_pos: usize) -> YasonResult<Object<'a>> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        let size = self.read_size(value_pos)? as usize + DATA_TYPE_SIZE + OBJECT_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.0.slice(value_pos, value_pos + size)?) };
        Ok(unsafe { Object::new_unchecked(yason) })
    }

    #[inline]
    fn read_array(&self, value_entry_pos: usize) -> YasonResult<Array<'a>> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        let size = self.read_size(value_pos)? as usize + DATA_TYPE_SIZE + ARRAY_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.0.slice(value_pos, value_pos + size)?) };
        Ok(unsafe { Array::new_unchecked(yason) })
    }

    #[inline]
    fn read_string(&self, value_entry_pos: usize) -> YasonResult<&'a str> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        self.0.read_string(value_pos)
    }

    #[inline]
    fn read_number(&self, value_entry_pos: usize) -> YasonResult<Number> {
        let value_pos = self.read_value_pos(value_entry_pos)?;
        self.0.read_number(value_pos)
    }

    #[inline]
    fn read_bool(&self, value_entry_pos: usize) -> YasonResult<bool> {
        // bool can be inlined
        Ok(self.0.read_u32(value_entry_pos + DATA_TYPE_SIZE)? == 1)
    }

    #[inline]
    fn read_value(&self, index: usize) -> YasonResult<Value<'a>> {
        let value_entry_pos = DATA_TYPE_SIZE + ARRAY_SIZE + ELEMENT_COUNT_SIZE + index * VALUE_ENTRY_SIZE;
        let data_type = self.0.read_type(value_entry_pos)?;

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
