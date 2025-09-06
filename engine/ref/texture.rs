use core::{
    ffi::{c_int, c_uint},
    num::NonZeroU32,
};

use shared::macros::const_assert_size_eq;

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
