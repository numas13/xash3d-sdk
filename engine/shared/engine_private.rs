mod str;

use core::ffi::c_char;

use csz::CStrThin;

pub use self::str::{AsCStrPtr, ToEngineStr};

pub type GetCvarStringFn = unsafe extern "C" fn(name: *const c_char) -> *const c_char;

pub fn get_cvar_string<'a>(func: GetCvarStringFn, name: impl ToEngineStr) -> &'a CStrThin {
    let name = name.to_engine_str();
    // FIXME: The lifetime of the returned string is valid only until the cvar is modified.
    let ptr = unsafe { (func)(name.as_ptr()) };
    // SAFETY: the engine returns an empty string if cvar is not found
    unsafe { CStrThin::from_ptr(ptr) }
}
