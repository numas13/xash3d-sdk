use core::{any::Any, ffi::CStr};

use xash3d_shared::ffi::{common::vec3_t, server::edict_s};

use crate::{
    engine::ServerEngineRef,
    entity::{Entity, EntityPlayer},
    global_state::GlobalStateRef,
};

pub trait GameRules: Any {
    fn as_any(&self) -> &dyn Any;

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
        let pv = player.vars_mut();
        pv.set_origin(sev.origin + vec3_t::new(0.0, 0.0, 1.0));
        pv.set_view_angle(vec3_t::ZERO);
        pv.set_velocity(vec3_t::ZERO);
        pv.set_angles(sev.angles);
        pv.set_punch_angle(vec3_t::ZERO);
        pv.set_fix_angle(1);
        spawn_spot
    }

    #[allow(unused_variables)]
    fn player_spawn(&self, player: &mut dyn EntityPlayer) {}

    fn allow_flashlight(&self) -> bool {
        false
    }
}

impl dyn GameRules {
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.as_any().downcast_ref::<T>()
    }
}

pub struct StubGameRules {
    engine: ServerEngineRef,
}

impl StubGameRules {
    pub fn install(engine: ServerEngineRef, global_state: GlobalStateRef) {
        global_state.set_game_rules(Self::new(engine));
    }

    pub fn new(engine: ServerEngineRef) -> Self {
        Self { engine }
    }
}

impl GameRules for StubGameRules {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn get_game_description(&self) -> &'static CStr {
        c"Stub"
    }
}
