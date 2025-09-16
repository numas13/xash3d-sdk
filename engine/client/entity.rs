use core::{ffi::c_int, iter, mem, ptr};

use bitflags::bitflags;
use shared::ffi::api::efx::TEMPENTITY;

pub use shared::entity::*;

bitflags! {
    #[derive(Copy, Clone, Debug)]
    #[repr(transparent)]
    pub struct TempEntityFlags: c_int {
        const NONE                  = 0;
        const SINEWAVE              = 1 << 0;
        const GRAVITY               = 1 << 1;
        const ROTATE                = 1 << 2;
        const SLOWGRAVITY           = 1 << 3;
        const SMOKETRAIL            = 1 << 4;
        const COLLIDEWORLD          = 1 << 5;
        const FLICKER               = 1 << 6;
        const FADEOUT               = 1 << 7;
        const SPRANIMATE            = 1 << 8;
        const HITSOUND              = 1 << 9;
        const SPIRAL                = 1 << 10;
        const SPRCYCLE              = 1 << 11;
        const COLLIDEALL            = 1 << 12;
        const PERSIST               = 1 << 13;
        const COLLIDEKILL           = 1 << 14;
        const PLYRATTACHMENT        = 1 << 15;
        const SPRANIMATELOOP        = 1 << 16;
        const SPARKSHOWER           = 1 << 17;
        const NOMODEL               = 1 << 18;
        const CLIENTCUSTOM          = 1 << 19;
        const SCALE                 = 1 << 20;
    }
}

pub trait TempEntityExt {
    fn flags(&self) -> &TempEntityFlags;

    fn flags_mut(&mut self) -> &mut TempEntityFlags;
}

impl TempEntityExt for TEMPENTITY {
    fn flags(&self) -> &TempEntityFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    fn flags_mut(&mut self) -> &mut TempEntityFlags {
        unsafe { mem::transmute(&mut self.flags) }
    }
}

pub struct TempEntityList {
    head: *mut *mut TEMPENTITY,
    free: *mut *mut TEMPENTITY,
}

impl TempEntityList {
    /// Creates a `TempEntityList` directly from pointers.
    pub(crate) unsafe fn from_raw_parts(
        head: *mut *mut TEMPENTITY,
        free: *mut *mut TEMPENTITY,
    ) -> Self {
        Self { head, free }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut TEMPENTITY) -> bool,
    {
        unsafe {
            let mut prev = ptr::null_mut();
            let mut temp = *self.head;
            while !temp.is_null() {
                let next = (*temp).next;
                if f(&mut *temp) {
                    // keep item
                    prev = temp;
                } else {
                    // remove item
                    (*temp).next = *self.free;
                    *self.free = temp;
                    if prev.is_null() {
                        // remove from head
                        *self.head = next;
                    } else {
                        (*prev).next = next;
                    }
                }
                temp = next;
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TEMPENTITY> {
        let mut temp = unsafe { *self.head };
        iter::from_fn(move || {
            if !temp.is_null() {
                let ret = unsafe { &mut *temp };
                temp = ret.next;
                Some(ret)
            } else {
                None
            }
        })
    }
}
