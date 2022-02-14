//! Object manipulation.

use crate::binary::{ARRAY_SIZE, DATA_TYPE_SIZE, ELEMENT_COUNT_SIZE, KEY_LENGTH_SIZE, KEY_OFFSET_SIZE, OBJECT_SIZE};
use crate::yason::array::Array;
use crate::yason::{Value, Yason, YasonResult};
use crate::{DataType, Number};

/// An object in yason binary format.
pub struct Object<'a>(&'a Yason);

impl<'a> Object<'a> {
    /// Gets an iterator over the entries of the object.
    #[inline]
    pub fn iter(&self) -> YasonResult<ObjectIter> {
        ObjectIter::try_new(self)
    }

    /// Gets an iterator over the keys of the object.
    #[inline]
    pub fn key_iter(&self) -> YasonResult<KeyIter> {
        KeyIter::try_new(self)
    }

    /// Gets an iterator over the keys of the object.
    #[inline]
    pub fn value_iter(&self) -> YasonResult<ValueIter> {
        ValueIter::try_new(self)
    }

    /// Creates an `Object`.
    ///
    /// # Safety
    ///
    /// Callers should guarantee the `yason` is a valid `Object`.
    pub const unsafe fn new_unchecked(yason: &'a Yason) -> Self {
        debug_assert!(yason.bytes.len() >= DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE);
        Self(yason)
    }

    /// Returns the number of elements in the object.
    #[inline]
    pub fn len(&self) -> YasonResult<usize> {
        Ok(self.0.read_u16(DATA_TYPE_SIZE + OBJECT_SIZE)? as usize)
    }

    /// Returns true if the object contains no elements.
    #[inline]
    pub fn is_empty(&self) -> YasonResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Returns the value corresponding to the key, if it exists.
    #[inline]
    pub fn get<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Value>> {
        let found = self.find_key(key.as_ref())?;
        if let Some(value_pos) = found {
            return Ok(Some(self.read_value(value_pos)?));
        }

        Ok(None)
    }

    /// Returns the value's type corresponding to the key, if it exists.
    #[inline]
    pub fn type_of<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<DataType>> {
        let found = self.find_key(key.as_ref())?;
        if let Some(value_pos) = found {
            return Ok(Some(self.0.read_type(value_pos)?));
        }

        Ok(None)
    }

    /// Returns whether the value's type is the specified type corresponding to the key, if it exists.
    #[inline]
    pub fn is_type<T: AsRef<str>>(&self, key: T, data_type: DataType) -> YasonResult<Option<bool>> {
        let found = self.find_key(key.as_ref())?;
        if let Some(value_pos) = found {
            return Ok(Some(self.0.is_type(value_pos, data_type as u8)?));
        }

        Ok(None)
    }

    /// Returns whether this key has a null value.
    #[inline]
    pub fn is_null<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<bool>> {
        self.is_type(key, DataType::Null)
    }

    /// Returns true if the object contains a value for the specified key.
    #[inline]
    pub fn contains_key<T: AsRef<str>>(&self, key: T) -> YasonResult<bool> {
        let found = self.find_key(key.as_ref())?;
        Ok(found.is_some())
    }

    /// Gets an object for this key if it exists and has the correct type, returns `None` if this
    /// key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn object<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Object>> {
        let found = self.check_key(key.as_ref(), DataType::Object)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.read_object(value_pos)?));
        }
        Ok(None)
    }

    /// Gets an array for this key if it exists and has the correct type, returns `None` if this
    /// key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn array<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Array>> {
        let found = self.check_key(key.as_ref(), DataType::Array)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.read_array(value_pos)?));
        }
        Ok(None)
    }

    /// Gets a string value for this key if it exists and has the correct type, returns `None` if
    /// this key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn string<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<&str>> {
        let found = self.check_key(key.as_ref(), DataType::String)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.read_string(value_pos)?));
        }
        Ok(None)
    }

    /// Gets a number value for this key if it exists and has the correct type, returns `None`
    /// if this key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn number<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Number>> {
        let found = self.check_key(key.as_ref(), DataType::Number)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.read_number(value_pos)?));
        }
        Ok(None)
    }

    /// Gets a bool value for this key if it exists and has the correct type, returns `None` if
    /// this key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn bool<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<bool>> {
        let found = self.check_key(key.as_ref(), DataType::Bool)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.read_bool(value_pos)?));
        }
        Ok(None)
    }
}

impl<'a> Object<'a> {
    #[inline]
    fn skip_key(&self, offset: usize) -> YasonResult<usize> {
        let key_pos = offset + DATA_TYPE_SIZE + OBJECT_SIZE;
        let key_len = self.0.read_u16(key_pos)? as usize;
        Ok(key_pos + KEY_LENGTH_SIZE + key_len)
    }

    #[inline]
    fn read_key(&self, key_offset: usize) -> YasonResult<(&str, usize)> {
        let len_pos = key_offset + DATA_TYPE_SIZE + OBJECT_SIZE;
        let len = self.0.read_u16(len_pos)? as usize;
        let key_pos = len_pos + KEY_LENGTH_SIZE;
        let bytes = self.0.slice(key_pos, key_pos + len)?;
        let key = unsafe { std::str::from_utf8_unchecked(bytes) };
        Ok((key, key_pos + len))
    }

    #[inline]
    fn read_key_offset(&self, offset_pos: usize) -> YasonResult<u32> {
        self.0.read_u32(offset_pos)
    }

    #[inline]
    fn read_size(&self, value_pos: usize) -> YasonResult<i32> {
        let size_pos = value_pos + DATA_TYPE_SIZE;
        self.0.read_i32(size_pos)
    }

