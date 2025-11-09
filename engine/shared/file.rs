use core::{
    ffi::{CStr, FromBytesWithNulError},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    slice,
    str::{self, Utf8Error},
};

use crate::{
    engine::EngineFile,
    parser::{Tokens, TokensBytes},
};

pub struct File<T: EngineFile> {
    data: *mut u8,
    len: usize,
    phantom: PhantomData<T>,
}

impl<T: EngineFile> File<T> {
    pub(crate) unsafe fn new(data: *mut u8, len: usize) -> Self {
        Self {
            data,
            len,
            phantom: PhantomData,
        }
    }

    #[deprecated(note = "use as_bytes")]
    pub fn as_slice(&self) -> &[u8] {
        self.as_bytes()
    }

    #[deprecated(note = "use as_bytes_mut")]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }

    /// Returns a byte slice of this file.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }

    /// Returns a mutable byte slice of this file.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }

    /// Converts this file to a C string slice.
    pub fn as_c_str(&self) -> Result<&CStr, FromBytesWithNulError> {
        CStr::from_bytes_with_nul(self.as_bytes())
    }

    /// Converts this file to a string slice.
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.as_bytes())
    }

    /// Returns an opaque cursor to the data of this file.
    pub fn cursor(&mut self) -> Cursor<'_> {
        unsafe { Cursor::from_ptr(self.data) }
    }

    pub fn tokens_bytes(&self) -> TokensBytes<'_> {
        TokensBytes::new(self.as_bytes())
    }

    pub fn tokens_str(&self) -> Result<Tokens<'_>, Utf8Error> {
        self.as_str().map(Tokens::new)
    }
}

impl<T: EngineFile> Deref for File<T> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl<T: EngineFile> DerefMut for File<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_bytes_mut()
    }
}

impl<T: EngineFile> Drop for File<T> {
    fn drop(&mut self) {
        // SAFETY: we can not create a file without engine being initialized
        let engine = unsafe { T::global_assume_init_ref() };
        // SAFETY: this is a valid file
        unsafe {
            engine.free_file_raw(self.data.cast());
        }
    }
}

pub struct Cursor<'a> {
    data: *mut u8,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> Cursor<'a> {
    /// Creates cursor from a raw pointer.
    ///
    /// # Safety
    ///
    /// `data` must contain a nul-byte at the end.
    pub unsafe fn from_ptr(data: *mut u8) -> Self {
        Self {
            data,
            phantom: PhantomData,
        }
    }

    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        assert_eq!(slice.last(), Some(&0));
        unsafe { Self::from_ptr(slice.as_mut_ptr()) }
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.data
    }
}
