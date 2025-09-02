use core::ffi::c_int;

use crate::raw;

pub struct RefGlobals {
    raw: *mut raw::ref_globals_s,
}

shared::export::impl_unsync_global!(RefGlobals);

impl RefGlobals {
    pub(crate) fn new(raw: *mut raw::ref_globals_s) -> Self {
        Self { raw }
    }

    pub fn screen_width(&self) -> c_int {
        unsafe { (*self.raw).width }
    }

    pub fn set_screen_width(&self, width: c_int) {
        unsafe { (*self.raw).width = width }
    }

    pub fn screen_height(&self) -> c_int {
        unsafe { (*self.raw).height }
    }

    pub fn set_screen_height(&self, height: c_int) {
        unsafe { (*self.raw).height = height }
    }

    pub fn screen_size(&self) -> (c_int, c_int) {
        (self.screen_width(), self.screen_height())
    }
}
