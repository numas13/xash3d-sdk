use shared::ffi;

use crate::{engine::UiEngine, export::UnsyncGlobal};

/// Initialize the global [UiEngine] and [crate::globals::UiGlobals] instances.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(
    engine_funcs: &ffi::menu::ui_enginefuncs_s,
    globals: *mut ffi::menu::ui_globalvars_s,
) {
    unsafe {
        (*UiEngine::global_as_mut_ptr()).write(UiEngine::new(engine_funcs, globals));
    }
    crate::logger::init_console_logger();
}

/// Initialize extended functions for global [UiEngine] instance.
///
/// # Safety
///
/// Must be called only once after [init_engine].
pub unsafe fn init_engine_ext(ext: &ffi::menu::ui_extendedfuncs_s) {
    unsafe {
        (*UiEngine::global_as_mut_ptr())
            .assume_init_mut()
            .set_extended(*ext);
    }
}
