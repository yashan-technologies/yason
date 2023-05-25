use std::{
    borrow::{Borrow, Cow},
    collections::BTreeMap,
};

// for yason rust, number just be string is ok
pub type Number<'a> = &'a str;
pub type Map<String, Value> = BTreeMap<String, Value>;

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    Null,
    String(Cow<'a, str>),
    Number(Number<'a>),
    Bool(bool),
    Array(Vec<Value<'a>>),
    Object(Map<Cow<'a, str>, Value<'a>>),
}

#[allow(dead_code)]
impl<'a> Value<'a> {
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.borrow()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    #[inline]
    pub fn as_number(&self) -> Option<&'a str> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    #[inline]
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    #[inline]
    pub fn as_object(&self) -> Option<&BTreeMap<Cow<str>, Value>> {
        match self {
            Value::Object(object) => Some(object),
            _ => None,
        }
    }
}
