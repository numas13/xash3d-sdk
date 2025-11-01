#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod base_door;

pub mod func_door;
pub mod func_door_rotating;

#[doc(hidden)]
pub use xash3d_server::export::export_entity;
