#![no_std]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

pub struct System {}

impl System {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(windows))]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        use core::ptr;
        let size = layout.size();
        let align = layout.align();
        if align < core::mem::size_of::<*const u8>() {
            unsafe { libc::malloc(size).cast() }
        } else {
            let mut ret = ptr::null_mut();
            if unsafe { libc::posix_memalign(&mut ret, align, size) } == 0 {
                ret.cast()
            } else {
                ptr::null_mut()
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { libc::free(ptr.cast()) }
    }
}

#[cfg(windows)]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        match layout.align() {
            1 | 2 => unsafe { libc::malloc(size).cast() },
            align => unsafe { libc::aligned_malloc(size, align).cast() },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match layout.align() {
            1 | 2 => unsafe { libc::free(ptr.cast()) },
            _ => unsafe { libc::aligned_free(ptr.cast()) },
        }
    }
}
