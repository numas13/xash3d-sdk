pub mod bsp;

use core::{
    ffi::{c_int, c_uint},
    mem, str,
};

use bitflags::bitflags;
use xash3d_ffi::{
    common::{
        byte, entity_state_s, kbutton_t, model_s, ref_viewpass_s, usercmd_s, vec3_t, wrect_s,
    },
    player_move::physent_s,
};

use crate::consts::MAX_MOVEENTS;

bitflags! {
    /// kbutton_t.state
    #[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct KeyState: c_int {
        const DOWN = 1 << 0;
        const IMPULSE_DOWN = 1 << 1;
        const ANY_DOWN = Self::DOWN.union(Self::IMPULSE_DOWN).bits();
        const IMPULSE_UP = 1 << 2;
    }
}

pub trait KButtonExt {
    fn new() -> Self;

    fn state(&self) -> &KeyState;

    fn state_mut(&mut self) -> &mut KeyState;

    fn is_down(&self) -> bool {
        self.state().contains(KeyState::DOWN)
    }

    fn is_up(&self) -> bool {
        !self.is_down()
    }

    fn is_impulse_down(&self) -> bool {
        self.state().intersects(KeyState::IMPULSE_DOWN)
    }

    fn is_impulse_up(&self) -> bool {
        self.state().intersects(KeyState::IMPULSE_UP)
    }
}

impl KButtonExt for kbutton_t {
    fn new() -> Self {
        kbutton_t {
            down: [0; 2],
            state: 0,
        }
    }

    fn state(&self) -> &KeyState {
        unsafe { mem::transmute(&self.state) }
    }

    fn state_mut(&mut self) -> &mut KeyState {
        unsafe { mem::transmute(&mut self.state) }
    }
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct PictureFlags: u32 {
        const EXPAND_SOURCE = 1 << 0;
        const KEEP_SOURCE   = 1 << 1;
        const NEAREST       = 1 << 2;
        const NOFLIP_TGA    = 1 << 3;
    }
}

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

/// entity_state_s.rendermode
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(i32)]
pub enum RenderMode {
    /// src
    Normal,
    /// c*a+dest*(1-a)
    TransColor,
    /// src*a+dest*(1-a)
    TransTexture,
    /// src*a+dest -- No Z buffer checks
    Glow,
    /// src*srca+dest*(1-srca)
    TransAlpha,
    /// src*a+dest
    TransAdd,

    /// Special rendermode for screenfade modulate.
    ///
    /// Probably will be expanded at some point.
    ScreenFadeModulate = 0x1000,
}
const_assert_size_eq!(RenderMode, c_int);

impl RenderMode {
    pub fn from_raw(raw: c_int) -> Option<RenderMode> {
        match raw {
            0 => Some(Self::Normal),
            1 => Some(Self::TransColor),
            2 => Some(Self::TransTexture),
            3 => Some(Self::Glow),
            4 => Some(Self::TransAlpha),
            5 => Some(Self::TransAdd),
            0x1000 => Some(Self::ScreenFadeModulate),
            _ => None,
        }
    }

    pub const fn is_opaque(&self) -> bool {
        matches!(self, Self::Normal)
    }
}

/// entity_state_s.renderfx
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum RenderFx {
    None = 0,
    PulseSlow,
    PulseFast,
    PulseSlowWide,
    PulseFastWide,
    FadeSlow,
    FadeFast,
    SolidSlow,
    SolidFast,
    StrobeSlow,
    StrobeFast,
    StrobeFaster,
    FlickerSlow,
    FlickerFast,
    NoDissipation,
    /// Distort/scale/translate flicker
    Distort,
    /// kRenderFxDistort + distance fade
    Hologram,
    /// kRenderAmt is the player index
    DeadPlayer,
    /// Scale up really big!
    Explode,
    /// Glowing Shell
    GlowShell,
    /// Keep this sprite from getting very small (SPRITES only!)
    ClampMinScale,
    LightMultiplier,
}
const_assert_size_eq!(RenderFx, c_int);

impl RenderFx {
    #[deprecated]
    pub fn from_u32(value: u32) -> Option<RenderFx> {
        Self::from_raw(value as c_int)
    }

