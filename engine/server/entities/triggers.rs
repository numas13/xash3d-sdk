use xash3d_shared::{
    entity::{Effects, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    entity::{Entity, EntityVars, Solid},
    prelude::*,
};

pub fn init_trigger(engine: &ServerEngine, v: &EntityVars) {
    if v.angles() != vec3_t::ZERO {
        v.set_move_dir_from_angles();
    }
    v.set_solid(Solid::Trigger);
    v.set_move_type(MoveType::None);
    v.reload_model();
    if !engine.get_cvar::<bool>(c"showtriggers") {
        v.with_effects(|f| f | Effects::NODRAW);
    }
}

pub fn toggle_use(ent: &impl Entity) {
    let engine = ent.engine();
    let v = ent.vars();
    match v.solid() {
        Solid::Not => {
            v.set_solid(Solid::Trigger);
            engine.globals.force_retouch();
        }
        _ => {
            v.set_solid(Solid::Not);
        }
    }
    v.link();
}
