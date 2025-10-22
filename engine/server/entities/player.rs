use core::{cell::Cell, ffi::c_int, ptr};

use xash3d_shared::{
    entity::{Buttons, EdictFlags, MoveType},
    ffi::common::vec3_t,
    math::ToAngleVectors,
};

use crate::{
    engine::TraceIgnore,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Dead, Entity, EntityHandle,
        EntityPlayer, EntityVars, GetPrivateData, LastSound, ObjectCaps, Solid, TakeDamage,
        UseType,
    },
    utils::{self, ViewField},
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[derive(Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Input {
    buttons_last: Cell<Buttons>,
    buttons_pressed: Cell<Buttons>,
    buttons_released: Cell<Buttons>,
}

impl Input {
    pub fn is_changed(&self, buttons: Buttons) -> bool {
        self.pressed().union(self.released()).contains(buttons)
    }

    pub fn is_pressed(&self, buttons: Buttons) -> bool {
        self.pressed().contains(buttons)
    }

    pub fn is_released(&self, buttons: Buttons) -> bool {
        self.released().contains(buttons)
    }

    pub fn pressed(&self) -> Buttons {
        self.buttons_pressed.get()
    }

    pub fn released(&self) -> Buttons {
        self.buttons_released.get()
    }

    pub fn pre_think(&self, vars: &EntityVars) {
        let buttons = vars.buttons();
        let buttons_changed = self.buttons_last.get().symmetric_difference(buttons);
        self.buttons_pressed
            .set(buttons_changed.intersection(buttons));
        self.buttons_released
            .set(buttons_changed.difference(buttons));
    }

    pub fn post_think(&self, vars: &EntityVars) {
        self.buttons_last.set(vars.buttons());
    }
}

struct UseTarget<'a> {
    entity: &'a dyn Entity,
    use_type: UseType,
    // TODO: physics flags
    dot: f32,
}

impl<'a> UseTarget<'a> {
    fn new(entity: &'a dyn Entity, use_type: UseType, dot: f32) -> Self {
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

    #[cfg_attr(feature = "save", save(skip))]
    last_sound: Cell<Option<LastSound>>,

    pub input: Input,
}

impl CreateEntity for Player {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            last_sound: Default::default(),

            input: Input::default(),
        }
    }
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
        target: &'a dyn Entity,
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

    fn player_use_target(
        &self,
        mut target: Option<UseTarget>,
        f: impl FnOnce(&dyn Entity, UseType),
    ) {
        let engine = self.engine();
        let pv = self.base.vars();

        if let Some(i) = &target {
            // check if there is something between the player and the target
            trace!("player use target {}", i.entity.classname());
            let tv = i.entity.vars();
            let start = pv.origin() + pv.view_ofs();
            let end = tv.bmodel_origin();
            let trace = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(pv));
            if trace.fraction() < 0.9
                && !ptr::eq(
                    tv.containing_entity_raw(),
                    trace.hit_entity().vars().containing_entity_raw(),
                )
            {
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
            let name = i.entity.pretty_name();
            let use_type = i.use_type;
            trace!("player use target {name} type {use_type:?}");
            f(i.entity, i.use_type);
        }
    }

    pub fn player_use_with_custom(
        &self,
        search_radius: f32,
        view_field: ViewField,
        f: impl FnOnce(&dyn Entity, UseType),
    ) {
        let engine = self.engine();
        let pv = self.base.vars();
        let view_origin = pv.origin() + pv.view_ofs();
        let forward = pv.view_angle().angle_vectors().forward();
        let closest = engine
            .entities()
            .in_sphere(pv.origin(), search_radius)
            .filter_map(|i| i.get_entity())
            .filter_map(|i| self.new_use_target(i, view_origin, forward))
            .reduce(|a, b| if a.dot <= b.dot { b } else { a });

        if let Some(target) = closest {
            if view_field.to_dot() <= target.dot {
                // a target is in the view cone
                self.player_use_target(Some(target), f);
            } else {
                // a target is not in the view cone
                self.player_use_target(None, f);
            }
        } else {
            // a target is not found
            self.player_use_target(None, f);
        }
    }

    pub fn player_use_with(&self, search_radius: f32, view_field: ViewField) {
        self.player_use_with_custom(search_radius, view_field, |target, use_type| {
            target.used(use_type, Some(self), self);
        });
    }

    pub fn player_use_custom(&self, f: impl FnOnce(&dyn Entity, UseType)) {
        self.player_use_with_custom(Self::USE_SEARCH_RADIUS, Self::USE_VIEW_FIELD, f);
    }

    pub fn player_use(&self) {
        self.player_use_with(Self::USE_SEARCH_RADIUS, Self::USE_VIEW_FIELD)
    }

    pub fn set_custom_decal_frames(&mut self, frames: c_int) {
        debug!("Player::set_custom_decal_frames({frames})");
    }
}

impl_entity_cast!(Player);

#[cfg(feature = "save")]
impl crate::save::OnRestore for Player {
    fn on_restore(&self) {
        let engine = self.base.engine();

        // TODO:

        let v = self.vars();
        v.with_view_angle(|v| v.with_z(0.0));
        v.set_angles(v.view_angle());
        v.set_fix_angle(1);

        engine.set_physics_key_value(self, c"hl", c"1");
    }
}

impl Entity for Player {
    delegate_entity!(base not { object_caps, restore, spawn, is_player });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let engine = self.base.engine();
        let v = self.base.vars();
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

    fn is_player(&self) -> bool {
        true
    }
}

impl EntityPlayer for Player {
    fn select_spawn_point(&self) -> EntityHandle {
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

        if let Some(spot) = engine.entities().by_class_name(start_spot).first() {
            self.global_state().set_last_spawn(Some(spot));
            spot
        } else {
            error!("No info_player_start on level");
            engine.get_world_spawn_entity()
        }
    }

    fn pre_think(&self) {
        self.input.pre_think(self.base.vars());
    }

    fn post_think(&self) {
        self.input.post_think(self.base.vars());
    }

    fn env_sound(&self) -> Option<LastSound> {
        self.last_sound.get()
    }

    fn set_env_sound(&self, last: Option<LastSound>) {
        self.last_sound.set(last);
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        debug!("drop Player");
    }
}
