use shared::{export::UnsyncGlobal, ffi};

use crate::{engine::RefEngine, globals::RefGlobals};

/// Initialize the global [RefEngine] and [RefGlobals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(
    engine_funcs: &ffi::render::ref_api_t,
    globals: *mut ffi::render::ref_globals_t,
) {
    unsafe {
        (*RefEngine::global_as_mut_ptr()).write(RefEngine::new(engine_funcs));
        (*RefGlobals::global_as_mut_ptr()).write(RefGlobals::new(globals));
    }
    crate::logger::init_console_logger();
}

/// Returns a reference to the global [RefEngine] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn engine() -> &'static RefEngine {
    unsafe { RefEngine::global_assume_init_ref() }
}

/// Returns a reference to the global [RefGlobals] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn globals() -> &'static RefGlobals {
    unsafe { RefGlobals::global_assume_init_ref() }
}
