use shared::export::UnsyncGlobal;

use crate::{engine::ClientEngine, raw, studio::Studio};

/// Initialize the global [ClientEngine] instance.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(engine_funcs: &raw::cl_enginefuncs_s) {
    unsafe {
        (*ClientEngine::global_as_mut_ptr()).write(ClientEngine::new(engine_funcs));
    }
    crate::logger::init_console_logger();
}

/// Returns a reference to the global [ClientEngine] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn engine() -> &'static ClientEngine {
    unsafe { ClientEngine::global_assume_init_ref() }
}

/// Initialize the global [Studio] instance.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_studio(funcs: &raw::engine_studio_api_s) {
    unsafe {
        (*Studio::global_as_mut_ptr()).write(Studio::new(funcs));
    }
}

/// Returns a reference to the global [Studio] instance.
///
/// # Safety
///
/// Must not be called before [init_studio].
pub fn studio<'a>() -> &'a Studio {
    unsafe { Studio::global_assume_init_ref() }
}
