use core::{
    alloc::{GlobalAlloc, Layout},
    ffi::{c_int, c_void},
    ptr,
};

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn posix_memalign(memptr: *mut *mut c_void, alignment: usize, size: usize) -> c_int;
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
