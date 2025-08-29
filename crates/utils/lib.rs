#![no_std]

extern crate alloc;

#[macro_use]
pub mod macros;

pub mod allocator;
pub mod logger;
pub mod str;

pub fn array_from_slice<T: Copy + Default, const N: usize>(slice: &[T]) -> [T; N] {
    let mut arr = [T::default(); N];
    arr.copy_from_slice(&slice[..N]);
    arr
}

pub fn abort() -> ! {
    unsafe { libc::abort() }
}
