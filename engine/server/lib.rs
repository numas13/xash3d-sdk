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
pub mod globals;
pub mod instance;
mod logger;
pub mod prelude;
pub mod save;
pub mod str;
pub mod utils;

pub use shared::{cell, color, cvar, ffi, math, str::ToEngineStr};
