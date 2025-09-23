use core::{ffi::c_int, ptr};

use xash3d_server::{
    consts::{DAMAGE_AIM, DEAD_NO, SOLID_SLIDEBOX},
    entity::{
        AsEdict, BaseEntity, CreateEntity, EdictFlags, Effects, Entity, EntityPlayer, MoveType,
        ObjectCaps, PrivateData,
    },
    export::export_entity,
    ffi::server::edict_s,
    prelude::*,
};

use crate::{
    entity::{impl_cast, EntityTest, Private},
    export::global_state,
};

pub struct Player {
    base: BaseEntity,
}

impl_cast!(Player);

impl Player {
    pub fn set_custom_decal_frames(&mut self, frames: c_int) {
        debug!("Player::set_custom_decal_frames({frames})");
    }
}

impl CreateEntity for Player {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for Player {
    fn object_caps(&self) -> ObjectCaps {
        ObjectCaps::DONT_SAVE
    }

    fn restore(&mut self) {
        let engine = self.base.engine;

        // TODO:

        let ev = self.vars_mut().as_raw_mut();
        ev.v_angle.set_z(0.0);
        ev.angles = ev.v_angle;
        ev.fixangle = 1;

        engine.set_physics_key_value(self.as_edict_mut(), c"hl", c"1");
    }

    fn spawn(&mut self) {
        let engine = self.base.engine;
        let ev = self.vars_mut().as_raw_mut();
        ev.classname = engine.try_alloc_map_string(c"player").unwrap().index();
        ev.health = 100.0;
        ev.armorvalue = 0.0;
        ev.takedamage = DAMAGE_AIM as f32;
        ev.solid = SOLID_SLIDEBOX as c_int;
        ev.movetype = MoveType::Walk.into();
        ev.max_health = ev.health;
        ev.flags &= EdictFlags::PROXY.bits();
        ev.flags |= EdictFlags::CLIENT.bits();
        ev.air_finished = engine.globals.map_time_f32() + 12.0;
        ev.dmg = 2.0;
        ev.effects = Effects::NONE.bits();
        ev.deadflag = DEAD_NO;
        ev.dmg_take = 0.0;
        ev.dmg_save = 0.0;
        ev.friction = 1.0;
        ev.gravity = 1.0;
        ev.fov = 0.0;
        ev.view_ofs = xash3d_player_move::VEC_VIEW;

        engine.set_physics_key_value(self.as_edict_mut(), c"slj", c"0");
        engine.set_physics_key_value(self.as_edict_mut(), c"hl", c"1");

        self.game_rules().unwrap().get_player_spawn_spot(self);

        engine.set_model(self.as_edict_mut(), res::valve::models::PLAYER);
    }
}

impl EntityPlayer for Player {
    fn select_spawn_point(&self) -> *mut edict_s {
        let engine = self.base.engine;
        let game_rules = self.game_rules().expect("GameRules is not intialized.");

        if game_rules.is_coop() {
            todo!();
        } else if game_rules.is_deathmatch() {
            todo!();
        }

        let start_spot = engine.globals.start_spot();
        let mut start_spot = start_spot.as_ref().map_or(c"".into(), |s| s.as_thin());
        if start_spot.is_empty() {
            start_spot = c"info_player_start".into();
        }
        let spot = engine.find_ent_by_classname(ptr::null_mut(), start_spot);

        if !spot.is_null() {
            *global_state().last_spawn.borrow_mut() = spot;
            spot
        } else {
            error!("No info_player_start on level");
            engine.entity_of_ent_index(0)
        }
    }
}

impl EntityTest for Player {
    fn do_test_work(&self) {
        // trace!("do some test work for player entity");
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        debug!("drop Player");
    }
}

pub fn client_put_in_server(engine: ServerEngineRef, ent: &mut edict_s) {
    let player = unsafe { PrivateData::create::<Private<Player>>(engine, &mut ent.v) };

    // TODO: testing, remove later
    player.set_custom_decal_frames(-1);
    player.spawn();

    ent.v.effects_mut().insert(Effects::NOINTERP);
    ent.v.iuser1 = 0;
    ent.v.iuser2 = 0;
}

export_entity!(player, Private<Player>);
