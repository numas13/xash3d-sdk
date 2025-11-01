#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod func_train;
pub mod path_corner;

#[doc(hidden)]
pub use xash3d_server::export::export_entity;
