use core::{
    ffi::CStr,
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
    str::Utf8Error,
};

use alloc::{ffi::CString, string::String};
use bitflags::bitflags;
use csz::{CStrBox, CStrThin};
use xash3d_ffi::{self as ffi, common::cvar_s};

use crate::{
    cell::SyncOnceCell,
    engine::{EngineCvar, EngineRef},
    export::UnsyncGlobal,
    str::ToEngineStr,
};

bitflags! {
    // TODO: add docs for cvar flags
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct CvarFlags: u32 {
        const NONE = 0;
        const ARCHIVE = ffi::common::FCVAR_ARCHIVE;
        const USER_INFO = ffi::common::FCVAR_USERINFO;
        const SERVER = ffi::common::FCVAR_SERVER;
        const EXT_DLL = ffi::common::FCVAR_EXTDLL;
        const CLIENT_DLL = ffi::common::FCVAR_CLIENTDLL;
        const PROTECTED = ffi::common::FCVAR_PROTECTED;
        const SP_ONLY = ffi::common::FCVAR_SPONLY;
        const PRINTABLE_ONLY = ffi::common::FCVAR_PRINTABLEONLY;
        const UNLOGGED = ffi::common::FCVAR_UNLOGGED;
        const NO_EXTRA_WHITESPACE = ffi::common::FCVAR_NOEXTRAWHITESPACE;
        const PRIVILEGED = ffi::common::FCVAR_PRIVILEGED;
        const FILTERABLE = ffi::common::FCVAR_FILTERABLE;
        const GL_CONFIG = ffi::common::FCVAR_GLCONFIG;
        const CHANGED = ffi::common::FCVAR_CHANGED;
        const GAMEUI_DLL = ffi::common::FCVAR_GAMEUIDLL;
        const CHEAT = ffi::common::FCVAR_CHEAT;
        const LATCH = ffi::common::FCVAR_LATCH;
    }
}

pub const NO_FLAGS: CvarFlags = CvarFlags::NONE;
pub const ARCHIVE: CvarFlags = CvarFlags::ARCHIVE;
pub const USER_INFO: CvarFlags = CvarFlags::USER_INFO;
pub const SERVER: CvarFlags = CvarFlags::SERVER;
pub const EXT_DLL: CvarFlags = CvarFlags::EXT_DLL;
pub const CLIENT_DLL: CvarFlags = CvarFlags::CLIENT_DLL;
pub const PROTECTED: CvarFlags = CvarFlags::PROTECTED;
pub const SP_ONLY: CvarFlags = CvarFlags::SP_ONLY;
pub const PRINTABLE_ONLY: CvarFlags = CvarFlags::PRINTABLE_ONLY;
pub const UNLOGGED: CvarFlags = CvarFlags::UNLOGGED;
pub const NO_EXTRA_WHITESPACE: CvarFlags = CvarFlags::NO_EXTRA_WHITESPACE;
pub const PRIVILEGED: CvarFlags = CvarFlags::PRIVILEGED;
pub const FILTERABLE: CvarFlags = CvarFlags::FILTERABLE;
pub const GL_CONFIG: CvarFlags = CvarFlags::GL_CONFIG;
pub const CHANGED: CvarFlags = CvarFlags::CHANGED;
pub const GAMEUI_DLL: CvarFlags = CvarFlags::GAMEUI_DLL;
pub const CHEAT: CvarFlags = CvarFlags::CHEAT;
pub const LATCH: CvarFlags = CvarFlags::LATCH;

/// A console variable wrapper.
pub struct Cvar<E, T: ?Sized = f32> {
    engine: EngineRef<E>,
    raw: NonNull<cvar_s>,
    phantom: PhantomData<T>,
}

