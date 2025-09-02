use core::{
    ffi::{c_int, CStr},
    fmt,
    num::NonZeroI32,
    ops::Deref,
};

use csz::CStrThin;
use shared::macros::const_assert_size_eq;

use crate::instance::engine;

pub use shared::str::ToEngineStr;

/// A string that valid until the end of the current map.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct MapString {
    // TODO: replace with NonZero<c_int> when MSRV >= 1.79
    index: NonZeroI32,
}
const_assert_size_eq!(c_int, MapString);
const_assert_size_eq!(c_int, Option<MapString>);

impl MapString {
    /// Creates a new string from the given index.
    pub fn from_index(index: c_int) -> Option<Self> {
        NonZeroI32::new(index).map(|index| Self { index })
    }

    /// Tries to create a new map string from a given `string`.
    pub fn try_new(string: impl ToEngineStr) -> Option<Self> {
        engine().alloc_map_string(string)
    }

    /// Creates a new map string from a given `string`.
    pub fn new(string: impl ToEngineStr) -> Self {
        Self::try_new(string).expect("failed to allocate a map string")
    }

    pub const fn index(&self) -> c_int {
        self.index.get()
    }

    pub fn as_thin(&self) -> &CStrThin {
        engine()
            .find_map_string(self)
            .unwrap_or(c"<invalid MapString>".into())
    }

    pub fn as_c_str(&self) -> &CStr {
        self.as_thin().as_c_str()
    }
}

impl Deref for MapString {
    type Target = CStrThin;

    fn deref(&self) -> &Self::Target {
        self.as_thin()
    }
}

impl AsRef<CStrThin> for MapString {
    fn as_ref(&self) -> &CStrThin {
        self.as_thin()
    }
}

impl fmt::Debug for MapString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_thin(), f)
    }
}

impl fmt::Display for MapString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_thin(), f)
    }
}

impl<'a> ToEngineStr for &'a MapString {
    type Output = &'a CStrThin;

    fn to_engine_str(&self) -> Self::Output {
        self.as_thin()
    }
}
