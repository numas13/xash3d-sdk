#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use bitflags::bitflags;

use crate::ffi;

pub const MAX_MAP_TEXTURES: usize = ffi::common::MAX_MAP_TEXTURES as usize;

pub const MAX_MAP_LEAFS: usize = ffi::common::MAX_MAP_LEAFS as usize;
pub const MAX_MAP_LEAFS_BYTES: usize = MAX_MAP_LEAFS.div_ceil(8);

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct SurfaceFlags: u32 {
        /// Plane should be negated.
        const PLANEBACK        = 1 << 1;
        /// Sky surface.
        const DRAWSKY          = 1 << 2;
        /// All subidivided polygons are quads.
        const DRAWTURB_QUADS   = 1 << 3;
        /// Warp surface.
        const DRAWTURB         = 1 << 4;
        /// Face without lighmap.
        const DRAWTILED        = 1 << 5;
        /// Scrolled texture (was SURF_DRAWBACKGROUND).
        const CONVEYOR         = 1 << 6;
        /// Caustics.
        const UNDERWATER       = 1 << 7;
        /// It's a transparent texture (was SURF_DONTWARP).
        const TRANSPARENT      = 1 << 8;
    }
}
