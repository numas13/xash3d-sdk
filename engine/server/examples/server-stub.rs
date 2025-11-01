use core::ffi::{c_int, CStr};

use xash3d_entities::{player::Player, world::World};
use xash3d_server::{
    entity::BaseEntity,
    export::{export_dll, impl_unsync_global, ServerDll},
    game_rules::StubGameRules,
    global_state::GlobalStateRef,
    prelude::*,
};

struct Dll {
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
}

impl_unsync_global!(Dll);

impl ServerDll for Dll {
    type World = World;
    type Player = Player;

    fn new(engine: ServerEngineRef, global_state: GlobalStateRef) -> Self {
        Self {
            engine,
            global_state,
        }
    }

    fn create_world(base: BaseEntity) -> World {
        World::create(base, StubGameRules::install)
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }

    fn get_game_description_static() -> &'static CStr {
        c"ServerStub"
    }
}

export_dll!(Dll);
