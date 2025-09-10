use core::{ffi::c_int, mem};

use bitflags::bitflags;
use csz::CStrThin;
use shared::ffi::{self, menu::gameinfo2_s};

pub use shared::raw::*;

pub const GAMEINFO_VERSION: c_int = ffi::menu::GAMEINFO_VERSION as c_int;

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

pub trait GameInfo2Ext {
    fn gamefolder(&self) -> &CStrThin;
    fn startmap(&self) -> &CStrThin;
    fn trainmap(&self) -> &CStrThin;
    fn demomap(&self) -> &CStrThin;
    fn title(&self) -> &CStrThin;
    fn iconpath(&self) -> &CStrThin;
    fn version(&self) -> &CStrThin;
    fn game_url(&self) -> &CStrThin;
    fn update_url(&self) -> &CStrThin;
    fn type_(&self) -> &CStrThin;
    fn date(&self) -> &CStrThin;
    fn flags(&self) -> &GameInfoFlags;
    fn gamemode(&self) -> GameType;
}

macro_rules! get_cstr {
    ($($field:ident),* $(,)?) => {
        $(fn $field(&self) -> &CStrThin {
            unsafe { CStrThin::from_ptr(self.$field.as_ptr()) }
        })*
    };
}

impl GameInfo2Ext for gameinfo2_s {
    get_cstr! {
        gamefolder,
        startmap,
        trainmap,
        demomap,
        title,
        iconpath,
        version,
        game_url,
        update_url,
        type_,
        date,
    }

    fn flags(&self) -> &GameInfoFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    fn gamemode(&self) -> GameType {
        match self.gamemode {
            1 => GameType::SingleplayerOnly,
            2 => GameType::MultiplayerOnly,
            _ => GameType::Normal,
        }
    }
}
