//! Query by path expression.

use crate::path::parse::{ArrayStep, FuncStep, ObjectStep, SingleIndex, SingleStep, Step};
use crate::path::push_value;
use crate::yason::YasonResult;
use crate::{Number, Value, YasonError};
use std::cmp::Ordering;

pub struct Selector<'a, 'b> {
    steps: &'b [Step],
    with_wrapper: bool,
    query_buf: &'b mut Vec<Value<'a>>,
    for_exists: bool,
}

impl<'a, 'b> Selector<'a, 'b> {
    #[inline]
    pub fn new(steps: &'b [Step], with_wrapper: bool, query_buf: &'b mut Vec<Value<'a>>, for_exists: bool) -> Self {
        Self {
            steps,
            with_wrapper,
            query_buf,
            for_exists,
        }
    }

    #[inline]
    pub fn query(&mut self, value: Value<'a>, step_index: usize) -> YasonResult<bool> {
        debug_assert!(step_index <= self.steps.len());

        if step_index == self.steps.len() {
            if !self.for_exists {
                if !self.with_wrapper && !self.query_buf.is_empty() {
                    return Err(YasonError::MultiValuesWithoutWrapper);
                }
                push_value(self.query_buf, value)?;
            }
            return Ok(true);
        }

        let cur_step = &self.steps[step_index];
        match cur_step {
            Step::Root => unreachable!(),
            Step::Object(obj_step) => match obj_step {
                ObjectStep::Key(key) => self.object_key_match(value, step_index, key.as_str()),
                ObjectStep::Wildcard => self.object_wildcard_match(value, step_index),
            },
            Step::Array(arr_step) => match arr_step {
                ArrayStep::Index(index) => self.array_index_match(value, step_index, *index),
                ArrayStep::Last(minus) => self.array_last_match(value, step_index, *minus),
                ArrayStep::Range(begin, end) => self.array_range_match(value, step_index, begin, end),
                ArrayStep::Multiple(arr_steps) => self.array_multi_steps_match(value, step_index, arr_steps),
                ArrayStep::Wildcard => self.array_wildcard_match(value, step_index),
            },
            Step::Descendent(key) => self.descendent_step_match(value, step_index, key.as_str()),
            Step::Func(func) => self.func_step_match(value, step_index, func),
        }
    }

