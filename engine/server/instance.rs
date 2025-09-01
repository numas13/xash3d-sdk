use shared::export::UnsyncGlobal;

use crate::{engine::ServerEngine, globals::ServerGlobals, raw};

/// Initialize the global [ServerEngine] and [ServerGlobals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(funcs: &raw::enginefuncs_s, globals: *mut raw::globalvars_t) {
    unsafe {
        (*ServerEngine::global_as_mut_ptr()).write(ServerEngine::new(funcs));
        (*ServerGlobals::global_as_mut_ptr()).write(ServerGlobals::new(globals));
    }
    crate::logger::init_console_logger();
}

/// Returns a reference to the global [ServerEngine] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn engine() -> &'static ServerEngine {
    unsafe { ServerEngine::global_assume_init_ref() }
}

/// Returns a reference to the global [ServerGlobals] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn globals() -> &'static ServerGlobals {
    unsafe { ServerGlobals::global_assume_init_ref() }
}
