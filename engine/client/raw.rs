use core::{ffi::c_int, mem};

use bitflags::bitflags;
use csz::CStrThin;

use shared::{
    ffi::{
        api::efx::TEMPENTITY,
        client::{hud_player_info_s, SCREENINFO},
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

bitflags! {
    /// SCREENINFO.flags
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct ScreenInfoFlags: c_int {
        const NONE = 0;
        const SCREENFLASH = 1;
        const STRETCHED = 2;
    }
}

pub trait ScreenInfoExt {
    fn new() -> Self;

    fn width(&self) -> c_int;

    fn height(&self) -> c_int;

    fn size(&self) -> (c_int, c_int) {
        (self.width(), self.height())
    }

    fn sprite_resolution(&self) -> u32 {
        let (width, height) = self.size();
        if width > 2560 && height > 1600 {
            2560
        } else if width >= 1280 && height > 720 {
            1280
        } else if width >= 640 {
            640
        } else {
            320
        }
    }

    fn scale(&self) -> u32 {
        let (width, height) = self.size();
        if width > 2560 && height > 1600 {
            4
        } else if width >= 1280 && height > 720 {
            3
        } else if width >= 640 {
            2
        } else {
            1
        }
    }
}

impl ScreenInfoExt for SCREENINFO {
    fn new() -> Self {
        Self {
            iSize: mem::size_of::<Self>() as c_int,
            ..unsafe { mem::zeroed() }
        }
    }

    fn width(&self) -> c_int {
        self.iWidth
    }

    fn height(&self) -> c_int {
        self.iHeight
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
