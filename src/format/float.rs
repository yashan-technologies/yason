//! Floating-point format

use crate::format::WriteExt;
use std::fmt::{Display, Formatter};
use std::mem::MaybeUninit;
use std::num::FpCategory;
use std::ops::Deref;
#[cfg(target_os = "windows")]
use std::os::raw::{c_char, c_int};

pub(crate) trait Floating: Copy {
    const FORMAT: &'static str;

    fn classify(self) -> FpCategory;
    fn is_sign_positive(self) -> bool;
    /// This function is needed because [`libc::snprintf`] is a C variadic function, and all C variadic functions expect
    /// `u8` and `u16` to promote to `u32`, and `f32` to promote to `f64`.
    ///
    /// For more details, see <https://github.com/rust-lang/rust/issues/21812>.
    fn into_f64(self) -> f64;
}

impl Floating for f32 {
    const FORMAT: &'static str = "%.9g\0";

    #[inline(always)]
    fn classify(self) -> FpCategory {
        self.classify()
    }

    #[inline(always)]
    fn is_sign_positive(self) -> bool {
        self.is_sign_positive()
    }

    #[inline(always)]
    fn into_f64(self) -> f64 {
        self as f64
    }
}

impl Floating for f64 {
    const FORMAT: &'static str = "%.17lg\0";

    #[inline(always)]
    fn classify(self) -> FpCategory {
        self.classify()
    }

    #[inline(always)]
    fn is_sign_positive(self) -> bool {
        self.is_sign_positive()
    }

    #[inline(always)]
    fn into_f64(self) -> f64 {
        self
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct FloatingFormat<T: Floating>(pub T);

impl<T: Floating> Deref for FloatingFormat<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg_attr(target_os = "windows", link(name = "legacy_stdio_definitions"))]
unsafe extern "C" {
    fn snprintf(s: *mut c_char, n: usize, format: *const c_char, ...) -> c_int;
}

#[cfg(not(target_os = "windows"))]
use libc::snprintf;

impl<T: Floating> Display for FloatingFormat<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.classify() {
            FpCategory::Nan => write!(f, "Nan"),
            FpCategory::Infinite => {
                if self.is_sign_positive() {
                    write!(f, "Inf")
                } else {
                    write!(f, "-Inf")
                }
            }
            _ => {
                const BUF_SIZE: usize = 32;
                let mut buf = MaybeUninit::<[u8; BUF_SIZE]>::uninit();
                let s = buf.as_mut_ptr() as *mut _;
                let fmt = T::FORMAT.as_ptr() as *const _;

                // SAFETY: snprintf should always work with a valid and sufficiently-large buffer.
                let (buf, size) = unsafe {
                    // See the docstring of Floating::into_f64() for why we have this function.
                    let size = snprintf(s, BUF_SIZE, fmt, self.into_f64());
                    (buf.assume_init(), size as usize)
                };

                f.write_bytes(&buf[..size])
            }
        }
    }
}
