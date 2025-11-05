use core::{
    ffi::{CStr, c_int, c_uint},
    mem,
    num::NonZeroU32,
    ops::{Deref, DerefMut},
};

use bitflags::bitflags;
use xash3d_shared::{
    color::RGBA,
    ffi::{
        self,
        render::{ilFlags_t, imgFlags_t, pixformat_t, rgbdata_s},
    },
    macros::{const_assert_size_eq, define_enum_for_primitive},
    math::sqrtf,
    render::TextureFlags,
};

use crate::engine::RefEngineRef;

pub const SKYBOX_MAX_SIDES: usize = ffi::render::SKYBOX_MAX_SIDES as usize;

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum PixelFormat: pixformat_t {
        /// An unknown pixel format.
        #[default]
        Unknown(ffi::render::pixformat_t_PF_UNKNOWN),
        /// Inflated palette (768 bytes).
        Indexed24(ffi::render::pixformat_t_PF_INDEXED_24),
        /// Deflated palette (1024 bytes).
        Indexed32(ffi::render::pixformat_t_PF_INDEXED_32),
        /// Normal RGBA buffer.
        Rgba32(ffi::render::pixformat_t_PF_RGBA_32),
        /// big endian RGBA (MacOS).
        Bgra32(ffi::render::pixformat_t_PF_BGRA_32),
        /// Uncompressed dds or another 24-bit image.
        Rgb24(ffi::render::pixformat_t_PF_RGB_24),
        /// Big-endian RGB (MacOS).
        Bgr24(ffi::render::pixformat_t_PF_BGR_24),
        Luminance(ffi::render::pixformat_t_PF_LUMINANCE),
        /// s3tc DXT1/BC1 format.
        Dxt1(ffi::render::pixformat_t_PF_DXT1),
        /// s3tc DXT3/BC2 format.
        Dxt3(ffi::render::pixformat_t_PF_DXT3),
        /// s3tc DXT5/BC3 format.
        Dxt5(ffi::render::pixformat_t_PF_DXT5),
        /// latc ATI2N/BC5 format.
        Ati2(ffi::render::pixformat_t_PF_ATI2),
        Bc4Signed(ffi::render::pixformat_t_PF_BC4_SIGNED),
        Bc4Unsigned(ffi::render::pixformat_t_PF_BC4_UNSIGNED),
        Bc5Signed(ffi::render::pixformat_t_PF_BC5_SIGNED),
        Bc5Unsigned(ffi::render::pixformat_t_PF_BC5_UNSIGNED),
        /// bptc BC6H signed FP16 format.
        Bc6hSigned(ffi::render::pixformat_t_PF_BC6H_SIGNED),
        /// bptc BC6H unsigned FP16 format.
        Bc6hUnsigned(ffi::render::pixformat_t_PF_BC6H_UNSIGNED),
        /// bptc BC7 format.
        Bc7Unorm(ffi::render::pixformat_t_PF_BC7_UNORM),
        Bc7Srgb(ffi::render::pixformat_t_PF_BC7_SRGB),
        /// Raw KTX2 data, used for yet unsupported KTX2 subformats.
        Ktx2Raw(ffi::render::pixformat_t_PF_KTX2_RAW),
    }
}

impl PixelFormat {
    pub const fn is_raw(&self) -> bool {
        matches!(
            self,
            Self::Rgba32 | Self::Bgra32 | Self::Rgb24 | Self::Bgr24 | Self::Luminance
        )
    }

    pub const fn is_compressed(&self) -> bool {
        matches!(
            self,
            Self::Dxt1
                | Self::Dxt3
                | Self::Dxt5
                | Self::Ati2
                | Self::Bc4Signed
                | Self::Bc4Unsigned
                | Self::Bc5Signed
                | Self::Bc5Unsigned
                | Self::Bc6hSigned
                | Self::Bc6hUnsigned
                | Self::Bc7Unorm
                | Self::Bc7Srgb
                | Self::Ktx2Raw
        )
    }
}

