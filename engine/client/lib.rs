#![no_std]

#[macro_use]
extern crate log;

#[macro_use]
pub mod macros;
mod efx;
mod engine;
mod event;
pub mod input;
pub mod message;
pub mod raw;
pub mod sprite;
mod studio;

use core::ffi::c_int;

pub use shared::{cell, color, consts, cvar, math};
pub use utils;

pub use crate::{efx::EfxApi, engine::*, event::EventApi, studio::*};

pub const CLDLL_INTERFACE_VERSION: c_int = 7;
