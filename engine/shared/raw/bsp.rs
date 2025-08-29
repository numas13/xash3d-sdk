use core::ffi::{c_char, c_int, c_short, c_ushort};

use bitflags::bitflags;

use crate::raw::{
    byte,
    consts::{CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_WATER, MAX_MAP_HULLS},
    vec3_t, vec4_t,
};

pub type word = c_ushort;

pub const Q1BSP_VERSION: u32 = 29;
pub const HLBSP_VERSION: u32 = 30;
pub const QBSP2_VERSION: u32 = u32::from_be_bytes(*b"BSP2");

pub const IDEXTRAHEADER: u32 = u32::from_le_bytes(*b"XASH");
pub const EXTRA_VERSION: u32 = 4;

pub const DELUXEMAP_VERSION: u32 = 1;
pub const IDDELUXEMAPHEADER: u32 = u32::from_le_bytes(*b"QLIT");

// worldcraft predefined angles
pub const ANGLE_UP: i32 = -1;
pub const ANGLE_DOWN: i32 = -2;

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Surface: u32 {
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

// MAXLIGHTMAPS
pub const LM_STYLES: usize = 4;
pub const LS_NORMAL: u32 = 0x00;
pub const LS_UNUSED: u32 = 0xfe;
pub const LS_NONE: u32 = 0xff;

pub const MAX_MAP_CLIPNODES_HLBSP: u32 = 32767;

#[cfg(not(feature = "bsp2"))]
mod bsp_version_consts {
    pub const MAX_MAP_MODELS: usize = 1024;
    pub const MAX_MAP_ENTSTRING: usize = 0x100000;
    pub const MAX_MAP_PLANES: usize = 65536;
    pub const MAX_MAP_NODES: usize = 32767;
    pub const MAX_MAP_CLIPNODES: usize = 32767;
    pub const MAX_MAP_LEAFS: usize = 32767;
    pub const MAX_MAP_VERTS: usize = 65535;
    pub const MAX_MAP_FACES: usize = 65535;
    pub const MAX_MAP_MARKSURFACES: usize = 65535;
}

#[cfg(feature = "bsp2")]
mod bsp_version_consts {
    pub const MAX_MAP_CLIPNODES_BSP2: usize = 524288;
    pub const MAX_MAP_MODELS: usize = 2048;
    pub const MAX_MAP_ENTSTRING: usize = 0x200000;
    pub const MAX_MAP_PLANES: usize = 131072;
    pub const MAX_MAP_NODES: usize = 262144;
    pub const MAX_MAP_CLIPNODES: usize = MAX_MAP_CLIPNODES_BSP2;
    pub const MAX_MAP_LEAFS: usize = 131072;
    pub const MAX_MAP_VERTS: usize = 524288;
    pub const MAX_MAP_FACES: usize = 262144;
    pub const MAX_MAP_MARKSURFACES: usize = 524288;
}

pub use self::bsp_version_consts::*;

pub const MAX_MAP_LEAFS_BYTES: usize = MAX_MAP_LEAFS.div_ceil(8);

pub const MAX_MAP_ENTITIES: u32 = 8192;
pub const MAX_MAP_TEXINFO: u32 = 65535;
pub const MAX_MAP_EDGES: u32 = 1048576;
pub const MAX_MAP_SURFEDGES: u32 = 2097152;
pub const MAX_MAP_TEXTURES: usize = 2048;
pub const MAX_MAP_MIPTEX: u32 = 33554432;
pub const MAX_MAP_LIGHTING: u32 = 33554432;
pub const MAX_MAP_VISIBILITY: u32 = 16777216;
pub const MAX_MAP_FACEINFO: u32 = 8192;

pub const LUMP_ENTITIES: u32 = 0;
pub const LUMP_PLANES: u32 = 1;
pub const LUMP_TEXTURES: u32 = 2;
pub const LUMP_VERTEXES: u32 = 3;
pub const LUMP_VISIBILITY: u32 = 4;
pub const LUMP_NODES: u32 = 5;
pub const LUMP_TEXINFO: u32 = 6;
pub const LUMP_FACES: u32 = 7;
pub const LUMP_LIGHTING: u32 = 8;
pub const LUMP_CLIPNODES: u32 = 9;
pub const LUMP_LEAFS: u32 = 10;
pub const LUMP_MARKSURFACES: u32 = 11;
pub const LUMP_EDGES: u32 = 12;
pub const LUMP_SURFEDGES: u32 = 13;
pub const LUMP_MODELS: u32 = 14;
pub const HEADER_LUMPS: usize = 15;

pub const LUMP_LIGHTVECS: u32 = 0;
pub const LUMP_FACEINFO: u32 = 1;
pub const LUMP_CUBEMAPS: u32 = 2;
pub const LUMP_VERTNORMALS: u32 = 3;
pub const LUMP_LEAF_LIGHTING: u32 = 4;
pub const LUMP_WORLDLIGHTS: u32 = 5;
pub const LUMP_COLLISION: u32 = 6;
pub const LUMP_AINODEGRAPH: u32 = 7;
pub const LUMP_SHADOWMAP: u32 = 8;
pub const LUMP_VERTEX_LIGHT: u32 = 9;
pub const LUMP_UNUSED0: u32 = 10;
pub const LUMP_UNUSED1: u32 = 11;
pub const EXTRA_LUMPS: usize = 12;

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct TextureFlags: u32 {
        /// Sky or slime, no lightmap or 256 subdivision.
        const SPECIAL           = 1 << 0;
        /// Alternative lightmap matrix will be used.
        ///
        /// Luxels per world units instead of luxels per texels.
        const WORLD_LUXELS      = 1 << 1;
        /// Force world luxels to axial positive scales.
        const AXIAL_LUXELS      = 1 << 2;
        /// bsp31 legacy - using 8 texels per luxel instead of 16 texels per luxel.
        const EXTRA_LIGHTMAP    = 1 << 3;
        /// Doom special FX
        const SCROLL            = 1 << 6;
    }
}