bitflags! {
    /// Image flags.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct ImageFlags: ilFlags_t {
        const NONE              = 0;
        /// Lerping images during resample.
        const USE_LERPING       = ffi::render::ilFlags_t_IL_USE_LERPING;
        /// Do not expand paletted images.
        const KEEP_8BIT         = ffi::render::ilFlags_t_IL_KEEP_8BIT;
        /// Allow to overwrite stored images.
        const ALLOW_OVERWRITE   = ffi::render::ilFlags_t_IL_ALLOW_OVERWRITE;
        /// Steam background completely ignore tga attribute 0x20 (stupid lammers!).
        const DONTFLIP_TGA      = ffi::render::ilFlags_t_IL_DONTFLIP_TGA;
        /// DXT compression is support.
        const DDS_HARDWARE      = ffi::render::ilFlags_t_IL_DDS_HARDWARE;
        /// The special mode for load gradient decals.
        const LOAD_DECAL        = ffi::render::ilFlags_t_IL_LOAD_DECAL;
        /// Overview required some unique operations.
        const OVERVIEW          = ffi::render::ilFlags_t_IL_OVERVIEW;
        /// Special mode for player decals.
        const LOAD_PLAYER_DECAL = ffi::render::ilFlags_t_IL_LOAD_PLAYER_DECAL;
        /// Renderer can consume raw KTX2 files (e.g. ref_vk).
        const KTX2_RAW          = ffi::render::ilFlags_t_IL_KTX2_RAW;
    }
}

bitflags! {
    /// Output image flags.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct OutputImageFlags: imgFlags_t {
        const NONE          = 0;
        /// The image is a 6-sides cubemap buffer.
        const CUBEMAP       = ffi::render::imgFlags_t_IMAGE_CUBEMAP;
        /// The image contains an alpha channel.
        const HAS_ALPHA     = ffi::render::imgFlags_t_IMAGE_HAS_ALPHA;
        /// The image contains a RGB channels.
        const HAS_COLOR     = ffi::render::imgFlags_t_IMAGE_HAS_COLOR;
        /// All colors in palette is gradients of last color (decals).
        const COLORINDEX    = ffi::render::imgFlags_t_IMAGE_COLORINDEX;
        /// The image has luma pixels (q1-style maps).
        const HAS_LUMA      = ffi::render::imgFlags_t_IMAGE_HAS_LUMA;
        /// Only used by [crate::engine::RefEngine::fs_save_image] for write right suffixes.
        const SKYBOX        = ffi::render::imgFlags_t_IMAGE_SKYBOX;
        /// It is a quake sky double layered clouds (so keep it as 8 bit).
        const QUAKESKY      = ffi::render::imgFlags_t_IMAGE_QUAKESKY;
        /// A hint for GL loader.
        const DDS_FORMAT    = ffi::render::imgFlags_t_IMAGE_DDS_FORMAT;
        /// To differentiate from 3D texture.
        const MULTILAYER    = ffi::render::imgFlags_t_IMAGE_MULTILAYER;
        /// The alpha channel is 1 bit long.
        const ONEBIT_ALPHA  = ffi::render::imgFlags_t_IMAGE_ONEBIT_ALPHA;
        /// The image has quake1 palette.
        const QUAKEPAL      = ffi::render::imgFlags_t_IMAGE_QUAKEPAL;

        /// Flip the image by width.
        const FLIP_X        = ffi::render::imgFlags_t_IMAGE_FLIP_X;
        /// Flip the image by height.
        const FLIP_Y        = ffi::render::imgFlags_t_IMAGE_FLIP_Y;
        /// Flip from upper left corner to down right corner.
        const ROT_90        = ffi::render::imgFlags_t_IMAGE_ROT_90;
        const ROT180        = ffi::render::imgFlags_t_IMAGE_ROT180;
        const ROT270        = ffi::render::imgFlags_t_IMAGE_ROT270;
        /// Resample the image to specified dims.
        const RESAMPLE      = ffi::render::imgFlags_t_IMAGE_RESAMPLE;
        /// Force the image to RGBA buffer.
        const FORCE_RGBA    = ffi::render::imgFlags_t_IMAGE_FORCE_RGBA;
        /// Create the luma texture from indexed.
        const MAKE_LUMA     = ffi::render::imgFlags_t_IMAGE_MAKE_LUMA;
        /// Make the indexed image from 24-bit or 32-bit image.
        const QUANTIZE      = ffi::render::imgFlags_t_IMAGE_QUANTIZE;
        /// Apply gamma for the image.
        const LIGHTGAMMA    = ffi::render::imgFlags_t_IMAGE_LIGHTGAMMA;
        /// Interpret the width and the height as top and bottom color.
        const REMAP         = ffi::render::imgFlags_t_IMAGE_REMAP;
    }
}

/// A special name for missing texture.
pub const UNUSED_TEXTURE_NAME: &CStr = c"*unused*";

/// Default texture information.
pub struct DefaultTexture {
    pub id: TextureId,
    pub name: &'static CStr,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub flags: TextureFlags,
    pub pixel: fn(u16, u16) -> RGBA,
}

