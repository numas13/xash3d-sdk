use core::{cell::UnsafeCell, ffi::CStr, ptr};

use xash3d_shared::{csz::CStrThin, ffi::common::cvar_s, macros::const_assert_size_eq};

use crate::prelude::*;

pub use xash3d_shared::cvar::*;

pub type Cvar<T = f32> = xash3d_shared::cvar::Cvar<ServerEngine, T>;

const_assert_size_eq!(*mut cvar_s, Cvar);
const_assert_size_eq!(*mut cvar_s, Option<Cvar>);

/// A storage for a server console variable.
///
/// # Examples
///
/// ```no_run
/// use xash3d_server::{
///     cvar::{Cvar, CvarStorage},
///     prelude::*,
///     csz::CStrThin,
/// };
///
/// static SV_VERSION: CvarStorage = CvarStorage::new(c"sv_version", c"0.1");
///
/// fn init(engine: &ServerEngine) {
///     let sv_version = engine.create_cvar::<CStrThin>(&SV_VERSION);
///     assert_eq!(sv_version.get(), c"0.1");
/// }
/// ```
pub struct CvarStorage {
    raw: UnsafeCell<cvar_s>,
}

unsafe impl Sync for CvarStorage {}

impl CvarStorage {
    /// Creates a new cvar storage.
    pub const fn new(name: &'static CStr, default_value: &'static CStr) -> Self {
        Self::with_flags(name, default_value, NO_FLAGS)
    }

    /// Creates a new cvar storage with the given flags.
    pub const fn with_flags(
        name: &'static CStr,
        default_value: &'static CStr,
        flags: CvarFlags,
    ) -> Self {
        Self {
            raw: UnsafeCell::new(cvar_s {
                name: name.as_ptr().cast_mut(),
                string: default_value.as_ptr().cast_mut(),
                flags: flags.bits(),
                value: 0.0,
                next: ptr::null_mut(),
            }),
        }
    }

    pub(crate) fn as_ptr(&self) -> *mut cvar_s {
        self.raw.get()
    }

    /// Gets the cvar name.
    pub fn name(&self) -> &CStrThin {
        unsafe {
            let name = (*self.raw.get()).name;
            CStrThin::from_ptr(name)
        }
    }
}
