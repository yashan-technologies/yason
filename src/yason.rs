//! Yason manipulation.

use crate::{DataType, InvalidDataType};
use std::borrow::Borrow;
use std::ops::Deref;

#[repr(transparent)]
pub struct YasonBuf {
    bytes: Vec<u8>,
}

impl YasonBuf {
    /// Creates a new `YasonBuf` from `Vec<u8>`.
    ///
    /// # Safety
    ///
    /// Callers should guarantee the `bytes` is a valid `YASON`.
    #[inline]
    pub const unsafe fn new_unchecked(bytes: Vec<u8>) -> Self {
        YasonBuf { bytes }
    }
}

#[repr(transparent)]
pub struct Yason {
    bytes: [u8],
}

impl Deref for YasonBuf {
    type Target = Yason;

    #[inline]
    fn deref(&self) -> &Yason {
        unsafe { Yason::new_unchecked(&self.bytes) }
    }
}

impl Borrow<Yason> for YasonBuf {
    #[inline]
    fn borrow(&self) -> &Yason {
        self.deref()
    }
}

impl ToOwned for Yason {
    type Owned = YasonBuf;

    #[inline]
    fn to_owned(&self) -> YasonBuf {
        self.to_yason_buf()
    }
}

impl AsRef<Yason> for YasonBuf {
    #[inline]
    fn as_ref(&self) -> &Yason {
        self
    }
}

impl Yason {
    /// Creates a new `Yason` from the reference of `[u8]`.
    ///
    /// # Safety
    ///
    /// Callers should guarantee the `bytes` is a valid `YASON`.
    #[inline]
    pub unsafe fn new_unchecked<B: AsRef<[u8]> + ?Sized>(bytes: &B) -> &Yason {
        &*(bytes.as_ref() as *const [u8] as *const Yason)
    }

    #[inline]
    pub fn to_yason_buf(&self) -> YasonBuf {
        YasonBuf {
            bytes: self.bytes.to_vec(),
        }
    }

    #[inline]
    pub fn data_type(&self) -> Result<DataType, InvalidDataType> {
        DataType::try_from(self.bytes[4])
    }
}
