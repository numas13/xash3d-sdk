#![no_std]

#[macro_use]
pub mod macros;
mod efx;
pub mod engine;
mod event;
pub mod input;
pub mod instance;
mod logger;
pub mod message;
pub mod prelude;
pub mod raw;
pub mod sprite;
mod studio;
pub mod utils;

use core::ffi::c_int;

pub use shared::{cell, color, consts, cvar, math, str::ToEngineStr};

pub use crate::{efx::EfxApi, event::EventApi, studio::*};

pub const CLDLL_INTERFACE_VERSION: c_int = 7;
