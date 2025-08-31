use core::ops::{Deref, DerefMut};

use csz::CStrThin;

use crate::raw::{self, string_t};

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
