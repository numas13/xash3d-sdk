use core::ffi::{c_int, CStr};

use xash3d_server::{
    entities::{player::Player, world::World},
    entity::{BaseEntity, EntityHandle},
    export::{export_dll, impl_unsync_global, ServerDll},
    game_rules::StubGameRules,
    global_state::GlobalStateRef,
    prelude::*,
    private::Downcast,
};

// A custom interface to entities.
trait EntityCustom: Entity {
    fn custom(&self);
}

// A custom wrapper for private data.
struct CustomPrivate<T>(core::marker::PhantomData<T>);

impl<T: Entity + EntityCustom> PrivateEntity for CustomPrivate<T> {
    type Entity = T;

    fn downcast(t: &Downcast<'_, Self::Entity>) -> bool {
        // cast an entity to the custom interface
        t.downcast(|i| Some(i))
    }
}

// Implement EntityCustom for the Player type
impl EntityCustom for Player {
    fn custom(&self) {
        log::warn!("Player: custom method");
    }
}

struct Dll {
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
}

impl_unsync_global!(Dll);

impl ServerDll for Dll {
    type World = World;
    type Player = CustomPrivate<Player>;

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

    fn dispatch_touch(&self, touched: EntityHandle, _other: EntityHandle) {
        // call the custom method for player on touch
        if let Some(touched) = touched.get_entity() {
            if let Some(touched) = touched.downcast_ref::<dyn EntityCustom>() {
                touched.custom();
            }
        }
    }
}

export_dll!(Dll);
