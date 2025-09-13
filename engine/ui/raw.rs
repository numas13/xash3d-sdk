use core::{ffi::c_int, mem};

use csz::CStrThin;
use shared::ffi::{self, menu::gameinfo2_s};

pub use shared::raw::*;

#[deprecated(note = "use xash3d_ui::ffi::menu::GAMEINFO_VERSION instead")]
pub const GAMEINFO_VERSION: c_int = ffi::menu::GAMEINFO_VERSION as c_int;

#[deprecated(note = "use xash3d_ui::game_info::GameInfoFlags instead")]
pub type GameInfoFlags = crate::game_info::GameInfoFlags;

#[deprecated(note = "use xash3d_ui::game_info::GameType instead")]
pub type GameType = crate::game_info::GameType;

#[deprecated(note = "the trait will be removed")]
#[allow(deprecated)]
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

#[allow(deprecated)]
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
