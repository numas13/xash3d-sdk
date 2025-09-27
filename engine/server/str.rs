use core::{
    ffi::{c_int, CStr},
    fmt,
    num::NonZeroI32,
    ops::Deref,
};

use csz::CStrThin;

pub use xash3d_shared::str::ToEngineStr;

use crate::prelude::*;

/// A string that valid until the end of the current map.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MapString {
    engine: ServerEngineRef,
    // TODO: replace with NonZero<c_int> when MSRV >= 1.79
    index: NonZeroI32,
}

impl MapString {
    /// Creates a new string from the given index.
    pub fn from_index(engine: ServerEngineRef, index: c_int) -> Option<Self> {
        NonZeroI32::new(index).map(|index| Self { engine, index })
    }

    pub const fn index(&self) -> c_int {
        self.index.get()
    }

    pub fn as_thin(&self) -> &CStrThin {
        self.engine
            .find_map_string(self)
            .unwrap_or(c"<invalid MapString>".into())
    }

    pub fn as_c_str(&self) -> &CStr {
        self.as_thin().as_c_str()
    }

    pub fn is_empty(&self) -> bool {
        self.as_thin().is_empty()
    }

    pub fn is_null_or_empty(engine: ServerEngineRef, index: c_int) -> bool {
        MapString::from_index(engine, index)
            .map(|s| s.is_empty())
            .unwrap_or(true)
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
