use core::{ffi::CStr, fmt};

use shared::ffi::menu::GAMEINFO;

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
