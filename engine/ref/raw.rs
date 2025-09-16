#![allow(non_camel_case_types)]

use core::{
    ffi::{c_int, CStr},
    mem, ptr,
};

use bitflags::bitflags;
use csz::CStrThin;
use shared::{
    consts::RefParm,
    cvar::CVarFlags,
    ffi::common::uint,
    ffi::render::{convar_s, rgbdata_t},
};

pub use shared::raw::*;

pub use crate::bsp;

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct FContext: c_int {
        const CORE_PROFILE  = 1 << 0;
        const DEBUG_ARB     = 1 << 1;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct ScreenshotType(c_int);

impl ScreenshotType {
    pub const VID_SCREENSHOT: Self = ScreenshotType(0);
    pub const VID_LEVELSHOT: Self = ScreenshotType(1);
    pub const VID_MINISHOT: Self = ScreenshotType(2);
    /// Special case for overview layer.
    pub const VID_MAPSHOT: Self = ScreenshotType(3);
    /// Save screenshot into root dir and no gamma correction.
    pub const VID_SNAPSHOT: Self = ScreenshotType(4);
}

bitflags! {
    /// goes into world.flags
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    #[repr(transparent)]
    pub struct WorldFlags: c_int {
        const SKYSPHERE         = 1 << 0;
        const CUSTOM_SKYBOX     = 1 << 1;
        const WATERALPHA        = 1 << 2;
        const HAS_DELUXEMAP     = 1 << 3;
    }
}

pub const SKYBOX_MAX_SIDES: usize = 6;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum demo_mode {
    INACTIVE = 0,
    XASH3D = 1,
    QUAKE1 = 2,
}

/// rgbdata_s.type_
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
#[non_exhaustive]
#[repr(C)]
pub enum PixelFormat {
    #[default]
    UNKNOWN = 0,
    INDEXED_24,
    INDEXED_32,
    RGBA_32,
    BGRA_32,
    RGB_24,
    BGR_24,
    LUMINANCE,
    DXT1,
    DXT3,
    DXT5,
    ATI2,
    BC4_SIGNED,
    BC4_UNSIGNED,
    BC5_SIGNED,
    BC5_UNSIGNED,
    BC6H_SIGNED,
    BC6H_UNSIGNED,
    BC7_UNORM,
    BC7_SRGB,
    KTX2_RAW,
    TOTALCOUNT,
}

impl PixelFormat {
    pub fn from_raw(raw: uint) -> Option<Self> {
        if raw <= PixelFormat::TOTALCOUNT as uint {
            Some(unsafe { mem::transmute::<uint, Self>(raw) })
        } else {
            None
        }
    }
    pub const fn is_raw(&self) -> bool {
        matches!(
            self,
            Self::RGBA_32 | Self::BGRA_32 | Self::RGB_24 | Self::BGR_24 | Self::LUMINANCE
        )
    }

    pub const fn is_compressed(&self) -> bool {
        matches!(
            self,
            Self::DXT1
                | Self::DXT3
                | Self::DXT5
                | Self::ATI2
                | Self::BC4_SIGNED
                | Self::BC4_UNSIGNED
                | Self::BC5_SIGNED
                | Self::BC5_UNSIGNED
                | Self::BC6H_SIGNED
                | Self::BC6H_UNSIGNED
                | Self::BC7_UNORM
                | Self::BC7_SRGB
                | Self::KTX2_RAW
        )
    }
}

#[deprecated(note = "use texture::ImageFlags instead")]
pub type ImageFlags = crate::texture::ImageFlags;

#[deprecated(note = "use texture::OutputImageFlags instead")]
pub type OutputImageFlags = crate::texture::OutputImageFlags;

#[deprecated(note = "use engine::GraphicApi instead")]
pub type GraphicApi = crate::engine::GraphicApi;

pub const CVAR_SENTINEL: usize = 0xdeadbeefdeadbeef_u64 as usize;

pub trait ConVarExt {
    fn builder(name: &'static CStr) -> ConVarBuilder {
        ConVarBuilder::new(name)
    }

    fn name(&self) -> &CStrThin;

    fn value_c_str(&self) -> &CStrThin;

    fn value(&self) -> f32;
}

impl ConVarExt for convar_s {
    fn name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.name) }
    }

    fn value_c_str(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.string) }
    }

    fn value(&self) -> f32 {
        self.value
    }
}

