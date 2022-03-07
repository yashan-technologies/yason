//! Yason builder.

mod array;
mod object;
mod scalar;

pub use array::{ArrBuilder, ArrayBuilder, ArrayRefBuilder};
pub use object::{ObjBuilder, ObjectBuilder, ObjectRefBuilder};
pub use scalar::Scalar;

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt::{Display, Formatter};

const DEFAULT_SIZE: usize = 128;
const MAX_NESTED_DEPTH: usize = 100;

/// Possible errors that can arise during dealing with number.
#[derive(Debug)]
pub enum NumberError {
    Overflow,
    FormatError,
}

impl Display for NumberError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberError::Overflow => write!(f, "numeric overflow"),
            NumberError::FormatError => write!(f, "an error occurred when formatting a number"),
        }
    }
}

impl Error for NumberError {}

/// Possible errors that can arise during building.
#[derive(Debug)]
pub enum BuildError {
    TryReserveError(TryReserveError),
    InnerUncompletedError,
    InconsistentElementCount { expected: u16, actual: u16 },
    StringTooLong(usize),
    JsonError(serde_json::Error),
    NumberError(NumberError),
    NestedTooDeeply,
}

impl Display for BuildError {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            BuildError::TryReserveError(e) => write!(f, "{}", e),
            BuildError::InnerUncompletedError => write!(f, "inner builder is not finished"),
            BuildError::InconsistentElementCount { expected, actual } => write!(
                f,
                "inconsistent element count, expected {}, actual {}",
                expected, actual
            ),
            BuildError::StringTooLong(e) => write!(f, "string too long, length is {}", e),
            BuildError::JsonError(e) => write!(f, "{}", e),
            BuildError::NumberError(e) => write!(f, "{}", e),
            BuildError::NestedTooDeeply => write!(f, "nested too many depth"),
        }
    }
}

impl Error for BuildError {}

impl From<TryReserveError> for BuildError {
    #[inline]
    fn from(e: TryReserveError) -> Self {
        BuildError::TryReserveError(e)
    }
}

pub type BuildResult<T> = std::result::Result<T, BuildError>;

pub(crate) enum Depth<'a> {
    Owned(usize),
    Borrowed(&'a mut usize),
}

impl<'a> Depth<'a> {
    #[inline]
    const fn new() -> Self {
        Depth::Owned(0)
    }

    #[inline]
    fn borrow_mut(&mut self) -> Depth<'_> {
        match self {
            Depth::Owned(d) => Depth::Borrowed(d),
            Depth::Borrowed(d) => Depth::Borrowed(*d),
        }
    }

    #[inline]
    fn depth(&self) -> usize {
        match self {
            Depth::Owned(d) => *d,
            Depth::Borrowed(d) => **d,
        }
    }

    #[inline]
    fn increase(&mut self) {
        match self {
            Depth::Borrowed(d) => **d += 1,
            Depth::Owned(d) => *d += 1,
        }
    }

    #[inline]
    fn decrease(&mut self) {
        match self {
            Depth::Borrowed(d) => **d -= 1,
            Depth::Owned(d) => *d -= 1,
        }
    }
}
