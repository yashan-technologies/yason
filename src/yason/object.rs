//! Object manipulation.

use crate::binary::{DATA_TYPE_SIZE, ELEMENT_COUNT_SIZE, KEY_LENGTH_SIZE, KEY_OFFSET_SIZE, OBJECT_SIZE};
use crate::yason::array::Array;
use crate::yason::{LazyValue, Value, Yason, YasonResult};
use crate::{DataType, Number};

/// An object in yason binary format.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Object<'a>(&'a Yason);

impl<'a> Object<'a> {
    /// Gets an iterator over the entries of the object.
    #[inline]
    pub fn iter(&self) -> YasonResult<ObjectIter<'a>> {
        ObjectIter::try_new(self.0)
    }

    #[inline]
    pub(crate) fn lazy_iter(&self) -> YasonResult<LazyObjectIter<'a>> {
        LazyObjectIter::try_new(self.0)
    }

    /// Gets an iterator over the keys of the object.
    #[inline]
    pub fn key_iter(&self) -> YasonResult<KeyIter<'a>> {
        KeyIter::try_new(self.0)
    }

    /// Gets an iterator over the keys of the object.
    #[inline]
    pub fn value_iter(&self) -> YasonResult<ValueIter<'a>> {
        ValueIter::try_new(self.0)
    }

    #[inline]
    pub(crate) fn lazy_value_iter(&self) -> YasonResult<LazyObjectValueIter<'a>> {
        LazyObjectValueIter::try_new(self.0)
    }

    /// Creates an `Object`.
    ///
    /// # Safety
    ///
    /// Callers should guarantee the `yason` is a valid `Object`.
    #[inline]
    pub const unsafe fn new_unchecked(yason: &'a Yason) -> Self {
        debug_assert!(yason.bytes.len() >= DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE);
        Self(yason)
    }

    /// Returns the number of elements in the object.
    #[inline]
    pub fn len(&self) -> YasonResult<usize> {
        Ok(self.0.read_u16(DATA_TYPE_SIZE + OBJECT_SIZE)? as usize)
    }

    #[inline]
    pub fn yason(&self) -> &'a Yason {
        self.0
    }

    /// Returns true if the object contains no elements.
    #[inline]
    pub fn is_empty(&self) -> YasonResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Returns the value corresponding to the key, if it exists.
    #[inline]
    pub fn get<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Value<'a>>> {
        let found = self.find_key(key.as_ref())?;
        if let Some(value_pos) = found {
            return Ok(Some(self.read_value(value_pos)?));
        }

        Ok(None)
    }

    #[inline]
    pub(crate) fn lazy_get<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<LazyValue<'a, false>>> {
        let found = self.find_key(key.as_ref())?;
        if let Some(value_pos) = found {
            let data_type = self.0.read_type(value_pos)?;
            return Ok(Some(LazyValue::new(self.0, data_type, value_pos)));
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
    pub fn object<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Object<'a>>> {
        let found = self.check_key(key.as_ref(), DataType::Object)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.0.read_object(value_pos)?));
        }
        Ok(None)
    }

    /// Gets an array for this key if it exists and has the correct type, returns `None` if this
    /// key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn array<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Array<'a>>> {
        let found = self.check_key(key.as_ref(), DataType::Array)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.0.read_array(value_pos)?));
        }
        Ok(None)
    }

    /// Gets a string value for this key if it exists and has the correct type, returns `None` if
    /// this key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn string<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<&'a str>> {
        let found = self.check_key(key.as_ref(), DataType::String)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.0.read_string(value_pos)?));
        }
        Ok(None)
    }

    /// Gets a number value for this key if it exists and has the correct type, returns `None`
    /// if this key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn number<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<Number>> {
        let found = self.check_key(key.as_ref(), DataType::Number)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.0.read_number(value_pos)?));
        }
        Ok(None)
    }

    /// Gets a bool value for this key if it exists and has the correct type, returns `None` if
    /// this key does not exist, returns `YasonError` otherwise.
    #[inline]
    pub fn bool<T: AsRef<str>>(&self, key: T) -> YasonResult<Option<bool>> {
        let found = self.check_key(key.as_ref(), DataType::Bool)?;

        if let Some(value_pos) = found {
            return Ok(Some(self.0.read_bool(value_pos)?));
        }
        Ok(None)
    }

    #[inline]
    pub(crate) fn equals<T: AsRef<Object<'a>>>(&self, other: T) -> YasonResult<bool> {
        let other = other.as_ref();
        if self.len()? != other.len()? {
            return Ok(false);
        }

        for (l_entry, r_entry) in self.lazy_iter()?.zip(other.lazy_iter()?) {
            let (l_key, l_value) = l_entry?;
            let (r_key, r_value) = r_entry?;

            if l_key != r_key {
                return Ok(false);
            }
            let res = l_value.equals(r_value)?;
            if !res {
                return Ok(false);
            }
        }

        Ok(true)
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
    fn read_key(&self, key_offset: usize) -> YasonResult<(&'a str, usize)> {
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
    fn read_value(&self, value_pos: usize) -> YasonResult<Value<'a>> {
        let data_type = self.0.read_type(value_pos)?;
        let value = match data_type {
            DataType::Object => Value::Object(self.0.read_object(value_pos)?),
            DataType::Array => Value::Array(self.0.read_array(value_pos)?),
            DataType::String => Value::String(self.0.read_string(value_pos)?),
            DataType::Number => Value::Number(self.0.read_number(value_pos)?),
            DataType::Bool => Value::Bool(self.0.read_bool(value_pos)?),
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

    #[inline]
    unsafe fn nth_key_offset(&self, index: usize) -> YasonResult<u32> {
        debug_assert!(index < self.len()?);
        let key_offset_pos = DATA_TYPE_SIZE + OBJECT_SIZE + ELEMENT_COUNT_SIZE + index * KEY_OFFSET_SIZE;
        self.read_key_offset(key_offset_pos)
    }

    #[inline]
    unsafe fn read_nth_value_pos(&self, index: usize) -> YasonResult<usize> {
        let key_offset = self.nth_key_offset(index)?;
        self.skip_key(key_offset as usize)
    }

    #[inline]
    unsafe fn read_nth_key_and_value_pos(&self, index: usize) -> YasonResult<(&'a str, usize)> {
        let key_offset = self.nth_key_offset(index)?;
        self.read_key(key_offset as usize)
    }

    #[inline]
    unsafe fn read_nth_type_and_value_pos(&self, index: usize) -> YasonResult<(DataType, usize)> {
        let value_pos = self.read_nth_value_pos(index)?;
        let data_type = self.0.read_type(value_pos)?;
        Ok((data_type, value_pos))
    }
}

impl<'a> AsRef<Object<'a>> for Object<'a> {
    #[inline]
    fn as_ref(&self) -> &Object<'a> {
        self
    }
}