pub struct ConVarBuilder {
    var: convar_s,
}

impl ConVarBuilder {
    pub const fn new(name: &'static CStr) -> Self {
        ConVarBuilder {
            var: convar_s {
                name: name.as_ptr().cast_mut(),
                string: c"".as_ptr().cast_mut(),
                flags: CVarFlags::NONE.bits(),
                value: 0.0,
                next: CVAR_SENTINEL as *mut convar_s,
                desc: ptr::null_mut(),
                def_string: ptr::null_mut(),
            },
        }
    }

    pub const fn value(mut self, value: &'static CStr) -> Self {
        self.var.string = value.as_ptr().cast_mut();
        self
    }

    pub const fn flags(mut self, flags: CVarFlags) -> Self {
        self.var.flags = flags.bits();
        self
    }

    pub const fn description(mut self, desc: &'static CStr) -> Self {
        self.var.desc = desc.as_ptr().cast_mut();
        self
    }

    pub const fn build(self) -> convar_s {
        self.var
    }
}

pub const PARM_DEV_OVERVIEW: RefParm = RefParm::new(-1);
pub const PARM_THIRDPERSON: RefParm = RefParm::new(-2);
pub const PARM_QUAKE_COMPATIBLE: RefParm = RefParm::new(-3);
pub const PARM_GET_CLIENT_PTR: RefParm = RefParm::new(-4);
pub const PARM_GET_HOST_PTR: RefParm = RefParm::new(-5);
pub const PARM_CONNSTATE: RefParm = RefParm::new(-6);
pub const PARM_PLAYING_DEMO: RefParm = RefParm::new(-7);
pub const PARM_WATER_LEVEL: RefParm = RefParm::new(-8);
pub const PARM_GET_WORLD_PTR: RefParm = RefParm::new(-9);
pub const PARM_LOCAL_HEALTH: RefParm = RefParm::new(-10);
pub const PARM_LOCAL_GAME: RefParm = RefParm::new(-11);
pub const PARM_NUMENTITIES: RefParm = RefParm::new(-12);
pub const PARM_GET_MOVEVARS_PTR: RefParm = RefParm::new(-13);
pub const PARM_GET_PALETTE_PTR: RefParm = RefParm::new(-14);
pub const PARM_GET_VIEWENT_PTR: RefParm = RefParm::new(-15);
pub const PARM_GET_TEXGAMMATABLE_PTR: RefParm = RefParm::new(-16);
pub const PARM_GET_LIGHTGAMMATABLE_PTR: RefParm = RefParm::new(-17);
pub const PARM_GET_SCREENGAMMATABLE_PTR: RefParm = RefParm::new(-18);
pub const PARM_GET_LINEARGAMMATABLE_PTR: RefParm = RefParm::new(-19);
pub const PARM_GET_LIGHTSTYLES_PTR: RefParm = RefParm::new(-20);
pub const PARM_GET_DLIGHTS_PTR: RefParm = RefParm::new(-21);
pub const PARM_GET_ELIGHTS_PTR: RefParm = RefParm::new(-22);

/// Returns non-null integer if filtering is enabled for texture.
///
/// Pass -1 to query global filtering settings.
pub const PARM_TEX_FILTERING: RefParm = RefParm::new(-0x10000);

#[allow(deprecated)]
pub trait RgbDataExt {
    fn flags(&self) -> &OutputImageFlags;

    fn flags_mut(&mut self) -> &mut OutputImageFlags;

    fn type_(&self) -> PixelFormat;
}

#[allow(deprecated)]
impl RgbDataExt for rgbdata_t {
    fn flags(&self) -> &OutputImageFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    fn flags_mut(&mut self) -> &mut OutputImageFlags {
        unsafe { mem::transmute(&mut self.flags) }
    }

    fn type_(&self) -> PixelFormat {
        PixelFormat::from_raw(self.type_).unwrap()
    }
}
