//! Yason builder.

use crate::{DataType, YasonBuf};
use std::collections::TryReserveError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::size_of;

#[derive(Debug)]
pub enum BuildError {
    TryReserveError(TryReserveError),
}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            BuildError::TryReserveError(e) => write!(f, "{}", e),
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

type BuildResult<T> = std::result::Result<T, BuildError>;

pub struct Builder {
    bytes: Vec<u8>,
}

impl Builder {
    #[inline]
    const fn new() -> Self {
        Builder { bytes: Vec::new() }
    }

    #[inline(always)]
    fn try_reserve(&mut self, additional: usize) -> BuildResult<()> {
        self.bytes.try_reserve(additional)?;
        Ok(())
    }

    #[inline]
    fn try_with_capacity(capacity: usize) -> BuildResult<Self> {
        let mut builder = Builder::new();
        builder.try_reserve(capacity)?;
        Ok(builder)
    }

    #[inline]
    fn write_u8(&mut self, val: u8) {
        debug_assert!(size_of::<u8>() <= self.bytes.capacity().wrapping_sub(self.bytes.len()));
        self.bytes.push(val);
    }

    #[inline]
    fn write_i32(&mut self, val: i32) {
        debug_assert!(size_of::<i32>() <= self.bytes.capacity().wrapping_sub(self.bytes.len()));
        self.bytes.extend_from_slice(&val.to_le_bytes());
    }

    #[inline]
    fn write_data_type(&mut self, data_type: DataType) {
        self.write_u8(data_type as u8);
    }

    #[inline]
    fn write_str(&mut self, s: &str) {
        debug_assert!(s.len() <= self.bytes.capacity().wrapping_sub(self.bytes.len()));
        self.bytes.extend_from_slice(s.as_bytes());
    }

    #[allow(dead_code)]
    #[inline]
    fn try_write_i32(&mut self, val: i32) -> BuildResult<()> {
        self.try_reserve(size_of::<i32>())?;
        self.write_i32(val);
        Ok(())
    }

    #[inline]
    fn skip_size(&mut self) {
        self.write_i32(0);
    }

    #[inline]
    fn write_total_size(&mut self) {
        let total_size = self.bytes.len() as i32;
        let s = &mut self.bytes[0..size_of::<i32>()];
        s.copy_from_slice(&total_size.to_le_bytes());
    }

    #[inline]
    fn finish(self) -> YasonBuf {
        unsafe { YasonBuf::new_unchecked(self.bytes) }
    }

    #[inline]
    pub fn string<T: AsRef<str>>(s: T) -> BuildResult<YasonBuf> {
        let s = s.as_ref();
        let size = size_of::<i32>() + size_of::<DataType>() + 1 + s.len();
        let mut builder = Builder::try_with_capacity(size)?;
        builder.skip_size();
        builder.write_data_type(DataType::String);
        builder.write_u8(s.len() as u8);
        builder.write_str(s);
        builder.write_total_size();
        Ok(builder.finish())
    }
}
