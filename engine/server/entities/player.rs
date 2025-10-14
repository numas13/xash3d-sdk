use core::{ffi::c_int, ptr};

use xash3d_shared::{
    entity::{Buttons, EdictFlags, EntityIndex, MoveType},
    ffi::{common::vec3_t, server::edict_s},
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

struct UseTarget<'a> {
    entity: &'a mut dyn Entity,
    use_type: UseType,
    // TODO: physics flags
    dot: f32,
}

impl<'a> UseTarget<'a> {
    fn new(entity: &'a mut dyn Entity, use_type: UseType, dot: f32) -> Self {
        Self {
            entity,
            use_type,
            dot,
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Player {
    base: BaseEntity,

    pub input: Input,
}

impl Player {
    /// Default search radius for player use action.
    pub const USE_SEARCH_RADIUS: f32 = 64.0;

    /// Default view field for player use action.
    pub const USE_VIEW_FIELD: ViewField = ViewField::NARROW;

    fn is_use_button_active(&self) -> bool {
        self.vars().buttons().intersects(Buttons::USE) || self.input.is_changed(Buttons::USE)
    }

    /// Checks if [player_use](Self::player_use) should be called.
    pub fn check_player_use(&self) -> bool {
        !self.is_observer() && self.is_use_button_active()
    }

    fn new_use_target<'a>(
        &self,
        target: &'a mut dyn Entity,
        view_origin: vec3_t,
        forward: vec3_t,
    ) -> Option<UseTarget<'a>> {
        let v = self.base.vars();
        let caps = target.object_caps();
        if !caps.is_player_use() {
            return None;
        }

        let use_type = if v.buttons().is_use() && caps.is_continuous_use() {
            warn!("player: set physics flags USING is not implemented yet");
            Some(UseType::Set(1.0))
        } else if self.input.pressed().is_use() && (caps.is_impulse_use() || caps.is_on_off_use()) {
            Some(UseType::Set(1.0))
        } else if self.input.released().is_use() && caps.is_on_off_use() {
            Some(UseType::Set(0.0))
        } else {
            None
        };

        use_type.map(|use_type| {
            let v = target.vars();
            let los = v.bmodel_origin() - view_origin;
            // This essentially moves the origin of the target to the corner nearest
            // the player to test to see if it's "hull" is in the view cone.
            let los = utils::clamp_vector_to_box(los, v.size() * 0.5);
            let dot = los.dot(forward);
            UseTarget::new(target, use_type, dot)
        })
    }

    fn player_use_target(&mut self, mut target: Option<UseTarget>) {
        let engine = self.engine();
        let pv = self.base.vars();

        if let Some(i) = &target {
            // check if there is something between the player and the target
            trace!("player use target {}", i.entity.classname());
            let tv = i.entity.vars();
            let start = pv.origin() + pv.view_ofs();
            let end = tv.bmodel_origin();
            let trace = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(pv));
            if trace.fraction() < 0.9 && !ptr::eq(tv.containing_entity_raw(), trace.hit_entity()) {
                let classname = trace.hit_entity().get_entity().map(|e| e.classname());
                trace!("player use trace hit {classname:?} ({})", trace.fraction());
                target = None;
            }

            if false {
                let msg = crate::user_message::Line {
                    start: start.into(),
                    end: (start + (end - start) * trace.fraction()).into(),
                    duration: 10.0.into(),
                    color: if target.is_some() {
                        crate::color::RGB::GREEN
                    } else {
                        crate::color::RGB::RED
                    },
                };
                engine.msg_one_reliable(pv, &msg);
            }
        }

        if self.input.pressed().is_use() {
            engine.build_sound().channel_item().volume(0.4).emit_dyn(
                if target.is_some() {
                    res::valve::sound::common::WPN_SELECT
                } else {
                    res::valve::sound::common::WPN_DENYSELECT
                },
                pv,
            );
        }

        if let Some(i) = target {
            let classname = i.entity.classname();
            let name = i.entity.name();
            let use_type = i.use_type;
            trace!("player use target {classname}({name}) type {use_type:?}");
            i.entity.used(use_type, None, self);
        }
    }

    pub fn player_use_with(&mut self, search_radius: f32, view_field: ViewField) {
        let engine = self.engine();
        let pv = self.base.vars();
        let view_origin = pv.origin() + pv.view_ofs();
        let forward = pv.view_angle().angle_vectors().forward();
        let closest = engine
            .find_entity_in_sphere_iter(None::<&edict_s>, pv.origin(), search_radius)
            .filter_map(|mut i| unsafe { i.as_mut() }.get_entity_mut())
            .filter_map(|i| self.new_use_target(i, view_origin, forward))
            .reduce(|a, b| if a.dot <= b.dot { b } else { a });

        if let Some(target) = closest {
            if view_field.to_dot() <= target.dot {
                // a target is in the view cone
                self.player_use_target(Some(target));
            } else {
                // a target is not in the view cone
                self.player_use_target(None);
            }
        } else {
            // a target is not found
            self.player_use_target(None);
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

        let v = self.vars();
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
        let v = self.vars();
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

        let v = self.vars();
        v.set_model(res::valve::models::PLAYER);
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
