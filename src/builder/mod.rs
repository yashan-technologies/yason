//! Yason builder.

mod array;
mod object;
mod scalar;

pub use array::{ArrayBuilder, ArrayRefBuilder};
pub use object::{ObjectBuilder, ObjectRefBuilder};
pub use scalar::Scalar;

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt::{Display, Formatter};

const DEFAULT_SIZE: usize = 128;

/// Possible errors that can arise during building.
#[derive(Debug)]
pub enum BuildError {
    TryReserveError(TryReserveError),
    InnerUncompletedError,
    InconsistentElementCount { expected: u16, actual: u16 },
    StringTooLong(usize),
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

struct BytesWrapper<B: AsMut<Vec<u8>>> {
    bytes: B,
    depth: usize,
}

impl<B: AsMut<Vec<u8>>> BytesWrapper<B> {
    #[inline]
    fn new(bytes: B) -> Self {
        BytesWrapper { bytes, depth: 0 }
    }
}
