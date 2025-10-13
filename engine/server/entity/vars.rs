use core::{
    ffi::{c_int, CStr},
    fmt, mem,
    num::NonZeroI32,
    ptr::{self, NonNull},
};

use bitflags::bitflags;
use csz::CStrThin;
use xash3d_shared::{
    entity::{Buttons, EdictFlags, Effects, MoveType},
    ffi::{
        self,
        common::vec3_t,
        server::{edict_s, entvars_s},
    },
    macros::{const_assert_size_of_field_eq, define_enum_for_primitive},
    math::ToAngleVectors,
    render::{RenderFx, RenderMode},
    str::ToEngineStr,
};

use crate::{
    engine::ServerEngineRef,
    entity::{AsEntityHandle, KeyValue},
    global_state::GlobalStateRef,
    save::{FieldType, SaveFields},
    str::MapString,
    time::MapTime,
};

#[cfg(feature = "save")]
use crate::save::{self, SaveResult};

trait RawBitflags<T> {
    fn from_bits_retain(bits: T) -> Self;

    fn bits(self) -> T;
}

impl RawBitflags<i32> for u32 {
    fn from_bits_retain(bits: i32) -> Self {
        bits as u32
    }

    fn bits(self) -> i32 {
        self as i32
    }
}

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum Solid: i32 {
        #[default]
        Not(ffi::common::SOLID_NOT),
        Trigger(ffi::common::SOLID_TRIGGER),
        BBox(ffi::common::SOLID_BBOX),
        SlideBox(ffi::common::SOLID_SLIDEBOX),
        Bsp(ffi::common::SOLID_BSP),
        Custom(ffi::common::SOLID_CUSTOM),
        Portal(ffi::common::SOLID_PORTAL),
    }
}

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum TakeDamage: f32 {
        #[default]
        No(Self::DAMAGE_NO),
        Yes(Self::DAMAGE_YES),
        Aim(Self::DAMAGE_AIM),
    }
}

impl TakeDamage {
    const DAMAGE_NO: f32 = ffi::common::DAMAGE_NO as f32;
    const DAMAGE_YES: f32 = ffi::common::DAMAGE_YES as f32;
    const DAMAGE_AIM: f32 = ffi::common::DAMAGE_AIM as f32;
}

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum Dead: i32 {
        /// Alive.
        #[default]
        No(ffi::common::DEAD_NO),
        /// Playing death animation or still falling off of a ledge waiting to hit ground.
        Dying(ffi::common::DEAD_DYING),
        /// Dead. Lying still.
        Yes(ffi::common::DEAD_DEAD),
        Respawnable(ffi::common::DEAD_RESPAWNABLE),
        DiscardBody(ffi::common::DEAD_DISCARDBODY),
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Weapons: u32 {
        const SUIT          = 1 << 31;
    }
}

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum WaterLevel: i32 {
        // Not in water.
        #[default]
        Dry(0),
        // Feet underwater.
        Feet(1),
        // Waist underwater.
        Waist(2),
        // Head underwater (completely submerged).
        Head(3),
    }
}

