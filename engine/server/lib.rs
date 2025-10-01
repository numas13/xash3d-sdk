#![no_std]

//#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

#[macro_use]
pub mod macros;

pub mod consts;
pub mod engine;
pub mod entity;
pub mod export;
pub mod game_rules;
pub mod global_state;
pub mod globals;
pub mod instance;
mod logger;
pub mod prelude;
pub mod save;
pub mod sound;
pub mod str;
pub mod svc;
pub mod utils;

pub use xash3d_shared::{cell, color, cvar, ffi, math, render};

// HACK: used by delegate macros to access xash3d_server types
#[doc(hidden)]
pub use crate as xash3d_server;
