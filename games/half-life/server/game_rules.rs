use core::ffi::CStr;

use xash3d_server::{game_rules::GameRules, global_state::GlobalStateRef, prelude::*};

pub struct HalfLifeRules {
    engine: ServerEngineRef,
}

impl HalfLifeRules {
    pub fn new(engine: ServerEngineRef) -> Self {
        engine.server_command("exec spserver.cfg\n");
        // TODO: refresh skill data
        Self { engine }
    }
}

impl GameRules for HalfLifeRules {
    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn get_game_description(&self) -> &'static CStr {
        c"Half-Life"
    }

    fn allow_flashlight(&self) -> bool {
        true
    }
}

pub fn install_game_rules(engine: ServerEngineRef, global_state: GlobalStateRef) {
    engine.server_command(c"exec game.cfg\n");
    engine.server_execute();

    if !engine.globals.is_deathmatch() {
        // TODO: g_teamplay = 0;
        global_state.set_game_rules(HalfLifeRules::new(engine));
        return;
    } else {
        // TODO:
    }
    todo!();
}
