use core::{
    ffi::{c_int, CStr},
    fmt, mem,
    num::NonZeroI32,
    ptr::{self, NonNull},
};

use csz::CStrThin;
use xash3d_shared::{
    entity::{EdictFlags, Effects, MoveType},
    ffi::{
        self,
        common::vec3_t,
        server::{edict_s, entvars_s},
    },
    macros::{const_assert_size_of_field_eq, define_enum_for_primitive},
    math::ToAngleVectors,
    render::RenderMode,
    str::ToEngineStr,
};

use crate::{
    engine::ServerEngineRef,
    entity::{AsEdict, KeyValue},
    global_state::GlobalStateRef,
    save::{FieldType, SaveFields},
    str::MapString,
    time::MapTime,
};

#[cfg(feature = "save")]
use crate::save::{self, SaveResult};

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

macro_rules! entvars_value {
    (
        $field:ident: $ty:ty,
        $( #[$get_attr:meta] )* fn $get:ident
        $(, $( #[$set_attr:meta] )* fn $set:ident)? $(,)?
    ) => {
        $( #[$get_attr] )*
        pub fn $get(&self) -> $ty {
            self.as_raw().$field
        }

        $(
            $( #[$set_attr] )*
            pub fn $set(&mut self, $get: $ty) {
                self.as_raw_mut().$field = $get;
            }
        )?
    }
}

macro_rules! entvars_str {
    (
        $field:ident,
        $( #[$get_attr:meta] )* fn $get:ident
        $(, $( #[$set_attr:meta] )* fn $set:ident)? $(,)?
    ) => {
        $( #[$get_attr] )*
        pub fn $get(&self) -> Option<MapString> {
            MapString::from_index(self.engine, self.as_raw().$field)
        }

        $(
            $( #[$set_attr] )*
            pub fn $set(&mut self, $get: impl Into<Option<MapString>>) {
                self.as_raw_mut().$field = $get.into().map_or(0, |s| s.index());
            }
        )?
    }
}

macro_rules! entvars_enum {
    (
        $field:ident: $ty:ty,
        $( #[$get_attr:meta] )* fn $get:ident
        $(, $( #[$set_attr:meta] )* fn $set:ident)? $(,)?
    ) => {
        $( #[$get_attr] )*
        pub fn $get(&self) -> $ty {
            <$ty>::from_raw(self.as_raw().$field).unwrap()
        }

        $(
            $( #[$set_attr] )*
            pub fn $set(&mut self, $get: $ty) {
                self.as_raw_mut().$field = $get.into_raw();
            }
        )?
    }
}

macro_rules! entvars_time {
    (
        $field:ident,
        $( #[$get_attr:meta] )* fn $get:ident
        $(, $( #[$set_attr:meta] )* fn $set:ident)? $(,)?
    ) => {
        $( #[$get_attr] )*
        pub fn $get(&self) -> MapTime {
            MapTime::from_secs_f32(self.as_raw().$field)
        }

        $(
            $( #[$set_attr] )*
            pub fn $set(&mut self, $get: MapTime) {
                self.as_raw_mut().$field = $get.as_secs_f32();
            }
        )?
    }
}

macro_rules! entvars_bitflags {
    (
        $field:ident: $ty:ty,
        $( #[$get_attr:meta] )* fn $get:ident
        $(, $( #[$set_attr:meta] )* fn $set:ident $(, $( #[$mut_attr:meta] )* fn $get_mut:ident)?)?
        $(,)?
    ) => {
        $( #[$get_attr] )*
        pub fn $get(&self) -> $ty {
            <$ty>::from_bits_retain(self.as_raw().$field)
        }

        $(
            $(#[$set_attr])*
            pub fn $set(&mut self, $get: $ty) {
                self.as_raw_mut().$field = $get.bits();
            }

            $(
                $(#[$mut_attr])*
                pub fn $get_mut(&mut self) -> &mut $ty {
                    const_assert_size_of_field_eq!($ty, entvars_s, $field);
                    unsafe { mem::transmute(&mut self.as_raw_mut().$field) }
                }
            )?
        )?
    }
}

macro_rules! entvars_vec3 {
    (
        $field:ident,
        $( #[$get_attr:meta] )* fn $get:ident
        $(, $( #[$set_attr:meta] )* fn $set:ident $(, $( #[$mut_attr:meta] )* fn $get_mut:ident)?)?
        $(,)?
    ) => {
        $( #[$get_attr] )*
        pub fn $get(&self) -> vec3_t {
            self.as_raw().$field
        }

        $(
            $(#[$set_attr])*
            pub fn $set(&mut self, $get: impl Into<vec3_t>) {
                self.as_raw_mut().$field = $get.into();
            }

            $(
                $(#[$mut_attr])*
                pub fn $get_mut(&mut self) -> &mut vec3_t {
                    &mut self.as_raw_mut().$field
                }
            )?
        )?
    }
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

    pub fn as_raw(&self) -> &entvars_s {
        unsafe { &*self.raw }
    }

    pub fn as_raw_mut(&mut self) -> &mut entvars_s {
        unsafe { &mut *self.raw }
    }

    pub fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    pub fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }

    pub fn owner(&self) -> Option<NonNull<edict_s>> {
        NonNull::new(self.as_raw().owner)
    }

    pub fn set_owner<T: ?Sized + AsEdict>(&mut self, owner: &mut T) {
        self.as_raw_mut().owner = owner.as_edict_mut();
    }

    entvars_str!(classname, fn classname, fn set_classname);
    entvars_str!(globalname, fn globalname, fn set_globalname);
    entvars_str!(targetname, fn target_name, fn set_target_name);
    entvars_str!(target, fn target, fn set_target);
    entvars_str!(netname, fn net_name, fn set_net_name);
    entvars_str!(model, fn model_name);

    pub fn model_index_raw(&self) -> i32 {
        self.as_raw().modelindex
    }

    pub fn model_index(&self) -> Option<NonZeroI32> {
        NonZeroI32::new(self.model_index_raw())
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

    pub fn remove_model(&mut self) {
        let ev = self.as_raw_mut();
        ev.model = 0;
        ev.modelindex = 0;
    }

    pub fn viewmodel(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().viewmodel)
    }

    pub fn weaponmodel(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().weaponmodel)
    }

    pub fn message(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().message)
    }

    entvars_bitflags!(flags: EdictFlags, fn flags, fn set_flags, fn flags_mut);

    pub fn spawn_flags(&self) -> u32 {
        self.as_raw().spawnflags as u32
    }

    pub fn set_spawn_flags(&mut self, flags: u32) {
        self.as_raw_mut().spawnflags = flags as c_int;
    }

    pub fn spawn_flags_mut(&mut self) -> &mut u32 {
        const_assert_size_of_field_eq!(u32, entvars_s, spawnflags);
        unsafe { mem::transmute(&mut self.as_raw_mut().spawnflags) }
    }

    /// Ask the engine to remove this entity at the appropriate time.
    pub fn delayed_remove(&mut self) {
        self.flags_mut().insert(EdictFlags::KILLME);
        self.as_raw_mut().targetname = 0;
    }

    entvars_bitflags!(effects: Effects, fn effects, fn set_effects, fn effects_mut);

    pub fn remove_effects(&mut self) {
        self.as_raw_mut().effects = 0;
    }

    /// Sets the next think time relative to the map time.
    pub fn set_next_think_time(&mut self, relative_time: f32) {
        self.as_raw_mut().nextthink = self.engine.globals.map_time_f32() + relative_time;
    }

    /// Sets the next think time relative to the last think time.
    pub fn set_next_think_time_from_last(&mut self, relative_time: f32) {
        self.as_raw_mut().nextthink = self.as_raw().ltime + relative_time;
    }

    pub fn set_next_think_time_absolute(&mut self, time: MapTime) {
        self.as_raw_mut().nextthink = time.as_secs_f32();
    }

    pub fn stop_thinking(&mut self) {
        // numas13: is there any difference between -1.0 and 0.0???
        self.as_raw_mut().nextthink = -1.0;
    }

    entvars_enum!(takedamage: TakeDamage, fn take_damage, fn set_take_damage);

    entvars_value!(health: f32, fn health, fn set_health);
    entvars_value!(max_health: f32, fn max_health, fn set_max_health);
    entvars_value!(dmg: f32, fn damage, fn set_damage);

    entvars_time!(dmgtime,
        /// Returns the map time at which this entity last took damage.
        fn damage_time,
        fn set_damage_time,
    );

    entvars_time!(pain_finished, fn pain_finished, fn set_pain_finished);

    entvars_enum!(deadflag: Dead, fn dead, fn set_dead);
    entvars_enum!(movetype: MoveType, fn move_type, fn set_move_type);
    entvars_enum!(solid: Solid, fn solid, fn set_solid);

    entvars_value!(friction: f32, fn friction, fn set_friction);
    entvars_value!(skin: i32, fn skin, fn set_skin);

    entvars_vec3!(origin,
        fn origin,
        /// Sets a new world position of this entity.
        ///
        /// Call [EntityVars::link] to link the entity to the list.
        fn set_origin,
        fn origin_mut,
    );

    /// Links this entity into the list.
    pub fn link(&mut self) {
        let engine = self.engine();
        engine.set_origin(self.origin(), self);
    }

    /// Sets a new world position of this entity and links it into the list.
    pub fn set_origin_and_link(&mut self, origin: impl Into<vec3_t>) {
        self.set_origin(origin);
        self.link();
    }

    entvars_vec3!(view_ofs, fn view_ofs, fn set_view_ofs, fn view_ofs_mut);
    entvars_vec3!(mins, fn min_size, fn set_min_size, fn min_size_mut);
    entvars_vec3!(maxs, fn max_size, fn set_max_size, fn max_size_mut);
    entvars_vec3!(absmin, fn abs_min, fn set_abs_min, fn abs_min_mut);
    entvars_vec3!(absmax, fn abs_max, fn set_abs_max, fn abs_max_mut);

    /// Returns an absolute center position in the world.
    pub fn abs_center(&self) -> vec3_t {
        (self.abs_min() + self.abs_max()) * 0.5
    }

    pub fn bmodel_origin(&self) -> vec3_t {
        self.abs_min() + self.size() * 0.5
    }

    entvars_vec3!(angles, fn angles, fn set_angles, fn angles_mut);
    entvars_vec3!(size, fn size, fn set_size, fn size_mut);

    entvars_value!(scale: f32,
        /// Returns this entity rendering scale. Applies to studio and sprite models.
        fn scale,
        fn set_scale,
    );

    entvars_vec3!(velocity, fn velocity, fn set_velocity, fn velocity_mut);
    entvars_vec3!(
        basevelocity,
        fn base_velocity,
        fn set_base_velocity,
        fn base_velocity_mut
    );

    entvars_value!(speed: f32, fn speed, fn set_speed);
    entvars_value!(maxspeed: f32, fn max_speed, fn set_max_speed);

    entvars_vec3!(movedir, fn move_dir);

    pub fn set_move_dir(&mut self) {
        let ev = self.as_raw_mut();
        if ev.angles == vec3_t::new(0.0, -1.0, 0.0) {
            ev.movedir = vec3_t::new(0.0, 0.0, 1.0);
        } else if ev.angles == vec3_t::new(0.0, -2.0, 0.0) {
            ev.movedir = vec3_t::new(0.0, 0.0, -1.0);
        } else {
            ev.movedir = ev.angles.angle_vectors().forward();
        }
        ev.angles = vec3_t::ZERO;
    }

    entvars_value!(frame: f32, fn frame, fn set_frame);
    entvars_value!(framerate: f32, fn framerate, fn set_framerate);

    entvars_enum!(rendermode: RenderMode, fn render_mode, fn set_render_mode);
    entvars_value!(renderamt: f32, fn render_amount, fn set_render_amount);

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
        save::write_fields(state, cur, self.as_raw())
    }
}

#[cfg(feature = "save")]
impl save::Restore for EntityVars {
    fn restore(&mut self, state: &save::RestoreState, cur: &mut save::Cursor) -> SaveResult<()> {
        save::read_fields(state, cur, self.as_raw_mut())
    }
}

impl fmt::Debug for EntityVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EntityVars").finish()
    }
}
