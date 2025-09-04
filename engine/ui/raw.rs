#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]

use core::ffi::{c_int, c_short};

use bitflags::bitflags;
use csz::CStrArray;

pub use shared::raw::*;

pub type HIMAGE = c_int;

pub const GAMEINFO_VERSION: c_int = 2;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GAMEINFO {
    pub gamefolder: CStrArray<64>,
    pub startmap: CStrArray<64>,
    pub trainmap: CStrArray<64>,
    pub title: CStrArray<64>,
    pub version: CStrArray<14>,
    pub flags: c_short,
    pub game_url: CStrArray<256>,
    pub update_url: CStrArray<256>,
    pub type_: CStrArray<64>,
    pub date: CStrArray<64>,
    pub size: CStrArray<64>,
    pub gamemode: c_int,
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct GameInfoFlags: u32 {
        const NONE                  = 0;
        const NOMODELS              = 1 << 0;
        const NOSKILLS              = 1 << 1;
        const RENDER_PICBUTTON_TEXT = 1 << 2;
        const HD_BACKGROUND         = 1 << 3;
        const ANIMATED_TITLE        = 1 << 4;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[repr(C)]
pub enum GameType {
    Normal = 0,
    SingleplayerOnly = 1,
    MultiplayerOnly = 2,
}

impl GameType {
    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    pub fn is_singleplayer_only(&self) -> bool {
        matches!(self, Self::SingleplayerOnly)
    }

    pub fn is_multiplayer_only(&self) -> bool {
        matches!(self, Self::MultiplayerOnly)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct gameinfo2_s {
    pub gi_version: c_int,
    pub gamefolder: CStrArray<64>,
    pub startmap: CStrArray<64>,
    pub trainmap: CStrArray<64>,
    pub demomap: CStrArray<64>,
    pub title: CStrArray<64>,
    pub iconpath: CStrArray<64>,
    pub version: CStrArray<16>,
    pub flags: GameInfoFlags,
    pub game_url: CStrArray<256>,
    pub update_url: CStrArray<256>,
    pub type_: CStrArray<64>,
    pub date: CStrArray<64>,
    pub size: u64,
    pub gamemode: GameType,
}
