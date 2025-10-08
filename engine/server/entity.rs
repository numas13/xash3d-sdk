mod macros;

mod private_data;

use core::{
    ffi::{c_int, c_short, CStr},
    mem, ptr,
};

use bitflags::bitflags;
use csz::CStrThin;
use xash3d_shared::{
    consts::{SOLID_BSP, SOLID_NOT, SOLID_SLIDEBOX, SOLID_TRIGGER},
    ffi::{
        common::{entity_state_s, vec3_t},
        server::{edict_s, entvars_s, KeyValueData},
    },
    macros::const_assert_size_of_field_eq,
    math::{fabsf, ToAngleVectors},
    utils::cstr_or_none,
};

#[cfg(feature = "save")]
use crate::save::{self, Restore, Save, SaveResult};
use crate::{
    engine::ServerEngineRef,
    global_state::{EntityState, GlobalStateRef},
    save::{FieldType, SaveFields},
    str::MapString,
};

pub use self::macros::*;
pub use self::private_data::*;
pub use xash3d_shared::entity::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RestoreResult {
    Delete,
    Ok,
    Moved,
}

impl From<RestoreResult> for c_int {
    fn from(val: RestoreResult) -> Self {
        match val {
            RestoreResult::Delete => -1,
            RestoreResult::Ok => 0,
            RestoreResult::Moved => 1,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EntityOffset(u32);

impl EntityOffset {
    /// Create a new `EntityOffset` from a value.
    ///
    /// # Safety
    ///
    /// The offset value must be received from the engine.
    pub const unsafe fn new_unchecked(offset: u32) -> Self {
        EntityOffset(offset)
    }

    /// Converts this offset to a raw value.
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    pub const fn is_first(&self) -> bool {
        self.0 == 0
    }
}

/// Used to get a reference to [edict_s].
pub trait AsEdict {
    /// Converts this type into a shared reference to [edict_s].
    fn as_edict(&self) -> &edict_s;

    /// Converts this type into a mutable reference to [edict_s].
    fn as_edict_mut(&mut self) -> &mut edict_s;
}

impl AsEdict for edict_s {
    fn as_edict(&self) -> &edict_s {
        self
    }

    fn as_edict_mut(&mut self) -> &mut edict_s {
        self
    }
}

impl AsEdict for entvars_s {
    fn as_edict(&self) -> &edict_s {
        unsafe { &*self.pContainingEntity }
    }

    fn as_edict_mut(&mut self) -> &mut edict_s {
        unsafe { &mut *self.pContainingEntity }
    }
}

impl<T: Entity> AsEdict for T {
    fn as_edict(&self) -> &edict_s {
        self.vars().as_edict()
    }

    fn as_edict_mut(&mut self) -> &mut edict_s {
        self.vars_mut().as_edict_mut()
    }
}

impl AsEdict for EntityVars {
    fn as_edict(&self) -> &edict_s {
        self.as_raw().as_edict()
    }

    fn as_edict_mut(&mut self) -> &mut edict_s {
        self.as_raw_mut().as_edict_mut()
    }
}

#[repr(transparent)]
pub struct KeyValue {
    raw: KeyValueData,
}

impl KeyValue {
    pub fn new(raw: &mut KeyValueData) -> &mut KeyValue {
        unsafe { &mut *(raw as *mut KeyValueData as *mut Self) }
    }

    /// Returns the class name of an entity related to the data.
    pub fn class_name(&self) -> Option<&CStrThin> {
        unsafe { cstr_or_none(self.raw.szClassName) }
    }

    pub fn key_name(&self) -> &CStrThin {
        unsafe { cstr_or_none(self.raw.szKeyName) }.unwrap()
    }

    pub fn key_name_str(&self) -> &str {
        self.key_name().to_str().unwrap_or("")
    }

    pub fn value(&self) -> &CStrThin {
        unsafe { cstr_or_none(self.raw.szValue) }.unwrap()
    }

    pub fn value_str(&self) -> &str {
        self.value().to_str().unwrap_or("")
    }

    /// Returns `true` if the server DLL knows the key name.
    pub fn handled(&self) -> bool {
        self.raw.fHandled != 0
    }

    pub fn set_handled(&mut self, handled: bool) {
        self.raw.fHandled = handled.into();
    }
}

/// A safe wrapper for [entvars_s].
#[derive(Debug)]
pub struct EntityVars {
    engine: ServerEngineRef,
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
    pub unsafe fn from_raw(engine: ServerEngineRef, raw: *mut entvars_s) -> Self {
        Self { engine, raw }
    }

    pub fn as_raw(&self) -> &entvars_s {
        unsafe { &*self.raw }
    }

    pub fn as_raw_mut(&mut self) -> &mut entvars_s {
        unsafe { &mut *self.raw }
    }

    pub fn classname(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().classname)
    }

    pub fn globalname(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().globalname)
    }

    pub fn target_name(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().targetname)
    }

