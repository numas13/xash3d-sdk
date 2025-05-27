use core::ffi::CStr;

use sv::{
    engine,
    raw::{entvars_s, KeyValueData},
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
        engine.cvar_set_string(c"sv_gravity", c"800");
        engine.cvar_set_string(c"sv_stepsize", c"18");
        engine.cvar_set_string(c"room_type", c"0");
        install_game_rules();
    }

    fn key_value(&mut self, data: &mut KeyValueData) {
        let class_name = unsafe { CStr::from_ptr(data.szClassName) };
        let key_name = unsafe { CStr::from_ptr(data.szKeyName) };
        let value = unsafe { CStr::from_ptr(data.szValue) };
        let handled = data.fHandled;
        debug!("World::key_value({class_name:?}, {key_name:?}, {value:?}, {handled})");
        data.fHandled = 1;
    }
}

link_entity!(worldspawn, World::new);
