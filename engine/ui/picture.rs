use core::{
    ffi::{c_int, CStr},
    fmt,
};

use bitflags::bitflags;
use shared::ffi::{self, menu::HIMAGE};

use crate::{color::RGBA, engine_types::Size, prelude::*};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct PictureFlags: u32 {
        const NONE          = 0;
        /// Disable a texture filtering.
        const NEAREST       = ffi::menu::PIC_NEAREST as u32;
        /// Keep an image source.
        const KEEP_SOURCE   = ffi::menu::PIC_KEEP_SOURCE as u32;
        const NOFLIP_TGA    = ffi::menu::PIC_NOFLIP_TGA as u32;
        /// Expand an image source to 32-bit RGBA.
        const EXPAND_SOURCE = ffi::menu::PIC_EXPAND_SOURCE as u32;
    }
}

#[derive(Debug)]
pub enum PictureError {
    LoadError,
    InvalidPathError,
}

impl fmt::Display for PictureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LoadError => "failed to load a picture".fmt(f),
            Self::InvalidPathError => "invalid picture path".fmt(f),
        }
    }
}

// FIXME: use core::error::Error when MSRV >= 1.81
#[cfg(feature = "std")]
impl std::error::Error for PictureError {}

pub struct Picture<T: AsRef<CStr> = &'static CStr> {
    raw: HIMAGE,
    path: T,
}

impl<T: AsRef<CStr>> Picture<T> {
    fn new(path: T, buf: Option<&[u8]>, flags: PictureFlags) -> Result<Self, PictureError> {
        engine()
            .pic_load_with_flags(path.as_ref(), buf, flags)
            .ok_or(PictureError::LoadError)
            .map(|raw| Self { raw, path })
    }

    pub fn raw(&self) -> HIMAGE {
        self.raw
    }

    pub fn path(&self) -> &T {
        &self.path
    }

    pub fn create_with_flags(
        path: T,
        buf: &[u8],
        flags: PictureFlags,
    ) -> Result<Self, PictureError> {
        if path.as_ref().to_bytes().starts_with(b"#") {
            Self::new(path, Some(buf), flags)
        } else {
            Err(PictureError::InvalidPathError)
        }
    }

    pub fn create(path: T, buf: &[u8]) -> Result<Self, PictureError> {
        Self::create_with_flags(path, buf, PictureFlags::empty())
    }

    pub fn load(path: T, flags: PictureFlags) -> Result<Self, PictureError> {
        Self::new(path, None, flags)
    }

    pub fn as_raw(&self) -> c_int {
        self.raw
    }

    pub fn width(&self) -> c_int {
        engine().pic_width(self.raw)
    }

    pub fn height(&self) -> c_int {
        engine().pic_height(self.raw)
    }

    pub fn size(&self) -> Size {
        engine().pic_size(self.raw)
    }

    pub fn set_with_color<C: Into<RGBA>>(&self, color: C) {
        engine().pic_set(self.raw, color);
    }

    pub fn set(&self) {
        self.set_with_color(RGBA::WHITE);
    }
}

// FIXME: The engine uses one id for all pictures with same path.
// impl<T: AsRef<CStr>> Drop for Picture<T> {
//     fn drop(&mut self) {
//         engine().pic_free(self.path.as_ref());
//     }
// }
