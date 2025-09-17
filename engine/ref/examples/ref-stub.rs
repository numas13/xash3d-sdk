use std::cell::RefCell;

use xash3d_ref::{
    buffer::SwBuffer,
    engine::{GraphicApi, RefEngineRef},
    export::{export_dll, impl_unsync_global, RefDll},
};

pub struct Instance {
    engine: RefEngineRef,
    buffer: RefCell<SwBuffer>,
}

impl_unsync_global!(Instance);

impl RefDll for Instance {
    fn new(engine: RefEngineRef) -> Option<Self> {
        if !engine.r_init_video(GraphicApi::Software) {
            engine.r_free_video();
            return None;
        }
        Some(Instance {
            engine,
            buffer: SwBuffer::new(engine).into(),
        })
    }

    fn end_frame(&self) {
        let mut buffer = self.buffer.borrow_mut();
        let (width, height) = self.engine.globals.screen_size();
        let Some(mut lock) = buffer.lock(width, height) else {
            // a resolution changed or the buffer is not available
            if let Some(new_buffer) = self.engine.sw_create_buffer(width, height) {
                *buffer = new_buffer;
            }
            return;
        };
        // acquired the buffer lock
        lock.as_bytes_mut().fill(0xff);
    }
}

export_dll!(Instance);
