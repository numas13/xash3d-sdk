use core::ffi::c_int;

use xash3d_shared::ffi::render::ref_globals_s;

pub struct RefGlobals {
    raw: *mut ref_globals_s,
}

impl RefGlobals {
    pub(crate) fn new(raw: *mut ref_globals_s) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> *const ref_globals_s {
        self.raw
    }

    pub fn raw_mut(&self) -> *mut ref_globals_s {
        self.raw
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
