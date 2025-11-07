#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
extern crate log;

pub mod consts;
pub mod engine;
pub mod export;
pub mod file;
pub mod game_info;
pub mod globals;
pub mod instance;
mod logger;
pub mod picture;
pub mod prelude;
pub mod utils;

pub use xash3d_shared::{
    cell, color, csz, cvar, entity, ffi, math, misc, parser, render, str::ToEngineStr,
};
