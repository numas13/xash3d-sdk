#![no_std]

#[macro_use]
extern crate log;

mod base_button;

pub mod func_button;
pub mod func_rot_button;

#[doc(hidden)]
pub use xash3d_server::export::export_entity;
