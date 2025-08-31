use core::ffi::c_char;

use csz::CStrThin;

use crate::str::{AsCStrPtr, ToEngineStr};

pub type GetCvarFloatFn = unsafe extern "C" fn(name: *const c_char) -> f32;
pub type SetCvarValueFn = unsafe extern "C" fn(name: *const c_char, value: f32);
pub type GetCvarStringFn = unsafe extern "C" fn(name: *const c_char) -> *const c_char;
pub type SetCvarStringFn = unsafe extern "C" fn(name: *const c_char, value: *const c_char);

pub fn get_cvar_float(func: GetCvarFloatFn, name: impl ToEngineStr) -> f32 {
    let name = name.to_engine_str();
    unsafe { (func)(name.as_ptr()) }
}

pub fn set_cvar_float(func: SetCvarValueFn, name: impl ToEngineStr, value: f32) {
    let name = name.to_engine_str();
    unsafe { (func)(name.as_ptr(), value) }
}

pub fn get_cvar_string<'a>(func: GetCvarStringFn, name: impl ToEngineStr) -> &'a CStrThin {
    let name = name.to_engine_str();
    // FIXME: The lifetime of the returned string is valid only until the cvar is modified.
    let ptr = unsafe { (func)(name.as_ptr()) };
    // SAFETY: the engine returns an empty string if cvar is not found
    unsafe { CStrThin::from_ptr(ptr) }
}

pub fn set_cvar_string(func: SetCvarStringFn, name: impl ToEngineStr, value: impl ToEngineStr) {
    let name = name.to_engine_str();
    let value = value.to_engine_str();
    unsafe { (func)(name.as_ptr(), value.as_ptr()) }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_engine_cvar {
    ($ty:ty {
        $get_float:ident,
        $set_float:ident,
        $get_string:ident,
        $set_string:ident $(,)?
    }) => {
        impl EngineCvar for $ty {
            fn get_cvar_float(&self, name: impl ToEngineStr) -> f32 {
                let name = name.to_engine_str();
                unsafe { unwrap!(self, $get_float)(name.as_ptr()) }
            }

            fn set_cvar_float(&self, name: impl ToEngineStr, value: f32) {
                let name = name.to_engine_str();
                unsafe { unwrap!(self, $set_float)(name.as_ptr(), value) }
            }

            fn get_cvar_string(&self, name: impl ToEngineStr) -> &CStrThin {
                let name = name.to_engine_str();
                // FIXME: The lifetime of the returned string is valid only until the cvar is modified.
                let ptr = unsafe { unwrap!(self, $get_string)(name.as_ptr()) };
                // SAFETY: the engine returns an empty string if cvar is not found
                unsafe { CStrThin::from_ptr(ptr) }
            }

            fn set_cvar_string(&self, name: impl ToEngineStr, value: impl ToEngineStr) {
                let name = name.to_engine_str();
                let value = value.to_engine_str();
                unsafe { unwrap!(self, $set_string)(name.as_ptr(), value.as_ptr()) }
            }
        }
    };
}
pub use impl_engine_cvar;
