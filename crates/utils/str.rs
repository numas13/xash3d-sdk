use core::{
    cmp,
    ffi::{c_char, CStr},
    mem::MaybeUninit,
    ptr,
};

use alloc::{boxed::Box, ffi::CString, string::String};
use csz::{CStrArray, CStrBox, CStrThin};

pub fn cstr_copy(dst: &mut [u8], src: &[u8]) -> usize {
    let len = src.len() - src.ends_with(b"\0") as usize;
    let len = cmp::min(len, dst.len() - 1);
    dst[..len].copy_from_slice(&src[..len]);
    dst[len] = b'\0';
    len
}

pub enum CStrBuf<const N: usize = 512> {
    Stack([u8; N]),
    Heap(Box<[u8]>),
}

impl<const N: usize> CStrBuf<N> {
    pub fn new(s: &str) -> Self {
        if s.len() < N {
            unsafe {
                let mut bytes = MaybeUninit::<[u8; N]>::uninit();
                let dst = bytes.as_mut_ptr().cast::<u8>();
                ptr::copy(s.as_ptr(), dst, s.len());
                ptr::write(dst.add(s.len()), 0);
                Self::Stack(bytes.assume_init())
            }
        } else {
            let bytes = CString::new(s).unwrap().into_bytes_with_nul();
            Self::Heap(bytes.into_boxed_slice())
        }
    }

    pub fn as_bytes_with_null(&self) -> &[u8] {
        match self {
            Self::Stack(s) => s,
            Self::Heap(s) => s,
        }
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.as_bytes_with_null().as_ptr().cast()
    }

    pub fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.as_ptr()) }
    }

    pub fn is_stack(&self) -> bool {
        matches!(self, Self::Stack(_))
    }

    pub fn is_heap(&self) -> bool {
        matches!(self, Self::Heap(_))
    }
}

impl From<&'_ str> for CStrBuf {
    fn from(src: &str) -> Self {
        Self::new(src)
    }
}

impl AsRef<CStr> for CStrBuf {
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

impl AsPtr<c_char> for CStrBuf {
    fn as_ptr(&self) -> *const c_char {
        self.as_ptr()
    }
}

pub trait ToEngineStr {
    type Output: AsPtr<c_char>;

    fn to_engine_str(&self) -> Self::Output;
}

impl ToEngineStr for &'_ str {
    type Output = CStrBuf;

    fn to_engine_str(&self) -> Self::Output {
        CStrBuf::new(self)
    }
}

impl ToEngineStr for String {
    type Output = CStrBuf;

    fn to_engine_str(&self) -> Self::Output {
        CStrBuf::new(self.as_str())
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
        let s = CStrBuf::<N>::new(src.to_str().unwrap());
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
