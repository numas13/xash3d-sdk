use xash3d_server::{
    entity::{BaseEntity, CreateEntity, Entity},
    export::export_entity,
    ffi::server::KeyValueData,
    prelude::*,
};

use crate::{
    entity::{impl_cast, Private},
    game_rules::install_game_rules,
};

pub struct World {
    base: BaseEntity,
}

impl_cast!(World);

impl CreateEntity for World {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for World {
    fn key_value(&mut self, data: &mut KeyValueData) {
        let class_name = data.class_name();
        let key_name = data.key_name();
        let value = data.value();
        let handled = data.handled();
        debug!("World::key_value({class_name:?}, {key_name}, {value}, {handled})");
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.base.engine;
        engine.set_cvar(c"sv_gravity", c"800");
        engine.set_cvar(c"sv_stepsize", c"18");
        engine.set_cvar(c"room_type", c"0");
        install_game_rules(engine);
    }

    fn spawn(&mut self) {
        // TODO: global_game_over = false;
        self.precache();
    }
}

export_entity!(worldspawn, Private<World>);
