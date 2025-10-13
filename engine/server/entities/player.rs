use core::{ffi::c_int, ptr};

use xash3d_shared::{
    consts::{DAMAGE_AIM, DEAD_NO, SOLID_SLIDEBOX},
    entity::{Buttons, EdictFlags, Effects, EntityIndex, MoveType},
    ffi::server::edict_s,
};

use crate::entity::{
    delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityPlayer, EntityVars,
    ObjectCaps,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Input {
    buttons_last: Buttons,
    buttons_pressed: Buttons,
    buttons_released: Buttons,
}

impl Input {
    pub fn is_changed(&self, buttons: Buttons) -> bool {
        self.buttons_pressed
            .union(self.buttons_released)
            .contains(buttons)
    }

    pub fn is_pressed(&self, buttons: Buttons) -> bool {
        self.buttons_pressed.contains(buttons)
    }

    pub fn is_released(&self, buttons: Buttons) -> bool {
        self.buttons_released.contains(buttons)
    }

    pub fn pre_think(&mut self, vars: &EntityVars) {
        let buttons = vars.buttons();
        let buttons_changed = self.buttons_last.symmetric_difference(buttons);
        self.buttons_pressed = buttons_changed.intersection(buttons);
        self.buttons_released = buttons_changed.difference(buttons);
    }

    pub fn post_think(&mut self, vars: &EntityVars) {
        self.buttons_last = vars.buttons();
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Player {
    base: BaseEntity,

    pub input: Input,
}

impl_entity_cast!(Player);

#[cfg(feature = "save")]
impl crate::save::OnRestore for Player {
    fn on_restore(&mut self) {
        let engine = self.base.engine();

        // TODO:

        let ev = self.vars_mut().as_raw_mut();
        ev.v_angle.z = 0.0;
        ev.angles = ev.v_angle;
        ev.fixangle = 1;

        engine.set_physics_key_value(self, c"hl", c"1");
    }
}

impl Player {
    pub fn set_custom_decal_frames(&mut self, frames: c_int) {
        debug!("Player::set_custom_decal_frames({frames})");
    }
}

impl CreateEntity for Player {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            input: Input::default(),
        }
    }
}

impl Entity for Player {
    delegate_entity!(base not { object_caps, restore, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let engine = self.base.engine();
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

        engine.set_physics_key_value(self, c"slj", c"0");
        engine.set_physics_key_value(self, c"hl", c"1");

        self.global_state().game_rules().get_player_spawn_spot(self);

        engine.set_model(self, res::valve::models::PLAYER);
    }
}

impl EntityPlayer for Player {
    fn select_spawn_point(&self) -> *mut edict_s {
        let engine = self.engine();
        let global_state = self.global_state();
        let game_rules = global_state.game_rules();

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
            self.global_state().set_last_spawn(spot);
            spot
        } else {
            error!("No info_player_start on level");
            engine.entity_of_ent_index(EntityIndex::ZERO)
        }
    }

    fn pre_think(&mut self) {
        self.input.pre_think(self.base.vars());
    }

    fn post_think(&mut self) {
        self.input.post_think(self.base.vars());
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        debug!("drop Player");
    }
}
