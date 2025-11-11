#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(any(test, feature = "std"))]
#[allow(unused_imports)]
#[macro_use]
extern crate std;

#[macro_use]
extern crate log;

#[macro_use]
pub mod macros;

pub mod borrow;
pub mod bsp;
pub mod cell;
pub mod color;
pub mod consts;
pub mod cvar;
pub mod engine;
pub mod entity;
pub mod export;
pub mod file;
pub mod global_state;
pub mod input;
pub mod logger;
pub mod math;
pub mod misc;
pub mod model;
pub mod parser;
pub mod prelude;
pub mod render;
pub mod sound;
pub mod str;
pub mod user_message;
pub mod utils;

pub use csz;
pub use xash3d_ffi as ffi;
