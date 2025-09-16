use core::{
    ffi::{c_int, c_uint, CStr},
    num::NonZeroU32,
};

use bitflags::bitflags;
use shared::{
    color::RGBA,
    ffi::{self, render::imgFlags_t},
    macros::const_assert_size_eq,
    math::sqrtf,
    render::TextureFlags,
};

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
