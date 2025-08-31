use csz::CStrThin;
use shared::logger::EngineConsole;

use crate::prelude::*;

struct Console;

impl EngineConsole for Console {
    fn get_cvar_float(name: &CStrThin) -> f32 {
        engine().get_cvar(name)
    }

    fn console_print(s: &CStrThin) {
        engine().con_print(s);
    }
}

pub fn init_console_logger() {
    shared::logger::init_console_logger::<Console>();
}