/// An iterator over the object's entries.
pub struct ObjectIter<'a> {
    object: Object<'a>,
    len: usize,
    index: usize,
}

impl<'a> ObjectIter<'a> {
    #[inline]
    fn try_new(yason: &'a Yason) -> YasonResult<Self> {
        let object = Object(yason);
        Ok(Self {
            len: object.len()?,
            object,
            index: 0,
        })
    }

    #[inline]
    fn next_entry(&mut self) -> YasonResult<(&'a str, Value<'a>)> {
        let (key, value_pos) = unsafe { self.object.read_nth_key_and_value_pos(self.index)? };
        let value = self.object.read_value(value_pos)?;
        Ok((key, value))
    }

    #[inline]
    fn next_key(&mut self) -> YasonResult<&'a str> {
        Ok(unsafe { self.object.read_nth_key_and_value_pos(self.index)?.0 })
    }

    #[inline]
    fn next_value(&mut self) -> YasonResult<Value<'a>> {
        let value_pos = unsafe { self.object.read_nth_value_pos(self.index)? };
        let value = self.object.read_value(value_pos)?;
        Ok(value)
    }
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = YasonResult<(&'a str, Value<'a>)>;

    #[inline]
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

pub struct LazyObjectIter<'a> {
    object: Object<'a>,
    len: usize,
    index: usize,
}

impl<'a> LazyObjectIter<'a> {
    #[inline]
    fn try_new(yason: &'a Yason) -> YasonResult<Self> {
        let object = Object(yason);
        Ok(Self {
            len: object.len()?,
            object,
            index: 0,
        })
    }

    #[inline]
    fn next(&mut self) -> YasonResult<(&'a str, LazyValue<'a, false>)> {
        let (key, value_pos) = unsafe { self.object.read_nth_key_and_value_pos(self.index)? };
        let data_type = self.object.0.read_type(value_pos)?;
        self.index += 1;
        Ok((key, LazyValue::new(self.object.0, data_type, value_pos)))
    }
}

impl<'a> Iterator for LazyObjectIter<'a> {
    type Item = YasonResult<(&'a str, LazyValue<'a, false>)>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            Some(self.next())
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
    fn try_new(yason: &'a Yason) -> YasonResult<Self> {
        Ok(Self {
            inner: ObjectIter::try_new(yason)?,
        })
    }
}

impl<'a> Iterator for KeyIter<'a> {
    type Item = YasonResult<&'a str>;

    #[inline]
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
    fn try_new(yason: &'a Yason) -> YasonResult<Self> {
        Ok(Self {
            inner: ObjectIter::try_new(yason)?,
        })
    }
}

impl<'a> Iterator for ValueIter<'a> {
    type Item = YasonResult<Value<'a>>;

    #[inline]
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

pub struct LazyObjectValueIter<'a> {
    object: Object<'a>,
    len: usize,
    index: usize,
}

impl<'a> LazyObjectValueIter<'a> {
    #[inline]
    fn try_new(yason: &'a Yason) -> YasonResult<LazyObjectValueIter<'a>> {
        let object = Object(yason);
        Ok(Self {
            len: object.len()?,
            object,
            index: 0,
        })
    }

    #[inline]
    fn next(&mut self) -> YasonResult<LazyValue<'a, false>> {
        let (data_type, value_pos) = unsafe { self.object.read_nth_type_and_value_pos(self.index)? };
        self.index += 1;
        Ok(LazyValue::new(self.object.0, data_type, value_pos))
    }
}

impl<'a> Iterator for LazyObjectValueIter<'a> {
    type Item = YasonResult<LazyValue<'a, false>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            Some(self.next())
        } else {
            None
        }
    }
}