    pub fn from_raw(value: c_int) -> Option<RenderFx> {
        Some(match value {
            0 => Self::None,
            1 => Self::PulseSlow,
            2 => Self::PulseFast,
            3 => Self::PulseSlowWide,
            4 => Self::PulseFastWide,
            5 => Self::FadeSlow,
            6 => Self::FadeFast,
            7 => Self::SolidSlow,
            8 => Self::SolidFast,
            9 => Self::StrobeSlow,
            10 => Self::StrobeFast,
            11 => Self::StrobeFaster,
            12 => Self::FlickerSlow,
            13 => Self::FlickerFast,
            14 => Self::NoDissipation,
            15 => Self::Distort,
            16 => Self::Hologram,
            17 => Self::DeadPlayer,
            18 => Self::Explode,
            19 => Self::GlowShell,
            20 => Self::ClampMinScale,
            21 => Self::LightMultiplier,
            _ => return None,
        })
    }
}

bitflags! {
    /// entity_state_s.effects
    #[derive(Copy, Clone, Debug)]
    #[repr(transparent)]
    pub struct Effects: c_int {
        const NONE                  = 0;
        const BRIGHTFIELD           = 1 << 0;
        const MUZZLEFLASH           = 1 << 1;
        const BRIGHTLIGHT           = 1 << 2;
        const DIMLIGHT              = 1 << 3;
        const INVLIGHT              = 1 << 4;
        const NOINTERP              = 1 << 5;
        const LIGHT                 = 1 << 6;
        const NODRAW                = 1 << 7;
        const WATERSIDES            = 1 << 26;
        const FULLBRIGHT            = 1 << 27;
        const NOSHADOW              = 1 << 28;
        const MERGE_VISIBILITY      = 1 << 29;
        const REQUEST_PHS           = 1 << 30;
    }
}

/// model_s.type_
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum ModelType {
    Bad = -1,
    Brush = 0,
    Sprite = 1,
    Alias = 2,
    Studio = 3,
}

impl ModelType {
    pub fn from_raw(raw: c_int) -> Option<Self> {
        match raw {
            -1 => Some(Self::Bad),
            0 => Some(Self::Brush),
            1 => Some(Self::Sprite),
            2 => Some(Self::Alias),
            3 => Some(Self::Studio),
            _ => None,
        }
    }
}

/// used in mplane_s.type_
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
#[repr(transparent)]
pub struct PlaneType(pub byte);

impl PlaneType {
    // 0 - 2 are axial planes
    pub const X: Self = Self(0);
    pub const Y: Self = Self(1);
    pub const Z: Self = Self(2);
    // 3 needs alternate calc
    pub const NONAXIAL: Self = Self(3);
}

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

bitflags! {
    /// model_s.flags
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    #[repr(transparent)]
    pub struct ModelFlags: c_int {
        const CONVEYOR          = 1 << 0;
        const HAS_ORIGIN        = 1 << 1;
        // Model has only point hull.
        const LIQUID            = 1 << 2;
        // Model has transparent surfaces.
        const TRANSPARENT       = 1 << 3;
        // Lightmaps stored as RGB.
        const COLORED_LIGHTING  = 1 << 4;

        // uses 32-bit types.
        const QBSP2             = 1 << 28;
        /// It's a world model.
        const WORLD             = 1 << 29;
        /// A client sprite.
        const CLIENT            = 1 << 30;
    }
}

bitflags! {
    /// ref_viewpass_s.flags
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    #[repr(transparent)]
    pub struct RefFlags: c_int {
        /// pass should draw the world (otherwise it's player menu model)
        const DRAW_WORLD        = 1 << 0;
        /// special 6x pass to render cubemap/skybox sides
        const DRAW_CUBEMAP      = 1 << 1;
        /// overview mode is active
        const DRAW_OVERVIEW     = 1 << 2;
        /// nothing is drawn by the engine except clientDraw functions
        const ONLY_CLIENTDRAW   = 1 << 3;
    }
}

pub trait UserCmdExt {
    fn default() -> Self;

    fn move_vector(&self) -> vec3_t;

    fn move_vector_set(&mut self, vec: vec3_t);

    fn is_button(&self, button: u32) -> bool;
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

