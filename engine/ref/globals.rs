use core::{
    ffi::c_int,
    ops::{Deref, DerefMut},
};

use crate::raw;

pub struct Globals {
    raw: *mut raw::ref_globals_s,
}

shared::export::impl_unsync_global!(Globals);

impl Globals {
    pub(crate) fn new(raw: *mut raw::ref_globals_s) -> Self {
        Self { raw }
    }

    pub fn screen_size(&self) -> (c_int, c_int) {
        (self.width, self.height)
    }
}

impl Deref for Globals {
    type Target = raw::ref_globals_s;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for Globals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}
