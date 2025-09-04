use core::ffi::c_int;

use shared::{
    export::impl_unsync_global,
    raw::{qboolean, vec3_t},
};

use crate::raw::sortedface_s;

#[allow(non_camel_case_types)]
pub type ref_globals_s = RefGlobalsRaw;

#[repr(C)]
pub struct RefGlobalsRaw {
    pub developer: qboolean,
    pub screen_width: c_int,
    pub screen_height: c_int,
    pub full_screen: qboolean,
    pub wide_screen: qboolean,
    pub vieworg: vec3_t,
    pub viewangles: vec3_t,
    pub draw_surfaces: *mut sortedface_s,
    pub max_surfaces: c_int,
    pub visbytes: usize,
    pub desktop_bits_pixel: c_int,
}

pub struct RefGlobals {
    raw: *mut RefGlobalsRaw,
}

impl_unsync_global!(RefGlobals);

impl RefGlobals {
    pub(crate) fn new(raw: *mut RefGlobalsRaw) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> *const RefGlobalsRaw {
        self.raw
    }

    pub fn raw_mut(&self) -> *mut RefGlobalsRaw {
        self.raw
    }

    pub fn screen_width(&self) -> c_int {
        unsafe { (*self.raw).screen_width }
    }

    pub fn set_screen_width(&self, width: c_int) {
        unsafe { (*self.raw).screen_width = width }
    }

    pub fn screen_height(&self) -> c_int {
        unsafe { (*self.raw).screen_height }
    }

    pub fn set_screen_height(&self, height: c_int) {
        unsafe { (*self.raw).screen_height = height }
    }

    pub fn screen_size(&self) -> (c_int, c_int) {
        (self.screen_width(), self.screen_height())
    }
}
