use core::{
    cmp,
    ffi::{c_int, CStr},
    fmt,
    num::NonZero,
    ops::Deref,
};

use csz::CStrThin;

use crate::prelude::*;

#[cfg(feature = "save")]
use crate::save;

pub use xash3d_shared::str::ToEngineStr;

/// A string that valid until the end of the current map.
#[derive(Copy, Clone)]
pub struct MapString {
    engine: ServerEngineRef,
    index: NonZero<c_int>,
}

impl MapString {
    /// Creates a new string from the given index.
    pub fn from_index(engine: ServerEngineRef, index: c_int) -> Option<Self> {
        NonZero::new(index).map(|index| Self { engine, index })
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

    pub fn is_none_or_empty(s: Option<MapString>) -> bool {
        s.map(|s| s.is_empty()).unwrap_or(true)
    }

    pub fn is_null_or_empty(engine: ServerEngineRef, index: c_int) -> bool {
        Self::is_none_or_empty(MapString::from_index(engine, index))
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

impl PartialOrd for MapString {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MapString {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_thin().cmp(other.as_thin())
    }
}

impl PartialEq for MapString {
    fn eq(&self, other: &Self) -> bool {
        self.as_thin() == other.as_thin()
    }
}

impl Eq for MapString {}

impl PartialEq<&CStrThin> for MapString {
    fn eq(&self, other: &&CStrThin) -> bool {
        self.as_thin() == *other
    }
}

impl PartialEq<&CStr> for MapString {
    fn eq(&self, other: &&CStr) -> bool {
        self.as_thin() == <&CStrThin>::from(*other)
    }
}

impl PartialEq<MapString> for &CStrThin {
    fn eq(&self, other: &MapString) -> bool {
        *self == other.as_thin()
    }
}

impl PartialEq<MapString> for &CStr {
    fn eq(&self, other: &MapString) -> bool {
        <&CStrThin>::from(*self) == other.as_thin()
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

impl ToEngineStr for MapString {
    type Output = MapString;

    fn to_engine_str(&self) -> Self::Output {
        *self
    }
}

impl<'a> ToEngineStr for &'a MapString {
    type Output = &'a CStrThin;

    fn to_engine_str(&self) -> Self::Output {
        self.as_thin()
    }
}

#[cfg(feature = "save")]
impl Save for Option<MapString> {
    fn save(&self, _: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        let bytes = self.as_ref().map_or(&[][..], |s| s.to_bytes_with_nul());
        cur.write_bytes_with_size(bytes)?;
        Ok(())
    }
}

#[cfg(feature = "save")]
impl Restore for Option<MapString> {
    fn restore(
        &mut self,
        state: &save::RestoreState,
        cur: &mut save::Cursor,
    ) -> save::SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        if !bytes.is_empty() {
            let s = CStr::from_bytes_with_nul(bytes).map_err(|_| save::SaveError::InvalidString)?;
            *self = Some(state.engine().new_map_string(s));
        } else {
            *self = None;
        }
        Ok(())
    }
}
