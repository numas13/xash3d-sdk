#![no_std]

#[macro_use]
pub mod macros;

pub mod engine;
pub mod entity;
pub mod export;
pub mod input;
pub mod instance;
mod logger;
pub mod message;
pub mod prelude;
pub mod render;
pub mod screen;
pub mod sprite;
mod studio;
pub mod utils;

pub use shared::{cell, color, consts, cvar, ffi, math, misc, model, sound, str::ToEngineStr};

pub use crate::studio::*;
