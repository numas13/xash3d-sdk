use core::{
    ffi::{c_int, CStr},
    fmt, mem,
};

use bitflags::bitflags;
use csz::CStrThin;
use shared::{
    ffi::{
        self,
        menu::{gameinfo2_s, gametype_e, GAMEINFO},
    },
    macros::{const_assert_size_eq, define_enum_for_primitive},
};

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct GameInfoFlags: u32 {
        const NONE                  = 0;
        const NOMODELS              = ffi::menu::GFL_NOMODELS as u32;
        const NOSKILLS              = ffi::menu::GFL_NOSKILLS as u32;
        const RENDER_PICBUTTON_TEXT = ffi::menu::GFL_RENDER_PICBUTTON_TEXT as u32;
        const HD_BACKGROUND         = ffi::menu::GFL_HD_BACKGROUND as u32;
        const ANIMATED_TITLE        = ffi::menu::GFL_ANIMATED_TITLE as u32;
    }
}

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum GameType: gametype_e {
        Normal(ffi::menu::gametype_e_GAME_NORMAL),
        SingleplayerOnly(ffi::menu::gametype_e_GAME_SINGLEPLAYER_ONLY),
        MultiplayerOnly(ffi::menu::gametype_e_GAME_MULTIPLAYER_ONLY),
    }
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

#[derive(Clone)]
pub struct GameInfo {
    raw: GAMEINFO,
}

impl GameInfo {
    pub(crate) fn new(raw: GAMEINFO) -> Self {
        Self { raw }
    }

    pub fn game_folder(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.raw.gamefolder.as_ptr())
                .to_str()
                .unwrap()
        }
    }

    pub fn start_map(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.startmap.as_ptr()).to_str().unwrap() }
    }

    pub fn train_map(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.trainmap.as_ptr()).to_str().unwrap() }
    }

    pub fn title(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.title.as_ptr()).to_str().unwrap() }
    }

    pub fn version(&self) -> Option<&str> {
        unsafe { CStr::from_ptr(self.raw.version.as_ptr()).to_str().ok() }
    }

    #[inline(always)]
    pub fn flags(&self) -> u16 {
        self.raw.flags as u16
    }

    pub fn game_url(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.game_url.as_ptr()).to_str().unwrap() }
    }

    pub fn update_url(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.raw.update_url.as_ptr())
                .to_str()
                .unwrap()
        }
    }

    pub fn type_(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.type_.as_ptr()).to_str().unwrap() }
    }

    pub fn date(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.date.as_ptr()).to_str().unwrap() }
    }

    pub fn size(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.size.as_ptr()).to_str().unwrap() }
    }

    #[inline(always)]
    pub fn game_mode(&self) -> u32 {
        self.raw.gamemode as u32
    }
}

#[allow(deprecated)]
impl fmt::Debug for GameInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("GameInfo")
            .field("gamefolder", &self.game_folder())
            .field("startmap", &self.start_map())
            .field("trainmap", &self.train_map())
            .field("title", &self.title())
            .field("version", &self.version())
            .field("flags", &self.flags())
            .field("game_url", &self.game_url())
            .field("update_url", &self.update_url())
            .field("type", &self.type_())
            .field("date", &self.date())
            .field("size", &self.size())
            .field("gamemode", &self.game_mode())
            .finish()
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct GameInfo2 {
    raw: gameinfo2_s,
}

macro_rules! impl_get_cstr {
    ($(fn $meth:ident = $field:ident;)*) => {
        $(pub fn $meth(&self) -> &CStrThin {
            unsafe { CStrThin::from_ptr(self.raw.$field.as_ptr()) }
        })*
    };
}

impl GameInfo2 {
    pub(crate) fn from_raw_ref(raw: &gameinfo2_s) -> &GameInfo2 {
        const_assert_size_eq!(gameinfo2_s, GameInfo2);
        unsafe { mem::transmute(raw) }
    }

    pub fn as_raw(&self) -> &gameinfo2_s {
        &self.raw
    }

    pub fn info_version(&self) -> c_int {
        self.raw.gi_version
    }

    impl_get_cstr! {
        fn game_dir = gamefolder;
        fn start_map = startmap;
        fn train_map = trainmap;
        fn demo_map = demomap;
        fn title = title;
        fn icon_path = iconpath;
        fn game_version = version;
        fn game_url = game_url;
        fn update_url = update_url;
        fn game_type = type_;
        fn date = date;
    }

    pub fn flags(&self) -> GameInfoFlags {
        GameInfoFlags::from_bits_retain(self.raw.flags)
    }

    pub fn size(&self) -> u64 {
        self.raw.size
    }

    pub fn game_mode(&self) -> GameType {
        GameType::from_raw(self.raw.gamemode).unwrap_or(GameType::Normal)
    }
}

impl fmt::Debug for GameInfo2 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("GameInfo")
            .field("info_version", &self.info_version())
            .field("game_dir", &self.game_dir())
            .field("start_map", &self.start_map())
            .field("train_map", &self.train_map())
            .field("demo_map", &self.demo_map())
            .field("title", &self.title())
            .field("icon_path", &self.icon_path())
            .field("game_version", &self.game_version())
            .field("game_url", &self.game_url())
            .field("update_url", &self.update_url())
            .field("game_type", &self.game_type())
            .field("date", &self.date())
            .field("flags", &self.flags())
            .field("size", &self.size())
            .field("gamemode", &self.game_type())
            .finish()
    }
}
