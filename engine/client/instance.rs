use core::ffi::c_int;

use shared::export::UnsyncGlobal;

use crate::{
    engine::prelude::*,
    engine::{event::EVENT_API_VERSION, tri::TRI_API_VERSION, ClientEngine, ClientEngineFunctions},
    raw,
    studio::Studio,
};

fn check_version(engine: &ClientEngine, name: &str, version: c_int, expected: c_int) -> bool {
    if version == expected {
        return true;
    }
    let msg = format_args!("^1Error:^7 {name} version is {version} (expected {expected})\n");
    engine.console_print(msg);
    false
}

/// Initialize the global [ClientEngine] instance.
///
/// Returns `true` if the initialization was successful.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(engine_funcs: &ClientEngineFunctions) -> bool {
    let engine = ClientEngine::new(engine_funcs);
    let mut ok = true;

    let tri_version = engine.tri_api().version();
    ok &= check_version(&engine, "TriangleAPI", tri_version, TRI_API_VERSION);

    let event_version = engine.event_api().version();
    ok &= check_version(&engine, "EventAPI", event_version, EVENT_API_VERSION);

    if !ok {
        return false;
    }

    unsafe {
        (*ClientEngine::global_as_mut_ptr()).write(engine);
    }
    crate::logger::init_console_logger();
    true
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
