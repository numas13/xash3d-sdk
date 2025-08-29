#![no_std]

#[macro_use]
extern crate log;

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

pub use shared::{cell, color, consts, cvar, math};

pub use crate::{efx::EfxApi, engine::*, event::EventApi, studio::*};

pub const CLDLL_INTERFACE_VERSION: c_int = 7;
