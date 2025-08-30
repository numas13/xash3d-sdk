#![no_std]

#[macro_use]
pub mod macros;
mod engine;
mod globals;
mod logger;
pub mod raw;
pub mod utils;

use shared::export::UnsyncGlobal;

pub use shared::{cell, consts, cvar, math};

pub use crate::engine::*;
pub use crate::globals::Globals;

/// Initialize the global [Engine] and [Globals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(funcs: &raw::enginefuncs_s, globals: *mut raw::globalvars_t) {
    unsafe {
        (*Engine::global_as_mut_ptr()).write(Engine::new(funcs));
        (*Globals::global_as_mut_ptr()).write(Globals::new(globals));
    }
    crate::logger::init_console_logger();
}

/// Returns a reference to the global [Engine] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn engine() -> &'static Engine {
    unsafe { Engine::global_assume_init_ref() }
}

/// Returns a reference to the global [Globals] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn globals() -> &'static Globals {
    unsafe { Globals::global_assume_init_ref() }
}

/// Returns a mutable reference to the global [Globals] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn globals_mut() -> &'static mut Globals {
    unsafe { Globals::global_assume_init_mut() }
}
