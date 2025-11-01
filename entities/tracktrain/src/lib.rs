#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod func_tracktrain;
pub mod path_track;

#[doc(hidden)]
pub use xash3d_server::export::export_entity;
