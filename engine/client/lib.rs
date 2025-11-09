#![no_std]

#[macro_use]
pub mod macros;

pub mod engine;
pub mod entity;
pub mod export;
pub mod file;
pub mod input;
pub mod instance;
mod logger;
pub mod prelude;
pub mod render;
pub mod screen;
pub mod sprite;
mod studio;
pub mod user_message;
pub mod utils;

pub use xash3d_shared::{
    cell, color, consts, csz, cvar, ffi, math, misc, model, parser, sound, str::ToEngineStr,
};

pub use crate::studio::*;
