#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]

pub mod bsp;

use core::{
    ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_ushort, c_void},
    fmt,
    marker::PhantomData,
    slice, str,
};

use bitflags::bitflags;
use csz::CStrArray;

use self::bsp::dmodel_t;

use crate::{
    color::{RGB, RGBA},
    consts::{
        self, HISTORY_MAX, MAXLIGHTMAPS, MAX_MAP_HULLS, MAX_MOVEENTS, MAX_PHYSINFO_STRING,
        MAX_SKINS, NUM_GLYPHS, VERTEXSIZE,
    },
    cvar::cvar_s,
};

pub use math::{vec2_t, vec3_t, vec4_t, Vector};

pub type playermove_s = c_void;

#[derive(Default)]
#[repr(C)]
pub struct DynArray<T>(PhantomData<T>, [T; 0]);

impl<T> DynArray<T> {
    #[inline]
    pub const fn new() -> Self {
        DynArray(PhantomData, [])
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self as *const _ as *const T
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self as *mut _ as *mut T
    }

    #[inline]
    pub unsafe fn as_slice(&self, len: usize) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), len) }
    }

    #[inline]
    pub unsafe fn as_mut_slice(&mut self, len: usize) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), len) }
    }
}

impl<T> fmt::Debug for DynArray<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("DynArray")
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct KeyState: c_int {
        const DOWN = 1 << 0;
        const IMPULSE_DOWN = 1 << 1;
        const ANY_DOWN = Self::DOWN.union(Self::IMPULSE_DOWN).bits();
        const IMPULSE_UP = 1 << 2;
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct kbutton_t {
    pub down: [c_int; 2],
    pub state: KeyState,
}

impl kbutton_t {
    pub const fn new() -> Self {
        kbutton_t {
            down: [0; 2],
            state: KeyState::empty(),
        }
    }

    pub fn is_down(&self) -> bool {
        self.state.contains(KeyState::DOWN)
    }

    pub fn is_up(&self) -> bool {
        !self.is_down()
    }

    pub fn is_impulse_down(&self) -> bool {
        self.state.intersects(KeyState::IMPULSE_DOWN)
    }

    pub fn is_impulse_up(&self) -> bool {
        self.state.intersects(KeyState::IMPULSE_UP)
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

pub type byte = c_uchar;
pub type poolhandle_t = u32;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct qboolean(pub c_uint);

impl qboolean {
    pub const FALSE: qboolean = qboolean(0);
    pub const TRUE: qboolean = qboolean(1);

    pub const fn to_bool(self) -> bool {
        self.0 != Self::FALSE.0
    }
}

impl From<bool> for qboolean {
    fn from(value: bool) -> Self {
        qboolean(value as c_uint)
    }
}

impl From<qboolean> for bool {
    fn from(value: qboolean) -> Self {
        value != qboolean::FALSE
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct string_t(pub c_int);

impl string_t {
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl Default for string_t {
    fn default() -> Self {
        Self::null()
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct link_s {
    pub prev: *mut link_s,
    pub next: *mut link_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct plane_t {
    pub normal: vec3_t,
    pub dist: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct trace_t {
    pub allsolid: qboolean,
    pub startsolid: qboolean,
    pub inopen: qboolean,
    pub inwater: qboolean,
    pub fraction: f32,
    pub endpos: vec3_t,
    pub plane: plane_t,
    pub ent: *mut edict_s,
    pub hitgroup: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct pmplane_t {
    pub normal: vec3_t,
    pub dist: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct pmtrace_s {
    pub allsolid: qboolean,
    pub startsolid: qboolean,
    pub inopen: qboolean,
    pub inwater: qboolean,
    pub fraction: f32,
    pub endpos: vec3_t,
    pub plane: pmplane_t,
    pub ent: c_int,
    pub deltavelocity: vec3_t,
    pub hitgroup: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct entvars_s {
    pub classname: string_t,
    pub globalname: string_t,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub velocity: vec3_t,
    pub basevelocity: vec3_t,
    pub clbasevelocity: vec3_t,
    pub movedir: vec3_t,
    pub angles: vec3_t,
    pub avelocity: vec3_t,
    pub punchangle: vec3_t,
    pub v_angle: vec3_t,
    pub endpos: vec3_t,
    pub startpos: vec3_t,
    pub impacttime: f32,
    pub starttime: f32,
    pub fixangle: c_int,
    pub idealpitch: f32,
    pub pitch_speed: f32,
    pub ideal_yaw: f32,
    pub yaw_speed: f32,
    pub modelindex: c_int,
    pub model: string_t,
    pub viewmodel: string_t,
    pub weaponmodel: string_t,
    pub absmin: vec3_t,
    pub absmax: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub size: vec3_t,
    pub ltime: f32,
    pub nextthink: f32,
    pub movetype: MoveType,
    pub solid: c_int,
    pub skin: c_int,
    pub body: c_int,
    pub effects: Effects,
    pub gravity: f32,
    pub friction: f32,
    pub light_level: c_int,
    pub sequence: c_int,
    pub gaitsequence: c_int,
    pub frame: f32,
    pub animtime: f32,
    pub framerate: f32,
    pub controller: [byte; 4],
    pub blending: [byte; 2],
    pub scale: f32,
    pub rendermode: RenderMode,
    pub renderamt: f32,
    pub rendercolor: vec3_t,
    pub renderfx: c_int,
    pub health: f32,
    pub frags: f32,
    pub weapons: c_int,
    pub takedamage: f32,
    pub deadflag: c_int,
    pub view_ofs: vec3_t,
    pub button: c_int,
    pub impulse: c_int,
    pub chain: *mut edict_s,
    pub dmg_inflictor: *mut edict_s,
    pub enemy: *mut edict_s,
    pub aiment: *mut edict_s,
    pub owner: *mut edict_s,
    pub groundentity: *mut edict_s,
    pub spawnflags: c_int,
    pub flags: EdictFlags,
    pub colormap: c_int,
    pub team: c_int,
    pub max_health: f32,
    pub teleport_time: f32,
    pub armortype: f32,
    pub armorvalue: f32,
    pub waterlevel: c_int,
    pub watertype: c_int,
    pub target: string_t,
    pub targetname: string_t,
    pub netname: string_t,
    pub message: string_t,
    pub dmg_take: f32,
    pub dmg_save: f32,
    pub dmg: f32,
    pub dmgtime: f32,
    pub noise: string_t,
    pub noise1: string_t,
    pub noise2: string_t,
    pub noise3: string_t,
    pub speed: f32,
    pub air_finished: f32,
    pub pain_finished: f32,
    pub radsuit_finished: f32,
    pub pContainingEntity: *mut edict_s,
    pub playerclass: c_int,
    pub maxspeed: f32,
    pub fov: f32,
    pub weaponanim: c_int,
    pub pushmsec: c_int,
    pub bInDuck: c_int,
    pub flTimeStepSound: c_int,
    pub flSwimTime: c_int,
    pub flDuckTime: c_int,
    pub iStepLeft: c_int,
    pub flFallVelocity: f32,
    pub gamestate: c_int,
    pub oldbuttons: c_int,
    pub groupinfo: c_int,
    pub iuser1: c_int,
    pub iuser2: c_int,
    pub iuser3: c_int,
    pub iuser4: c_int,
    pub fuser1: f32,
    pub fuser2: f32,
    pub fuser3: f32,
    pub fuser4: f32,
    pub vuser1: vec3_t,
    pub vuser2: vec3_t,
    pub vuser3: vec3_t,
    pub vuser4: vec3_t,
    pub euser1: *mut edict_s,
    pub euser2: *mut edict_s,
    pub euser3: *mut edict_s,
    pub euser4: *mut edict_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct edict_s {
    pub free: qboolean,
    pub serialnumber: c_int,
    pub area: link_s,
    pub headnode: c_int,
    pub num_leafs: c_int,
    pub leafnums: edits_s_leafnums,
    pub freetime: f32,
    pub pvPrivateData: *mut c_void,
    pub v: entvars_s,
}

pub const MAX_ENT_LEAFS_32: usize = 24; // originally was 16
pub const MAX_ENT_LEAFS_16: usize = 48;

#[derive(Copy, Clone)]
#[repr(C)]
pub union edits_s_leafnums {
    pub leafnums32: [c_int; MAX_ENT_LEAFS_32],
    pub leafnums16: [c_short; MAX_ENT_LEAFS_16],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct efrag_s {
    pub leaf: *mut mleaf_s,
    pub leafnext: *mut efrag_s,
    pub entity: *mut cl_entity_s,
    pub entnext: *mut efrag_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mouth_t {
    pub mouthopen: byte,
    pub sndcount: byte,
    pub sndavg: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct latchedvars_t {
    pub prevanimtime: f32,
    pub sequencetime: f32,
    pub prevseqblending: [byte; 2],
    pub prevorigin: vec3_t,
    pub prevangles: vec3_t,
    pub prevsequence: c_int,
    pub prevframe: f32,
    pub prevcontroller: [byte; 4],
    pub prevblending: [byte; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct position_history_t {
    pub animtime: f32,
    pub origin: vec3_t,
    pub angles: vec3_t,
}

bitflags! {
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct entity_state_s {
    pub entityType: c_int,
    pub number: c_int,
    pub msg_time: f32,
    pub messagenum: c_int,
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub modelindex: c_int,
    pub sequence: c_int,
    pub frame: f32,
    pub colormap: c_int,
    pub skin: c_short,
    pub solid: c_short,
    pub effects: Effects,
    pub scale: f32,
    pub eflags: byte,
    pub rendermode: RenderMode,
    pub renderamt: c_int,
    pub rendercolor: RGB,
    pub renderfx: c_int,
    pub movetype: MoveType,
    pub animtime: f32,
    pub framerate: f32,
    pub body: c_int,
    pub controller: [byte; 4],
    pub blending: [byte; 4],
    pub velocity: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub aiment: c_int,
    pub owner: c_int,
    pub friction: f32,
    pub gravity: f32,
    pub team: c_int,
    pub playerclass: c_int,
    pub health: c_int,
    pub spectator: qboolean,
    pub weaponmodel: c_int,
    pub gaitsequence: c_int,
    pub basevelocity: vec3_t,
    pub usehull: c_int,
    pub oldbuttons: c_int,
    pub onground: c_int,
    pub iStepLeft: c_int,
    pub flFallVelocity: f32,
    pub fov: f32,
    pub weaponanim: c_int,
    pub startpos: vec3_t,
    pub endpos: vec3_t,
    pub impacttime: f32,
    pub starttime: f32,
    pub iuser1: c_int,
    pub iuser2: c_int,
    pub iuser3: c_int,
    pub iuser4: c_int,
    pub fuser1: f32,
    pub fuser2: f32,
    pub fuser3: f32,
    pub fuser4: f32,
    pub vuser1: vec3_t,
    pub vuser2: vec3_t,
    pub vuser3: vec3_t,
    pub vuser4: vec3_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct clientdata_s {
    pub origin: vec3_t,
    pub velocity: vec3_t,
    pub viewmodel: c_int,
    pub punchangle: vec3_t,
    pub flags: EdictFlags,
    pub waterlevel: c_int,
    pub watertype: c_int,
    pub view_ofs: vec3_t,
    pub health: f32,
    pub bInDuck: c_int,
    pub weapons: c_int,
    pub flTimeStepSound: c_int,
    pub flDuckTime: c_int,
    pub flSwimTime: c_int,
    pub waterjumptime: c_int,
    pub maxspeed: f32,
    pub fov: f32,
    pub weaponanim: c_int,
    pub m_iId: c_int,
    pub ammo_shells: c_int,
    pub ammo_nails: c_int,
    pub ammo_cells: c_int,
    pub ammo_rockets: c_int,
    pub m_flNextAttack: f32,
    pub tfstate: c_int,
    pub pushmsec: c_int,
    pub deadflag: c_int,
    pub physinfo: CStrArray<MAX_PHYSINFO_STRING>,
    pub iuser1: c_int,
    pub iuser2: c_int,
    pub iuser3: c_int,
    pub iuser4: c_int,
    pub fuser1: f32,
    pub fuser2: f32,
    pub fuser3: f32,
    pub fuser4: f32,
    pub vuser1: vec3_t,
    pub vuser2: vec3_t,
    pub vuser3: vec3_t,
    pub vuser4: vec3_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct weapon_data_s {
    pub m_iId: c_int,
    pub m_iClip: c_int,
    pub m_flNextPrimaryAttack: f32,
    pub m_flNextSecondaryAttack: f32,
    pub m_flTimeWeaponIdle: f32,
    pub m_fInReload: c_int,
    pub m_fInSpecialReload: c_int,
    pub m_flNextReload: f32,
    pub m_flPumpTime: f32,
    pub m_fReloadTime: f32,
    pub m_fAimedDamage: f32,
    pub m_fNextAimBonus: f32,
    pub m_fInZoom: c_int,
    pub m_iWeaponState: c_int,
    pub iuser1: c_int,
    pub iuser2: c_int,
    pub iuser3: c_int,
    pub iuser4: c_int,
    pub fuser1: f32,
    pub fuser2: f32,
    pub fuser3: f32,
    pub fuser4: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cl_entity_s {
    pub index: c_int,
    pub player: qboolean,
    pub baseline: entity_state_s,
    pub prevstate: entity_state_s,
    pub curstate: entity_state_s,
    pub current_position: c_int,
    pub ph: [position_history_t; HISTORY_MAX],
    pub mouth: mouth_t,
    pub latched: latchedvars_t,
    pub lastmove: f32,
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub attachment: [vec3_t; 4],
    pub trivial_accept: c_int,
    pub model: *mut model_s,
    pub efrag: *mut efrag_s,
    pub topnode: *mut mnode_s,
    pub syncbase: f32,
    pub visframe: c_int,
    pub cvFloorColor: RGBA,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum modtype_t {
    Bad = -1,
    Brush = 0,
    Sprite = 1,
    Alias = 2,
    Studio = 3,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mplane_s {
    pub normal: vec3_t,
    pub dist: f32,
    pub type_: byte,
    pub signbits: byte,
    pub pad: [byte; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mvertex_t {
    pub position: vec3_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mclipnode_t {
    pub planenum: c_int,
    pub children: [c_short; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct medge_t {
    pub v: [c_ushort; 2],
    pub cachededgeoffset: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct texture_s {
    pub name: [c_char; 16],
    pub width: c_uint,
    pub height: c_uint,
    pub gl_texturenum: c_int,
    pub texturechain: *mut msurface_s,
    pub anim_total: c_int,
    pub anim_min: c_int,
    pub anim_max: c_int,
    pub anim_next: *mut texture_s,
    pub alternate_anims: *mut texture_s,
    pub fb_texturenum: c_ushort,
    pub dt_texturenum: c_ushort,
    pub unused: [c_uint; 3],
}
pub type texture_t = texture_s;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mfaceinfo_t {
    pub landname: [c_char; 16],
    pub texture_step: c_ushort,
    pub max_extent: c_ushort,
    pub groupid: c_short,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub reserved: [isize; 32],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mfacebevel_t {
    pub edges: *mut mplane_s,
    pub numedges: c_int,
    pub origin: vec3_t,
    pub radius: f32,
    pub contents: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mtexinfo_t {
    pub vecs: [vec4_t; 2],
    pub faceinfo: *mut mfaceinfo_t,
    pub texture: *mut texture_t,
    pub flags: c_int,
}

#[repr(C)]
pub struct glpoly2_s {
    pub next: *mut glpoly2_s,
    pub chain: *mut glpoly2_s,
    numverts: c_int,
    pub flags: c_int,
    verts: DynArray<[f32; VERTEXSIZE]>,
}

impl glpoly2_s {
    pub fn verts(&self) -> &[[f32; VERTEXSIZE]] {
        unsafe { self.verts.as_slice(self.numverts as usize) }
    }

    pub fn verts_mut(&mut self) -> &mut [[f32; VERTEXSIZE]] {
        unsafe { self.verts.as_mut_slice(self.numverts as usize) }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mnode_s {
    pub contents: c_int,
    pub visframe: c_int,
    pub minmaxs: [f32; 6],
    pub parent: *mut mnode_s,
    pub plane: *mut mplane_s,
    pub children: [*mut mnode_s; 2],
    pub firstsurface: c_ushort,
    pub numsurfaces: c_ushort,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct decal_s {
    pub pnext: *mut decal_s,
    pub psurface: *mut msurface_s,
    pub dx: f32,
    pub dy: f32,
    pub scale: f32,
    pub texture: c_short,
    pub flags: c_short,
    pub entityIndex: c_short,
    pub position: vec3_t,
    pub polys: *mut glpoly2_s,
    pub reserved: [isize; 4],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mleaf_s {
    pub contents: c_int,
    pub visframe: c_int,
    pub minmaxs: [f32; 6],
    pub parent: *mut mnode_s,
    pub compressed_vis: *mut byte,
    pub efrags: *mut efrag_s,
    pub firstmarksurface: *mut *mut msurface_s,
    pub nummarksurfaces: c_int,
    pub cluster: c_int,
    pub ambient_sound_level: [byte; 4],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mextrasurf_s {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub origin: vec3_t,
    pub surf: *mut msurface_s,
    pub dlight_s: c_int,
    pub dlight_t: c_int,
    pub lightmapmins: [c_short; 2],
    pub lightextents: [c_short; 2],
    pub lmvecs: [vec4_t; 2],
    pub deluxemap: *mut RGB,
    pub shadowmap: *mut byte,
    pub lightmapchain: *mut msurface_s,
    pub detailchain: *mut mextrasurf_s,
    pub bevel: *mut mfacebevel_t,
    pub lumachain: *mut mextrasurf_s,
    pub parent: *mut cl_entity_s,
    pub mirrortexturenum: c_int,
    pub mirrormatrix: [[f32; 4]; 4],
    pub grass: *mut grasshdr_s,
    pub grasscount: c_ushort,
    pub numverts: c_ushort,
    pub firstvertex: c_int,
    pub reserved: [isize; 32],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct msurface_s {
    pub visframe: c_int,
    pub plane: *mut mplane_s,
    pub flags: c_int,
    pub firstedge: c_int,
    pub numedges: c_int,
    pub texturemins: [c_short; 2],
    pub extents: [c_short; 2],
    pub light_s: c_int,
    pub light_t: c_int,
    pub polys: *mut glpoly2_s,
    pub texturechain: *mut msurface_s,
    pub texinfo: *mut mtexinfo_t,
    pub dlightframe: c_int,
    pub dlightbits: c_int,
    pub lightmaptexturenum: c_int,
    pub styles: [byte; MAXLIGHTMAPS],
    pub cached_light: [c_int; MAXLIGHTMAPS],
    pub info: *mut mextrasurf_s,
    pub samples: *mut RGB,
    pub pdecals: *mut decal_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mclipnode32_s {
    pub planenum: c_int,
    pub children: [c_int; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mclipnode16_s {
    pub planenum: c_int,
    pub children: [c_short; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union hull_s_clipnodes {
    pub clipnodes16: *mut mclipnode16_s,
    pub clipnodes32: *mut mclipnode32_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct hull_s {
    pub clipnodes: hull_s_clipnodes,
    pub planes: *mut mplane_s,
    pub firstclipnode: c_int,
    pub lastclipnode: c_int,
    pub clip_mins: vec3_t,
    pub clip_maxs: vec3_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cache_user_s {
    pub data: *mut c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct medge32_s {
    pub v: [c_uint; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct medge16_s {
    pub v: [c_ushort; 2],
    pub cachededgeoffset: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union medges {
    pub edges16: *mut medge16_s,
    pub edges32: *mut medge32_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union mclipnodes {
    pub clipnodes16: *mut mclipnode16_s,
    pub clipnodes32: *mut mclipnode32_s,
}

bitflags! {
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

        /// It's a world model.
        const WORLD             = 1 << 29;
        /// A client sprite.
        const CLIENT            = 1 << 30;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct model_s {
    pub name: [c_char; 64],
    pub needload: qboolean,
    pub type_: modtype_t,
    pub numframes: c_int,
    pub mempool: poolhandle_t,
    pub flags: ModelFlags,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub radius: f32,
    pub firstmodelsurface: c_int,
    pub nummodelsurfaces: c_int,
    pub numsubmodels: c_int,
    pub submodels: *mut dmodel_t,
    pub numplanes: c_int,
    pub planes: *mut mplane_s,
    pub numleafs: c_int,
    pub leafs: *mut mleaf_s,
    pub numvertexes: c_int,
    pub vertexes: *mut mvertex_t,
    pub numedges: c_int,
    pub edges: medges,
    pub numnodes: c_int,
    pub nodes: *mut mnode_s,
    pub numtexinfo: c_int,
    pub texinfo: *mut mtexinfo_t,
    pub numsurfaces: c_int,
    pub surfaces: *mut msurface_s,
    pub numsurfedges: c_int,
    pub surfedges: *mut c_int,
    pub numclipnodes: c_int,
    pub clipnodes: mclipnodes,
    pub nummarksurfaces: c_int,
    pub marksurfaces: *mut *mut msurface_s,
    pub hulls: [hull_s; MAX_MAP_HULLS],
    pub numtextures: c_int,
    pub textures: *mut *mut texture_t,
    pub visdata: *mut byte,
    pub lightdata: *mut RGB,
    pub entities: *mut c_char,
    pub cache: cache_user_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct auxvert_s {
    pub fv: [f32; 3],
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum resourcetype_t {
    Sound = 0,
    Skin = 1,
    Model = 2,
    Decal = 3,
    Generic = 4,
    Eventscript = 5,
    World = 6,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _resourceinfo_t {
    pub size: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct resourceinfo_s {
    pub info: [_resourceinfo_t; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct resource_s {
    pub szFileName: [c_char; 64],
    pub type_: resourcetype_t,
    pub nIndex: c_int,
    pub nDownloadSize: c_int,
    pub ucFlags: c_uchar,
    pub rgucMD5_hash: [c_uchar; 16],
    pub playernum: c_uchar,
    pub rguc_reserved: [c_uchar; 32],
    pub ucExtraFlags: c_ushort,
    pub pNext: *mut resource_s,
    pub pPrev: *mut resource_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct customization_s {
    pub bInUse: qboolean,
    pub resource: resource_s,
    pub bTranslated: qboolean,
    pub nUserData1: c_int,
    pub nUserData2: c_int,
    pub pInfo: *mut c_void,
    pub pBuffer: *mut c_void,
    pub pNext: *mut customization_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct player_info_s {
    pub userid: c_int,
    pub userinfo: [c_char; 256],
    pub name: [c_char; 32],
    pub spectator: c_int,
    pub ping: c_int,
    pub packet_loss: c_int,
    pub model: [c_char; 64],
    pub topcolor: c_int,
    pub bottomcolor: c_int,
    pub renderframe: c_int,
    pub gaitsequence: c_int,
    pub gaitframe: f32,
    pub gaityaw: f32,
    pub prevgaitorigin: vec3_t,
    pub customdata: customization_s,
    pub hashedcdkey: [c_char; 16],
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum spriteframetype_t {
    Single = 0,
    Group = 1,
    Angled = 2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mspriteframe_s {
    pub width: c_int,
    pub height: c_int,
    pub up: f32,
    pub down: f32,
    pub left: f32,
    pub right: f32,
    pub gl_texturenum: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mspritegroup_t {
    pub numframes: c_int,
    pub intervals: *mut f32,
    pub frames: [*mut mspriteframe_s; 1],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mspriteframedesc_t {
    pub type_: spriteframetype_t,
    pub frameptr: *mut mspriteframe_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct msprite_t {
    pub type_: c_short,
    pub texFormat: c_short,
    pub maxwidth: c_int,
    pub maxheight: c_int,
    pub numframes: c_int,
    pub radius: c_int,
    pub facecull: c_int,
    pub synctype: c_int,
    pub frames: [mspriteframedesc_t; 1],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct trivertex_t {
    pub v: [byte; 3],
    pub lightnormalindex: byte,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct maliasframedesc_t {
    pub firstpose: c_int,
    pub numposes: c_int,
    pub bboxmin: trivertex_t,
    pub bboxmax: trivertex_t,
    pub interval: f32,
    pub name: [c_char; 16],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct aliashdr_t {
    pub ident: c_int,
    pub version: c_int,
    pub scale: vec3_t,
    pub scale_origin: vec3_t,
    pub boundingradius: f32,
    pub eyeposition: vec3_t,
    pub numskins: c_int,
    pub skinwidth: c_int,
    pub skinheight: c_int,
    pub numverts: c_int,
    pub numtris: c_int,
    pub numframes: c_int,
    pub synctype: c_int,
    pub flags: c_int,
    pub size: f32,
    pub pposeverts: *mut *const trivertex_t,
    pub reserved: [isize; 7],
    pub numposes: c_int,
    pub poseverts: c_int,
    pub posedata: *mut trivertex_t,
    pub commands: *mut c_int,
    pub gl_texturenum: [[c_ushort; 4]; MAX_SKINS],
    pub fb_texturenum: [[c_ushort; 4]; MAX_SKINS],
    pub gl_reserved0: [[c_ushort; 4]; MAX_SKINS],
    pub gl_reserved1: [[c_ushort; 4]; MAX_SKINS],
    pub gl_reserved2: [[c_ushort; 4]; MAX_SKINS],
    pub frames: [maliasframedesc_t; 1],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct con_nprint_s {
    pub index: c_int,
    pub time_to_live: f32,
    pub color: [f32; 3],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dlight_s {
    pub origin: vec3_t,
    pub radius: f32,
    pub color: RGB,
    pub die: f32,
    pub decay: f32,
    pub minlight: f32,
    pub key: c_int,
    pub dark: qboolean,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum netadrtype_t {
    Loopback = 1,
    Broadcast = 2,
    Ip = 3,
    Ipx = 4,
    BroadcastIpx = 5,
    Ip6 = 6,
    MulticastIp6 = 7,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct netadr_s {
    pub netadr_ip_s: netadr_ip_u,
    pub port: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union netadr_ip_u {
    pub ip6: netadr_ip6_s,
    pub ip: netadr_ip_s,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct netadr_ip6_s {
    pub type6: u16,
    pub ip6: [u8; 16],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct netadr_ip_s {
    pub type_: u32,
    pub ip4: [u8; 4],
    pub ipx: [u8; 10],
}

pub type net_api_response_func_t = Option<unsafe extern "C" fn(response: *mut net_response_s)>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_adrlist_s {
    pub next: *mut net_adrlist_s,
    pub remote_address: netadr_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_response_s {
    pub error: c_int,
    pub context: c_int,
    pub type_: c_int,
    pub remote_address: netadr_s,
    pub ping: f64,
    pub response: *mut c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_status_s {
    pub connected: c_int,
    pub local_address: netadr_s,
    pub remote_address: netadr_s,
    pub packet_loss: c_int,
    pub latency: f64,
    pub connection_time: f64,
    pub rate: f64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_api_s {
    pub InitNetworking: Option<unsafe extern "C" fn()>,
    pub Status: Option<unsafe extern "C" fn(status: *mut net_status_s)>,
    pub SendRequest: Option<
        unsafe extern "C" fn(
            context: c_int,
            request: c_int,
            flags: c_int,
            timeout: f64,
            remote_address: *mut netadr_s,
            response: net_api_response_func_t,
        ),
    >,
    pub CancelRequest: Option<unsafe extern "C" fn(context: c_int)>,
    pub CancelAllRequests: Option<unsafe extern "C" fn()>,
    pub AdrToString: Option<unsafe extern "C" fn(a: *const netadr_s) -> *const c_char>,
    pub CompareAdr: Option<unsafe extern "C" fn(a: *const netadr_s, b: *const netadr_s) -> c_int>,
    pub StringToAdr: Option<unsafe extern "C" fn(s: *const c_char, a: *mut netadr_s) -> c_int>,
    pub ValueForKey:
        Option<unsafe extern "C" fn(s: *const c_char, key: *const c_char) -> *const c_char>,
    pub RemoveKey: Option<unsafe extern "C" fn(s: *mut c_char, key: *const c_char)>,
    pub SetValueForKey: Option<
        unsafe extern "C" fn(
            s: *mut c_char,
            key: *const c_char,
            value: *const c_char,
            maxsize: c_int,
        ),
    >,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct charinfo {
    pub startoffset: c_short,
    pub charwidth: c_short,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct qfont_s {
    pub width: c_int,
    pub height: c_int,
    pub rowcount: c_int,
    pub rowheight: c_int,
    pub fontinfo: [charinfo; NUM_GLYPHS],
    pub data: [byte; 4],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ref_overview_s {
    pub origin: vec3_t,
    pub rotated: qboolean,
    pub xLeft: f32,
    pub xRight: f32,
    pub yTop: f32,
    pub yBottom: f32,
    pub zFar: f32,
    pub zNear: f32,
    pub flZoom: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ref_viewpass_s {
    pub viewport: [c_int; 4],
    pub vieworigin: vec3_t,
    pub viewangles: vec3_t,
    pub viewentity: c_int,
    pub fov_x: f32,
    pub fov_y: f32,
    pub flags: c_int,
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct usercmd_s {
    pub lerp_msec: c_short,
    pub msec: byte,
    pub viewangles: vec3_t,
    pub forwardmove: f32,
    pub sidemove: f32,
    pub upmove: f32,
    pub lightlevel: byte,
    pub buttons: c_ushort,
    pub impulse: byte,
    pub weaponselect: byte,
    pub impact_index: c_int,
    pub impact_position: vec3_t,
}

impl usercmd_s {
    pub fn move_vector(&self) -> vec3_t {
        vec3_t::new(self.forwardmove, self.sidemove, self.upmove)
    }

    pub fn move_vector_set(&mut self, vec: vec3_t) {
        self.forwardmove = vec[0];
        self.sidemove = vec[1];
        self.upmove = vec[2];
    }

    pub fn is_button(&self, button: u32) -> bool {
        self.buttons as u32 & button != 0
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lightstyle_t {
    pub pattern: [c_char; 256],
    pub map: [f32; 256],
    pub length: c_int,
    pub value: f32,
    pub interp: qboolean,
    pub time: f32,
}

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

pub type texFlags_t = TextureFlags;
bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    #[repr(transparent)]
    pub struct TextureFlags: c_int {
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum gl_context_type_t {
    GL = 0,
    GLES_1_X = 1,
    GLES_2_X = 2,
    GL_CORE = 3,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum gles_wrapper_t {
    None = 0,
    NanoGL = 1,
    WES = 2,
    GL4ES = 3,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct modelstate_s {
    pub sequence: c_short,
    pub frame: c_short,
    pub blending: [byte; 2],
    pub controller: [byte; 4],
    pub poseparam: [byte; 16],
    pub body: byte,
    pub skin: byte,
    pub scale: c_short,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct decallist_s {
    pub position: vec3_t,
    pub name: [c_char; 64],
    pub entityIndex: c_short,
    pub depth: byte,
    pub flags: byte,
    pub scale: f32,
    pub impactPlaneNormal: vec3_t,
    pub studio_state: modelstate_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct physent_s {
    pub name: [c_char; 32],
    pub player: c_int,
    pub origin: vec3_t,
    pub model: *mut model_s,
    pub studiomodel: *mut model_s,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub info: c_int,
    pub angles: vec3_t,
    pub solid: c_int,
    pub skin: c_int,
    pub rendermode: RenderMode,
    pub frame: f32,
    pub sequence: c_int,
    pub controller: [byte; 4],
    pub blending: [byte; 2],
    pub movetype: c_int,
    pub takedamage: c_int,
    pub blooddecal: c_int,
    pub team: c_int,
    pub classnumber: c_int,
    pub iuser1: c_int,
    pub iuser2: c_int,
    pub iuser3: c_int,
    pub iuser4: c_int,
    pub fuser1: f32,
    pub fuser2: f32,
    pub fuser3: f32,
    pub fuser4: f32,
    pub vuser1: vec3_t,
    pub vuser2: vec3_t,
    pub vuser3: vec3_t,
    pub vuser4: vec3_t,
}

bitflags! {
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

#[derive(Clone)]
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct movevars_s {
    pub gravity: f32,
    pub stopspeed: f32,
    pub maxspeed: f32,
    pub spectatormaxspeed: f32,
    pub accelerate: f32,
    pub airaccelerate: f32,
    pub wateraccelerate: f32,
    pub friction: f32,
    pub edgefriction: f32,
    pub waterfriction: f32,
    pub entgravity: f32,
    pub bounce: f32,
    pub stepsize: f32,
    pub maxvelocity: f32,
    pub zmax: f32,
    pub waveHeight: f32,
    pub footsteps: qboolean,
    pub skyName: [c_char; 32],
    pub rollangle: f32,
    pub rollspeed: f32,
    pub skycolor_r: f32,
    pub skycolor_g: f32,
    pub skycolor_b: f32,
    pub skyvec_x: f32,
    pub skyvec_y: f32,
    pub skyvec_z: f32,
    pub features: c_int,
    pub fog_settings: c_int,
    pub wateralpha: f32,
    pub skydir_x: f32,
    pub skydir_y: f32,
    pub skydir_z: f32,
    pub skyangle: f32,
}

impl movevars_s {
    pub fn is_footsteps(&self) -> bool {
        self.footsteps != qboolean::FALSE
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct wrect_s {
    pub left: c_int,
    pub right: c_int,
    pub top: c_int,
    pub bottom: c_int,
}

impl wrect_s {
    pub const fn width(&self) -> c_int {
        self.right - self.left
    }

    pub const fn height(&self) -> c_int {
        self.bottom - self.top
    }

    pub const fn size(&self) -> (c_int, c_int) {
        (self.width(), self.height())
    }
}

// #[derive(Copy, Clone)]
// #[repr(C)]
// pub struct CDStatus {
//     pub fPlaying: c_int,
//     pub fWasPlaying: c_int,
//     pub fInitialized: c_int,
//     pub fEnabled: c_int,
//     pub fPlayLooping: c_int,
//     pub cdvolume: f32,
//     pub fCDRom: c_int,
//     pub fPlayTrack: c_int,
// }

#[derive(Copy, Clone)]
#[repr(C)]
pub struct grasshdr_s {
    pub _address: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mstudiotex_s {
    pub _address: u8,
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct beam_s {
    pub next: *mut beam_s,
    pub type_: c_int,
    pub flags: c_int,
    pub source: vec3_t,
    pub target: vec3_t,
    pub delta: vec3_t,
    pub t: f32,
    pub freq: f32,
    pub die: f32,
    pub width: f32,
    pub amplitude: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub brightness: f32,
    pub speed: f32,
    pub frameRate: f32,
    pub frame: f32,
    pub segments: c_int,
    pub startEntity: c_int,
    pub endEntity: c_int,
    pub modelIndex: c_int,
    pub frameCount: c_int,
    pub pFollowModel: *mut model_s,
    pub particles: *mut particle_s,
}
pub type BEAM = beam_s;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mstudioevent_s {
    pub frame: i32,
    pub event: i32,
    pub unused: i32,
    pub options: [c_char; 64],
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum ptype_t {
    Static = 0,
    Grav = 1,
    SlowGrav = 2,
    Fire = 3,
    Explode = 4,
    Explode2 = 5,
    Blob = 6,
    Blob2 = 7,
    VoxSlowGrav = 8,
    VoxGrav = 9,
    ClientCustom = 10,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct particle_s {
    pub org: vec3_t,
    pub color: c_short,
    pub packedColor: c_short,
    pub next: *mut particle_s,
    pub vel: vec3_t,
    pub ramp: f32,
    pub die: f32,
    pub type_: ptype_t,
    pub deathfunc: Option<unsafe extern "C" fn(particle: *mut particle_s)>,
    pub callback: Option<unsafe extern "C" fn(particle: *mut particle_s, frametime: f32)>,
    pub context: c_uchar,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct screenfade_s {
    pub fadeSpeed: f32,
    pub fadeEnd: f32,
    pub fadeTotalEnd: f32,
    pub fadeReset: f32,
    pub fader: byte,
    pub fadeg: byte,
    pub fadeb: byte,
    pub fadealpha: byte,
    pub fadeFlags: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct r_studio_interface_s {
    pub version: c_int,
    pub StudioDrawModel: Option<unsafe extern "C" fn(flags: c_int) -> c_int>,
    pub StudioDrawPlayer:
        Option<unsafe extern "C" fn(flags: c_int, pplayer: *mut entity_state_s) -> c_int>,
}
pub type r_studio_interface_t = r_studio_interface_s;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct alight_s {
    pub ambientlight: c_int,
    pub shadelight: c_int,
    pub color: vec3_t,
    pub plightvec: *mut f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct engine_studio_api_s {
    pub Mem_Calloc: Option<unsafe extern "C" fn(number: c_int, size: usize) -> *mut c_void>,
    pub Cache_Check: Option<unsafe extern "C" fn(c: *mut cache_user_s) -> *mut c_void>,
    pub LoadCacheFile: Option<unsafe extern "C" fn(path: *const c_char, cu: *mut cache_user_s)>,
    pub Mod_ForName:
        Option<unsafe extern "C" fn(name: *const c_char, crash_if_missing: c_int) -> *mut model_s>,
    pub Mod_Extradata: Option<unsafe extern "C" fn(mod_: *mut model_s) -> *mut c_void>,
    pub GetModelByIndex: Option<unsafe extern "C" fn(index: c_int) -> *mut model_s>,
    pub GetCurrentEntity: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    pub PlayerInfo: Option<unsafe extern "C" fn(index: c_int) -> *mut player_info_s>,
    pub GetPlayerState: Option<unsafe extern "C" fn(index: c_int) -> *mut entity_state_s>,
    pub GetViewEntity: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    pub GetTimes:
        Option<unsafe extern "C" fn(framecount: *mut c_int, current: *mut f64, old: *mut f64)>,
    pub GetCvar: Option<unsafe extern "C" fn(name: *const c_char) -> *mut cvar_s>,
    pub GetViewInfo: Option<
        unsafe extern "C" fn(origin: *mut f32, upv: *mut f32, rightv: *mut f32, vpnv: *mut f32),
    >,
    pub GetChromeSprite: Option<unsafe extern "C" fn() -> *mut model_s>,
    pub GetModelCounters: Option<unsafe extern "C" fn(s: *mut *mut c_int, a: *mut *mut c_int)>,
    pub GetAliasScale: Option<unsafe extern "C" fn(x: *mut f32, y: *mut f32)>,
    pub StudioGetBoneTransform: Option<unsafe extern "C" fn() -> *mut *mut *mut *mut f32>,
    pub StudioGetLightTransform: Option<unsafe extern "C" fn() -> *mut *mut *mut *mut f32>,
    pub StudioGetAliasTransform: Option<unsafe extern "C" fn() -> *mut *mut *mut f32>,
    pub StudioGetRotationMatrix: Option<unsafe extern "C" fn() -> *mut *mut *mut f32>,
    pub StudioSetupModel: Option<
        unsafe extern "C" fn(
            bodypart: c_int,
            ppbodypart: *mut *mut c_void,
            ppsubmodel: *mut *mut c_void,
        ),
    >,
    pub StudioCheckBBox: Option<unsafe extern "C" fn() -> c_int>,
    pub StudioDynamicLight:
        Option<unsafe extern "C" fn(ent: *mut cl_entity_s, plight: *mut alight_s)>,
    pub StudioEntityLight: Option<unsafe extern "C" fn(plight: *mut alight_s)>,
    pub StudioSetupLighting: Option<unsafe extern "C" fn(plighting: *mut alight_s)>,
    pub StudioDrawPoints: Option<unsafe extern "C" fn()>,
    pub StudioDrawHulls: Option<unsafe extern "C" fn()>,
    pub StudioDrawAbsBBox: Option<unsafe extern "C" fn()>,
    pub StudioDrawBones: Option<unsafe extern "C" fn()>,
    pub StudioSetupSkin: Option<unsafe extern "C" fn(ptexturehdr: *mut c_void, index: c_int)>,
    pub StudioSetRemapColors: Option<unsafe extern "C" fn(top: c_int, bottom: c_int)>,
    pub SetupPlayerModel: Option<unsafe extern "C" fn(index: c_int) -> *mut model_s>,
    pub StudioClientEvents: Option<unsafe extern "C" fn()>,
    pub GetForceFaceFlags: Option<unsafe extern "C" fn() -> c_int>,
    pub SetForceFaceFlags: Option<unsafe extern "C" fn(flags: c_int)>,
    pub StudioSetHeader: Option<unsafe extern "C" fn(header: *mut c_void)>,
    pub SetRenderModel: Option<unsafe extern "C" fn(model: *mut model_s)>,
    pub SetupRenderer: Option<unsafe extern "C" fn(rendermode: c_int)>,
    pub RestoreRenderer: Option<unsafe extern "C" fn()>,
    pub SetChromeOrigin: Option<unsafe extern "C" fn()>,
    pub IsHardware: Option<unsafe extern "C" fn() -> c_int>,
    pub GL_StudioDrawShadow: Option<unsafe extern "C" fn()>,
    pub GL_SetRenderMode: Option<unsafe extern "C" fn(mode: c_int)>,
    pub StudioSetRenderamt: Option<unsafe extern "C" fn(iRenderamt: c_int)>,
    pub StudioSetCullState: Option<unsafe extern "C" fn(iCull: c_int)>,
    pub StudioRenderShadow: Option<
        unsafe extern "C" fn(
            iSprite: c_int,
            p1: *mut f32,
            p2: *mut f32,
            p3: *mut f32,
            p4: *mut f32,
        ),
    >,
}
pub type engine_studio_api_t = engine_studio_api_s;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum TRICULLSTYLE {
    Front = 0,
    None = 1,
}
