use core::mem;

use shared::ffi::{
    api::efx::TEMPENTITY,
    common::{movevars_s, ref_params_s, usercmd_s},
};

use crate::entity::TempEntityFlags;

pub use shared::raw::*;

pub trait TempEntityExt {
    fn flags(&self) -> &TempEntityFlags;

    fn flags_mut(&mut self) -> &mut TempEntityFlags;
}

impl TempEntityExt for TEMPENTITY {
    fn flags(&self) -> &TempEntityFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    fn flags_mut(&mut self) -> &mut TempEntityFlags {
        unsafe { mem::transmute(&mut self.flags) }
    }
}

pub trait RefParamsExt {
    fn movevars(&self) -> &movevars_s;

    fn cmd(&self) -> &usercmd_s;
}

impl RefParamsExt for ref_params_s {
    fn movevars(&self) -> &movevars_s {
        unsafe { &*self.movevars }
    }

    fn cmd(&self) -> &usercmd_s {
        unsafe { &*self.cmd }
    }
}