    #[inline]
    fn read_object(&self, value_pos: usize) -> YasonResult<Object> {
        let size = self.read_size(value_pos)? as usize + DATA_TYPE_SIZE + OBJECT_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.0.slice(value_pos, value_pos + size)?) };
        Ok(unsafe { Object::new_unchecked(yason) })
    }

    #[inline]
    fn read_array(&self, value_pos: usize) -> YasonResult<Array> {
        let size = self.read_size(value_pos)? as usize + DATA_TYPE_SIZE + ARRAY_SIZE;
        let yason = unsafe { Yason::new_unchecked(self.0.slice(value_pos, value_pos + size)?) };
        Ok(unsafe { Array::new_unchecked(yason) })
    }

    #[inline]
    fn read_string(&self, value_pos: usize) -> YasonResult<&str> {
        self.0.read_string(value_pos + DATA_TYPE_SIZE)
    }

    #[inline]
    fn read_number(&self, value_pos: usize) -> YasonResult<Number> {
        self.0.read_number(value_pos + DATA_TYPE_SIZE)
    }

    #[inline]
    fn read_bool(&self, value_pos: usize) -> YasonResult<bool> {
        Ok(self.0.read_u8(value_pos + DATA_TYPE_SIZE)? == 1)
    }

    #[inline]
    fn read_value(&self, value_pos: usize) -> YasonResult<Value> {
        let data_type = self.0.read_type(value_pos)?;
        let value = match data_type {
            DataType::Object => Value::Object(self.read_object(value_pos)?),
            DataType::Array => Value::Array(self.read_array(value_pos)?),
            DataType::String => Value::String(self.read_string(value_pos)?),
            DataType::Number => Value::Number(self.read_number(value_pos)?),
            DataType::Bool => Value::Bool(self.read_bool(value_pos)?),
            DataType::Null => Value::Null,
        };
        Ok(value)
    }

    #[inline]
    fn find_key(&self, key: &str) -> YasonResult<Option<usize>> {
        let mut left = 0;
        let mut right = self.len()?;

        while left < right {
            let mid = left + (right - left) / 2;
            let key_offset_pos = DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE + mid * KEY_OFFSET_SIZE;
            let key_offset = self.read_key_offset(key_offset_pos)?;
            let (cur_key, value_pos) = self.read_key(key_offset as usize)?;
            if cur_key.len() < key.len() {
                left = mid + 1;
            } else if cur_key.len() > key.len() {
                right = mid;
            } else if cur_key < key {
                left = mid + 1;
            } else if cur_key > key {
                right = mid
            } else {
                return Ok(Some(value_pos));
            }
        }
        Ok(None)
    }

    #[inline]
    fn check_key(&self, key: &str, expected: DataType) -> YasonResult<Option<usize>> {
        let found = self.find_key(key.as_ref())?;
        if let Some(value_pos) = found {
            self.0.check_type(value_pos, expected)?;
            return Ok(Some(value_pos));
        }
        Ok(None)
    }
}

/// An iterator over the object's entries.
pub struct ObjectIter<'a> {
    object: &'a Object<'a>,
    len: usize,
    index: usize,
}

impl<'a> ObjectIter<'a> {
    #[inline]
    fn try_new(object: &'a Object) -> YasonResult<Self> {
        Ok(Self {
            object,
            len: object.len()?,
            index: 0,
        })
    }

    #[inline]
    fn next_entry(&mut self) -> YasonResult<(&'a str, Value<'a>)> {
        let key_offset_pos = DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE + self.index * KEY_OFFSET_SIZE;
        let key_offset = self.object.read_key_offset(key_offset_pos)?;
        let (key, value_pos) = self.object.read_key(key_offset as usize)?;
        let value = self.object.read_value(value_pos)?;
        Ok((key, value))
    }

    #[inline]
    fn next_key(&mut self) -> YasonResult<&'a str> {
        let key_offset_pos = DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE + self.index * KEY_OFFSET_SIZE;
        let key_offset = self.object.read_key_offset(key_offset_pos)?;
        let (key, _) = self.object.read_key(key_offset as usize)?;
        Ok(key)
    }

    #[inline]
    fn next_value(&mut self) -> YasonResult<Value<'a>> {
        let key_offset_pos = DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE + self.index * KEY_OFFSET_SIZE;
        let key_offset = self.object.read_key_offset(key_offset_pos)?;
        let value_pos = self.object.skip_key(key_offset as usize)?;
        let value = self.object.read_value(value_pos)?;
        Ok(value)
    }
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = YasonResult<(&'a str, Value<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let entry = self.next_entry();
            self.index += 1;
            Some(entry)
        } else {
            None
        }
    }
}

/// An iterator over the object's keys.
pub struct KeyIter<'a> {
    inner: ObjectIter<'a>,
}

impl<'a> KeyIter<'a> {
    #[inline]
    fn try_new(object: &'a Object) -> YasonResult<Self> {
        Ok(Self {
            inner: ObjectIter::try_new(object)?,
        })
    }
}

impl<'a> Iterator for KeyIter<'a> {
    type Item = YasonResult<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.index < self.inner.len {
            let key = self.inner.next_key();
            self.inner.index += 1;
            Some(key)
        } else {
            None
        }
    }
}

/// An iterator over the object's values.
pub struct ValueIter<'a> {
    inner: ObjectIter<'a>,
}

impl<'a> ValueIter<'a> {
    #[inline]
    fn try_new(object: &'a Object) -> YasonResult<Self> {
        Ok(Self {
            inner: ObjectIter::try_new(object)?,
        })
    }
}

impl<'a> Iterator for ValueIter<'a> {
    type Item = YasonResult<Value<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.index < self.inner.len {
            let key = self.inner.next_value();
            self.inner.index += 1;
            Some(key)
        } else {
            None
        }
    }
}