    fn is_button(&self, button: u32) -> bool {
        self.buttons as u32 & button != 0
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

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    #[repr(transparent)]
    pub struct TextureFlags: c_uint {
        /// Just for tabulate source.
        const COLORMAP          = 0;
        /// Disable texfilter.
        const NEAREST           = 1 << 0;
        /// Some images keep source.
        const KEEP_SOURCE       = 1 << 1;
        /// Steam background completely ignore tga attribute 0x20.
        const NOFLIP_TGA        = 1 << 2;
        /// Don't keep source as 8-bit expand to RGBA.
        const EXPAND_SOURCE     = 1 << 3;

        /// This is GL_TEXTURE_RECTANGLE.
        const RECTANGLE         = 1 << 5;
        /// It's cubemap texture.
        const CUBEMAP           = 1 << 6;
        /// Custom texture filter used.
        const DEPTHMAP          = 1 << 7;
        /// Image has an quake1 palette.
        const QUAKEPAL          = 1 << 8;
        /// Force image to grayscale.
        const LUMINANCE         = 1 << 9;
        /// This is a part of skybox.
        const SKYSIDE           = 1 << 10;
        /// Clamp texcoords to [0..1] range.
        const CLAMP             = 1 << 11;
        /// Don't build mips for this image.
        const NOMIPMAP          = 1 << 12;
        /// Sets by GL_UploadTexture.
        const HAS_LUMA          = 1 << 13;
        /// Create luma from quake texture (only q1 textures contain luma-pixels).
        const MAKELUMA          = 1 << 14;
        /// Is a normalmap.
        const NORMALMAP         = 1 << 15;
        /// Image has alpha (used only for GL_CreateTexture).
        const HAS_ALPHA         = 1 << 16;
        /// Force upload monochrome textures as RGB (detail textures).
        const FORCE_COLOR       = 1 << 17;
        /// Allow to update already loaded texture.
        const UPDATE            = 1 << 18;
        /// Zero clamp for projected textures.
        const BORDER            = 1 << 19;
        /// This is GL_TEXTURE_3D.
        const TEXTURE_3D        = 1 << 20;
        /// Bit who indicate lightmap page or deluxemap page.
        const ATLAS_PAGE        = 1 << 21;
        /// Special texture mode for A2C.
        const ALPHACONTRAST     = 1 << 22;

        /// This is set for first time when called glTexImage, otherwise it will be call glTexSubImage.
        const IMG_UPLOADED      = 1 << 25;
        /// Float textures.
        const ARB_FLOAT         = 1 << 26;
        /// Disable comparing for depth textures.
        const NOCOMPARE         = 1 << 27;
        /// Keep image as 16-bit (not 24).
        const ARB_16BIT         = 1 << 28;
        /// Multisampling texture.
        const MULTISAMPLE       = 1 << 29;
        /// Allows toggling nearest filtering for TF_NOMIPMAP textures.
        const ALLOW_NEAREST     = 1 << 30;
    }
}

impl TextureFlags {
    pub const SKY: Self = Self::SKYSIDE
        .union(Self::NOMIPMAP)
        .union(Self::ALLOW_NEAREST);
    pub const FONT: Self = Self::NOMIPMAP.union(Self::CLAMP).union(Self::ALLOW_NEAREST);
    pub const IMAGE: Self = Self::NOMIPMAP.union(Self::CLAMP);
    pub const DECAL: Self = Self::CLAMP;
}

/// Max rendering decals per a level.
pub const MAX_RENDER_DECALS: usize = 4096;

bitflags! {
    /// clientdata_s.flags
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct EdictFlags: c_int {
        /// Changes the SV_Movestep() behavior to not need to be on ground.
        const FLY           = 1 << 0;
        /// Changes the SV_Movestep() behavior to not need to be on ground (but stay in water).
        const SWIM          = 1 << 1;
        const CONVEYOR      = 1 << 2;
        const CLIENT        = 1 << 3;
        const INWATER       = 1 << 4;
        const MONSTER       = 1 << 5;
        const GODMODE       = 1 << 6;
        const NOTARGET      = 1 << 7;
        /// Don't send entity to local host, it's predicting this entity itself.
        const SKIPLOCALHOST = 1 << 8;
        /// At rest / on the ground.
        const ONGROUND      = 1 << 9;
        /// Not all corners are valid.
        const PARTIALGROUND = 1 << 10;
        /// Player jumping out of water.
        const WATERJUMP     = 1 << 11;
        /// Player is frozen for 3rd person camera.
        const FROZEN        = 1 << 12;
        /// JAC: fake client, simulated server side; don't send network messages to them.
        const FAKECLIENT    = 1 << 13;
        /// Player flag -- Player is fully crouched.
        const DUCKING       = 1 << 14;
        /// Apply floating force to this entity when in water.
        const FLOAT         = 1 << 15;
        /// Worldgraph has this ent listed as something that blocks a connection.
        const GRAPHED       = 1 << 16;

        // UNDONE: Do we need these?
        const IMMUNE_WATER  = 1 << 17;
        const IMMUNE_SLIME  = 1 << 18;
        const IMMUNE_LAVA   = 1 << 19;

        /// This is a spectator proxy.
        const PROXY         = 1 << 20;
        /// Brush model flag.
        ///
        /// Call think every frame regardless of nextthink - ltime (for
        /// constantly changing velocity/path).
        const ALWAYSTHINK   = 1 << 21;
        /// Base velocity has been applied this frame.
        ///
        /// Used to convert base velocity into momentum.
        const BASEVELOCITY  = 1 << 22;
        /// Only collide in with monsters who have FL_MONSTERCLIP set.
        const MONSTERCLIP   = 1 << 23;
        /// Player is _controlling_ a train.
        ///
        /// Movement commands should be ignored on client during prediction.
        const ONTRAIN       = 1 << 24;
        /// Not moveable/removeable brush entity.
        ///
        /// Really part of the world, but represented as an entity for transparency or something.
        const WORLDBRUSH    = 1 << 25;
        /// This client is a spectator.
        ///
        /// Don't run touch functions, etc.
        const SPECTATOR     = 1 << 26;
        /// Predicted laser spot from rocket launcher.
        const LASERDOT      = 1 << 27;

        /// This is a custom entity.
        const CUSTOMENTITY  = 1 << 29;
        /// This entity is marked for death.
        ///
        /// This allows the engine to kill ents at the appropriate time.
        const KILLME        = 1 << 30;
        /// Entity is dormant, no updates to client.
        const DORMANT       = 1 << 31;
    }
}

/// entity_state_s.movetype
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum MoveType {
    None = 0,
    // AngleNoClip = 1,
    // AngleClip = 2,
    Walk = 3,
    Step = 4,
    Fly = 5,
    Toss = 6,
    Push = 7,
    NoClip = 8,
    FlyMissile = 9,
    Bounce = 10,
    BounceMissile = 11,
    Follow = 12,
    PushStep = 13,
    Compound = 14,
}
const_assert_size_eq!(MoveType, c_int);

impl MoveType {
    pub fn from_raw(raw: c_int) -> Option<Self> {
        match raw {
            0 => Some(Self::None),
            // 1 => Some(Self::AngleNoClip),
            // 2 => Some(Self::AngleClip),
            3 => Some(Self::Walk),
            4 => Some(Self::Step),
            5 => Some(Self::Fly),
            6 => Some(Self::Toss),
            7 => Some(Self::Push),
            8 => Some(Self::NoClip),
            9 => Some(Self::FlyMissile),
            10 => Some(Self::Bounce),
            11 => Some(Self::BounceMissile),
            12 => Some(Self::Follow),
            13 => Some(Self::PushStep),
            14 => Some(Self::Compound),
            _ => None,
        }
    }
}

impl From<MoveType> for c_int {
    fn from(value: MoveType) -> c_int {
        value as c_int
    }
}

#[repr(C)]
pub struct MoveEnts {
    pub num: c_int,
    pub ents: [physent_s; MAX_MOVEENTS],
}

impl MoveEnts {
    pub fn as_slice(&self) -> &[physent_s] {
        &self.ents[..self.num as usize]
    }