impl DefaultTexture {
    pub const DEFAULT: Self = Self {
        id: unsafe { TextureId::new_unchecked(1) },
        name: c"*default",
        width: 16,
        height: 16,
        depth: 1,
        flags: TextureFlags::COLORMAP,
        pixel: |x, y| {
            if (y < 8) ^ (x < 8) {
                RGBA::rgb(255, 0, 255)
            } else {
                RGBA::BLACK
            }
        },
    };

    pub const PARTICLE: Self = Self {
        id: unsafe { TextureId::new_unchecked(2) },
        name: c"*particle",
        width: 16,
        height: 16,
        depth: 1,
        flags: TextureFlags::CLAMP.union(TextureFlags::HAS_ALPHA),
        pixel: |x, y| {
            let dx2 = ((x as i32) - 8).pow(2);
            let dy2 = ((y as i32) - 8).pow(2);
            let d = 255.0 - 35.0 * sqrtf((dx2 + dy2) as f32);
            RGBA::new(255, 255, 255, d.clamp(0.0, 255.0) as u8)
        },
    };

    pub const WHITE: Self = Self {
        id: unsafe { TextureId::new_unchecked(3) },
        name: c"*white",
        width: 4,
        height: 4,
        depth: 1,
        flags: TextureFlags::COLORMAP,
        pixel: |_, _| RGBA::WHITE,
    };

    pub const GRAY: Self = Self {
        id: unsafe { TextureId::new_unchecked(4) },
        name: c"*gray",
        width: 4,
        height: 4,
        depth: 1,
        flags: TextureFlags::COLORMAP,
        pixel: |_, _| RGBA::rgb(127, 127, 127),
    };

    pub const BLACK: Self = Self {
        id: unsafe { TextureId::new_unchecked(5) },
        name: c"*black",
        width: 4,
        height: 4,
        depth: 1,
        flags: TextureFlags::COLORMAP,
        pixel: |_, _| RGBA::BLACK,
    };

    pub const CINEMATIC_DUMMY: Self = Self {
        id: unsafe { TextureId::new_unchecked(6) },
        name: c"*cintexture",
        width: 640,
        height: 100,
        depth: 1,
        flags: TextureFlags::NOMIPMAP.union(TextureFlags::CLAMP),
        pixel: |_, _| RGBA::WHITE,
    };
}

/// Default textures exposed by ref dll.
pub const DEFAULT_TEXTURES: &[DefaultTexture] = &[
    DefaultTexture::DEFAULT,
    DefaultTexture::PARTICLE,
    DefaultTexture::WHITE,
    DefaultTexture::GRAY,
    DefaultTexture::BLACK,
    DefaultTexture::CINEMATIC_DUMMY,
];

/// A valid texture id.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TextureId(NonZeroU32);

const_assert_size_eq!(c_int, TextureId);
const_assert_size_eq!(c_int, Option<TextureId>);

impl TextureId {
    /// Creates a texture id if the given value is not zero.
    pub const fn new(raw: c_int) -> Option<TextureId> {
        match NonZeroU32::new(raw as c_uint) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }

    /// Creates a `TextureId` without checking whether the value is valid.
    ///
    /// # Safety
    ///
    /// The value must not be zero.
    pub const unsafe fn new_unchecked(raw: c_int) -> TextureId {
        Self(unsafe { NonZeroU32::new_unchecked(raw as u32) })
    }

    /// Returns the texture id as a raw primitive type.
    pub const fn raw(&self) -> u32 {
        self.0.get()
    }

    pub(crate) fn to_ffi(id: Option<Self>) -> c_int {
        id.map_or(0, |i| i.raw() as c_int)
    }
}

pub struct RgbData {
    pub(crate) engine: RefEngineRef,
    pub(crate) raw: *mut rgbdata_s,
}

impl RgbData {
    pub fn flags(&self) -> &OutputImageFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    pub fn flags_mut(&mut self) -> &mut OutputImageFlags {
        unsafe { mem::transmute(&mut self.flags) }
    }

    pub fn type_(&self) -> PixelFormat {
        PixelFormat::from_raw(self.type_).unwrap()
    }
}

impl Clone for RgbData {
    fn clone(&self) -> Self {
        let raw = unsafe { self.engine.fs_copy_image(self.raw) };
        assert!(!raw.is_null());
        Self {
            engine: self.engine,
            raw,
        }
    }
}

impl Drop for RgbData {
    fn drop(&mut self) {
        unsafe {
            self.engine.fs_free_image(self.raw);
        }
    }
}

impl Deref for RgbData {
    type Target = rgbdata_s;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for RgbData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}
