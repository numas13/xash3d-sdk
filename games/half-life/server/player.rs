use core::{ffi::c_int, ptr};

use shared::{
    consts::{DAMAGE_AIM, DEAD_NO, SOLID_SLIDEBOX},
    raw::{edict_s, entvars_s, EdictFlags, Effects, MoveType},
};
use sv::{engine, globals};

use crate::{
    entity::{impl_cast, Animating, Delay, Entity, EntityVars, Monster, ObjectCaps, Toggle},
    gamerules::game_rules,
    global_state::global_state,
    macros::link_entity,
    private_data::Private,
    save::{self, SaveRestore},
};

pub struct Player {
    vars: *mut entvars_s,
}

impl_cast!(Player);

impl Player {
    pub fn new(vars: *mut entvars_s) -> Self {
        Self { vars }
    }

    pub fn set_custom_decal_frames(&mut self, frames: c_int) {
        debug!("Player::set_custom_decal_frames({frames})");
    }

    pub fn ent_select_spawn_point(&self) -> *mut edict_s {
        let game_rules = game_rules().unwrap();

        if game_rules.is_coop() {
            todo!();
        } else if game_rules.is_deathmatch() {
            todo!();
        }

        let mut startspot = c"info_player_start";
        if !globals().startspot.is_null() && !globals().string(globals().startspot).is_empty() {
            startspot = globals().string(globals().startspot);
        }
        let spot = engine().find_ent_by_classname(ptr::null(), startspot);

        if !spot.is_null() {
            *global_state().last_spawn.borrow_mut() = spot;
            spot
        } else {
            error!("No info_player_start on level");
            engine().entity_of_ent_index(0)
        }
    }
}

impl EntityVars for Player {
    fn vars_ptr(&self) -> *mut entvars_s {
        self.vars
    }
}

impl Entity for Player {
    fn object_caps(&self) -> ObjectCaps {
        ObjectCaps::DONT_SAVE
    }

    fn spawn(&mut self) -> bool {
        let ev = self.vars_mut();
        ev.classname = globals().make_string(c"player");
        ev.health = 100.0;
        ev.armorvalue = 0.0;
        ev.takedamage = DAMAGE_AIM as f32;
        ev.solid = SOLID_SLIDEBOX as c_int;
        ev.movetype = MoveType::Walk;
        ev.max_health = ev.health;
        ev.flags &= EdictFlags::PROXY;
        ev.flags |= EdictFlags::CLIENT;
        ev.air_finished = globals().time + 12.0;
        ev.dmg = 2.0;
        ev.effects = Effects::NONE;
        ev.deadflag = DEAD_NO;
        ev.dmg_take = 0.0;
        ev.dmg_save = 0.0;
        ev.friction = 1.0;
        ev.gravity = 1.0;
        ev.fov = 0.0;
        ev.view_ofs = pm::VEC_VIEW;

        let engine = engine();
        engine.set_physics_key_value(self.ent_mut(), c"slj", c"0");
        engine.set_physics_key_value(self.ent_mut(), c"hl", c"1");

        game_rules().unwrap().get_player_spawn_spot(self);

        engine.set_model(self.ent_mut(), res::valve::models::PLAYER);

        true
    }

    fn restore(&mut self, restore: &mut SaveRestore) -> save::Result<()> {
        // TODO: call restore from base "classes"

        restore.read_ent_vars(c"ENTVARS", self.vars_mut())?;

        let fields = self.fields();
        restore.read_fields(c"BASE", self as *mut _ as *mut _, fields)?;

        let ev = self.vars_mut();
        if ev.modelindex != 0 && !ev.model.is_null() {
            let mins = ev.mins;
            let maxs = ev.maxs;
            let engine = engine();
            let model = globals().string(ev.model);
            engine.precache_model(model);
            engine.set_model(self.ent_mut(), model);
            engine.set_size(self.ent_mut(), mins, maxs);
        }

        // TODO:

        let ev = self.vars_mut();
        ev.v_angle.set_z(0.0);
        ev.fixangle = 1;

        let engine = engine();
        engine.set_physics_key_value(self.ent_mut(), c"hl", c"1");

        Ok(())
    }
}

impl Delay for Player {}
impl Animating for Player {}
impl Toggle for Player {}
impl Monster for Player {}

impl Drop for Player {
    fn drop(&mut self) {
        debug!("drop Player");
    }
}

pub fn client_put_in_server(ent: &mut edict_s) {
    let player = ent.private_init(Player::new);

    // TODO: testing, remove later
    let player: &mut dyn Entity = player;
    assert!(player.ent().downcast::<crate::world::World>().is_none());
    let player = player.ent_mut().downcast_mut::<Player>().unwrap();

    player.set_custom_decal_frames(-1);
    player.spawn();

    ent.v.effects.insert(Effects::NOINTERP);
    ent.v.iuser1 = 0;
    ent.v.iuser2 = 0;
}

link_entity!(player, Player::new);
