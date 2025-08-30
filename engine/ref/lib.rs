#![no_std]

#[macro_use]
extern crate log;

pub mod consts;
mod engine;
mod globals;
mod logger;
pub mod raw;
pub mod utils;

use shared::export::UnsyncGlobal;

pub use shared::{cell, color, cvar, math};

pub use crate::engine::*;
pub use crate::globals::Globals;

/// Initialize the global [Engine] and [Globals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(engine_funcs: &raw::ref_api_s, globals: *mut raw::ref_globals_s) {
    unsafe {
        (*Engine::global_as_mut_ptr()).write(Engine::new(engine_funcs));
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
