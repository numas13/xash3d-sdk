use core::{
    ffi::{c_int, c_uint},
    mem,
};

use bitflags::bitflags;
use xash3d_ffi::common::{ref_viewpass_s, vec3_t};

use crate::{
    ffi,
    macros::define_enum_for_primitive,
    math::{atanf, tanf},
};

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
        ViewPass::from(self.raw)
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

    pub const fn from_raw_ref(raw: &ref_viewpass_s) -> &ViewPass {
        unsafe { mem::transmute(raw) }
    }

    pub fn viewport(&self) -> [c_int; 4] {
        self.raw.viewport
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

impl From<ref_viewpass_s> for ViewPass {
    fn from(raw: ref_viewpass_s) -> Self {
        Self { raw }
    }
}

impl From<ViewPass> for ref_viewpass_s {
    fn from(value: ViewPass) -> Self {
        value.raw
    }
}

impl AsRef<ref_viewpass_s> for ViewPass {
    fn as_ref(&self) -> &ref_viewpass_s {
        &self.raw
    }
}

impl AsMut<ref_viewpass_s> for ViewPass {
    fn as_mut(&mut self) -> &mut ref_viewpass_s {
        &mut self.raw
    }
}

const RENDER_SCREEN_FADE_MODULATE: c_uint = ffi::render::kRenderScreenFadeModulate as c_uint;

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum RenderMode: c_int as c_uint {
        /// src
        #[default]
        Normal(ffi::common::kRenderNormal),
        /// c*a+dest*(1-a)
        TransColor(ffi::common::kRenderTransColor),
        /// src*a+dest*(1-a)
        TransTexture(ffi::common::kRenderTransTexture),
        /// src*a+dest -- No Z buffer checks
        Glow(ffi::common::kRenderGlow),
        /// src*srca+dest*(1-srca)
        TransAlpha(ffi::common::kRenderTransAlpha),
        /// src*a+dest
        TransAdd(ffi::common::kRenderTransAdd),

        /// Special rendermode for screenfade modulate.
        ScreenFadeModulate(RENDER_SCREEN_FADE_MODULATE),
    }
}

impl RenderMode {
    pub const fn is_opaque(&self) -> bool {
        matches!(self, Self::Normal)
    }
}

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum RenderFx: c_int as c_uint {
        #[default]
        None(ffi::common::kRenderFxNone),
        PulseSlow(ffi::common::kRenderFxPulseSlow),
        PulseFast(ffi::common::kRenderFxPulseFast),
        PulseSlowWide(ffi::common::kRenderFxPulseSlowWide),
        PulseFastWide(ffi::common::kRenderFxPulseFastWide),
        FadeSlow(ffi::common::kRenderFxFadeSlow),
        FadeFast(ffi::common::kRenderFxFadeFast),
        SolidSlow(ffi::common::kRenderFxSolidSlow),
        SolidFast(ffi::common::kRenderFxSolidFast),
        StrobeSlow(ffi::common::kRenderFxStrobeSlow),
        StrobeFast(ffi::common::kRenderFxStrobeFast),
        StrobeFaster(ffi::common::kRenderFxStrobeFaster),
        FlickerSlow(ffi::common::kRenderFxFlickerSlow),
        FlickerFast(ffi::common::kRenderFxFlickerFast),
        NoDissipation(ffi::common::kRenderFxNoDissipation),
        /// Distort/scale/translate flicker
        Distort(ffi::common::kRenderFxDistort),
        /// kRenderFxDistort + distance fade
        Hologram(ffi::common::kRenderFxHologram),
        /// kRenderAmt is the player index
        DeadPlayer(ffi::common::kRenderFxDeadPlayer),
        /// Scale up really big!
        Explode(ffi::common::kRenderFxExplode),
        /// Glowing Shell
        GlowShell(ffi::common::kRenderFxGlowShell),
        /// Keep this sprite from getting very small (SPRITES only!)
        ClampMinScale(ffi::common::kRenderFxClampMinScale),
        LightMultiplier(ffi::common::kRenderFxLightMultiplier),
    }
}
