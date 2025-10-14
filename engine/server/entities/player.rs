use core::{ffi::c_int, ptr};

use xash3d_shared::{
    entity::{Buttons, EdictFlags, EntityIndex, MoveType},
    ffi::server::edict_s,
    math::ToAngleVectors,
};

use crate::{
    engine::TraceIgnore,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Dead, Entity, EntityPlayer,
        EntityVars, GetPrivateData, ObjectCaps, Solid, TakeDamage, UseType,
    },
    utils::{self, ViewField},
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

    pub fn pressed(&self) -> Buttons {
        self.buttons_pressed
    }

    pub fn released(&self) -> Buttons {
        self.buttons_released
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

struct TargetUse {
    ty: UseType,
    // TODO: physics flags
}

impl TargetUse {
    fn new(ty: UseType) -> Self {
        Self { ty }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Player {
    base: BaseEntity,

    pub input: Input,
}

impl Player {
    /// Default search radius for use player action.
    pub const USE_SEARCH_RADIUS: f32 = 64.0;

    /// Default view field for use player action.
    pub const USE_VIEW_FIELD: ViewField = ViewField::NARROW;

    fn is_use_button_active(&self) -> bool {
        self.vars().buttons().intersects(Buttons::USE) || self.input.is_changed(Buttons::USE)
    }

    /// Checks if [player_use](Self::player_use) should be called.
    pub fn check_player_use(&self) -> bool {
        !self.is_observer() && self.is_use_button_active()
    }

    fn player_use_type(&mut self, target: &mut dyn Entity) -> Option<TargetUse> {
        let v = self.base.vars();
        let caps = target.object_caps();
        if v.buttons().is_use() && caps.is_continuous_use() {
            warn!("player: set physics flags USING is not implemented yet");
            Some(TargetUse::new(UseType::Set(1.0)))
        } else if self.input.pressed().is_use() && (caps.is_impulse_use() || caps.is_on_off_use()) {
            Some(TargetUse::new(UseType::Set(1.0)))
        } else if self.input.released().is_use() && caps.is_on_off_use() {
            Some(TargetUse::new(UseType::Set(0.0)))
        } else {
            None
        }
    }

    fn player_use_target(&mut self, target: &mut dyn Entity, mut target_use: Option<TargetUse>) {
        trace!("player use target {}", target.classname());
        let debug_use = false;
        let engine = self.engine();
        let pv = self.base.vars();
        let tv = target.vars();

        // check if there is something between the player and the button
        let start = pv.origin() + pv.view_ofs();
        let end = tv.bmodel_origin();
        let trace = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(pv));
        if trace.fraction() < 0.9 && !ptr::eq(tv.containing_entity_raw(), trace.hit_entity()) {
            if debug_use {
                let classname = trace.hit_entity().get_entity().map(|e| e.classname());
                trace!("player use trace hit {classname:?} ({})", trace.fraction());
            }
            target_use = None;
        }

        if debug_use {
            let msg = crate::user_message::Line {
                start: start.into(),
                end: (start + (end - start) * trace.fraction()).into(),
                duration: 10.0.into(),
                color: if target_use.is_some() {
                    crate::color::RGB::GREEN
                } else {
                    crate::color::RGB::RED
                },
            };
            engine.msg_one_reliable(pv, &msg);
        }

        if self.input.pressed().is_use() {
            engine.build_sound().channel_item().volume(0.4).emit_dyn(
                if target_use.is_some() {
                    res::valve::sound::common::WPN_SELECT
                } else {
                    res::valve::sound::common::WPN_DENYSELECT
                },
                pv,
            );
        }

        if let Some(target_use) = target_use {
            let classname = target.classname();
            let name = target.name();
            let ty = target_use.ty;
            trace!("player use target {classname}({name}) type {ty:?}");
            target.used(target_use.ty, None, self);
        }
    }

    pub fn player_use_with(&mut self, search_radius: f32, view_field: ViewField) {
        let debug_search = false;
        if debug_search {
            trace!("player use search:");
        }

        let engine = self.engine();
        let pv = self.base.vars();
        let forward = pv.view_angle().angle_vectors().forward();
        let mut target_use = None;
        let mut target = None;
        let mut target_dot = view_field.to_dot();
        let entities =
            engine.find_entity_in_sphere_iter(None::<&edict_s>, pv.origin(), search_radius);
        for mut ent in entities {
            let Some(ent) = unsafe { ent.as_mut() }.get_entity_mut() else {
                continue;
            };
            if !ent.object_caps().is_player_use() {
                continue;
            }
            let Some(use_type) = self.player_use_type(ent) else {
                continue;
            };

            let pv = self.base.vars();
            let los = ent.vars().bmodel_origin() - (pv.origin() + pv.view_ofs());
            // This essentially moves the origin of the target to the corner nearest
            // the player to test to see if it's "hull" is in the view cone.
            let los = utils::clamp_vector_to_box(los, ent.vars().size() * 0.5);
            let dot = los.dot(forward);
            if debug_search {
                trace!("  {}({}) dot={dot}", ent.classname(), ent.name());
            }
            if target_dot <= dot {
                target_use = Some(use_type);
                target = Some(ent);
                target_dot = dot;
            }
        }

        if let Some(target) = target {
            self.player_use_target(target, target_use);
        }
    }

    pub fn player_use(&mut self) {
        self.player_use_with(Self::USE_SEARCH_RADIUS, Self::USE_VIEW_FIELD);
    }
}

impl_entity_cast!(Player);

#[cfg(feature = "save")]
impl crate::save::OnRestore for Player {
    fn on_restore(&mut self) {
        let engine = self.base.engine();

        // TODO:

        let v = self.vars_mut();
        v.with_view_angle(|v| v.with_z(0.0));
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
        v.with_flags(|f| f & EdictFlags::PROXY);
        v.with_flags(|f| f | EdictFlags::CLIENT);
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
