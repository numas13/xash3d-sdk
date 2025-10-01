use core::{any::Any, ffi::CStr};

use xash3d_shared::ffi::{common::vec3_t, server::edict_s};

use crate::{
    engine::ServerEngineRef,
    entity::{Entity, EntityPlayer},
    global_state::GlobalStateRef,
};

pub trait InstallGameRules: Sized + 'static {
    fn install_game_rules(engine: ServerEngineRef, global_state: GlobalStateRef);
}

pub trait GameRules: Any {
    fn engine(&self) -> ServerEngineRef;

    fn is_multiplayer(&self) -> bool {
        false
    }

    fn is_deathmatch(&self) -> bool {
        false
    }

    fn is_teamplay(&self) -> bool {
        false
    }

    fn is_coop(&self) -> bool {
        false
    }

    fn get_game_description(&self) -> &'static CStr;

    #[allow(unused_variables)]
    fn is_allowed_to_spawn(&self, entity: &dyn Entity) -> bool {
        true
    }

    fn get_player_spawn_spot(&self, player: &mut dyn EntityPlayer) -> *mut edict_s {
        let spawn_spot = player.select_spawn_point();
        let sev = unsafe { &(*spawn_spot).v };
        let pev = player.vars_mut().as_raw_mut();
        pev.origin = sev.origin + vec3_t::new(0.0, 0.0, 1.0);
        pev.v_angle = vec3_t::ZERO;
        pev.velocity = vec3_t::ZERO;
        pev.angles = sev.angles;
        pev.punchangle = vec3_t::ZERO;
        pev.fixangle = 1;
        spawn_spot
    }

    #[allow(unused_variables)]
    fn player_spawn(&self, player: &mut dyn EntityPlayer) {}
}

pub struct StubGameRules {
    engine: ServerEngineRef,
}

impl StubGameRules {
    pub fn new(engine: ServerEngineRef) -> Self {
        Self { engine }
    }
}

impl GameRules for StubGameRules {
    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn get_game_description(&self) -> &'static CStr {
        c"Stub"
    }
}

pub struct InstallStubGameRules;

impl InstallGameRules for InstallStubGameRules {
    fn install_game_rules(engine: ServerEngineRef, global_state: GlobalStateRef) {
        global_state.set_game_rules(StubGameRules::new(engine));
    }
}
