pub mod bsp;

use core::{ffi::c_int, mem, str};

use bitflags::bitflags;
use xash3d_ffi::common::{usercmd_s, vec3_t, wrect_s};

#[deprecated(note = "use input::KeyState instead")]
pub type KeyState = crate::input::KeyState;

pub use crate::input::KButtonExt;

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct SoundFlags: c_int {
        const NONE              = 0;
        /// A scaled byte.
        const VOLUME            = 1 << 0;
        /// A byte.
        const ATTENUATION       = 1 << 1;
        /// Get sentence from a script.
        const SEQUENCE          = 1 << 2;
        /// A byte.
        const PITCH             = 1 << 3;
        /// Set if sound num is actually a sentence num.
        const SENTENCE          = 1 << 4;
        /// Stop the sound.
        const STOP              = 1 << 5;
        /// Change sound vol.
        const CHANGE_VOL        = 1 << 6;
        /// Change sound pitch.
        const CHANGE_PITCH      = 1 << 7;
        /// We're spawning, used in some cases for ambients (not sent across network).
        const SPAWNING          = 1 << 8;
        /// Not paused, not looped, for internal use.
        const LOCALSOUND        = 1 << 9;
        /// Stop all looping sounds on the entity.
        const STOP_LOOPING      = 1 << 10;
        /// Don't send sound from local player if prediction was enabled.
        const FILTER_CLIENT     = 1 << 11;
        /// Passed playing position and the forced end.
        const RESTORE_POSITION  = 1 << 12;
    }
}

#[deprecated(note = "use entity::Effects instead")]
pub type Effects = crate::entity::Effects;

#[deprecated(note = "use entity::EdictFlags instead")]
pub type EdictFlags = crate::entity::EdictFlags;

pub use crate::entity::EntityStateExt;

#[deprecated(note = "use model::ModelType instead")]
pub type ModelType = crate::model::ModelType;

#[deprecated(note = "use model::ModelFlags instead")]
pub type ModelFlags = crate::model::ModelFlags;

pub use crate::model::ModelExt;

bitflags! {
    /// msurface_s.flags
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    #[repr(transparent)]
    pub struct SurfaceFlags: c_int {
        const NONE              = 0;
        const PLANEBACK         = 1 << 1; // plane should be negated
        const DRAWSKY           = 1 << 2; // sky surface
        const DRAWTURB_QUADS    = 1 << 3; // all subidivided polygons are quads
        const DRAWTURB          = 1 << 4; // warp surface
        const DRAWTILED         = 1 << 5; // face without lighmap
        const CONVEYOR          = 1 << 6; // scrolled texture (was SURF_DRAWBACKGROUND)
        const UNDERWATER        = 1 << 7; // caustics
        const TRANSPARENT       = 1 << 8; // it's a transparent texture (was SURF_DONTWARP)
    }
}

#[deprecated]
pub type RefFlags = crate::render::DrawFlags;

pub trait UserCmdExt {
    fn default() -> Self;

    fn move_vector(&self) -> vec3_t;

    fn move_vector_set(&mut self, vec: vec3_t);

    fn is_button(&self, button: c_int) -> bool;
}

impl UserCmdExt for usercmd_s {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }

    fn move_vector(&self) -> vec3_t {
        vec3_t::new(self.forwardmove, self.sidemove, self.upmove)
    }

    fn move_vector_set(&mut self, vec: vec3_t) {
        self.forwardmove = vec[0];
        self.sidemove = vec[1];
        self.upmove = vec[2];
    }

    fn is_button(&self, button: c_int) -> bool {
        self.buttons as c_int & button != 0
    }
}

pub const MAX_LIGHTSTYLES: usize = 256;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum SkyboxOrdering {
    Right = 0,
    Back = 1,
    Left = 2,
    Forward = 3,
    Up = 4,
    Down = 5,
}

#[deprecated(note = "use render::TextureFlags instead")]
pub type TextureFlags = crate::render::TextureFlags;

/// Max rendering decals per a level.
pub const MAX_RENDER_DECALS: usize = 4096;

pub trait WRectExt {
    fn default() -> Self;

    fn width(&self) -> c_int;

    fn height(&self) -> c_int;

    fn size(&self) -> (c_int, c_int) {
        (self.width(), self.height())
    }
}

impl WRectExt for wrect_s {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }

    fn width(&self) -> c_int {
        self.right - self.left
    }

    fn height(&self) -> c_int {
        self.bottom - self.top
    }
}
