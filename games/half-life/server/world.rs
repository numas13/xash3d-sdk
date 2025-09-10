use sv::{
    ffi::server::{entvars_s, KeyValueData},
    prelude::*,
};

use crate::{
    entity::{impl_cast, Entity, EntityVars},
    gamerules::install_game_rules,
    macros::link_entity,
};

pub struct World {
    vars: *mut entvars_s,
}

impl World {
    fn new(vars: *mut entvars_s) -> Self {
        Self { vars }
    }
}

impl_cast!(World);

impl EntityVars for World {
    fn vars_ptr(&self) -> *mut entvars_s {
        self.vars
    }
}

impl Entity for World {
    fn spawn(&mut self) -> bool {
        // TODO: global_game_over = false;
        self.precache();
        true
    }

    fn precache(&mut self) {
        let engine = engine();
        engine.set_cvar(c"sv_gravity", c"800");
        engine.set_cvar(c"sv_stepsize", c"18");
        engine.set_cvar(c"room_type", c"0");
        install_game_rules();
    }

    fn key_value(&mut self, data: &mut KeyValueData) {
        let class_name = data.class_name();
        let key_name = data.key_name();
        let value = data.value();
        let handled = data.handled();
        debug!("World::key_value({class_name:?}, {key_name}, {value}, {handled})");
        data.set_handled(true);
    }
}

link_entity!(worldspawn, World::new);