    pub fn target(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().target)
    }

    pub fn model(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().model)
    }

    pub fn viewmodel(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().viewmodel)
    }

    pub fn weaponmodel(&self) -> Option<MapString> {
        MapString::from_index(self.engine, self.as_raw().weaponmodel)
    }

    pub fn flags(&self) -> EdictFlags {
        EdictFlags::from_bits_retain(self.as_raw().flags)
    }

    pub fn flags_mut(&mut self) -> &mut EdictFlags {
        const_assert_size_of_field_eq!(EdictFlags, entvars_s, flags);
        unsafe { mem::transmute(&mut self.as_raw_mut().flags) }
    }

    pub fn spawn_flags(&self) -> u32 {
        self.as_raw().spawnflags as u32
    }

    pub fn set_spawn_flags(&mut self, flags: u32) {
        self.as_raw_mut().spawnflags = flags as c_int;
    }

    /// Ask the engine to remove this entity at the appropriate time.
    pub fn delayed_remove(&mut self) {
        self.flags_mut().insert(EdictFlags::KILLME);
        self.as_raw_mut().targetname = 0;
    }

    pub fn effects(&self) -> Effects {
        Effects::from_bits_retain(self.as_raw().effects)
    }

    pub fn effects_mut(&mut self) -> &mut Effects {
        const_assert_size_of_field_eq!(Effects, entvars_s, effects);
        unsafe { mem::transmute(&mut self.as_raw_mut().effects) }
    }

    /// Sets the next think time relative to the map time.
    pub fn set_next_think_time(&mut self, relative_time: f32) {
        self.as_raw_mut().nextthink = self.engine.globals.map_time_f32() + relative_time;
    }

    /// Sets the next think time relative to the last think time.
    pub fn set_next_think_time_from_last(&mut self, relative_time: f32) {
        self.as_raw_mut().nextthink = self.as_raw().ltime + relative_time;
    }

    pub fn stop_thinking(&mut self) {
        self.as_raw_mut().nextthink = -1.0;
    }

    pub fn origin(&self) -> vec3_t {
        self.as_raw().origin
    }

    pub fn set_origin(&mut self, origin: vec3_t) {
        self.as_raw_mut().origin = origin;
    }

    pub fn angles(&self) -> vec3_t {
        self.as_raw().angles
    }

    pub fn set_angles(&mut self, angles: vec3_t) {
        self.as_raw_mut().angles = angles;
    }

    pub fn size(&self) -> vec3_t {
        self.as_raw().size
    }

    pub fn set_size(&mut self, size: vec3_t) {
        self.as_raw_mut().size = size;
    }

    pub fn velocity(&self) -> vec3_t {
        self.as_raw().velocity
    }

    pub fn set_velocity(&mut self, vel: vec3_t) {
        self.as_raw_mut().velocity = vel;
    }

    pub fn move_dir(&self) -> vec3_t {
        self.as_raw().movedir
    }

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

    pub fn key_value(&mut self, data: &mut KeyValue) {
        let key_name = data.key_name();
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

pub trait CreateEntity: Entity {
    fn create(base: BaseEntity) -> Self;
}

bitflags! {
    /// Flags to indicate an object's capabilities.
    ///
    /// Used for save/restore and level transitions.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct ObjectCaps: u32 {
        const NONE                  = 0;
        const CUSTOMSAVE            = 1 << 0;
        const ACROSS_TRANSITION     = 1 << 1;
        const MUST_SPAWN            = 1 << 2;
        const IMPULSE_USE           = 1 << 3;
        const CONTINUOUS_USE        = 1 << 4;
        const ONOFF_USE             = 1 << 5;
        const DIRECTIONAL_USE       = 1 << 6;
        const MASTER                = 1 << 7;
        const FORCE_TRANSITION      = ObjectCaps::MASTER.bits();
        const DONT_SAVE             = 1 << 31;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UseType {
    Off,
    On,
    Set,
    Toggle,
}

impl UseType {
    pub fn should_toggle(&self, current_state: bool) -> bool {
        !matches!(
            (self, current_state),
            (UseType::On, true) | (UseType::Off, false)
        )
    }
}

pub trait EntityCast: 'static {
    fn as_player(&self) -> Option<&dyn EntityPlayer>;
    fn as_player_mut(&mut self) -> Option<&mut dyn EntityPlayer>;
}

#[cfg(feature = "save")]
pub trait EntitySaveRestore: Save + Restore {}

#[cfg(feature = "save")]
impl<T: Save + Restore> EntitySaveRestore for T {}

#[cfg(not(feature = "save"))]
pub trait EntitySaveRestore {}

#[cfg(not(feature = "save"))]
impl<T> EntitySaveRestore for T {}

define_entity_trait! {
    /// The base trait for all entities.
    pub trait Entity(delegate_entity): (EntitySaveRestore + EntityCast + AsEdict) {
        fn private(&self) -> &::xash3d_server::entity::PrivateData;

        fn private_mut(&mut self) -> &mut ::xash3d_server::entity::PrivateData;

        /// Returns a reference to the server engine.
        fn engine(&self) -> ::xash3d_server::engine::ServerEngineRef;

        fn global_state(&self) -> ::xash3d_server::global_state::GlobalStateRef;

        /// Returns a shared reference to entity variables.
        fn vars(&self) -> &::xash3d_server::entity::EntityVars;

        /// Returns a mutable reference to entity variables.
        fn vars_mut(&mut self) -> &mut ::xash3d_server::entity::EntityVars;

        fn globalname(&self) -> Option<::xash3d_server::str::MapString> {
            self.vars().globalname()
        }

        fn is_globalname(&self, name: &::csz::CStrThin) -> bool {
            self.globalname().is_some_and(|s| name == s.as_thin())
        }

        fn classname(&self) -> ::xash3d_server::str::MapString {
            self.vars().classname().unwrap()
        }

        fn is_classname(&self, name: &::csz::CStrThin) -> bool {
            name == self.classname().as_thin()
        }

        fn name(&self) -> ::xash3d_server::str::MapString {
            self.vars().target_name().unwrap_or_else(|| self.classname())
        }

        fn target(&self) -> Option<::xash3d_server::str::MapString> {
            self.vars().target()
        }

        fn object_caps(&self) -> ::xash3d_server::entity::ObjectCaps {
            ObjectCaps::ACROSS_TRANSITION
        }

        fn make_dormant(&mut self) {
            let ev = self.vars_mut().as_raw_mut();
            ev.flags_mut().insert(EdictFlags::DORMANT);
            ev.solid = SOLID_NOT;
            ev.movetype = MoveType::None.into();
            ev.effects_mut().insert(Effects::NODRAW);
            ev.nextthink = 0.0;
        }

        fn is_dormant(&self) -> bool {
            self.vars().flags().intersects(EdictFlags::DORMANT)
        }

        fn key_value(&mut self, data: &mut ::xash3d_server::entity::KeyValue) {
            data.set_handled(false);
        }

        fn precache(&mut self) {}

        fn spawn(&mut self) {}

        fn think(&mut self) {}

        #[allow(unused_variables)]
        fn touched(&mut self, other: &mut dyn ::xash3d_server::entity::Entity) {}

        #[allow(unused_variables)]
        fn used(
            &mut self,
            activator: &mut dyn ::xash3d_server::entity::Entity,
            caller: Option<&mut dyn ::xash3d_server::entity::Entity>,
            use_type: ::xash3d_server::entity::UseType,
            value: f32,
        ) {}

        #[allow(unused_variables)]
        fn blocked(&mut self, other: &mut dyn ::xash3d_server::entity::Entity) {}

        #[allow(unused_variables)]
        fn is_triggered(&self, activator: &dyn ::xash3d_server::entity::Entity) -> bool {
            true
        }

        fn override_reset(&mut self) {}

        fn set_object_collision_box(&mut self) {
            set_object_collision_box(self.vars_mut().as_raw_mut());
        }

        fn intersects(&self, other: &dyn ::xash3d_server::entity::Entity) -> bool {
            let a = self.vars().as_raw();
            let b = other.vars().as_raw();
            !(b.absmin.x() > a.absmax.x()
                || b.absmin.y() > a.absmax.y()
                || b.absmin.z() > a.absmax.z()
                || b.absmax.x() < a.absmin.x()
                || b.absmax.y() < a.absmin.y()
                || b.absmax.z() < a.absmin.z())
        }

        /// Removes this entity from the world.
        fn remove_from_world(&mut self) {
            if self.vars().flags().contains(EdictFlags::GRAPHED) {
                // TODO: remove from the world graph
                warn!("Entity::update_on_remove(): remove from the world graph is not implemented");
            }

            if let Some(globalname) = self.globalname() {
                self.global_state().set_entity_state(globalname, EntityState::Dead);
            }

            let ev = self.vars_mut().as_raw_mut();
            if ev.health > 0.0 {
                ev.health = 0.0;
                warn!("Entity::remove_from_world(): called with health > 0");
            }

            self.vars_mut().delayed_remove();
        }
    }
}

impl dyn Entity {
    pub fn downcast_ref<U: Entity + ?Sized + 'static>(&self) -> Option<&U> {
        self.private().downcast_ref::<U>()
    }

    pub fn downcast_mut<U: Entity + ?Sized + 'static>(&mut self) -> Option<&mut U> {
        self.private_mut().downcast_mut::<U>()
    }
}

