#![no_std]

#[macro_use]
pub mod macros;
mod efx;
mod engine;
mod event;
pub mod input;
mod logger;
pub mod message;
pub mod raw;
pub mod sprite;
mod studio;
pub mod utils;

use core::ffi::c_int;

use shared::export::UnsyncGlobal;

pub use shared::{cell, color, consts, cvar, math};

pub use crate::{efx::EfxApi, engine::*, event::EventApi, studio::*};

pub const CLDLL_INTERFACE_VERSION: c_int = 7;

/// Initialize the global [Engine] instance.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(engine_funcs: &raw::cl_enginefuncs_s) {
    unsafe {
        (*Engine::global_as_mut_ptr()).write(Engine::new(engine_funcs));
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

/// Initialize the global [Studio] instance.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_studio(funcs: &raw::engine_studio_api_s) {
    unsafe {
        (*Studio::global_as_mut_ptr()).write(Studio::new(funcs));
    }
}

/// Returns a reference to the global [Studio] instance.
///
/// # Safety
///
/// Must not be called before [init_studio].
pub fn studio<'a>() -> &'a Studio {
    unsafe { Studio::global_assume_init_ref() }
}