impl<E, T> Cvar<E, T>
where
    T: ?Sized,
{
    /// Constructs a `Cvar` for the given raw console variable pointer.
    ///
    /// Returns a `None` if the pointer is null.
    ///
    /// # Safety
    ///
    /// The pointer must be received from `register_cvar` or `find_cvar` engine methods.
    #[doc(hidden)]
    pub unsafe fn new(engine: EngineRef<E>, raw: *mut cvar_s) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self {
            engine,
            raw,
            phantom: PhantomData,
        })
    }

    /// Returns a mutable pointer to the underlying [cvar_s].
    pub fn as_ptr(&self) -> *mut cvar_s {
        self.raw.as_ptr()
    }

    /// Gets the console variable's flags.
    pub fn flags(&self) -> CvarFlags {
        let flags = unsafe { self.raw.as_ref().flags };
        CvarFlags::from_bits_retain(flags)
    }

    /// Gets the console variable's name.
    pub fn name(&self) -> &CStrThin {
        let name = unsafe { self.raw.as_ref().name };
        assert!(!name.is_null());
        unsafe { CStrThin::from_ptr(name) }
    }

    /// Gets the console variable's value.
    pub fn get_f32(&self) -> f32 {
        unsafe { self.raw.as_ref().value }
    }

    /// Gets the console variable's value as a thin C string reference.
    pub fn get_thin(&self) -> &CStrThin {
        let value = unsafe { self.raw.as_ref().string };
        assert!(!value.is_null());
        unsafe { CStrThin::from_ptr(value) }
    }

    /// Gets the console variable's value as a fat C string reference.
    pub fn get_c_str(&self) -> &CStr {
        self.get_thin().as_c_str()
    }
}

impl<E> Cvar<E, f32> {
    /// Gets the console variable's value.
    pub fn get(&self) -> f32 {
        self.get_f32()
    }
}

macro_rules! impl_cvar_get_num_value {
    ($( $ty:ty ),* $(,)?) => {
        $(
            impl<E> Cvar<E, $ty> {
                #[doc = concat!("Gets the console variable's value as [", stringify!($ty), "].")]
                pub fn get(&self) -> $ty {
                    self.get_f32() as $ty
                }
            }
        )*
    };
}

impl_cvar_get_num_value! {
    f64,

    u8,
    u16,
    u32,
    u64,
    u128,
    usize,

    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
}

impl<E> Cvar<E, bool>
where
    E: UnsyncGlobal + EngineCvar,
{
    /// Gets the console variable's value as [bool].
    pub fn get(&self) -> bool {
        self.get_f32() != 0.0
    }

    pub fn toggle(&self) {
        self.set(!self.get());
    }
}

impl<E> Cvar<E, CStrThin> {
    /// Gets the console variable's value string.
    pub fn get(&self) -> &CStrThin {
        self.get_thin()
    }
}

impl<E> Cvar<E, CStr> {
    /// Gets the console variable's value string.
    pub fn get(&self) -> &CStr {
        self.get_c_str()
    }
}

impl<E, T> Cvar<E, T>
where
    E: UnsyncGlobal + EngineCvar,
{
    /// Sets the contained value from the given [f32] value.
    pub fn set_f32(&self, value: f32) {
        self.engine.direct_set_cvar_float(self, value);
    }

    /// Sets the contained value from the given string.
    pub fn set_string(&self, value: impl ToEngineStr) {
        self.engine.direct_set_cvar_string(self, value);
    }
}

impl<E, T> Cvar<E, T>
where
    T: SetCvar,
    E: UnsyncGlobal + EngineCvar,
{
    /// Sets the contained value from the given value.
    pub fn set(&self, value: T) {
        self.engine.direct_set_cvar(self, value);
    }
}

#[deprecated]
pub mod flags {
    #![allow(deprecated)]

    use bitflags::bitflags;

    bitflags! {
        #[deprecated]
        #[derive(Copy, Clone, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct CVarFlags: u32 {
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

#[allow(deprecated)]
pub use self::flags::*;

#[deprecated]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct CVarPtr {
    raw: *mut cvar_s,
}

#[allow(deprecated)]
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

    pub fn name(self) -> &'static CStrThin {
        if !self.raw.is_null() {
            unsafe { CStrThin::from_ptr((*self.raw).name) }
        } else {
            c"undefined".into()
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

    pub fn value_str(&self) -> &CStrThin {
        if !self.raw.is_null() {
            let ptr = unsafe { ptr::read(&(*self.raw).string) };
            assert!(!ptr.is_null());
            unsafe { CStrThin::from_ptr(ptr) }
        } else {
            error!("CVarPtr: read from null cvar");
            c"".into()
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

#[allow(deprecated)]
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

#[deprecated]
#[allow(deprecated)]
pub type GetCVarFn = fn(&CStrThin, &CStrThin, CVarFlags) -> CVarPtr;

#[allow(deprecated)]
static GET_CVAR: SyncOnceCell<GetCVarFn> = unsafe { SyncOnceCell::new() };

#[deprecated]
#[allow(deprecated)]
pub fn init(get_cvar: GetCVarFn) {
    GET_CVAR.get_or_init(|| get_cvar);
}

#[deprecated]
#[allow(deprecated)]
pub struct CVar {
    ptr: SyncOnceCell<CVarPtr>,
    name: &'static CStr,
    value: &'static CStr,
    flags: CVarFlags,
}

#[allow(deprecated)]
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

#[allow(deprecated)]
impl Deref for CVar {
    type Target = CVarPtr;

    fn deref(&self) -> &Self::Target {
        self.ptr.get_or_init(|| {
            GET_CVAR.get().unwrap()(self.name.into(), self.value.into(), self.flags)
        })
    }
}

#[deprecated]
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
                    $crate::macros::cstringify!($name),
                    value,
                    flags,
                )
            };
        )*
    };
}
#[allow(deprecated)]
#[doc(inline)]
pub use define;

