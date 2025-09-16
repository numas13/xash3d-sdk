use std::cell::RefCell;

use xash3d_ref::{
    buffer::SwBuffer,
    engine::GraphicApi,
    export::{export_dll, impl_unsync_global, RefDll},
    prelude::*,
};

#[derive(Default)]
pub struct Instance {
    buffer: RefCell<SwBuffer>,
}

impl_unsync_global!(Instance);

impl RefDll for Instance {
    fn new() -> Option<Self> {
        let engine = engine();
        if !engine.r_init_video(GraphicApi::Software) {
            engine.r_free_video();
            return None;
        }
        Some(Instance::default())
    }

    fn end_frame(&self) {
        let mut buffer = self.buffer.borrow_mut();
        let (width, height) = globals().screen_size();
        let Some(mut lock) = buffer.lock(width, height) else {
            // a resolution changed or the buffer is not available
            if let Some(new_buffer) = engine().sw_create_buffer(width, height) {
                *buffer = new_buffer;
            }
            return;
        };
        // acquired the buffer lock
        lock.as_bytes_mut().fill(0xff);
    }
}

export_dll!(Instance);
