#![cfg_attr(not(feature = "std"), no_std)]

//#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

mod camera;
mod entity;
mod events;
mod export;
mod helpers;
mod hud;
mod input;
mod studio;
mod view;
mod weapons;

#[cfg(not(feature = "std"))]
#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: xash3d_allocator::System = xash3d_allocator::System::new();

#[cfg(not(feature = "std"))]
#[cfg(panic = "abort")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{info}");
    unsafe { libc::abort() }
}

#[cfg(not(feature = "std"))]
#[cfg(panic = "abort")]
#[no_mangle]
fn rust_eh_personality() {}