    #[inline]
    fn object_key_match(&mut self, value: Value<'a>, step_index: usize, key: &'b str) -> YasonResult<bool> {
        match value {
            Value::Object(object) => {
                let val = object.get(key)?;
                if let Some(v) = val {
                    return self.query(v, step_index + 1);
                }
            }
            Value::Array(array) => {
                for val in array.iter()? {
                    let found = self.query(val?, step_index)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    #[inline]
    fn object_wildcard_match(&mut self, value: Value<'a>, step_index: usize) -> YasonResult<bool> {
        match value {
            Value::Object(object) => {
                for val in object.value_iter()? {
                    let found = self.query(val?, step_index + 1)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            Value::Array(array) => {
                for val in array.iter()? {
                    let found = self.query(val?, step_index)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    #[inline]
    fn array_index_match(&mut self, value: Value<'a>, step_index: usize, index: usize) -> YasonResult<bool> {
        match value {
            Value::Array(array) => {
                if index < array.len()? {
                    let val = array.get(index)?;
                    return self.query(val, step_index + 1);
                }
            }
            _ => {
                if index == 0 {
                    return self.query(value, step_index + 1);
                }
            }
        }
        Ok(false)
    }

    #[inline]
    fn array_last_match(&mut self, value: Value<'a>, step_index: usize, minus: usize) -> YasonResult<bool> {
        match value {
            Value::Array(array) => {
                let len = array.len()?;
                if len - 1 > minus {
                    let val = unsafe { array.get_unchecked(len - 1 - minus)? };
                    return self.query(val, step_index + 1);
                }
            }
            _ => {
                if minus == 0 {
                    return self.query(value, step_index + 1);
                }
            }
        }
        Ok(false)
    }

    #[inline]
    fn array_range_match(
        &mut self,
        value: Value<'a>,
        step_index: usize,
        begin: &'b SingleIndex,
        end: &'b SingleIndex,
    ) -> YasonResult<bool> {
        match value {
            Value::Array(array) => {
                let len = array.len()?;
                if let Some((b, e)) = find_range(begin, end, len) {
                    for i in b..e + 1 {
                        let val = unsafe { array.get_unchecked(i)? };
                        let found = self.query(val, step_index + 1)?;
                        if self.for_exists && found {
                            return Ok(true);
                        }
                    }
                }
            }
            _ => {
                if find_range(begin, end, 1).is_some() {
                    return self.query(value, step_index + 1);
                }
            }
        }
        Ok(false)
    }

    #[inline]
    fn array_multi_steps_match(
        &mut self,
        value: Value<'a>,
        step_index: usize,
        arr_steps: &'b [SingleStep],
    ) -> YasonResult<bool> {
        match value {
            Value::Array(array) => {
                let mut arr_steps_index = 0;
                while arr_steps_index < arr_steps.len() {
                    let cur_step = &arr_steps[arr_steps_index];
                    let len = array.len()?;

                    match cur_step {
                        SingleStep::Single(single_index) => match single_index {
                            SingleIndex::Index(index) => {
                                if *index < len {
                                    let val = unsafe { array.get_unchecked(*index)? };
                                    let found = self.query(val, step_index + 1)?;
                                    if self.for_exists && found {
                                        return Ok(true);
                                    }
                                }
                            }
                            SingleIndex::Last(minus) => {
                                if len - 1 > *minus {
                                    let val = unsafe { array.get_unchecked(len - 1 - minus)? };
                                    let found = self.query(val, step_index + 1)?;
                                    if self.for_exists && found {
                                        return Ok(true);
                                    }
                                }
                            }
                        },
                        SingleStep::Range(begin, end) => {
                            if let Some((b, e)) = find_range(begin, end, len) {
                                for i in b..e + 1 {
                                    let val = unsafe { array.get_unchecked(i)? };
                                    let found = self.query(val, step_index + 1)?;
                                    if self.for_exists && found {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                    arr_steps_index += 1;
                }
            }
            _ => {
                if non_array_relaxed_match(arr_steps) {
                    return self.query(value, step_index + 1);
                }
            }
        }
        Ok(false)
    }

    #[inline]
    fn array_wildcard_match(&mut self, value: Value<'a>, step_index: usize) -> YasonResult<bool> {
        match value {
            Value::Array(array) => {
                for val in array.iter()? {
                    let found = self.query(val?, step_index + 1)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            _ => return self.query(value, step_index + 1),
        };
        Ok(false)
    }

    #[inline]
    fn descendent_step_match(&mut self, value: Value<'a>, step_index: usize, key: &'b str) -> YasonResult<bool> {
        match value {
            Value::Object(ref object) => {
                if let Some(val) = object.get(key)? {
                    let found = self.query(val, step_index + 1)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }

                for val in object.value_iter()? {
                    let found = self.query(val?, step_index)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            Value::Array(ref array) => {
                for val in array.iter()? {
                    let found = self.query(val?, step_index)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            _ => {}
        }

        Ok(false)
    }

    #[inline]
    fn func_step_match(&mut self, value: Value<'a>, step_index: usize, func: &'b FuncStep) -> YasonResult<bool> {
        match func {
            FuncStep::Count => self.query(value, step_index + 1),
            FuncStep::Size => {
                let size = match value {
                    Value::Array(array) => array.len()?,
                    _ => 1,
                };
                let val = Value::Number(Number::from(size));
                self.query(val, step_index + 1)
            }
            FuncStep::Type => {
                let data_type = value.data_type() as u8;
                let val = Value::Number(Number::from(data_type));
                self.query(val, step_index + 1)
            }
        }
    }
}

#[inline]
fn non_array_relaxed_match(steps: &[SingleStep]) -> bool {
    for step in steps {
        match step {
            SingleStep::Single(single_index) => match single_index {
                SingleIndex::Index(index) => {
                    if *index == 0 {
                        return true;
                    }
                }
                SingleIndex::Last(minus) => {
                    if *minus == 0 {
                        return true;
                    }
                }
            },

            SingleStep::Range(left_field, right_field) => {
                let (left, right) = match (left_field, right_field) {
                    (SingleIndex::Index(i1), SingleIndex::Index(i2))
                    | (SingleIndex::Index(i1), SingleIndex::Last(i2))
                    | (SingleIndex::Last(i1), SingleIndex::Index(i2))
                    | (SingleIndex::Last(i1), SingleIndex::Last(i2)) => (i1, i2),
                };
                if *left == 0 || *right == 0 {
                    return true;
                }
            }
        }
    }

    false
}

#[inline]
fn find_range(begin: &SingleIndex, end: &SingleIndex, len: usize) -> Option<(usize, usize)> {
    fn inner(u1: usize, u2: usize, len: usize) -> Option<(usize, usize)> {
        let b = u1.min(u2);
        let e = u1.max(u2).min(len - 1);
        Some((b, e))
    }

    let last = len - 1;
    match (begin, end) {
        (SingleIndex::Index(i1), SingleIndex::Index(i2)) => inner(*i1, *i2, len),
        (SingleIndex::Index(index), SingleIndex::Last(minus)) => inner(*index, last.max(*minus) - minus, len),
        (SingleIndex::Last(minus), SingleIndex::Index(index)) => inner(last.max(*minus) - minus, *index, len),
        (SingleIndex::Last(m1), SingleIndex::Last(m2)) => match (last.cmp(m1), last.cmp(m2)) {
            (Ordering::Less, Ordering::Less) => None,
            (Ordering::Less, _) => inner(0, last - m2, len),
            (_, Ordering::Less) => inner(last - m1, 0, len),
            _ => inner(last - m1, last - m2, len),
        },
    }
}
