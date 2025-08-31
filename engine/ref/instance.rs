use shared::export::UnsyncGlobal;

use crate::{engine::RefEngine, globals::RefGlobals, raw};

/// Initialize the global [RefEngine] and [RefGlobals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(engine_funcs: &raw::ref_api_s, globals: *mut raw::ref_globals_s) {
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

/// Returns a mutable reference to the global [RefGlobals] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn globals_mut() -> &'static mut RefGlobals {
    unsafe { RefGlobals::global_assume_init_mut() }
}
