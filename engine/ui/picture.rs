use bitflags::bitflags;
use shared::{
    ffi,
    misc::{Rect, Size},
};

use crate::{color::RGBA, engine::UiEngineRef};

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

#[derive(Copy, Clone, Debug)]
pub struct Picture {
    engine: UiEngineRef,
    raw: HIMAGE,
}

impl Picture {
    pub const fn new(engine: UiEngineRef, raw: HIMAGE) -> Self {
        Self { engine, raw }
    }

    pub fn as_raw(&self) -> HIMAGE {
        self.raw
    }

    pub fn is_none(&self) -> bool {
        self.raw == 0
    }

    pub fn width(&self) -> u32 {
        self.engine.pic_width(self.raw)
    }

    pub fn height(&self) -> u32 {
        self.engine.pic_height(self.raw)
    }

    pub fn size(&self) -> Size {
        self.engine.pic_size(self.raw)
    }

    pub fn draw(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        self.engine.pic_set(self.raw, color);
        self.engine.pic_draw(area, pic_area);
    }

    pub fn draw_holes(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        self.engine.pic_set(self.raw, color);
        self.engine.pic_draw_holes(area, pic_area);
    }

    pub fn draw_trans(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        self.engine.pic_set(self.raw, color);
        self.engine.pic_draw_trans(area, pic_area);
    }

    pub fn draw_additive(&self, color: impl Into<RGBA>, area: Rect, pic_area: Option<Rect>) {
        self.engine.pic_set(self.raw, color);
        self.engine.pic_draw_additive(area, pic_area);
    }
}
