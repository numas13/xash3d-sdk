use shared::{export::UnsyncGlobal, ffi};

use crate::engine::RefEngine;

/// Initialize the global [RefEngine] and [crate::globals::RefGlobals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(
    engine_funcs: &ffi::render::ref_api_t,
    globals: *mut ffi::render::ref_globals_t,
) {
    let engine = RefEngine::new(engine_funcs, globals);
    unsafe {
        (*RefEngine::global_as_mut_ptr()).write(engine);
    }
    crate::logger::init_console_logger();
}
