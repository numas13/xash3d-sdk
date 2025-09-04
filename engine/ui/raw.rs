#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]

use core::ffi::{c_char, c_int, c_short, c_uchar};

use bitflags::bitflags;
use csz::CStrArray;

pub use shared::raw::*;

pub type HIMAGE = c_int;

pub const MENU_EXTENDED_API_VERSION: c_int = 1;

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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UI_FUNCTIONS {
    pub pfnVidInit: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnInit: Option<unsafe extern "C" fn()>,
    pub pfnShutdown: Option<unsafe extern "C" fn()>,
    pub pfnRedraw: Option<unsafe extern "C" fn(flTime: f32)>,
    pub pfnKeyEvent: Option<unsafe extern "C" fn(key: c_int, down: c_int)>,
    pub pfnMouseMove: Option<unsafe extern "C" fn(x: c_int, y: c_int)>,
    pub pfnSetActiveMenu: Option<unsafe extern "C" fn(active: c_int)>,
    pub pfnAddServerToList: Option<unsafe extern "C" fn(adr: netadr_s, info: *const c_char)>,
    pub pfnGetCursorPos: Option<unsafe extern "C" fn(pos_x: *mut c_int, pos_y: *mut c_int)>,
    pub pfnSetCursorPos: Option<unsafe extern "C" fn(pos_x: c_int, pos_y: c_int)>,
    pub pfnShowCursor: Option<unsafe extern "C" fn(show: c_int)>,
    pub pfnCharEvent: Option<unsafe extern "C" fn(key: c_int)>,
    pub pfnMouseInRect: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnIsVisible: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnCreditsActive: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnFinalCredits: Option<unsafe extern "C" fn()>,
}

pub type ADDTOUCHBUTTONTOLIST = Option<
    unsafe extern "C" fn(
        name: *const c_char,
        texture: *const c_char,
        command: *const c_char,
        color: *mut c_uchar,
        flags: c_int,
    ),
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UI_EXTENDED_FUNCTIONS {
    pub pfnAddTouchButtonToList: ADDTOUCHBUTTONTOLIST,
    pub pfnResetPing: Option<unsafe extern "C" fn()>,
    pub pfnShowConnectionWarning: Option<unsafe extern "C" fn()>,
    pub pfnShowUpdateDialog: Option<unsafe extern "C" fn(preferStore: c_int)>,
    pub pfnShowMessageBox: Option<unsafe extern "C" fn(text: *const c_char)>,
    pub pfnConnectionProgress_Disconnect: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_Download: Option<
        unsafe extern "C" fn(
            pszFileName: *const c_char,
            pszServerName: *const c_char,
            iCurrent: c_int,
            iTotal: c_int,
            comment: *const c_char,
        ),
    >,
    pub pfnConnectionProgress_DownloadEnd: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_Precache: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_Connect: Option<unsafe extern "C" fn(server: *const c_char)>,
    pub pfnConnectionProgress_ChangeLevel: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_ParseServerInfo: Option<unsafe extern "C" fn(server: *const c_char)>,
}
