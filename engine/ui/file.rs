use core::ffi::c_char;

use xash3d_shared::csz::CStrThin;

use crate::engine::UiEngine;

pub use xash3d_shared::file::Cursor;

pub type File = xash3d_shared::file::File<UiEngine>;

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
