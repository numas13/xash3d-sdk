use core::ops::Deref;

use csz::CStrThin;

use crate::raw::{self, string_t, vec3_t};

pub struct ServerGlobals {
    raw: *mut raw::globalvars_t,
}

shared::export::impl_unsync_global!(ServerGlobals);

impl ServerGlobals {
    pub(crate) fn new(raw: *mut raw::globalvars_t) -> Self {
        Self { raw }
    }

    pub fn string(&self, string: string_t) -> &'static CStrThin {
        unsafe { CStrThin::from_ptr(self.pStringBase.wrapping_byte_add(string.0 as usize)) }
    }

    pub fn set_landmark_offset(&self, landmark_offset: vec3_t) {
        unsafe {
            (*self.raw).vecLandmarkOffset = landmark_offset;
        }
    }
}

impl Deref for ServerGlobals {
    type Target = raw::globalvars_t;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}
