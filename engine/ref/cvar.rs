use core::{ffi::CStr, ptr};

use xash3d_shared::{
    csz::CStrThin,
    ffi::{common::cvar_s, render::convar_s},
    macros::const_assert_size_eq,
};

use crate::prelude::*;

pub use xash3d_shared::cvar::*;

pub type Cvar<T = f32> = xash3d_shared::cvar::Cvar<RefEngine, T>;

const_assert_size_eq!(*mut cvar_s, Cvar);
const_assert_size_eq!(*mut cvar_s, Option<Cvar>);

#[deprecated]
pub const CVAR_SENTINEL: usize = 0xdeadbeefdeadbeef_u64 as usize;

#[deprecated]
#[allow(deprecated)]
pub trait ConVarExt {
    fn builder(name: &'static CStr) -> ConVarBuilder {
        ConVarBuilder::new(name)
    }

    fn name(&self) -> &CStrThin;

    fn value_c_str(&self) -> &CStrThin;

    fn value(&self) -> f32;
}

#[allow(deprecated)]
impl ConVarExt for convar_s {
    fn name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.name) }
    }

    fn value_c_str(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.string) }
    }

    fn value(&self) -> f32 {
        self.value
    }
}

#[deprecated]
pub struct ConVarBuilder {
    var: convar_s,
}

#[allow(deprecated)]
impl ConVarBuilder {
    pub const fn new(name: &'static CStr) -> Self {
        ConVarBuilder {
            var: convar_s {
                name: name.as_ptr().cast_mut(),
                string: c"".as_ptr().cast_mut(),
                flags: CVarFlags::NONE.bits(),
                value: 0.0,
                next: CVAR_SENTINEL as *mut convar_s,
                desc: ptr::null_mut(),
                def_string: ptr::null_mut(),
            },
        }
    }

    pub const fn value(mut self, value: &'static CStr) -> Self {
        self.var.string = value.as_ptr().cast_mut();
        self
    }

    pub const fn flags(mut self, flags: CVarFlags) -> Self {
        self.var.flags = flags.bits();
        self
    }

    pub const fn description(mut self, desc: &'static CStr) -> Self {
        self.var.desc = desc.as_ptr().cast_mut();
        self
    }

    pub const fn build(self) -> convar_s {
        self.var
    }
}
