use core::{ffi::c_int, ptr};

use xash3d_shared::{
    entity::{Buttons, EdictFlags, EntityIndex, MoveType},
    ffi::server::edict_s,
};

use crate::entity::{
    delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Dead, Entity, EntityPlayer,
    EntityVars, ObjectCaps, Solid, TakeDamage,
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

        let v = self.vars_mut();
        v.view_angle_mut().z = 0.0;
        v.set_angles(v.view_angle());
        v.set_fix_angle(1);

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
        let v = self.vars_mut();
        v.set_classname(engine.try_alloc_map_string(c"player").unwrap());
        v.set_health(100.0);
        v.set_armor_value(0.0);
        v.set_take_damage(TakeDamage::Aim);
        v.set_solid(Solid::SlideBox);
        v.set_move_type(MoveType::Walk);
        v.set_max_health(v.health());
        *v.flags_mut() &= EdictFlags::PROXY;
        *v.flags_mut() |= EdictFlags::CLIENT;
        v.set_air_finished_time(engine.globals.map_time() + 12.0);
        v.set_damage(2.0);
        v.remove_effects();
        v.set_dead(Dead::No);
        v.set_damage_take(0.0);
        v.set_damage_save(0.0);
        v.set_friction(1.0);
        v.set_gravity(1.0);
        v.set_fov(0.0);
        v.set_view_ofs(xash3d_player_move::VEC_VIEW);

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
