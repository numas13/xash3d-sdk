use core::{any::Any, ffi::CStr};

use xash3d_shared::{ffi::common::vec3_t, utils::AsAny};

use crate::{
    engine::ServerEngineRef,
    entity::{Entity, EntityHandle, EntityPlayer},
    global_state::GlobalStateRef,
    time::MapTime,
};

pub trait GameRules: AsAny {
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

    fn get_player_spawn_spot(&self, player: &dyn EntityPlayer) -> EntityHandle {
        let spawn_spot = player.select_spawn_point();
        let sv = spawn_spot.vars();
        let pv = player.vars();
        pv.set_origin(sv.origin() + vec3_t::new(0.0, 0.0, 1.0));
        pv.set_view_angle(vec3_t::ZERO);
        pv.set_velocity(vec3_t::ZERO);
        pv.set_angles(sv.angles());
        pv.set_punch_angle(vec3_t::ZERO);
        pv.set_fix_angle(1);
        spawn_spot
    }

    #[allow(unused_variables)]
    fn player_spawn(&self, player: &dyn EntityPlayer) {}

    fn allow_flashlight(&self) -> bool {
        false
    }

    fn can_have_item(&self, player: &dyn EntityPlayer, item: &dyn Entity) -> bool;

    fn player_got_item(&self, player: &dyn EntityPlayer, item: &dyn Entity);

    fn item_respawn(&self, item: &dyn Entity) -> Option<(MapTime, vec3_t)>;
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

#[allow(unused_variables)]
impl GameRules for StubGameRules {
    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn get_game_description(&self) -> &'static CStr {
        c"Stub"
    }

    fn can_have_item(&self, player: &dyn EntityPlayer, item: &dyn Entity) -> bool {
        false
    }

    fn player_got_item(&self, player: &dyn EntityPlayer, item: &dyn Entity) {}

    fn item_respawn(&self, item: &dyn Entity) -> Option<(MapTime, vec3_t)> {
        None
    }
}