#[deprecated(note = "moved to utils module")]
pub fn fire_targets(
    target_name: &CStrThin,
    activator: &mut dyn Entity,
    caller: Option<&mut dyn Entity>,
    use_type: UseType,
    value: f32,
) {
    crate::utils::fire_targets(target_name, use_type, value, activator, caller)
}

/// Base type for all entities.
#[derive(Debug)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct BaseEntity {
    #[cfg_attr(feature = "save", save(skip))]
    pub engine: ServerEngineRef,
    #[cfg_attr(feature = "save", save(skip))]
    pub global_state: GlobalStateRef,
    pub vars: EntityVars,
}

#[cfg(feature = "save")]
impl save::OnRestore for BaseEntity {
    fn on_restore(&mut self) {
        let ev = self.vars.as_raw();
        if let (true, Some(model)) = (ev.modelindex != 0, ev.model()) {
            let mins = ev.mins;
            let maxs = ev.maxs;
            let engine = self.engine();
            engine.precache_model(&model);
            engine.set_model(self, &model);
            engine.set_size(self, mins, maxs);
        }
    }
}

impl_entity_cast!(BaseEntity);

impl Entity for BaseEntity {
    fn private(&self) -> &PrivateData {
        PrivateData::from_edict(self.as_edict()).unwrap()
    }

