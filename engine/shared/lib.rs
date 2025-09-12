#![no_std]

extern crate alloc;

#[cfg(any(test, feature = "std"))]
#[macro_use]
extern crate std;

#[macro_use]
extern crate log;

#[macro_use]
pub mod macros;

pub mod borrow;
pub mod cell;
pub mod color;
pub mod consts;
pub mod cvar;
pub mod engine;
pub mod engine_private;
pub mod entity;
pub mod export;
pub mod logger;
pub mod math;
pub mod message;
pub mod parser;
pub mod prelude;
pub mod raw;
pub mod render;
pub mod str;
pub mod utils;

pub use xash3d_ffi as ffi;
