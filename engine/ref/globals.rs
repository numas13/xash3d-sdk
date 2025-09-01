use core::{
    ffi::c_int,
    ops::Deref,
};

use crate::raw;

pub struct RefGlobals {
    raw: *mut raw::ref_globals_s,
}

shared::export::impl_unsync_global!(RefGlobals);

impl RefGlobals {
    pub(crate) fn new(raw: *mut raw::ref_globals_s) -> Self {
        Self { raw }
    }

    pub fn screen_size(&self) -> (c_int, c_int) {
        (self.width, self.height)
    }
}

impl Deref for RefGlobals {
    type Target = raw::ref_globals_s;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}