    fn private_mut(&mut self) -> &mut PrivateData {
        PrivateData::from_edict_mut(self.as_edict_mut()).unwrap()
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }

    fn vars(&self) -> &EntityVars {
        &self.vars
    }

    fn vars_mut(&mut self) -> &mut EntityVars {
        &mut self.vars
    }
}

define_entity_trait! {
    pub trait EntityPlayer(delegate_player): (Entity) {
        fn select_spawn_point(&self) -> *mut ::xash3d_server::ffi::server::edict_s;

        fn pre_think(&mut self);

        fn post_think(&mut self);
    }
}

pub fn set_object_collision_box(ev: &mut entvars_s) {
    if ev.solid == SOLID_BSP && ev.angles != vec3_t::ZERO {
        let mut max = 0.0;
        for i in 0..3 {
            let v = fabsf(ev.mins[i]);
            if v > max {
                max = v;
            }
            let v = fabsf(ev.maxs[i]);
            if v > max {
                max = v;
            }
        }

        ev.absmin = ev.origin - vec3_t::splat(max);
        ev.absmax = ev.origin + vec3_t::splat(max);
    } else {
        ev.absmin = ev.origin + ev.mins; // TODO: should it be sub?
        ev.absmax = ev.origin + ev.maxs;
    }

    ev.absmin -= vec3_t::splat(1.0);
    ev.absmax += vec3_t::splat(1.0);
}

pub fn create_baseline(
    player: bool,
    eindex: c_int,
    baseline: &mut entity_state_s,
    ent: &mut edict_s,
    player_model_index: c_int,
    player_mins: vec3_t,
    player_maxs: vec3_t,
) {
    baseline.origin = ent.v.origin;
    baseline.angles = ent.v.angles;
    baseline.frame = ent.v.frame;
    baseline.skin = ent.v.skin as c_short;

    baseline.rendermode = ent.v.rendermode as c_int;
    baseline.renderamt = ent.v.renderamt as u8 as c_int;
    baseline.rendercolor.r = ent.v.rendercolor[0] as u8;
    baseline.rendercolor.g = ent.v.rendercolor[1] as u8;
    baseline.rendercolor.b = ent.v.rendercolor[2] as u8;
    baseline.renderfx = ent.v.renderfx as c_int;

    if player {
        baseline.mins = player_mins;
        baseline.maxs = player_maxs;

        baseline.colormap = eindex;
        baseline.modelindex = player_model_index;
        baseline.friction = 1.0;
        baseline.movetype = MoveType::Walk.into();

        baseline.scale = ent.v.scale;
        baseline.solid = SOLID_SLIDEBOX as c_short;
        baseline.framerate = 1.0;
        baseline.gravity = 1.0;
    } else {
        baseline.mins = ent.v.mins;
        baseline.maxs = ent.v.maxs;

        baseline.colormap = 0;
        baseline.modelindex = ent.v.modelindex;
        baseline.movetype = ent.v.movetype as c_int;

        baseline.scale = ent.v.scale;
        baseline.solid = ent.v.solid as c_short;
        baseline.framerate = ent.v.framerate;
        baseline.gravity = ent.v.gravity;
    }
}

