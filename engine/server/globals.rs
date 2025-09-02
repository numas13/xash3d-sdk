use core::ops::Deref;

use crate::raw::{self, vec3_t};

pub struct ServerGlobals {
    raw: *mut raw::globalvars_t,
}

shared::export::impl_unsync_global!(ServerGlobals);

impl ServerGlobals {
    pub(crate) fn new(raw: *mut raw::globalvars_t) -> Self {
        Self { raw }
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
