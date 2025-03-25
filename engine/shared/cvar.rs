use core::{
    ffi::{c_char, c_int, CStr},
    ops::Deref,
    ptr,
};

use bitflags::bitflags;
use cell::SyncOnceCell;

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct CVarFlags: c_int {
        const NONE                  = 0;
        /// Set to cause it to be saved to vars.rc.
        const ARCHIVE               = 1 << 0;
        /// Changes the client's info string.
        const USERINFO              = 1 << 1;
        /// Notifies players when changed.
        const SERVER                = 1 << 2;
        /// Defined by external DLL.
        const EXTDLL                = 1 << 3;
        /// Defined by the client dll.
        const CLIENTDLL             = 1 << 4;
        /// It's a server cvar.
        ///
        /// But we don't send the data since it's a password, etc. Sends 1 if
        /// it's not bland/zero, 0 otherwise as value.
        const PROTECTED             = 1 << 5;
        /// This cvar cannot be changed by clients connected to a multiplayer server.
        const SPONLY                = 1 << 6;
        /// This cvar's string cannot contain unprintable characters.
        ///
        /// Used for player name, etc.
        const PRINTABLEONLY         = 1 << 7;
        /// If this is a FCVAR_SERVER.
        ///
        /// Don't log changes to the log file / console if we are creating a log.
        const UNLOGGED              = 1 << 8;
        /// Strip trailing/leading white space from this cvar.
        const NOEXTRAWHITEPACE      = 1 << 9;
        /// Not queryable/settable by unprivileged sources.
        const PRIVILEGED            = 1 << 10;
        /// Not queryable/settable if unprivileged and filterstufftext is enabled.
        const FILTERSTUFFTEXT       = 1 << 11;
        /// This cvar's string will be filtered for 'bad' characters (e.g. ';', '\n').
        const FILTERCHARS           = 1 << 12;
        /// This cvar's string cannot contain file paths that are above the current directory.
        const NOBADPATHS            = 1 << 13;
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cvar_s {
    pub name: *const c_char,
    pub string: *mut c_char,
    pub flags: CVarFlags,
    pub value: f32,
    pub next: *mut cvar_s,
}

/// SAFETY: engine is single-threaded
unsafe impl Sync for cvar_s {}

impl cvar_s {
    pub const fn new(name: &'static CStr, value: f32, flags: CVarFlags) -> Self {
        Self {
            name: name.as_ptr(),
            string: ptr::null_mut(),
            flags,
            value,
            next: ptr::null_mut(),
        }
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.name) }
    }

    pub fn string(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.string) }
    }

    pub fn flags(&self) -> CVarFlags {
        self.flags
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn value_as_bool(&self) -> bool {
        self.value() != 0.0
    }

    pub fn next_var(&mut self) -> Option<&mut Self> {
        if self.next.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.next })
        }
    }
}

pub mod flags {
    use super::CVarFlags;

    pub const NONE: CVarFlags = CVarFlags::NONE;
    pub const ARCHIVE: CVarFlags = CVarFlags::ARCHIVE;
    pub const USERINFO: CVarFlags = CVarFlags::USERINFO;
    pub const SERVER: CVarFlags = CVarFlags::SERVER;
    pub const EXTDLL: CVarFlags = CVarFlags::EXTDLL;
    pub const CLIENTDLL: CVarFlags = CVarFlags::CLIENTDLL;
    pub const PROTECTED: CVarFlags = CVarFlags::PROTECTED;
    pub const SPONLY: CVarFlags = CVarFlags::SPONLY;
    pub const PRINTABLEONLY: CVarFlags = CVarFlags::PRINTABLEONLY;
    pub const UNLOGGED: CVarFlags = CVarFlags::UNLOGGED;
    pub const NOEXTRAWHITEPACE: CVarFlags = CVarFlags::NOEXTRAWHITEPACE;
    pub const PRIVILEGED: CVarFlags = CVarFlags::PRIVILEGED;
    pub const FILTERSTUFFTEXT: CVarFlags = CVarFlags::FILTERSTUFFTEXT;
    pub const FILTERCHARS: CVarFlags = CVarFlags::FILTERCHARS;
    pub const NOBADPATHS: CVarFlags = CVarFlags::NOBADPATHS;
}

pub use self::flags::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct CVarPtr {
    raw: *mut cvar_s,
}

impl CVarPtr {
    pub const fn from_ptr(raw: *mut cvar_s) -> Self {
        Self { raw }
    }

    pub const fn null() -> Self {
        Self {
            raw: ptr::null_mut(),
        }
    }

    pub fn as_ptr(&self) -> *mut cvar_s {
        self.raw
    }

    pub fn is_null(self) -> bool {
        self.raw.is_null()
    }

    pub fn name(self) -> &'static CStr {
        if !self.raw.is_null() {
            unsafe { (*self.raw).name() }
        } else {
            c"undefined"
        }
    }

    pub fn value(self) -> f32 {
        if !self.raw.is_null() {
            unsafe { ptr::read(&(*self.raw).value) }
        } else {
            error!("CVarPtr: read from null cvar");
            0.0
        }
    }

    pub fn value_str(&self) -> &CStr {
        if !self.raw.is_null() {
            let ptr = unsafe { ptr::read(&(*self.raw).string) };
            assert!(!ptr.is_null());
            unsafe { CStr::from_ptr(ptr) }
        } else {
            error!("CVarPtr: read from null cvar");
            c""
        }
    }

    pub fn value_set(self, value: f32) {
        if !self.raw.is_null() {
            unsafe { ptr::write(&mut (*self.raw).value, value) }
        } else {
            error!("CVarPtr: write to null cvar");
        }
    }
}

impl Deref for CVarPtr {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        if !self.raw.is_null() {
            unsafe { &(*self.raw).value }
        } else {
            error!("CVarPtr: read from null cvar");
            &0.0
        }
    }
}

pub type GetCVarFn = fn(&CStr, &CStr, CVarFlags) -> CVarPtr;

static GET_CVAR: SyncOnceCell<GetCVarFn> = unsafe { SyncOnceCell::new() };

pub fn init(get_cvar: GetCVarFn) {
    GET_CVAR.get_or_init(|| get_cvar);
}

pub struct CVar {
    ptr: SyncOnceCell<CVarPtr>,
    name: &'static CStr,
    value: &'static CStr,
    flags: CVarFlags,
}

impl CVar {
    pub const fn new(name: &'static CStr, value: &'static CStr, flags: CVarFlags) -> Self {
        Self {
            ptr: unsafe { SyncOnceCell::new() },
            name,
            value,
            flags,
        }
    }

    pub fn get_ptr(&self) {
        self.value();
    }
}

impl Deref for CVar {
    type Target = CVarPtr;

    fn deref(&self) -> &Self::Target {
        self.ptr
            .get_or_init(|| GET_CVAR.get().unwrap()(self.name, self.value, self.flags))
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! define {
    ($($(#[$meta:meta])* $vis:vis static $name:ident$(($value:expr $(, $flags:expr)?))?;)*) => {
        $(
            #[allow(non_upper_case_globals)]
            $(#[$meta])*
            $vis static $name: $crate::cvar::CVar = {
                use $crate::cvar::flags::*;

                let value = c"";
                let flags = NONE;

                $(
                    let value = $value;
                    $(let flags = $flags;)?
                )?

                $crate::cvar::CVar::new(
                    shared::macros::cstringify!($name),
                    value,
                    flags,
                )
            };
        )*
    };
}
#[doc(inline)]
pub use define;