// TODO: add safe wrapper for entvars_s and remove this trait
#[doc(hidden)]
pub trait EntityVarsExt {
    fn classname(&self) -> Option<MapString>;

    fn globalname(&self) -> Option<MapString>;

    fn model(&self) -> Option<MapString>;

    fn viewmodel(&self) -> Option<MapString>;

    fn weaponmodel(&self) -> Option<MapString>;

    fn flags(&self) -> &EdictFlags;

    fn flags_mut(&mut self) -> &mut EdictFlags;

    fn effects(&self) -> &Effects;

    fn effects_mut(&mut self) -> &mut Effects;
}

impl EntityVarsExt for entvars_s {
    fn classname(&self) -> Option<MapString> {
        // TODO: remove me
        let engine = unsafe { ServerEngineRef::new() };
        MapString::from_index(engine, self.classname)
    }

    fn globalname(&self) -> Option<MapString> {
        // TODO: remove me
        let engine = unsafe { ServerEngineRef::new() };
        MapString::from_index(engine, self.globalname)
    }

    fn model(&self) -> Option<MapString> {
        // TODO: remove me
        let engine = unsafe { ServerEngineRef::new() };
        MapString::from_index(engine, self.model)
    }

    fn viewmodel(&self) -> Option<MapString> {
        // TODO: remove me
        let engine = unsafe { ServerEngineRef::new() };
        MapString::from_index(engine, self.viewmodel)
    }

    fn weaponmodel(&self) -> Option<MapString> {
        // TODO: remove me
        let engine = unsafe { ServerEngineRef::new() };
        MapString::from_index(engine, self.weaponmodel)
    }

    fn flags(&self) -> &EdictFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    fn flags_mut(&mut self) -> &mut EdictFlags {
        unsafe { mem::transmute(&mut self.flags) }
    }

    fn effects(&self) -> &Effects {
        unsafe { mem::transmute(&self.effects) }
    }

    fn effects_mut(&mut self) -> &mut Effects {
        unsafe { mem::transmute(&mut self.effects) }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct StubEntity {
    base: BaseEntity,
    dump_key_value: bool,
}

impl StubEntity {
    pub fn new(base: BaseEntity, dump_key_value: bool) -> Self {
        Self {
            base,
            dump_key_value,
        }
    }
}

impl CreateEntity for StubEntity {
    fn create(base: BaseEntity) -> Self {
        Self::new(base, false)
    }
}

impl_entity_cast!(StubEntity);

impl Entity for StubEntity {
    delegate_entity!(base not { object_caps, key_value, spawn, touched, used, blocked });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        self.base.key_value(data);

        if self.dump_key_value && !data.handled() {
            let name = self.name();
            let key = data.key_name();
            let value = data.value();
            trace!("{name}: key={key} value={value}");
        }
    }

    fn spawn(&mut self) {
        let classname = self.classname();
        let name = self.vars().target_name();
        let target = self.vars().target();
        trace!("spawn {classname}({name:?}), target={target:?}");

        let engine = self.engine();
        let v = self.vars_mut();
        v.set_move_dir();
        let ev = v.as_raw_mut();
        ev.solid = SOLID_TRIGGER;
        ev.movetype = MoveType::Push.into();
        if let Some(model) = v.model() {
            engine.set_model(self, &model);
        }
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        let classname = self.classname();
        if let Some(name) = self.vars().target_name() {
            trace!("{classname}({name}) touched by {}", other.name());
        } else {
            trace!("{classname} touched by {}", other.name());
        }
    }

    fn used(
        &mut self,
        activator: &mut dyn Entity,
        _caller: Option<&mut dyn Entity>,
        use_type: UseType,
        value: f32,
    ) {
        let classname = self.classname();
        if let Some(name) = self.vars().target_name() {
            trace!(
                "{classname}({name}) used({use_type:?}, {value}) by {}",
                activator.name()
            );
        } else {
            trace!(
                "{classname} used({use_type:?}, {value}) by {}",
                activator.name()
            );
        }
    }

    fn blocked(&mut self, other: &mut dyn Entity) {
        let classname = self.classname();
        if let Some(name) = self.vars().target_name() {
            trace!("{classname}({name}) blocked by {}", other.name());
        } else {
            trace!("{classname} blocked by {}", other.name());
        }
    }
}
