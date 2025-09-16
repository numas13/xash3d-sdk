use core::{ffi::c_int, mem};

use bitflags::bitflags;

pub use shared::entity::*;
use shared::ffi::api::efx::TEMPENTITY;

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
