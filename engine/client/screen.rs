use core::{
    ffi::{c_int, c_short},
    mem,
};

use bitflags::bitflags;
use xash3d_shared::ffi::{self, client::SCREENINFO};

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct ScreenInfoFlags: c_int {
        const NONE = 0;
        const SCREENFLASH = ffi::client::SCRINFO_SCREENFLASH;
        const STRETCHED = ffi::client::SCRINFO_STRETCHED;
    }
}

#[derive(Copy, Clone)]
pub struct ScreenInfo {
    raw: SCREENINFO,
}

impl Default for ScreenInfo {
    fn default() -> Self {
        Self {
            // TODO: replace with derive Default
            raw: SCREENINFO {
                iSize: mem::size_of::<Self>() as c_int,
                ..unsafe { mem::zeroed() }
            },
        }
    }
}

impl ScreenInfo {
    pub fn width(&self) -> c_int {
        self.raw.iWidth
    }

    pub fn height(&self) -> c_int {
        self.raw.iHeight
    }

    pub fn size(&self) -> (c_int, c_int) {
        (self.width(), self.height())
    }

    pub fn flags(&self) -> ScreenInfoFlags {
        ScreenInfoFlags::from_bits_retain(self.raw.iFlags)
    }

    pub fn char_height(&self) -> c_int {
        self.raw.iCharHeight
    }

    pub fn char_widths(&self) -> &[c_short; 256] {
        &self.raw.charWidths
    }

    pub fn char_width(&self, c: u8) -> c_short {
        self.char_widths()[c as usize]
    }

    pub fn sprite_resolution(&self) -> u32 {
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

    pub fn scale(&self) -> u32 {
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

impl From<ScreenInfo> for SCREENINFO {
    fn from(info: ScreenInfo) -> Self {
        info.raw
    }
}

impl From<SCREENINFO> for ScreenInfo {
    fn from(raw: SCREENINFO) -> Self {
        Self { raw }
    }
}

impl AsRef<SCREENINFO> for ScreenInfo {
    fn as_ref(&self) -> &SCREENINFO {
        &self.raw
    }
}

impl AsMut<SCREENINFO> for ScreenInfo {
    fn as_mut(&mut self) -> &mut SCREENINFO {
        &mut self.raw
    }
}
