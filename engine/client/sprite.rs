use core::{ffi::c_int, num::NonZeroI32, ops::Deref, slice};

use xash3d_shared::{
    csz::CStrThin,
    ffi::client::{HSPRITE, client_sprite_s},
};

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
#[repr(transparent)]
pub struct SpriteHandle {
    raw: NonZeroI32,
}

impl SpriteHandle {
    pub fn new(raw: HSPRITE) -> Option<Self> {
        NonZeroI32::new(raw).map(|raw| Self { raw })
    }

    pub fn raw(&self) -> HSPRITE {
        self.raw.get()
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
