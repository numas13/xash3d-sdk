use xash3d_shared::{
    csz::CStrThin,
    logger::{self, EngineConsoleLogger},
};

use crate::prelude::*;

struct Console;

impl EngineConsoleLogger for Console {
    unsafe fn console_print(s: &CStrThin) {
        let engine = unsafe { RefEngineRef::new() };
        engine.console_print(s);
    }
}

pub unsafe fn init_console_logger(engine: &RefEngine) {
    let developer = engine.get_cvar_float(c"developer");
    unsafe {
        logger::init_console_logger::<Console>(developer, None);
    }
}
