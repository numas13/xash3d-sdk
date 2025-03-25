#![no_std]

//#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

mod macros;

mod cvar;
mod entity;
mod export;
mod gamerules;
mod global_state;
mod player;
mod private_data;
mod save;
mod todo;
mod triggers;
mod world;

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
