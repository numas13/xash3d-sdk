use core::{
    ffi::c_char,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    slice,
};

use csz::CStrThin;

use crate::engine::UiEngineRef;

pub struct File {
    engine: UiEngineRef,
    data: *mut u8,
    len: usize,
}

impl File {
    pub(crate) unsafe fn new(engine: UiEngineRef, data: *mut u8, len: usize) -> Self {
        Self { engine, data, len }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }

    pub fn cursor(&mut self) -> Cursor<'_> {
        unsafe { Cursor::from_ptr(self.data) }
    }
}

impl Deref for File {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for File {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            self.engine.free_file(self.data.cast());
        }
    }
}

pub struct Cursor<'a> {
    pub(crate) data: *mut u8,
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
}

pub struct FileList<'a> {
    raw: &'a [*const c_char],
}

impl<'a> FileList<'a> {
    pub(crate) unsafe fn new(raw: &'a [*const c_char]) -> Self {
        Self { raw }
    }

    fn as_slice(&self) -> &'a [*const c_char] {
        self.raw
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a CStrThin> {
        self.as_slice()
            .iter()
            .map(|i| unsafe { CStrThin::from_ptr(*i) })
    }
}
