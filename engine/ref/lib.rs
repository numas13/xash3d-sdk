#![no_std]

#[macro_use]
extern crate log;

pub mod consts;
pub mod engine;
mod engine_types;
mod globals;
pub mod instance;
mod logger;
pub mod prelude;
pub mod raw;
pub mod utils;

pub use shared::{cell, color, cvar, math, parser, str::ToEngineStr};

pub use crate::globals::RefGlobals;

// TODO: remove me
pub use crate::engine_types::*;
