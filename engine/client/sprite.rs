use core::{ffi::c_int, num::NonZeroI32, ops::Deref, slice};

use xash3d_shared::{
    color::RGB,
    csz::CStrThin,
    ffi::{
        client::{HSPRITE, client_sprite_s},
        common::wrect_s,
    },
};

use crate::prelude::*;

pub trait ClientSpriteExt {
    fn sprite(&self) -> &CStrThin;

    fn name(&self) -> &CStrThin;
}

impl ClientSpriteExt for client_sprite_s {
    fn sprite(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.szSprite.as_ptr()) }
    }

    fn name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.szName.as_ptr()) }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpriteHandle {
    engine: ClientEngineRef,
    raw: NonZeroI32,
}

impl SpriteHandle {
    pub fn new(engine: ClientEngineRef, raw: HSPRITE) -> Option<Self> {
        NonZeroI32::new(raw).map(|raw| Self { engine, raw })
    }

    pub fn raw(&self) -> HSPRITE {
        self.raw.get()
    }

    pub fn frames(&self) -> i32 {
        self.engine.spr_frames(*self)
    }

    pub fn height(&self, frame: i32) -> i32 {
        self.engine.spr_height(*self, frame)
    }

    pub fn width(&self, frame: i32) -> i32 {
        self.engine.spr_width(*self, frame)
    }

    pub fn size(&self, frame: i32) -> (i32, i32) {
        self.engine.spr_size(*self, frame)
    }

    fn set(&self, color: RGB) {
        self.engine.spr_set(*self, color);
    }

    pub fn draw(&self, frame: i32, x: i32, y: i32, color: RGB) {
        self.set(color);
        self.engine.spr_draw(frame, x, y);
    }

    pub fn draw_rect(&self, frame: i32, x: i32, y: i32, color: RGB, rect: wrect_s) {
        self.set(color);
        self.engine.spr_draw_rect(frame, x, y, rect);
    }

    pub fn draw_holes(&self, frame: i32, x: i32, y: i32, color: RGB) {
        self.set(color);
        self.engine.spr_draw_holes(frame, x, y);
    }

    pub fn draw_holes_rect(&self, frame: i32, x: i32, y: i32, color: RGB, rect: wrect_s) {
        self.set(color);
        self.engine.spr_draw_holes_rect(frame, x, y, rect);
    }

    pub fn draw_additive(&self, frame: i32, x: i32, y: i32, color: RGB) {
        self.set(color);
        self.engine.spr_draw_additive(frame, x, y);
    }

    pub fn draw_additive_rect(&self, frame: i32, x: i32, y: i32, color: RGB, rect: wrect_s) {
        self.set(color);
        self.engine.spr_draw_additive_rect(frame, x, y, rect);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    handle: SpriteHandle,
    rect: wrect_s,
}

impl Sprite {
    pub fn new(handle: SpriteHandle, rect: wrect_s) -> Self {
        Sprite { handle, rect }
    }

    pub fn handle(&self) -> SpriteHandle {
        self.handle
    }

    pub fn frames(&self) -> i32 {
        self.handle.frames()
    }

    pub fn rect(&self) -> wrect_s {
        self.rect
    }

    pub fn width(&self) -> i32 {
        self.rect.right - self.rect.left
    }

    pub fn height(&self) -> i32 {
        self.rect.bottom - self.rect.top
    }

    pub fn size(&self) -> (i32, i32) {
        (self.width(), self.height())
    }

    pub fn draw(&self, frame: i32, x: i32, y: i32, color: RGB) {
        self.handle.draw_rect(frame, x, y, color, self.rect);
    }

    pub fn draw_holes(&self, frame: i32, x: i32, y: i32, color: RGB) {
        self.handle.draw_holes_rect(frame, x, y, color, self.rect);
    }

    pub fn draw_additive(&self, frame: i32, x: i32, y: i32, color: RGB) {
        self.handle
            .draw_additive_rect(frame, x, y, color, self.rect);
    }
}

pub struct SpriteList {
    data: *const client_sprite_s,
    len: usize,
}

impl SpriteList {
    // pub fn empty() -> Self {
    //     Self {
    //         data: ptr::null(),
    //         len: 0,
    //     }
    // }

    pub(crate) fn new(data: *const client_sprite_s, len: usize) -> Self {
        Self { data, len }
    }

    pub fn as_slice(&self) -> &[client_sprite_s] {
        if !self.data.is_null() {
            unsafe { slice::from_raw_parts(self.data, self.len) }
        } else {
            &[]
        }
    }

    pub fn find(&self, name: &CStrThin, res: c_int) -> Option<&client_sprite_s> {
        self.as_slice()
            .iter()
            .filter(|i| i.iRes <= res && i.name() == name)
            .max_by_key(|i| i.iRes)
    }
}

impl Deref for SpriteList {
    type Target = [client_sprite_s];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
