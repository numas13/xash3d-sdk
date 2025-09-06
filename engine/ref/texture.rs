use core::{
    ffi::{c_int, c_uint, CStr},
    num::NonZeroU32,
};

use shared::{color::RGBA, macros::const_assert_size_eq, math::sqrtf, raw::TextureFlags};

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
