use core::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
};

#[cfg(not(windows))]
use core::{ffi::c_int, ptr};

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);

    #[cfg(not(windows))]
    fn posix_memalign(memptr: *mut *mut c_void, alignment: usize, size: usize) -> c_int;
}

#[cfg(windows)]
extern "C" {
    fn _aligned_malloc(size: usize, alignment: usize) -> *mut c_void;
    fn _aligned_free(memblock: *mut c_void);
}

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
        let size = layout.size();
        match layout.align() {
            1 | 2 => unsafe { malloc(size).cast() },
            align => {
                let mut ret = ptr::null_mut();
                if unsafe { posix_memalign(&mut ret, align, size) } == 0 {
                    ret.cast()
                } else {
                    ptr::null_mut()
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { free(ptr.cast()) }
    }
}

#[cfg(windows)]
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        match layout.align() {
            1 | 2 => unsafe { malloc(size).cast() },
            align => unsafe { _aligned_malloc(size, align).cast() },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match layout.align() {
            1 | 2 => unsafe { free(ptr.cast()) },
            _ => unsafe { _aligned_free(ptr.cast()) },
        }
    }
}