/// Read a console variable.
///
/// # Note
///
/// Numbers are stored as [f32] and can not represent all possible values.
pub trait GetCvar<'a> {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self;
}

/// Modify a console variable.
pub trait SetCvar {
    fn set_cvar(engine: &impl EngineCvar, name: impl ToEngineStr, value: Self);

    fn direct_set_cvar<E: EngineCvar>(engine: &E, cvar: &Cvar<E, Self>, value: Self);
}

impl<'a> GetCvar<'a> for bool {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
        engine.get_cvar_float(name) != 0.0
    }
}

impl SetCvar for bool {
    fn set_cvar(engine: &impl EngineCvar, name: impl ToEngineStr, value: Self) {
        engine.set_cvar_float(name, if value { 1.0 } else { 0.0 });
    }

    fn direct_set_cvar<E: EngineCvar>(engine: &E, cvar: &Cvar<E, Self>, value: Self) {
        engine.direct_set_cvar_float(cvar, if value { 1.0 } else { 0.0 });
    }
}

macro_rules! impl_cvar_for_number {
    ($($ty:ty),* $(,)?) => {
        $(
            impl<'a> GetCvar<'a> for $ty {
                fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
                    engine.get_cvar_float(name) as $ty
                }
            }

            impl SetCvar for $ty {
                fn set_cvar(engine: &impl EngineCvar, name: impl ToEngineStr, value: Self) {
                    engine.set_cvar_float(name, value as f32);
                }

                fn direct_set_cvar<E: EngineCvar>(engine: &E, cvar: &Cvar<E, Self>, value: Self) {
                    engine.direct_set_cvar_float(cvar, value as f32);
                }
            }
        )*
    };
}

impl_cvar_for_number!(u8, u16, u32, u64, usize);
impl_cvar_for_number!(i8, i16, i32, i64, isize);
impl_cvar_for_number!(f32, f64);

impl<'a> GetCvar<'a> for &'a CStrThin {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
        engine.get_cvar_string(name)
    }
}

impl SetCvar for &CStrThin {
    fn set_cvar(engine: &impl EngineCvar, name: impl ToEngineStr, value: Self) {
        engine.set_cvar_string(name, value);
    }

    fn direct_set_cvar<E: EngineCvar>(engine: &E, cvar: &Cvar<E, Self>, value: Self) {
        engine.direct_set_cvar_string(cvar, value);
    }
}

impl<'a> GetCvar<'a> for &'a CStr {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
        engine.get_cvar_string(name).into()
    }
}

impl SetCvar for &CStr {
    fn set_cvar(engine: &impl EngineCvar, name: impl ToEngineStr, value: Self) {
        engine.set_cvar_string(name, value);
    }

    fn direct_set_cvar<E: EngineCvar>(engine: &E, cvar: &Cvar<E, Self>, value: Self) {
        engine.direct_set_cvar_string(cvar, value);
    }
}

impl<'a> GetCvar<'a> for Result<&'a str, Utf8Error> {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
        engine.get_cvar_string(name).to_str()
    }
}

impl<'a> GetCvar<'a> for CStrBox {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
        engine.get_cvar_string(name).into()
    }
}

impl<'a> GetCvar<'a> for CString {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
        engine.get_cvar_string(name).into()
    }
}

impl SetCvar for &str {
    fn set_cvar(engine: &impl EngineCvar, name: impl ToEngineStr, value: Self) {
        engine.set_cvar_string(name, value);
    }

    fn direct_set_cvar<E: EngineCvar>(engine: &E, cvar: &Cvar<E, Self>, value: Self) {
        engine.direct_set_cvar_string(cvar, value);
    }
}

impl<'a> GetCvar<'a> for Result<String, Utf8Error> {
    fn get_cvar(engine: &'a impl EngineCvar, name: impl ToEngineStr) -> Self {
        engine.get_cvar_string(name).to_str().map(Into::into)
    }
}
