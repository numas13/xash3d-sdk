use crate::{
    engine::{UiEngine, UiEngineFunctions, UiEngineFunctionsExtended},
    export::UnsyncGlobal,
    globals::{UiGlobals, UiGlobalsRaw},
};

/// Initialize the global [UiEngine] and [UiGlobals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(engine_funcs: &UiEngineFunctions, globals: *mut UiGlobalsRaw) {
    unsafe {
        (*UiEngine::global_as_mut_ptr()).write(UiEngine::new(engine_funcs));
        (*UiGlobals::global_as_mut_ptr()).write(UiGlobals::new(globals));
    }
    crate::logger::init_console_logger();
}

/// Initialize extended functions for global [UiEngine] instance.
///
/// # Safety
///
/// Must be called only once after [init_engine].
pub unsafe fn init_engine_ext(ext: &UiEngineFunctionsExtended) {
    unsafe {
        (*UiEngine::global_as_mut_ptr())
            .assume_init_mut()
            .set_extended(*ext);
    }
}

/// Returns a reference to the global [UiEngine] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn engine() -> &'static UiEngine {
    unsafe { UiEngine::global_assume_init_ref() }
}

/// Returns a reference to the global [UiGlobals] instance.
///
/// # Safety
///
/// Must not be called before [init_engine].
pub fn globals() -> &'static UiGlobals {
    unsafe { UiGlobals::global_assume_init_ref() }
}
