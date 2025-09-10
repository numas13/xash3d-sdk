#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::type_complexity)]

use core::{ffi::c_int, mem};

use bitflags::bitflags;
use csz::CStrThin;
use shared::utils::cstr_or_none;

pub use shared::raw::*;

// TODO: remove me
#[rustfmt::skip]
pub use shared::ffi::{
    common::local_state_s,
    common::ref_params_s,

    api::efx::TEMPENTITY,

    client::SCREENINFO,
    client::client_data_s,
    client::client_textmessage_s,
    client::cmdalias_s,
    client::hud_player_info_s,
    client::tagPOINT,
};

bitflags! {
    #[derive(Copy, Clone, Debug)]
    #[repr(transparent)]
    pub struct TempEntFlags: c_int {
        const NONE                  = 0;
        const SINEWAVE              = 1 << 0;
        const GRAVITY               = 1 << 1;
        const ROTATE                = 1 << 2;
        const SLOWGRAVITY           = 1 << 3;
        const SMOKETRAIL            = 1 << 4;
        const COLLIDEWORLD          = 1 << 5;
        const FLICKER               = 1 << 6;
        const FADEOUT               = 1 << 7;
        const SPRANIMATE            = 1 << 8;
        const HITSOUND              = 1 << 9;
        const SPIRAL                = 1 << 10;
        const SPRCYCLE              = 1 << 11;
        const COLLIDEALL            = 1 << 12;
        const PERSIST               = 1 << 13;
        const COLLIDEKILL           = 1 << 14;
        const PLYRATTACHMENT        = 1 << 15;
        const SPRANIMATELOOP        = 1 << 16;
        const SPARKSHOWER           = 1 << 17;
        const NOMODEL               = 1 << 18;
        const CLIENTCUSTOM          = 1 << 19;
        const SCALE                 = 1 << 20;
    }
}

pub trait TempEntityExt {
    fn flags(&self) -> &TempEntFlags;

    fn flags_mut(&mut self) -> &mut TempEntFlags;
}

impl TempEntityExt for TEMPENTITY {
    fn flags(&self) -> &TempEntFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    fn flags_mut(&mut self) -> &mut TempEntFlags {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum EntityType {
    Normal = 0,
    Player = 1,
    TempEntity = 2,
    Beam = 3,
    Fragmented = 4,
}

impl EntityType {
    pub fn from_raw(raw: c_int) -> Option<Self> {
        match raw {
            0 => Some(Self::Normal),
            1 => Some(Self::Player),
            2 => Some(Self::TempEntity),
            3 => Some(Self::Beam),
            4 => Some(Self::Fragmented),
            _ => None,
        }
    }
}
