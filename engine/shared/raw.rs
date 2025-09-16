use core::{ffi::c_int, mem};

use xash3d_ffi::common::wrect_s;

#[deprecated(note = "use input::KeyState instead")]
pub type KeyState = crate::input::KeyState;

pub use crate::input::KButtonExt;

#[deprecated(note = "use protocol::SoundFlags instead")]
pub type SoundFlags = crate::sound::SoundFlags;

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

#[deprecated(note = "use SurfaceFlags instead")]
pub type SurfaceFlags = crate::bsp::SurfaceFlags;

#[deprecated]
pub type RefFlags = crate::render::DrawFlags;

// TODO: remove when defined in ffi crate
pub const MAX_LIGHTSTYLES: usize = 256;
pub const MAX_RENDER_DECALS: usize = 4096;

#[deprecated(note = "will be removed")]
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
