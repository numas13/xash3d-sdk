use csz::CStrThin;
use shared::logger::EngineConsole;

use crate::engine;

struct Console;

impl EngineConsole for Console {
    fn get_cvar_float(name: &CStrThin) -> f32 {
        engine().cvar_get_float(name)
    }

    fn console_print(s: &CStrThin) {
        engine().server_print(s);
    }
}

pub fn init_console_logger() {
    shared::logger::init_console_logger::<Console>();
}