pub const fn is_liquid_contents(cnt: i32) -> bool {
    matches!(cnt, CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Ambient {
    Water = 0,
    Sky = 1,
    Slime = 2,
    Lava = 3,
    Count = 4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dlump_t {
    pub fileofs: c_int,
    pub filelen: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dheader_t {
    pub version: c_int,
    pub lumps: [dlump_t; HEADER_LUMPS],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dextrahdr_t {
    pub id: c_int,
    pub version: c_int,
    pub lumps: [dlump_t; EXTRA_LUMPS],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dmodel_t {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub origin: vec3_t,
    pub headnode: [c_int; MAX_MAP_HULLS],
    pub visleafs: c_int,
    pub firstface: c_int,
    pub numfaces: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dmiptexlump_t {
    pub nummiptex: c_int,
    pub dataofs: [c_int; 4],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dvertex_t {
    pub point: vec3_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub type_: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dnode_t {
    pub planenum: c_int,
    pub children: [c_short; 2],
    pub mins: [c_short; 3],
    pub maxs: [c_short; 3],
    pub firstface: word,
    pub numfaces: word,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dnode32_t {
    pub planenum: c_int,
    pub children: [c_int; 2],
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub firstface: c_int,
    pub numfaces: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dleaf_t {
    pub contents: c_int,
    pub visofs: c_int,
    pub mins: [c_short; 3],
    pub maxs: [c_short; 3],
    pub firstmarksurface: word,
    pub nummarksurfaces: word,
    pub ambient_level: [byte; Ambient::Count as usize],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dleaf32_t {
    pub contents: c_int,
    pub visofs: c_int,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub firstmarksurface: c_int,
    pub nummarksurfaces: c_int,
    pub ambient_level: [byte; Ambient::Count as usize],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dclipnode_t {
    pub planenum: c_int,
    pub children: [c_short; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dclipnode32_t {
    pub planenum: c_int,
    pub children: [c_int; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dtexinfo_t {
    pub vecs: [vec4_t; 2],
    pub miptex: c_int,
    pub flags: c_short,
    pub faceinfo: c_short,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dfaceinfo_t {
    pub landname: [c_char; 16],
    pub texture_step: c_ushort,
    pub max_extent: c_ushort,
    pub groupid: c_short,
}

pub type dmarkface_t = word;
pub type dmarkface32_t = c_int;
pub type dsurfedge_t = c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dedge_t {
    pub v: [word; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dedge32_t {
    pub v: [c_int; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dface_t {
    pub planenum: word,
    pub side: c_short,
    pub firstedge: c_int,
    pub numedges: c_short,
    pub texinfo: c_short,
    pub styles: [byte; LM_STYLES],
    pub lightofs: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dface32_t {
    pub planenum: c_int,
    pub side: c_int,
    pub firstedge: c_int,
    pub numedges: c_int,
    pub texinfo: c_int,
    pub styles: [byte; LM_STYLES],
    pub lightofs: c_int,
}
