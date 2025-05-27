use core::ffi::{c_char, CStr};

use alloc::{ffi::CString, string::String};
use csz::{CStrArray, CStrBox, CStrThin};

/// An internal buffer for [CStrTemp].
enum Buf<const N: usize = 512> {
    Stack(CStrArray<N>),
    Heap(CString),
}

/// A `CStrTemp` is a temporary C string stored on the stack or the heap.
pub struct CStrTemp<const N: usize = 512> {
    buf: Buf<N>,
}

impl<const N: usize> CStrTemp<N> {
    fn new(s: &str) -> Self {
        let buf = CStrArray::from_bytes(s.as_bytes())
            .map(Buf::Stack)
            .unwrap_or_else(|_| Buf::Heap(CString::new(s).unwrap()));
        Self { buf }
    }

    /// Converts this C string to a byte slice containing the trailing 0 byte.
    pub fn as_bytes_with_null(&self) -> &[u8] {
        match &self.buf {
            Buf::Stack(s) => s.to_bytes_with_nul(),
            Buf::Heap(s) => s.as_bytes_with_nul(),
        }
    }

    /// Returns the inner pointer to this C string.
    pub fn as_ptr(&self) -> *const c_char {
        match &self.buf {
            Buf::Stack(s) => s.as_ptr(),
            Buf::Heap(s) => s.as_ptr(),
        }
    }

    /// Extracts a [CStr] slice containing the entire string.
    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_ptr()) }
    }

    /// Returns `true` if the string is stored on the stack.
    pub fn is_stack(&self) -> bool {
        matches!(self.buf, Buf::Stack(_))
    }

    /// Returns `true` if the string is stored on the heap.
    pub fn is_heap(&self) -> bool {
        matches!(self.buf, Buf::Heap(_))
    }
}

impl From<&'_ str> for CStrTemp {
    fn from(src: &str) -> Self {
        Self::new(src)
    }
}

impl AsRef<CStr> for CStrTemp {
    fn as_ref(&self) -> &CStr {
        self.as_c_str()
    }
}

pub trait AsPtr<T> {
    fn as_ptr(&self) -> *const T;
}

impl AsPtr<c_char> for &'_ CStr {
    fn as_ptr(&self) -> *const c_char {
        CStr::as_ptr(self)
    }
}

impl AsPtr<c_char> for &'_ CStrThin {
    fn as_ptr(&self) -> *const c_char {
        CStrThin::as_ptr(self)
    }
}

impl AsPtr<c_char> for CStrTemp {
    fn as_ptr(&self) -> *const c_char {
        self.as_ptr()
    }
}

pub trait ToEngineStr {
    type Output: AsPtr<c_char>;

    fn to_engine_str(&self) -> Self::Output;
}

impl ToEngineStr for &'_ str {
    type Output = CStrTemp;

    fn to_engine_str(&self) -> Self::Output {
        CStrTemp::new(self)
    }
}

impl ToEngineStr for String {
    type Output = CStrTemp;

    fn to_engine_str(&self) -> Self::Output {
        CStrTemp::new(self.as_str())
    }
}

impl<'a> ToEngineStr for &'a CStr {
    type Output = &'a CStr;

    fn to_engine_str(&self) -> Self::Output {
        self
    }
}

impl<'a> ToEngineStr for &'a CStrThin {
    type Output = &'a CStrThin;

    fn to_engine_str(&self) -> Self::Output {
        self
    }
}

impl<'a, const N: usize> ToEngineStr for &'a CStrArray<N> {
    type Output = &'a CStrThin;

    fn to_engine_str(&self) -> Self::Output {
        self.as_thin()
    }
}

impl<'a> ToEngineStr for &'a CStrBox {
    type Output = &'a CStrThin;

    fn to_engine_str(&self) -> Self::Output {
        self.as_c_str()
    }
}

impl<'a> ToEngineStr for &'a CString {
    type Output = &'a CStr;

    fn to_engine_str(&self) -> Self::Output {
        self.as_c_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::ffi::CStr;

    fn test<const N: usize>(src: &CStr, stack: bool) {
        let s = CStrTemp::<N>::new(src.to_str().unwrap());
        assert_eq!(s.is_stack(), stack);
        assert_eq!(s.as_bytes_with_null(), src.to_bytes_with_nul());
        assert_eq!(s.as_c_str(), src);
    }

    #[test]
    fn compat_str() {
        test::<8>(c"0123456", true);
        test::<7>(c"0123456", false);
    }
}
