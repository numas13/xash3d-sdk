use core::ops::{Deref, DerefMut};

use crate::raw;

pub struct Globals {
    raw: *mut raw::ui_globalvars_s,
}

shared::export::impl_unsync_global!(Globals);

impl Globals {
    pub(crate) fn new(raw: *mut raw::ui_globalvars_s) -> Self {
        Self { raw }
    }
}

impl Deref for Globals {
    type Target = raw::ui_globalvars_s;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for Globals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}