    pub fn as_slice_mut(&mut self) -> &mut [physent_s] {
        &mut self.ents[..self.num as usize]
    }
}

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

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct BeamEntity(c_int);

impl BeamEntity {
    pub fn new(index: c_int, attachment: c_int) -> Self {
        assert!(index & !0xfff == 0);
        assert!(attachment & !0xf == 0);
        Self(index | (attachment << 12))
    }

    pub fn bits(&self) -> c_int {
        self.0
    }

    pub fn index(&self) -> c_int {
        self.0 & 0xfff
    }

    pub fn attachment(&self) -> c_int {
        (self.0 >> 12) & 0xf
    }
}

pub trait ModelExt {
    fn model_type(&self) -> ModelType;
}

impl ModelExt for model_s {
    fn model_type(&self) -> ModelType {
        ModelType::from_raw(self.type_).unwrap()
    }
}

pub trait EntityStateExt {
    fn renderfx(&self) -> RenderFx;

    fn rendermode(&self) -> RenderMode;

    fn effects(&self) -> &Effects;
}

impl EntityStateExt for entity_state_s {
    fn renderfx(&self) -> RenderFx {
        RenderFx::from_raw(self.renderfx).unwrap()
    }

    fn rendermode(&self) -> RenderMode {
        RenderMode::from_raw(self.rendermode).unwrap()
    }

    fn effects(&self) -> &Effects {
        unsafe { mem::transmute(&self.effects) }
    }
}

pub trait RefViewpassExt {
    fn flags(&self) -> &RefFlags;
}

impl RefViewpassExt for ref_viewpass_s {
    fn flags(&self) -> &RefFlags {
        unsafe { mem::transmute(&self.flags) }
    }
}
