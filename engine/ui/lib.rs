#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
extern crate log;

pub mod consts;
pub mod engine;
pub mod export;
pub mod picture;
pub mod raw;
pub mod utils;

use core::ptr;

pub use shared::{cell, color, cvar, math, parser};

pub use crate::engine::{ActiveMenu, Engine, Point, Size};

static mut ENGINE: Option<Engine> = None;
static mut GLOBALS: Option<&'static raw::ui_globalvars_s> = None;

/// # Safety
///
/// The lifetime of returned object is inferred at calling side and must not be `'static'.
pub fn engine<'a>() -> &'a Engine {
    unsafe { (*ptr::addr_of_mut!(ENGINE)).as_ref().unwrap() }
}

pub fn globals() -> &'static raw::ui_globalvars_s {
    unsafe { (*ptr::addr_of_mut!(GLOBALS)).unwrap() }
}

pub fn init(eng_funcs: &raw::ui_enginefuncs_s, globals: &'static raw::ui_globalvars_s) {
    unsafe {
        ENGINE = Some(Engine::new(*eng_funcs));
        GLOBALS = Some(globals);
    }
}

pub fn init_ext(ext: &raw::ui_extendedfuncs_s) {
    unsafe {
        (*ptr::addr_of_mut!(ENGINE))
            .as_mut()
            .unwrap()
            .set_extended(*ext);
    }
}
