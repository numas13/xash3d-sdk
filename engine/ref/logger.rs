use csz::CStrThin;
use xash3d_shared::logger::EngineConsoleLogger;

use crate::prelude::*;

struct Console;

impl EngineConsoleLogger for Console {
    fn get_cvar_float(name: &CStrThin) -> f32 {
        // TODO: remove me
        let engine = unsafe { RefEngineRef::new() };
        engine.get_cvar_float(name)
    }

    fn console_print(s: &CStrThin) {
        // TODO: remove me
        let engine = unsafe { RefEngineRef::new() };
        engine.console_print(s);
    }
}

pub fn init_console_logger() {
    xash3d_shared::logger::init_console_logger::<Console>();
}
