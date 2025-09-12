use core::{ffi::c_int, mem};

use bitflags::bitflags;
use xash3d_ffi::common::{ref_viewpass_s, vec3_t};

use crate::math::{atanf, tanf};

bitflags! {
    /// ref_viewpass_s.flags
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    #[repr(transparent)]
    pub struct DrawFlags: c_int {
        /// pass should draw the world (otherwise it's player menu model)
        const WORLD         = 1 << 0;
        /// special 6x pass to render cubemap/skybox sides
        const CUBEMAP       = 1 << 1;
        /// overview mode is active
        const OVERVIEW      = 1 << 2;
        /// nothing is drawn by the engine except clientDraw functions
        const CLIENT_ONLY   = 1 << 3;
    }
}

#[derive(Copy, Clone)]
pub struct ViewPassBuilder {
    raw: ref_viewpass_s,
}

impl Default for ViewPassBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewPassBuilder {
    fn new() -> Self {
        Self {
            raw: ref_viewpass_s {
                fov_x: 40.0,
                ..unsafe { mem::zeroed() }
            },
        }
    }

    pub fn x(mut self, x: c_int) -> Self {
        self.raw.viewport[0] = x;
        self
    }

    pub fn y(mut self, y: c_int) -> Self {
        self.raw.viewport[1] = y;
        self
    }

    pub fn pos(self, x: c_int, y: c_int) -> Self {
        self.x(x).y(y)
    }

    pub fn view_origin(mut self, origin: impl Into<vec3_t>) -> Self {
        self.raw.vieworigin = origin.into();
        self
    }

    pub fn view_angles(mut self, angles: impl Into<vec3_t>) -> Self {
        self.raw.viewangles = angles.into();
        self
    }

    pub fn view_entity(mut self, entity: c_int) -> Self {
        self.raw.viewentity = entity;
        self
    }

    pub fn fov(mut self, fov: f32) -> Self {
        self.raw.fov_x = fov;
        self
    }

    pub fn flags(mut self, flags: DrawFlags) -> Self {
        self.raw.flags = flags.bits();
        self
    }

    pub fn build(mut self, width: i32, height: i32) -> ViewPass {
        self.raw.viewport[2] = width;
        self.raw.viewport[3] = height;
        let x = width as f32 / tanf(self.raw.fov_x.to_radians() * 0.5);
        self.raw.fov_y = atanf(height as f32 / x).to_degrees() * 2.0;
        ViewPass::from_raw(self.raw)
    }
}

#[derive(Copy, Clone)]
pub struct ViewPass {
    raw: ref_viewpass_s,
}

impl ViewPass {
    pub fn builder() -> ViewPassBuilder {
        ViewPassBuilder::new()
    }

    pub const fn from_raw(raw: ref_viewpass_s) -> ViewPass {
        Self { raw }
    }

    pub const fn into_raw(self) -> ref_viewpass_s {
        self.raw
    }

    pub fn x(&self) -> i32 {
        self.raw.viewport[0]
    }

    pub fn y(&self) -> i32 {
        self.raw.viewport[1]
    }

    pub fn width(&self) -> i32 {
        self.raw.viewport[2]
    }

    pub fn height(&self) -> i32 {
        self.raw.viewport[3]
    }

    pub fn origin(&self) -> vec3_t {
        self.raw.vieworigin
    }

    pub fn angles(&self) -> vec3_t {
        self.raw.viewangles
    }

    pub fn entity(&self) -> c_int {
        self.raw.viewentity
    }

    pub fn fov_x(&self) -> f32 {
        self.raw.fov_x
    }

    pub fn fov_y(&self) -> f32 {
        self.raw.fov_y
    }

    pub fn flags(&self) -> DrawFlags {
        DrawFlags::from_bits_retain(self.raw.flags)
    }

    pub fn set_flags(&mut self, flags: DrawFlags) {
        self.raw.flags = flags.bits();
    }

    pub fn flags_mut(&mut self) -> &mut DrawFlags {
        const_assert_size_of_field_eq!(DrawFlags, ref_viewpass_s, flags);
        unsafe { mem::transmute(&mut self.raw.flags) }
    }
}
