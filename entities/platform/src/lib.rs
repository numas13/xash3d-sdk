#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod func_plat;
pub mod func_platrot;

#[doc(hidden)]
pub use xash3d_server::export::export_entity;
