use core::{
    ffi::{c_char, c_int},
    fmt,
};

use csz::CStrThin;

use crate::{
    cvar::{GetCvar, SetCvar},
    str::{AsCStrPtr, ToEngineStr},
};

/// Engine API to read and modify console variables.
pub trait EngineCvar: Sized {
    #[doc(hidden)]
    fn fn_get_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char) -> f32;

    #[doc(hidden)]
    fn fn_set_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char, value: f32);

    #[doc(hidden)]
    fn fn_get_cvar_string(&self) -> unsafe extern "C" fn(name: *const c_char) -> *const c_char;

    #[doc(hidden)]
    fn fn_set_cvar_string(&self)
        -> unsafe extern "C" fn(name: *const c_char, value: *const c_char);

    fn get_cvar_float(&self, name: impl ToEngineStr) -> f32 {
        let name = name.to_engine_str();
        unsafe { self.fn_get_cvar_float()(name.as_ptr()) }
    }

    fn set_cvar_float(&self, name: impl ToEngineStr, value: f32) {
        let name = name.to_engine_str();
        unsafe { self.fn_set_cvar_float()(name.as_ptr(), value) }
    }

    fn get_cvar_string(&self, name: impl ToEngineStr) -> &CStrThin {
        let name = name.to_engine_str();
        // FIXME: The lifetime of the returned string is valid only until the cvar is modified.
        let ptr = unsafe { self.fn_get_cvar_string()(name.as_ptr()) };
        // SAFETY: the engine returns an empty string if cvar is not found
        unsafe { CStrThin::from_ptr(ptr) }
    }

    fn set_cvar_string(&self, name: impl ToEngineStr, value: impl ToEngineStr) {
        let name = name.to_engine_str();
        let value = value.to_engine_str();
        unsafe { self.fn_set_cvar_string()(name.as_ptr(), value.as_ptr()) }
    }

    fn get_cvar<'a, T: GetCvar<'a>>(&'a self, name: impl ToEngineStr) -> T {
        T::get_cvar(self, name)
    }

    fn set_cvar<T: SetCvar>(&self, name: impl ToEngineStr, value: T) {
        T::set_cvar(self, name, value)
    }
}

/// Engine API to generate random numbers.
pub trait EngineRng {
    #[doc(hidden)]
    fn fn_random_float(&self) -> unsafe extern "C" fn(min: f32, max: f32) -> f32;

    #[doc(hidden)]
    fn fn_random_int(&self) -> unsafe extern "C" fn(min: c_int, max: c_int) -> c_int;

    fn random_float(&self, min: f32, max: f32) -> f32 {
        unsafe { self.fn_random_float()(min, max) }
    }

    fn random_int(&self, min: c_int, max: c_int) -> c_int {
        assert!(min >= 0, "min must be greater than or equal to zero");
        assert!(min <= max, "min must be less than or equal to max");
        unsafe { self.fn_random_int()(min, max) }
    }
}

/// Engine API to print messages to the console.
pub trait EngineConsole {
    fn console_print(&self, msg: impl ToEngineStr);
}

#[derive(Debug)]
pub struct AddCmdError;

impl fmt::Display for AddCmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "failed to add a console command".fmt(f)
    }
}

/// Engine API to register console commands and access arguments.
pub trait EngineCmd {
    #[doc(hidden)]
    fn fn_cmd_argc(&self) -> unsafe extern "C" fn() -> c_int;

    #[doc(hidden)]
    fn fn_cmd_argv(&self) -> unsafe extern "C" fn(argc: c_int) -> *const c_char;

    fn add_command(
        &self,
        name: impl ToEngineStr,
        func: unsafe extern "C" fn(),
    ) -> Result<(), AddCmdError>;

    fn cmd_argc(&self) -> usize {
        unsafe { self.fn_cmd_argc()() as usize }
    }

    fn cmd_argv(&self, index: usize) -> &CStrThin {
        let ptr = unsafe { self.fn_cmd_argv()(index as c_int) };
        // SAFETY: the engine returns an empty string if index is greater than
        // arguments count
        unsafe { CStrThin::from_ptr(ptr) }
    }

    /// Returns an iterator over command arguments.
    fn cmd_args(&self) -> impl Iterator<Item = &CStrThin> {
        (0..self.cmd_argc()).map(|i| self.cmd_argv(i))
    }
}

/// Engine API to access a raw command arguments string.
pub trait EngineCmdArgsRaw {
    #[doc(hidden)]
    fn fn_cmd_args_raw(&self) -> unsafe extern "C" fn() -> *const c_char;

    /// Returns a raw command arguments string without command name.
    fn cmd_args_raw(&self) -> Option<&CStrThin> {
        let ptr = unsafe { self.fn_cmd_args_raw()() };
        if !ptr.is_null() {
            Some(unsafe { CStrThin::from_ptr(ptr) })
        } else {
            None
        }
    }
}
