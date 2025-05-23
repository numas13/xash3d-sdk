#![no_std]

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

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: utils::allocator::System = utils::allocator::System::new();

#[cfg(panic = "abort")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{info}");
    utils::abort();
}

#[cfg(panic = "abort")]
#[no_mangle]
fn rust_eh_personality() {}
