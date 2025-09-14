use core::fmt;

use bitflags::bitflags;
use shared::{
    ffi,
    misc::{Rect, Size},
    str::ToEngineStr,
};

use crate::{color::RGBA, prelude::*};

pub use ffi::menu::HIMAGE;

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

// TODO: use core::error::Error when MSRV >= 1.81
#[cfg(feature = "std")]
impl std::error::Error for PictureError {}

#[derive(Copy, Clone, Debug, Default)]
pub struct Picture {
    raw: HIMAGE,
}

impl Picture {
    pub const NONE: Self = Self { raw: 0 };

    pub const fn new(raw: HIMAGE) -> Self {
        Self { raw }
    }

    fn load_impl(
        path: impl ToEngineStr,
        buf: Option<&[u8]>,
        flags: PictureFlags,
    ) -> Result<Self, PictureError> {
        engine()
            .pic_load(path, buf, flags)
            .ok_or(PictureError::LoadError)
            .map(|raw| Self { raw })
    }

    pub fn load(path: impl ToEngineStr) -> Result<Self, PictureError> {
        Self::load_impl(path, None, PictureFlags::empty())
    }

    pub fn load_with_flags(
        path: impl ToEngineStr,
        flags: PictureFlags,
    ) -> Result<Self, PictureError> {
        Self::load_impl(path, None, flags)
    }

    pub fn create(path: impl ToEngineStr, buf: &[u8]) -> Result<Self, PictureError> {
        Self::create_with_flags(path, buf, PictureFlags::empty())
    }

    pub fn create_with_flags(
        path: impl ToEngineStr,
        buf: &[u8],
        flags: PictureFlags,
    ) -> Result<Self, PictureError> {
        let path = path.to_engine_str();
        if path.as_ref().to_bytes().starts_with(b"#") {
            Self::load_impl(path.as_ref(), Some(buf), flags)
        } else {
            Err(PictureError::InvalidPathError)
        }
    }

    pub fn as_raw(&self) -> HIMAGE {
        self.raw
    }

    pub fn is_none(&self) -> bool {
        self.raw == 0
    }

    pub fn width(&self) -> u32 {
        engine().pic_width(self.raw)
    }

    pub fn height(&self) -> u32 {
        engine().pic_height(self.raw)
    }

    pub fn size(&self) -> Size {
        engine().pic_size(self.raw)
    }

    pub fn draw(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        let engine = engine();
        engine.pic_set(self.raw, color);
        engine.pic_draw(area, pic_area);
    }

    pub fn draw_holes(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        let engine = engine();
        engine.pic_set(self.raw, color);
        engine.pic_draw_holes(area, pic_area);
    }

    pub fn draw_trans(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        let engine = engine();
        engine.pic_set(self.raw, color);
        engine.pic_draw_trans(area, pic_area);
    }

    pub fn draw_additive(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        let engine = engine();
        engine.pic_set(self.raw, color);
        engine.pic_draw_additive(area, pic_area);
    }
}
