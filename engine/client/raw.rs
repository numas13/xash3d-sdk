use core::mem;

use csz::CStrThin;

use shared::{
    ffi::{
        api::efx::TEMPENTITY,
        client::hud_player_info_s,
        common::{movevars_s, ref_params_s, usercmd_s},
    },
    utils::cstr_or_none,
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

pub trait HudPlayerInfoExt {
    fn name(&self) -> Option<&CStrThin>;
}

impl HudPlayerInfoExt for hud_player_info_s {
    fn name(&self) -> Option<&CStrThin> {
        unsafe { cstr_or_none(self.name) }
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
