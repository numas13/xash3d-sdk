use csz::CStrThin;
use shared::logger::EngineConsoleLogger;

use crate::prelude::*;

struct Console;

impl EngineConsoleLogger for Console {
    fn get_cvar_float(name: &CStrThin) -> f32 {
        // TODO: remove me
        let engine = unsafe { ClientEngineRef::new() };
        engine.get_cvar_float(name)
    }

    fn console_print(s: &CStrThin) {
        // TODO: remove me
        let engine = unsafe { ClientEngineRef::new() };
        engine.console_print(s);
    }
}

pub fn init_console_logger() {
    shared::logger::init_console_logger::<Console>();
}
