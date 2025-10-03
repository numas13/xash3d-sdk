use core::ffi::c_int;

use xash3d_server::{
    entities::{player::Player, world::World},
    entity::Private,
    export::{export_dll, export_entity, impl_unsync_global, ServerDll},
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
    type Player = Private<Player>;

    fn new(engine: ServerEngineRef, global_state: GlobalStateRef) -> Self {
        Self {
            engine,
            global_state,
        }
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }
}

export_entity!(worldspawn, Private<World>, |base| World::create(
    base,
    StubGameRules::install
));
export_dll!(Dll);
