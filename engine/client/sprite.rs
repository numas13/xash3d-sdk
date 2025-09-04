use core::{ffi::c_int, num::NonZeroI32, ops::Deref, slice};

use csz::{CStrArray, CStrThin};
use shared::raw::wrect_s;

pub type HSPRITE = c_int;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct client_sprite_s {
    pub name: CStrArray<64>,
    pub sprite: CStrArray<64>,
    pub hspr: c_int,
    pub res: c_int,
    pub rc: wrect_s,
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
            .filter(|i| i.res <= res && i.name == *name)
            .max_by_key(|i| i.res)
    }
}

impl Deref for SpriteList {
    type Target = [client_sprite_s];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
