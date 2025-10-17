use core::ffi::c_int;

use xash3d_shared::{cvar::CVarPtr, export::UnsyncGlobal, ffi};

use crate::{prelude::*, studio::Studio};

fn check_version(engine: &ClientEngine, name: &str, version: c_int, expected: c_int) -> bool {
    if version == expected {
        return true;
    }
    engine.console_print(format_args!(
        "^1Error:^7 {name} version is {version} (expected {expected})\n"
    ));
    false
}

/// Initialize the global [ClientEngine] instance.
///
/// Returns `true` if the initialization was successful.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_engine(engine_funcs: &ffi::client::cl_enginefuncs_s) -> bool {
    let engine = ClientEngine::new(engine_funcs);
    let mut ok = true;

    let tri_version = engine.tri_api().version();
    ok &= check_version(
        &engine,
        "TriangleAPI",
        tri_version,
        ffi::api::tri::TRI_API_VERSION,
    );

    let event_version = engine.event_api().version();
    ok &= check_version(
        &engine,
        "EventAPI",
        event_version,
        ffi::api::event::EVENT_API_VERSION,
    );

    if !ok {
        return false;
    }

    unsafe {
        (*ClientEngine::global_as_mut_ptr()).write(engine);
    }

    crate::cvar::init(|name, value, flags| {
        // TODO: remove me
        let engine = unsafe { ClientEngineRef::new() };
        let ptr = engine.get_cvar(name);
        if ptr.is_null() {
            engine
                .register_variable(name, value, flags)
                .unwrap_or(CVarPtr::null())
        } else {
            ptr
        }
    });

    true
}

/// Initialize the global [Studio] instance.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn init_studio(funcs: &ffi::api::studio::engine_studio_api_s) {
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
