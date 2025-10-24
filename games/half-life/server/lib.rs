#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod cvar;
mod entities;
mod export;
mod game_rules;
mod sound;

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
