#![no_std]

#[macro_use]
pub mod macros;
pub mod engine;
mod engine_types;
mod globals;
pub mod instance;
mod logger;
pub mod prelude;
pub mod raw;
pub mod utils;

pub use shared::{cell, color, consts, cvar, math, str::ToEngineStr};

pub use crate::globals::ServerGlobals;

// TODO: remove me
pub use crate::engine_types::*;
