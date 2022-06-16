//! Query by path expression.

use crate::path::parse::{ArrayStep, FuncStep, ObjectStep, SingleIndex, SingleStep, Step};
use crate::path::push_value;
use crate::yason::{LazyValue, YasonResult};
use crate::{DataType, Number, Value, Yason, YasonError};

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
    pub fn query(&mut self, value: &'a Yason, step_index: usize) -> YasonResult<bool> {
        let lazy_value = LazyValue::try_from(value)?;
        self.query_internal(lazy_value, step_index)
    }

    #[inline]
    fn query_internal<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
    ) -> YasonResult<bool> {
        debug_assert!(step_index <= self.steps.len());

        if step_index == self.steps.len() {
            if !self.for_exists {
                if !self.with_wrapper && !self.query_buf.is_empty() {
                    return Err(YasonError::MultiValuesWithoutWrapper);
                }

                push_value(self.query_buf, value.value()?)?;
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
    fn object_key_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
        key: &'b str,
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Object => {
                let object = unsafe { value.object()? };
                let val = object.lazy_get(key)?;
                if let Some(v) = val {
                    return self.query_internal(v, step_index + 1);
                }
            }
            DataType::Array => {
                let array = unsafe { value.array()? };
                for val in array.lazy_iter()? {
                    let found = self.query_internal(val?, step_index)?;
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
    fn object_wildcard_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Object => {
                let object = unsafe { value.object()? };
                for val in object.lazy_value_iter()? {
                    let found = self.query_internal(val?, step_index + 1)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            DataType::Array => {
                let array = unsafe { value.array()? };
                for val in array.lazy_iter()? {
                    let found = self.query_internal(val?, step_index)?;
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
    fn array_index_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
        index: usize,
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Array => {
                let array = unsafe { value.array()? };
                if index < array.len()? {
                    let val = unsafe { array.lazy_get_unchecked(index)? };
                    return self.query_internal(val, step_index + 1);
                }
            }
            _ => {
                if index == 0 {
                    return self.non_array_relax_match(value, step_index + 1);
                }
            }
        }
        Ok(false)
    }

    #[inline]
    fn array_last_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
        minus: usize,
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Array => {
                let array = unsafe { value.array()? };
                let len = array.len()?;
                if len > minus {
                    let val = unsafe { array.lazy_get_unchecked(len - 1 - minus)? };
                    return self.query_internal(val, step_index + 1);
                }
            }
            _ => {
                if minus == 0 {
                    return self.non_array_relax_match(value, step_index + 1);
                }
            }
        }

        Ok(false)
    }

    #[inline]
    fn array_range_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
        begin: &'b SingleIndex,
        end: &'b SingleIndex,
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Array => {
                let array = unsafe { value.array()? };
                let len = array.len()?;
                if len == 0 {
                    return Ok(false);
                }

                let last = len - 1;
                if let Some((b, e)) = find_range(begin, end, last) {
                    for i in b..e + 1 {
                        let val = unsafe { array.lazy_get_unchecked(i)? };
                        let found = self.query_internal(val, step_index + 1)?;
                        if self.for_exists && found {
                            return Ok(true);
                        }
                    }
                }
            }
            _ => {
                if non_array_range_step_relaxed_match(begin, end) {
                    return self.non_array_relax_match(value, step_index + 1);
                }
            }
        }
        Ok(false)
    }

    #[inline]
    fn array_multi_steps_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
        arr_steps: &'b [SingleStep],
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Array => {
                let array = unsafe { value.array()? };
                let len = array.len()?;
                if len == 0 {
                    return Ok(false);
                }

                let mut arr_steps_index = 0;
                while arr_steps_index < arr_steps.len() {
                    let cur_step = &arr_steps[arr_steps_index];

                    match cur_step {
                        SingleStep::Single(single_index) => match single_index {
                            SingleIndex::Index(index) => {
                                if *index < len {
                                    let val = unsafe { array.lazy_get_unchecked(*index)? };
                                    let found = self.query_internal(val, step_index + 1)?;
                                    if self.for_exists && found {
                                        return Ok(true);
                                    }
                                }
                            }
                            SingleIndex::Last(minus) => {
                                if len > *minus {
                                    let val = unsafe { array.lazy_get_unchecked(len - 1 - minus)? };
                                    let found = self.query_internal(val, step_index + 1)?;
                                    if self.for_exists && found {
                                        return Ok(true);
                                    }
                                }
                            }
                        },
                        SingleStep::Range(begin, end) => {
                            let last = len - 1;
                            if let Some((b, e)) = find_range(begin, end, last) {
                                for i in b..e + 1 {
                                    let val = unsafe { array.lazy_get_unchecked(i)? };
                                    let found = self.query_internal(val, step_index + 1)?;
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
                if non_array_multi_steps_relaxed_match(arr_steps) {
                    return self.non_array_relax_match(value, step_index + 1);
                }
            }
        }
        Ok(false)
    }

    #[inline]
    fn array_wildcard_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Array => {
                let array = unsafe { value.array()? };
                for val in array.lazy_iter()? {
                    let found = self.query_internal(val?, step_index + 1)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            _ => return self.non_array_relax_match(value, step_index + 1),
        }

        Ok(false)
    }

    #[inline]
    fn non_array_relax_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
    ) -> YasonResult<bool> {
        let mut cur_step_index = step_index;

        while cur_step_index < self.steps.len() {
            let step = &self.steps[cur_step_index];
            match step {
                Step::Array(array_step) => match array_step {
                    ArrayStep::Index(index) => {
                        if *index == 0 {
                            cur_step_index += 1;
                        } else {
                            return Ok(false);
                        }
                    }
                    ArrayStep::Last(minus) => {
                        if *minus == 0 {
                            cur_step_index += 1;
                        } else {
                            return Ok(false);
                        }
                    }
                    ArrayStep::Range(begin, end) => {
                        if non_array_range_step_relaxed_match(begin, end) {
                            cur_step_index += 1;
                        } else {
                            return Ok(false);
                        }
                    }
                    ArrayStep::Multiple(steps) => {
                        if non_array_multi_steps_relaxed_match(steps) {
                            cur_step_index += 1;
                        } else {
                            return Ok(false);
                        }
                    }
                    ArrayStep::Wildcard => {
                        cur_step_index += 1;
                    }
                },
                _ => return self.query_internal(value, cur_step_index),
            }
        }

        self.query_internal(value, cur_step_index)
    }

    #[inline]
    fn descendent_step_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
        key: &'b str,
    ) -> YasonResult<bool> {
        match value.data_type() {
            DataType::Object => {
                let object = unsafe { value.object()? };
                if let Some(val) = object.lazy_get(key)? {
                    let found = self.query_internal(val, step_index + 1)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }

                for val in object.lazy_value_iter()? {
                    let found = self.query_internal(val?, step_index)?;
                    if self.for_exists && found {
                        return Ok(true);
                    }
                }
            }
            DataType::Array => {
                let array = unsafe { value.array()? };
                for val in array.lazy_iter()? {
                    let found = self.query_internal(val?, step_index)?;
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
    fn func_step_match<const IN_ARRAY: bool>(
        &mut self,
        value: LazyValue<'a, IN_ARRAY>,
        step_index: usize,
        func: &'b FuncStep,
    ) -> YasonResult<bool> {
        debug_assert!(step_index + 1 == self.steps.len());
        debug_assert!(self.with_wrapper);
        let val = match func {
            FuncStep::Count => Value::Null,
            FuncStep::Size => {
                let size = match value.data_type() {
                    DataType::Array => {
                        let array = unsafe { value.array()? };
                        array.len()?
                    }
                    _ => 1,
                };

                Value::Number(Number::from(size))
            }
            FuncStep::Type => {
                let data_type = value.data_type();
                Value::String(data_type.name())
            }
        };
        push_value(self.query_buf, val)?;
        Ok(false)
    }
}

#[inline]
fn non_array_multi_steps_relaxed_match(steps: &[SingleStep]) -> bool {
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
                if non_array_range_step_relaxed_match(left_field, right_field) {
                    return true;
                }
            }
        }
    }

    false
}

#[inline]
fn non_array_range_step_relaxed_match(begin: &SingleIndex, end: &SingleIndex) -> bool {
    // For non-array types, an array of size 1 is automatically encapsulated for relaxed matching
    // and only index 0 can be matched.
    if let Some((b, _)) = find_range(begin, end, 0) {
        if b == 0 {
            return true;
        }
    }

    false
}

// Find the index range to traverse based on the two SingleIndexes, both sides of this range are closed.
// For example, if the return value is Some((1, 3)), the indexes that need to be traversed are 1, 2, 3.
// The argument `last` is equal to the last index of the array (last = array.len() - 1).
#[inline]
fn find_range(begin: &SingleIndex, end: &SingleIndex, last: usize) -> Option<(usize, usize)> {
    #[inline]
    fn find_range_by_index(begin_index: usize, end_index: usize, last: usize) -> Option<(usize, usize)> {
        debug_assert!(begin_index <= end_index);
        let end_index = end_index.min(last);
        if begin_index <= end_index {
            Some((begin_index, end_index))
        } else {
            None
        }
    }

    #[inline]
    fn find_range_by_last(minus1: usize, minus2: usize, last: usize) -> Option<(usize, usize)> {
        debug_assert!(minus1 <= minus2);
        let begin_index = last.saturating_sub(minus2);

        if minus1 <= last {
            Some((begin_index, last - minus1))
        } else {
            None
        }
    }

    #[inline]
    fn order(l: usize, r: usize) -> (usize, usize) {
        if l <= r {
            (l, r)
        } else {
            (r, l)
        }
    }

    match (begin, end) {
        (SingleIndex::Index(i1), SingleIndex::Index(i2)) => {
            let (begin_index, end_index) = order(*i1, *i2);
            find_range_by_index(begin_index, end_index, last)
        }
        (SingleIndex::Index(i1), SingleIndex::Last(minus)) => {
            let i2 = last.saturating_sub(*minus);
            let (begin_index, end_index) = order(*i1, i2);
            find_range_by_index(begin_index, end_index, last)
        }
        (SingleIndex::Last(minus), SingleIndex::Index(i2)) => {
            let i1 = last.saturating_sub(*minus);
            let (begin_index, end_index) = order(i1, *i2);
            find_range_by_index(begin_index, end_index, last)
        }
        (SingleIndex::Last(m1), SingleIndex::Last(m2)) => {
            let (minus1, minus2) = order(*m1, *m2);
            find_range_by_last(minus1, minus2, last)
        }
    }
}