macro_rules! field {
    // get mutable reference to bitflags
    (get bitflags $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> &mut $ty:ty) => {
        $( #[$attr] )*
        pub fn $meth(&mut self) -> &mut $ty {
            const_assert_size_of_field_eq!($ty, entvars_s, $field);
            unsafe { mem::transmute(&mut self.borrow_mut().$field) }
        }
    };
    // get mutable reference
    (get $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> &mut $ty:ty) => {
        $( #[$attr] )*
        pub fn $meth(&mut self) -> &mut $ty {
            &mut self.borrow_mut().$field
        }
    };

    // get enum
    (get enum $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> $ty:ty) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> $ty {
            <$ty>::from_raw(self.borrow().$field).unwrap()
        }
    };
    // get bitflags with cast
    (get bitflags $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> $ty:ty, $from:ty as $to:ty) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> $ty {
            let bits: $from = self.borrow().$field;
            <$ty>::from_bits_retain(bits as $to)
        }
    };
    // get bitflags
    (get bitflags $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> $ty:ty) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> $ty {
            <$ty>::from_bits_retain(self.borrow().$field)
        }
    };
    // get optional map string
    (get $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> Option<MapString>) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> Option<MapString> {
            MapString::from_index(self.engine, self.borrow().$field)
        }
    };
    // get map time
    (get $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> MapTime) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> MapTime {
            MapTime::from_secs_f32(self.borrow().$field)
        }
    };
    // get optional non-null pointer
    (get $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> Option<NonNull<$ty:ty>>) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> Option<NonNull<$ty>> {
            NonNull::new(self.borrow().$field)
        }
    };
    // get type with cast to other type
    (get $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> $ty:ty as $to:ty) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> $to {
            let value: $ty = self.borrow().$field;
            value as $to
        }
    };
    // get field as is
    (get $field:ident, $( #[$attr:meta] )* fn $meth:ident() -> $ty:ty) => {
        $( #[$attr] )*
        pub fn $meth(&self) -> $ty {
            self.borrow().$field
        }
    };

    // set entity
    (set entity $field:ident, $( #[$attr:meta] )* fn $meth:ident($arg:ident)) => {
        $( #[$attr] )*
        pub fn $meth<'a, T, U>(&mut self, $arg: T)
        where T: Into<Option<&'a U>>,
              U: 'a + ?Sized + AsEntityHandle,
        {
            self.borrow_mut().$field = $arg.into().map_or(ptr::null_mut(), |v| v.as_entity_handle());
        }
    };
    // set enum
    (set enum $field:ident, $( #[$attr:meta] )* fn $meth:ident($arg:ident: $ty:ty)) => {
        $( #[$attr] )*
        pub fn $meth(&mut self, $arg: $ty) {
            self.borrow_mut().$field = $arg.into_raw();
        }
    };
    // set bitflags
    (set bitflags $field:ident, $( #[$attr:meta] )* fn $meth:ident($arg:ident: $ty:ty $( as $to:ty)?)) => {
        $( #[$attr] )*
        pub fn $meth(&mut self, $arg: $ty) {
            self.borrow_mut().$field = $arg.bits() $( as $to)?;
        }
    };
    // set optional map string
    (set $field:ident, $( #[$attr:meta] )* fn $meth:ident($arg:ident: Option<MapString>)) => {
        $( #[$attr] )*
        pub fn $meth(&mut self, $arg: impl Into<Option<MapString>>) {
            self.borrow_mut().$field = $arg.into().map_or(0, |s| s.index());
        }
    };
    // set map time
    (set $field:ident, $( #[$attr:meta] )* fn $meth:ident($arg:ident: MapTime)) => {
        $( #[$attr] )*
        pub fn $meth(&mut self, $arg: MapTime) {
            self.borrow_mut().$field = $arg.as_secs_f32();
        }
    };
    // set vector
    (set $field:ident, $( #[$attr:meta] )* fn $meth:ident($arg:ident: vec3_t)) => {
        $( #[$attr] )*
        pub fn $meth(&mut self, $arg: impl Into<vec3_t>) {
            self.borrow_mut().$field = $arg.into();
        }
    };
    // set field as is
    (set $field:ident, $( #[$attr:meta] )* fn $meth:ident($arg:ident: $ty:ty)) => {
        $( #[$attr] )*
        pub fn $meth(&mut self, $arg: $ty) {
            self.borrow_mut().$field = $arg;
        }
    };

    // unsafe set field as is
    (set $field:ident, $( #[$attr:meta] )* unsafe fn $meth:ident($arg:ident: $ty:ty)) => {
        $( #[$attr] )*
        pub unsafe fn $meth(&mut self, $arg: $ty) {
            self.borrow_mut().$field = $arg;
        }
    };
}

/// A safe wrapper for [entvars_s].
pub struct EntityVars {
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
    raw: *mut entvars_s,
}

impl EntityVars {
    /// Creates a wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// * The raw pointer must be non-null and received from the engine.
    pub unsafe fn from_raw(
        engine: ServerEngineRef,
        global_state: GlobalStateRef,
        raw: *mut entvars_s,
    ) -> Self {
        Self {
            engine,
            global_state,
            raw,
        }
    }

    #[deprecated]
    pub fn as_raw(&self) -> &entvars_s {
        unsafe { &*self.raw }
    }

    #[deprecated]
    pub fn as_raw_mut(&mut self) -> &mut entvars_s {
        unsafe { &mut *self.raw }
    }

    pub fn as_ptr(&self) -> *const entvars_s {
        self.raw
    }

    pub fn as_mut_ptr(&mut self) -> *mut entvars_s {
        self.raw
    }

    fn borrow(&self) -> &entvars_s {
        unsafe { &*self.raw }
    }

    fn borrow_mut(&mut self) -> &mut entvars_s {
        unsafe { &mut *self.raw }
    }

    pub fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    pub fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }

    field!(get classname, fn classname() -> Option<MapString>);
    field!(set classname, fn set_classname(s: Option<MapString>));

    field!(get globalname, fn globalname() -> Option<MapString>);
    field!(set globalname, fn set_globalname(s: Option<MapString>));

    field!(get origin, fn origin() -> vec3_t);
    field!(get origin, fn origin_mut() -> &mut vec3_t);
    field!(set origin,
        /// Sets a new world position of this entity without linking to the world.
        ///
        /// Call [EntityVars::link] to link the entity to the list.
        fn set_origin(v: vec3_t));

    /// Links this entity into the list.
    #[deprecated(note = "use engine.set_origin_and_link(v.origin(), v) instead")]
    pub fn link(&mut self) {
        let engine = self.engine();
        engine.set_origin_and_link(self.origin(), self);
    }

    /// Sets a new world position of this entity and links it into the list.
    #[deprecated(note = "use ServerEngine::set_origin_and_link instead")]
    #[allow(deprecated)]
    pub fn set_origin_and_link(&mut self, origin: impl Into<vec3_t>) {
        self.set_origin(origin);
        self.link();
    }

    field!(get oldorigin, fn old_origin() -> vec3_t);
    field!(get oldorigin, fn old_origin_mut() -> &mut vec3_t);
    field!(set oldorigin, fn set_old_origin(v: vec3_t));

    field!(get velocity, fn velocity() -> vec3_t);
    field!(get velocity, fn velocity_mut() -> &mut vec3_t);
    field!(set velocity, fn set_velocity(v: vec3_t));

    field!(get basevelocity, fn base_velocity() -> vec3_t);
    field!(get basevelocity, fn base_velocity_mut() -> &mut vec3_t);
    field!(set basevelocity, fn set_base_velocity(v: vec3_t));

    field!(get clbasevelocity, fn client_base_velocity() -> vec3_t);
    field!(get clbasevelocity, fn client_base_velocity_mut() -> &mut vec3_t);
    field!(set clbasevelocity, fn set_client_base_velocity(v: vec3_t));

    field!(get movedir, fn move_dir() -> vec3_t);
    field!(get movedir, fn move_dir_mut() -> &mut vec3_t);
    // field!(set movedir, fn set_move_dir(v: vec3_t));

    #[deprecated(note = "use set_move_dir_from_angles instead")]
    pub fn set_move_dir(&mut self) {
        self.set_move_dir_from_angles();
    }

    pub fn set_move_dir_from_angles(&mut self) {
        let ev = self.borrow_mut();
        if ev.angles == vec3_t::new(0.0, -1.0, 0.0) {
            ev.movedir = vec3_t::new(0.0, 0.0, 1.0);
        } else if ev.angles == vec3_t::new(0.0, -2.0, 0.0) {
            ev.movedir = vec3_t::new(0.0, 0.0, -1.0);
        } else {
            ev.movedir = ev.angles.angle_vectors().forward();
        }
        ev.angles = vec3_t::ZERO;
    }

    field!(get angles, fn angles() -> vec3_t);
    field!(get angles, fn angles_mut() -> &mut vec3_t);
    field!(set angles, fn set_angles(v: vec3_t));

    field!(get avelocity, fn angular_velocity() -> vec3_t);
    field!(get avelocity, fn angular_velocity_mut() -> &mut vec3_t);
    field!(set avelocity, fn set_angular_velocity(v: vec3_t));

    field!(get punchangle, fn punch_angle() -> vec3_t);
    field!(get punchangle, fn punch_angle_mut() -> &mut vec3_t);
    field!(set punchangle, fn set_punch_angle(v: vec3_t));

    field!(get v_angle, fn view_angle() -> vec3_t);
    field!(get v_angle, fn view_angle_mut() -> &mut vec3_t);
    field!(set v_angle, fn set_view_angle(v: vec3_t));

    field!(get endpos, fn end_pos() -> vec3_t);
    field!(get endpos, fn end_pos_mut() -> &mut vec3_t);
    field!(set endpos, fn set_end_pos(v: vec3_t));

    field!(get startpos, fn start_pos() -> vec3_t);
    field!(get startpos, fn start_pos_mut() -> &mut vec3_t);
    field!(set startpos, fn set_start_pos(v: vec3_t));

    field!(get impacttime, fn impact_time() -> f32);
    field!(set impacttime, fn set_impact_time(v: f32));

    field!(get starttime, fn start_time() -> f32);
    field!(set starttime, fn set_start_time(v: f32));

    // TODO: fixangle: define enum or wrapper???
    // 0: nothing
    // 1: force view angles
    // 2: add angular velocity
    field!(get fixangle, fn fix_angle() -> i32);
    field!(set fixangle, fn set_fix_angle(v: i32));

    field!(get idealpitch, fn ideal_pitch() -> f32);
    field!(set idealpitch, fn set_ideal_pitch(v: f32));

    field!(get pitch_speed, fn pitch_speed() -> f32);
    field!(set pitch_speed, fn set_pitch_speed(v: f32));

    field!(get ideal_yaw, fn ideal_yaw() -> f32);
    field!(set ideal_yaw, fn set_ideal_yaw(v: f32));

    field!(get yaw_speed, fn yaw_speed() -> f32);
    field!(set yaw_speed, fn set_yaw_speed(v: f32));

    field!(get modelindex, fn model_index_raw() -> i32);
    field!(set modelindex, fn set_model_index_raw(v: i32));

    pub fn model_index(&self) -> Option<NonZeroI32> {
        NonZeroI32::new(self.model_index_raw())
    }

    field!(get model, fn model_name() -> Option<MapString>);
    field!(set model, fn set_model_name(v: Option<MapString>));

    pub fn remove_model(&mut self) {
        self.set_model_name(None);
        self.set_model_index_raw(0);
    }

    #[deprecated(note = "use ServerEngine::set_model instead")]
    pub fn set_model(&mut self, name: impl ToEngineStr) {
        let engine = self.engine;
        engine.set_model(self, name)
    }

    #[deprecated(note = "use ServerEngine::set_model_with_precache instead")]
    pub fn set_model_with_precache(&mut self, name: impl ToEngineStr) {
        let engine = self.engine;
        let name = name.to_engine_str();
        engine.precache_model(name.as_ref());
        engine.set_model(self, name.as_ref())
    }

    #[deprecated(note = "use ServerEngine::reload_model instead")]
    #[allow(deprecated)]
    pub fn reload_model(&mut self) {
        if let Some(name) = self.model_name() {
            self.set_model(name);
        }
    }

    #[deprecated(note = "use ServerEngine::reload_model_with_precache instead")]
    #[allow(deprecated)]
    pub fn reload_model_with_precache(&mut self) {
        if let Some(name) = self.model_name() {
            self.set_model_with_precache(name);
        }
    }

    field!(get viewmodel, fn view_model_name() -> Option<MapString>);
    field!(set viewmodel, fn set_view_model_name(v: Option<MapString>));

    #[deprecated(note = "use view_model_name instead")]
    pub fn viewmodel(&self) -> Option<MapString> {
        self.view_model_name()
    }

    field!(get weaponmodel, fn weapon_model_name() -> Option<MapString>);
    field!(set weaponmodel, fn set_weapon_model_name(v: Option<MapString>));

    #[deprecated(note = "use weapon_model_name instead")]
    pub fn weaponmodel(&self) -> Option<MapString> {
        self.weapon_model_name()
    }

    field!(get absmin, fn abs_min() -> vec3_t);
    field!(get absmin, fn abs_min_mut() -> &mut vec3_t);
    field!(set absmin, fn set_abs_min(v: vec3_t));

    field!(get absmax, fn abs_max() -> vec3_t);
    field!(get absmax, fn abs_max_mut() -> &mut vec3_t);
    field!(set absmax, fn set_abs_max(v: vec3_t));

    /// Returns an absolute center position in the world.
    pub fn abs_center(&self) -> vec3_t {
        (self.abs_min() + self.abs_max()) * 0.5
    }

    field!(get mins, fn min_size() -> vec3_t);
    field!(get mins, fn min_size_mut() -> &mut vec3_t);
    field!(set mins, fn set_min_size(v: vec3_t));

    field!(get maxs, fn max_size() -> vec3_t);
    field!(get maxs, fn max_size_mut() -> &mut vec3_t);
    field!(set maxs, fn set_max_size(v: vec3_t));

    field!(get size, fn size() -> vec3_t);
    field!(get size, fn size_mut() -> &mut vec3_t);
    field!(set size, fn set_size(v: vec3_t));

    pub fn bmodel_origin(&self) -> vec3_t {
        self.abs_min() + self.size() * 0.5
    }

    field!(get ltime, fn last_think_time() -> MapTime);
    field!(set ltime, fn set_last_think_time(time: MapTime));

    pub fn set_last_think_time_from_now(&mut self, relative: f32) {
        let now = self.engine.globals.map_time();
        self.set_last_think_time(now + relative);
    }

    field!(get nextthink, fn next_think_time() -> MapTime);
    // TODO:
    // field!(set nextthink, fn set_next_think_time(time: MapTime));

    pub fn set_next_think_time_absolute(&mut self, time: MapTime) {
        self.borrow_mut().nextthink = time.as_secs_f32();
    }

    /// Sets the next think time relative to the map time.
    #[deprecated(note = "use set_next_think_time_from_now instead")]
    pub fn set_next_think_time(&mut self, relative_time: f32) {
        self.borrow_mut().nextthink = self.engine.globals.map_time_f32() + relative_time;
    }

    /// Sets the next think time relative to the map time.
    pub fn set_next_think_time_from_now(&mut self, relative: f32) {
        self.borrow_mut().nextthink = self.engine.globals.map_time_f32() + relative;
    }

    /// Sets the next think time relative to the last think time.
    pub fn set_next_think_time_from_last(&mut self, relative: f32) {
        self.set_next_think_time_absolute(self.last_think_time() + relative);
    }

    pub fn stop_thinking(&mut self) {
        // numas13: is there any difference between -1.0 and 0.0???
        self.borrow_mut().nextthink = -1.0;
    }

    field!(get enum movetype, fn move_type() -> MoveType);
    field!(set enum movetype, fn set_move_type(v: MoveType));

    field!(get enum solid, fn solid() -> Solid);
    field!(set enum solid, fn set_solid(v: Solid));

    field!(get skin, fn skin() -> i32);
    field!(set skin, fn set_skin(v: i32));

    field!(get body, fn body() -> i32);
    field!(set body, fn set_body(v: i32));

    field!(get bitflags effects, fn effects() -> Effects);
    field!(get bitflags effects, fn effects_mut() -> &mut Effects);
    field!(set bitflags effects, fn set_effects(v: Effects));

    pub fn remove_effects(&mut self) {
        self.set_effects(Effects::NONE);
    }

    field!(get gravity, fn gravity() -> f32);
    field!(set gravity, fn set_gravity(v: f32));

    field!(get friction, fn friction() -> f32);
    field!(set friction, fn set_friction(v: f32));

    // TODO: pfnGetEntityIllum???
    // field!(get light_level, fn light_level() -> i32);
    // field!(set light_level, fn set_light_level(v: i32));

    field!(get sequence, fn sequence() -> i32);
    field!(set sequence, fn set_sequence(v: i32));

    field!(get gaitsequence, fn gaitsequence() -> i32);
    field!(set gaitsequence, fn set_gaitsequence(v: i32));

    field!(get frame, fn frame() -> f32);
    field!(set frame, fn set_frame(v: f32));

    field!(get animtime, fn animation_time() -> f32);
    field!(set animtime, fn set_animation_time(v: f32));

    field!(get framerate, fn framerate() -> f32);
    field!(set framerate, fn set_framerate(v: f32));

    // TODO:
    // pub controller: [byte; 4usize],
    // pub blending: [byte; 2usize],

    field!(get scale,
        /// Returns this entity rendering scale. Applies to studio and sprite models.
        fn scale() -> f32);
    field!(set scale, fn set_scale(v: f32));

    field!(get enum rendermode, fn render_mode() -> RenderMode);
    field!(set enum rendermode, fn set_render_mode(v: RenderMode));

    field!(get renderamt, fn render_amount() -> f32);
    field!(set renderamt, fn set_render_amount(v: f32));

    field!(get rendercolor, fn render_color() -> vec3_t);
    field!(get rendercolor, fn render_color_mut() -> &mut vec3_t);
    field!(set rendercolor, fn set_render_color(v: vec3_t));

    field!(get enum renderfx, fn render_fx() -> RenderFx);
    field!(set enum renderfx, fn set_render_fx(v: RenderFx));

    field!(get health, fn health() -> f32);
    field!(set health, fn set_health(v: f32));

    field!(get frags, fn frags() -> f32);
    field!(set frags, fn set_frags(v: f32));

    // TODO: use bitflags weapons?
    field!(get bitflags weapons, fn weapons() -> u32);
    field!(get bitflags weapons, fn weapons_mut() -> &mut u32);
    field!(set bitflags weapons, fn set_weapons(v: u32));

    pub fn has_suit(&self) -> bool {
        Weapons::from_bits_retain(self.weapons()).intersects(Weapons::SUIT)
    }

    field!(get enum takedamage, fn take_damage() -> TakeDamage);
    field!(set enum takedamage, fn set_take_damage(v: TakeDamage));

    field!(get enum deadflag, fn dead() -> Dead);
    field!(set enum deadflag, fn set_dead(v: Dead));

    field!(get view_ofs, fn view_ofs() -> vec3_t);
    field!(get view_ofs, fn view_ofs_mut() -> &mut vec3_t);
    field!(set view_ofs, fn set_view_ofs(v: vec3_t));

    field!(get bitflags button, fn buttons() -> Buttons);
    field!(get bitflags button, fn buttons_mut() -> &mut Buttons);
    field!(set bitflags button, fn set_buttons(v: Buttons));

    field!(get bitflags impulse, fn impulse() -> u32);
    field!(get bitflags impulse, fn impulse_mut() -> &mut u32);
    field!(set bitflags impulse, fn set_impulse(v: u32));

    field!(get chain, fn chain() -> Option<NonNull<edict_s>>);
    field!(set entity chain, fn set_chain(chain));

    field!(get dmg_inflictor, fn damage_inflictor() -> Option<NonNull<edict_s>>);
    field!(set entity dmg_inflictor, fn set_damage_inflictor(entity));

    field!(get enemy, fn enemy() -> Option<NonNull<edict_s>>);
    field!(set entity enemy, fn set_enemy(enemy));

    field!(get aiment, fn aim_entity() -> Option<NonNull<edict_s>>);
    field!(set entity aiment, fn set_aim_entity(owner));

    field!(get owner, fn owner() -> Option<NonNull<edict_s>>);
    field!(set entity owner, fn set_owner(owner));

    field!(get groundentity, fn ground_entity() -> Option<NonNull<edict_s>>);
    field!(set entity groundentity, fn set_ground_entity(ground));

    field!(get bitflags spawnflags, fn spawn_flags() -> u32);
    field!(get bitflags spawnflags, fn spawn_flags_mut() -> &mut u32);
    field!(set bitflags spawnflags, fn set_spawn_flags(v: u32));

    field!(get bitflags flags, fn flags() -> EdictFlags);
    field!(get bitflags flags, fn flags_mut() -> &mut EdictFlags);
    field!(set bitflags flags, fn set_flags(v: EdictFlags));

    field!(get colormap, fn color_map() -> i32);
    field!(set colormap, fn set_color_map(v: i32));

    field!(get team, fn team() -> i32);
    field!(set team, fn set_team(v: i32));

    field!(get max_health, fn max_health() -> f32);
    field!(set max_health, fn set_max_health(v: f32));

    field!(get teleport_time, fn teleport_time() -> MapTime);
    field!(set teleport_time, fn set_teleport_time(v: MapTime));

    field!(get armortype, fn armor_type() -> f32);
    field!(set armortype, fn set_armor_type(v: f32));

    field!(get armorvalue, fn armor_value() -> f32);
    field!(set armorvalue, fn set_armor_value(v: f32));

    field!(get enum waterlevel, fn water_level() -> WaterLevel);
    field!(set enum waterlevel, fn set_water_level(v: WaterLevel));

    // TODO: define WaterType(CONTENTS_*) enum for watertype???
    field!(get watertype, fn water_type() -> i32);
    field!(set watertype, fn set_water_type(v: i32));

    field!(get target, fn target() -> Option<MapString>);
    field!(set target, fn set_target(s: Option<MapString>));

    field!(get targetname, fn target_name() -> Option<MapString>);
    field!(set targetname, fn set_target_name(s: Option<MapString>));

    field!(get netname, fn net_name() -> Option<MapString>);
    field!(set netname, fn set_net_name(s: Option<MapString>));

    field!(get message, fn message() -> Option<MapString>);
    field!(set message, fn set_message(s: Option<MapString>));

    field!(get dmg_take, fn damage_take() -> f32);
    field!(set dmg_take, fn set_damage_take(v: f32));

    field!(get dmg_save, fn damage_save() -> f32);
    field!(set dmg_save, fn set_damage_save(v: f32));

    field!(get dmg, fn damage() -> f32);
    field!(set dmg, fn set_damage(v: f32));

    field!(get dmgtime,
        /// Returns the map time at which this entity last took damage.
        fn damage_time() -> MapTime);
    field!(set dmgtime, fn set_damage_time(v: MapTime));

    field!(get noise, fn noise() -> Option<MapString>);
    field!(set noise, fn set_noise(s: Option<MapString>));

    field!(get noise1, fn noise1() -> Option<MapString>);
    field!(set noise1, fn set_noise1(s: Option<MapString>));

    field!(get noise2, fn noise2() -> Option<MapString>);
    field!(set noise2, fn set_noise2(s: Option<MapString>));

    field!(get noise3, fn noise3() -> Option<MapString>);
    field!(set noise3, fn set_noise3(s: Option<MapString>));

    field!(get speed, fn speed() -> f32);
    field!(set speed, fn set_speed(v: f32));

    field!(get air_finished, fn air_finished_time() -> MapTime);
    field!(set air_finished, fn set_air_finished_time(v: MapTime));

    field!(get pain_finished, fn pain_finished_time() -> MapTime);
    field!(set pain_finished, fn set_pain_finished_time(v: MapTime));

    #[deprecated(note = "use pain_finished_time instead")]
    pub fn pain_finished(&self) -> MapTime {
        self.pain_finished_time()
    }

    #[deprecated(note = "use set_pain_finished_time instead")]
    pub fn set_pain_finished(&mut self, v: MapTime) {
        self.set_pain_finished_time(v)
    }

    field!(get radsuit_finished, fn radsuit_finished_time() -> MapTime);
    field!(set radsuit_finished, fn set_radsuit_finished_time(v: MapTime));

    field!(get pContainingEntity, fn containing_entity_raw() -> *mut edict_s);
    field!(set pContainingEntity, fn set_containing_entity_raw(v: *mut edict_s));

    field!(get pContainingEntity, fn containing_entity() -> Option<NonNull<edict_s>>);
    field!(set entity pContainingEntity, fn set_containing_entity(owner));

    field!(get playerclass, fn player_class() -> i32);
    field!(set playerclass, fn set_player_class(v: i32));

    field!(get maxspeed, fn max_speed() -> f32);
    field!(set maxspeed, fn set_max_speed(v: f32));

    field!(get fov, fn fov() -> f32);
    field!(set fov, fn set_fov(v: f32));

    field!(get weaponanim, fn weapon_animation() -> i32);
    field!(set weaponanim, fn set_weapon_animation(v: i32));

    field!(get pushmsec, fn push_msec() -> i32);
    field!(set pushmsec, fn set_push_msec(v: i32));

    field!(get bInDuck, fn in_duck_raw() -> i32);
    field!(set bInDuck, fn set_in_duck_raw(v: i32));

    pub fn in_duck(&self) -> bool {
        self.in_duck_raw() != 0
    }

    pub fn set_in_duck(&mut self, v: bool) {
        self.set_in_duck_raw(v as i32)
    }

    field!(get flTimeStepSound, fn time_step_sound() -> i32);
    field!(set flTimeStepSound, fn set_time_step_sound(v: i32));

    field!(get flSwimTime, fn swim_time() -> i32);
    field!(set flSwimTime, fn set_swim_time(v: i32));

    field!(get flDuckTime, fn duck_time() -> i32);
    field!(set flDuckTime, fn set_duck_time(v: i32));

    field!(get iStepLeft, fn step_left() -> i32);
    field!(set iStepLeft, fn set_step_left(v: i32));

    field!(get flFallVelocity, fn fall_velocity() -> f32);
    field!(set flFallVelocity, fn set_fall_velocity(v: f32));

    field!(get gamestate, fn game_state() -> i32);
    field!(set gamestate, fn set_game_state(v: i32));

    field!(get bitflags oldbuttons, fn old_buttons() -> Buttons);
    field!(get bitflags oldbuttons, fn old_buttons_mut() -> &mut Buttons);
    field!(set bitflags oldbuttons, fn set_old_buttons(v: Buttons));

    field!(get groupinfo, fn group_info() -> i32);
    field!(set groupinfo, fn set_group_info(v: i32));

    field!(get iuser1, fn iuser1() -> i32);
    field!(get iuser1, fn iuser1_mut() -> &mut i32);
    field!(set iuser1, fn set_iuser1(v: i32));

    field!(get iuser2, fn iuser2() -> i32);
    field!(get iuser2, fn iuser2_mut() -> &mut i32);
    field!(set iuser2, fn set_iuser2(v: i32));

    field!(get iuser3, fn iuser3() -> i32);
    field!(get iuser3, fn iuser3_mut() -> &mut i32);
    field!(set iuser3, fn set_iuser3(v: i32));

    field!(get iuser4, fn iuser4() -> i32);
    field!(get iuser4, fn iuser4_mut() -> &mut i32);
    field!(set iuser4, fn set_iuser4(v: i32));

    field!(get fuser1, fn fuser1() -> f32);
    field!(set fuser1, fn set_fuser1(v: f32));

    field!(get fuser2, fn fuser2() -> f32);
    field!(set fuser2, fn set_fuser2(v: f32));

    field!(get fuser3, fn fuser3() -> f32);
    field!(set fuser3, fn set_fuser3(v: f32));

    field!(get fuser4, fn fuser4() -> f32);
    field!(set fuser4, fn set_fuser4(v: f32));

    field!(get vuser1, fn vuser1() -> vec3_t);
    field!(get vuser1, fn vuser1_mut() -> &mut vec3_t);
    field!(set vuser1, fn set_vuser1(v: vec3_t));

    field!(get vuser2, fn vuser2() -> vec3_t);
    field!(get vuser2, fn vuser2_mut() -> &mut vec3_t);
    field!(set vuser2, fn set_vuser2(v: vec3_t));

    field!(get vuser3, fn vuser3() -> vec3_t);
    field!(get vuser3, fn vuser3_mut() -> &mut vec3_t);
    field!(set vuser3, fn set_vuser3(v: vec3_t));

    field!(get vuser4, fn vuser4() -> vec3_t);
    field!(get vuser4, fn vuser4_mut() -> &mut vec3_t);
    field!(set vuser4, fn set_vuser4(v: vec3_t));

    field!(get euser1, fn euser1_raw() -> *mut edict_s);
    field!(set euser1, fn set_euser1_raw(ent: *mut edict_s));

    field!(get euser2, fn euser2_raw() -> *mut edict_s);
    field!(set euser2, fn set_euser2_raw(ent: *mut edict_s));

    field!(get euser3, fn euser3_raw() -> *mut edict_s);
    field!(set euser3, fn set_euser3_raw(ent: *mut edict_s));

    field!(get euser4, fn euser4_raw() -> *mut edict_s);
    field!(set euser4, fn set_euser4_raw(ent: *mut edict_s));

    field!(get euser1, fn euser1() -> Option<NonNull<edict_s>>);
    field!(set entity euser1, fn set_euser1(ent));

    field!(get euser2, fn euser2() -> Option<NonNull<edict_s>>);
    field!(set entity euser2, fn set_euser2(ent));

    field!(get euser3, fn euser3() -> Option<NonNull<edict_s>>);
    field!(set entity euser3, fn set_euser3(ent));

    field!(get euser4, fn euser4() -> Option<NonNull<edict_s>>);
    field!(set entity euser4, fn set_euser4(ent));

    /// Ask the engine to remove this entity at the appropriate time.
    pub fn delayed_remove(&mut self) {
        self.flags_mut().insert(EdictFlags::KILLME);
        self.set_target_name(None);
    }

    pub fn key_value(&mut self, data: &mut KeyValue) {
        let key_name = data.key_name();

        if key_name == c"damage" {
            self.set_damage(data.value_str().parse().unwrap_or(0.0));
            data.set_handled(true);
            return;
        }

        let field = entvars_s::SAVE_FIELDS.iter().find(|i| {
            let name = unsafe { CStrThin::from_ptr(i.fieldName) };
            name.eq_ignore_case(key_name)
        });

        if let Some(field) = field {
            let field_type = FieldType::from_raw(field.fieldType).unwrap();
            let pev = self.raw as *mut u8;
            let p = unsafe { pev.offset(field.fieldOffset as isize) };

            match field_type {
                FieldType::MODELNAME | FieldType::SOUNDNAME | FieldType::STRING => {
                    let s = self.engine.new_map_string(data.value());
                    unsafe {
                        ptr::write(p.cast::<c_int>(), s.index());
                    }
                }
                FieldType::TIME | FieldType::FLOAT => {
                    let v = data.value_str().parse().unwrap_or(0.0);
                    unsafe {
                        ptr::write(p.cast::<f32>(), v);
                    }
                }
                FieldType::INTEGER => {
                    let v = data.value_str().parse().unwrap_or(0);
                    unsafe {
                        ptr::write(p.cast::<c_int>(), v);
                    }
                }
                FieldType::POSITION_VECTOR | FieldType::VECTOR => {
                    let mut v = vec3_t::ZERO;
                    for (i, s) in data.value_str().split(' ').enumerate() {
                        v[i] = s.parse().unwrap_or(0.0);
                    }
                    unsafe {
                        ptr::write(p.cast::<vec3_t>(), v);
                    }
                }
                _ => {
                    let name = unsafe { CStr::from_ptr(field.fieldName) };
                    error!(
                        "EntityVars::key_value: unimplemented field type {} for {name:?}",
                        field.fieldType
                    );
                }
            }
            data.set_handled(true);
        }
    }
}

#[cfg(feature = "save")]
impl save::Save for EntityVars {
    fn save(&self, state: &mut save::SaveState, cur: &mut save::CursorMut) -> SaveResult<()> {
        save::write_fields(state, cur, self.borrow())
    }
}

#[cfg(feature = "save")]
impl save::Restore for EntityVars {
    fn restore(&mut self, state: &save::RestoreState, cur: &mut save::Cursor) -> SaveResult<()> {
        save::read_fields(state, cur, self.borrow_mut())
    }
}

impl fmt::Debug for EntityVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EntityVars").finish()
    }
}
