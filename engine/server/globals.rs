use core::{
    ffi::CStr,
    ops::{Deref, DerefMut},
};

use csz::CStrThin;

use crate::{
    prelude::*,
    raw::{self, string_t},
};

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

    #[deprecated = "use Engine::alloc_string"]
    pub fn make_string(&self, s: &CStr) -> string_t {
        engine().alloc_string(s)
    }
}

impl Deref for ServerGlobals {
    type Target = raw::globalvars_t;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for ServerGlobals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}
