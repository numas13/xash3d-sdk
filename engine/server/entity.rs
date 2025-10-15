mod macros;
mod private_data;
mod vars;

use core::{
    ffi::{c_int, c_short, c_void},
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
    str::FromStr,
};

use bitflags::bitflags;
use csz::CStrThin;
use xash3d_shared::{
    ffi::{
        common::{entity_state_s, vec3_t},
        server::{edict_s, entvars_s, KeyValueData},
    },
    math::fabsf,
    utils::cstr_or_none,
};

use crate::{
    engine::ServerEngineRef,
    global_state::{EntityState, GlobalStateRef},
    str::MapString,
};

#[cfg(feature = "save")]
use crate::save::{self, Restore, Save};

pub use xash3d_shared::entity::*;

pub use self::macros::*;
pub use self::private_data::*;
pub use self::vars::*;

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

impl Default for EntityOffset {
    fn default() -> Self {
        Self::WORLD_SPAWN
    }
}

impl EntityOffset {
    /// The world spawn entity offset.
    pub const WORLD_SPAWN: Self = Self(0);

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

    #[deprecated(note = "use is_world_spawn instead")]
    pub const fn is_first(&self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if the offset is for the world spawn entity.
    pub const fn is_world_spawn(&self) -> bool {
        self.0 == Self::WORLD_SPAWN.0
    }
}

/// A reference to an entity handle.
#[derive(Copy, Clone)]
pub struct EntityHandleRef<'a> {
    engine: ServerEngineRef,
    raw: NonNull<edict_s>,
    phantom: PhantomData<&'a edict_s>,
}

impl<'a> EntityHandleRef<'a> {
    /// Create a new entity handle reference.
    ///
    /// # Safety
    ///
    /// The pointer must be non-null and received from the engine.
    pub(crate) unsafe fn new_unchecked(engine: ServerEngineRef, raw: *mut edict_s) -> Self {
        Self {
            engine,
            raw: unsafe { NonNull::new_unchecked(raw) },
            phantom: PhantomData,
        }
    }

    /// Create a new entity handle reference if the pointer is non-null.
    ///
    /// # Safety
    ///
    /// The pointer must be received from the engine.
    pub unsafe fn new(engine: ServerEngineRef, raw: *mut edict_s) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self {
            engine,
            raw,
            phantom: PhantomData,
        })
    }

    /// Create a new entity handle if the pointer is non-null and it is not the world entity.
    ///
    /// # Safety
    ///
    /// The pointer must be received from the engine.
    pub unsafe fn new_not_world_spawn(engine: ServerEngineRef, raw: *mut edict_s) -> Option<Self> {
        let ent = unsafe { Self::new(engine, raw)? };
        if !ent.is_world_spawn() {
            Some(ent)
        } else {
            None
        }
    }

    /// Returns the raw pointer to this entity.
    pub fn as_ptr(&self) -> *mut edict_s {
        self.raw.as_ptr()
    }

    fn as_handle(&self) -> EntityHandle {
        unsafe { EntityHandle::new_unchecked(self.engine, self.as_ptr()) }
    }

    /// Returns `true` if this is the world spawn entity.
    pub fn is_world_spawn(&self) -> bool {
        self.as_handle().is_world_spawn()
    }

    /// Returns a next entity in the same PVS as this entity.
    pub fn next(&self) -> Option<EntityHandleRef<'a>> {
        unsafe { Self::new_not_world_spawn(self.engine, self.raw.as_ref().v.chain) }
    }

    /// Returns variables of this entity.
    pub fn vars(&self) -> EntityVars {
        self.as_handle().vars()
    }

    /// Returns an index of this entity.
    pub fn entity_index(&self) -> EntityIndex {
        self.engine.get_entity_index(self)
    }

    /// Returns an offset of this entity.
    pub fn entity_offset(&self) -> EntityOffset {
        self.engine.get_entity_offset(self)
    }
}

impl From<EntityHandleRef<'_>> for EntityHandle {
    fn from(value: EntityHandleRef) -> Self {
        value.as_handle()
    }
}

/// An owned handle to an entity.
#[derive(Copy, Clone)]
pub struct EntityHandle {
    engine: ServerEngineRef,
    raw: NonNull<edict_s>,
}

impl EntityHandle {
    /// Create a new entity handle.
    ///
    /// # Safety
    ///
    /// The pointer must be non-null and received from the engine.
    pub unsafe fn new_unchecked(engine: ServerEngineRef, raw: *mut edict_s) -> Self {
        Self {
            engine,
            raw: unsafe { NonNull::new_unchecked(raw) },
        }
    }

    /// Create a new entity handle if the pointer is non-null.
    ///
    /// # Safety
    ///
    /// The pointer must be received from the engine.
    pub unsafe fn new(engine: ServerEngineRef, raw: *mut edict_s) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self { engine, raw })
    }

    /// Create a new entity handle if the pointer is non-null and it is not the world entity.
    ///
    /// # Safety
    ///
    /// The pointer must be received from the engine.
    pub unsafe fn new_not_world_spawn(engine: ServerEngineRef, raw: *mut edict_s) -> Option<Self> {
        let ent = unsafe { Self::new(engine, raw)? };
        if !ent.is_world_spawn() {
            Some(ent)
        } else {
            None
        }
    }

    /// Returns the raw pointer to this entity.
    pub fn as_ptr(&self) -> *mut edict_s {
        self.raw.as_ptr()
    }

    /// Returns `true` if this is the world spawn entity.
    pub fn is_world_spawn(&self) -> bool {
        self.engine.get_entity_offset(self).is_world_spawn()
    }

    /// Returns a next entity in the same PVS as this entity.
    pub fn next(&self) -> Option<EntityHandle> {
        unsafe { Self::new_not_world_spawn(self.engine, self.raw.as_ref().v.chain) }
    }

    /// Sets a private data for this entity.
    ///
    /// # Safety
    ///
    /// The private data must be allocated with
    /// [ServerEngine::alloc_ent_private_data](crate::engine::ServerEngine::alloc_ent_private_data).
    pub unsafe fn set_private_data(&mut self, private_data: *mut c_void) {
        unsafe {
            self.raw.as_mut().pvPrivateData = private_data;
        }
    }

    /// Returns variables of this entity.
    pub fn vars(&self) -> EntityVars {
        unsafe {
            let v = ptr::addr_of!(self.raw.as_ref().v).cast_mut();
            EntityVars::from_raw(self.engine, self.engine.global_state_ref(), v)
        }
    }

    /// Returns an index of this entity.
    pub fn entity_index(&self) -> EntityIndex {
        self.engine.get_entity_index(self)
    }

    /// Returns an offset of this entity.
    pub fn entity_offset(&self) -> EntityOffset {
        self.engine.get_entity_offset(self)
    }
}

impl<'a> From<&'a EntityHandle> for EntityHandleRef<'a> {
    fn from(value: &'a EntityHandle) -> Self {
        unsafe { EntityHandleRef::new_unchecked(value.engine, value.as_ptr()) }
    }
}

pub(crate) trait AsEntityHandleSealed {
    fn as_entity_handle(&self) -> *mut edict_s;
}

impl AsEntityHandleSealed for edict_s {
    fn as_entity_handle(&self) -> *mut edict_s {
        (self as *const edict_s).cast_mut()
    }
}

impl AsEntityHandleSealed for entvars_s {
    fn as_entity_handle(&self) -> *mut edict_s {
        self.pContainingEntity
    }
}

impl AsEntityHandleSealed for EntityHandleRef<'_> {
    fn as_entity_handle(&self) -> *mut edict_s {
        self.as_ptr()
    }
}

impl AsEntityHandleSealed for EntityHandle {
    fn as_entity_handle(&self) -> *mut edict_s {
        self.as_ptr()
    }
}

impl AsEntityHandleSealed for EntityVars {
    fn as_entity_handle(&self) -> *mut edict_s {
        self.containing_entity_raw()
    }
}

impl<T: Entity> AsEntityHandleSealed for T {
    fn as_entity_handle(&self) -> *mut edict_s {
        self.vars().as_entity_handle()
    }
}

impl AsEntityHandleSealed for &'_ dyn Entity {
    fn as_entity_handle(&self) -> *mut edict_s {
        self.vars().as_entity_handle()
    }
}

impl AsEntityHandleSealed for &'_ mut dyn Entity {
    fn as_entity_handle(&self) -> *mut edict_s {
        self.vars().as_entity_handle()
    }
}

#[allow(private_bounds)]
pub trait AsEntityHandle: AsEntityHandleSealed {}

impl<T: AsEntityHandleSealed> AsEntityHandle for T {}

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

    pub fn parse<T: FromStr>(&self) -> Result<T, T::Err> {
        self.value_str().parse()
    }

    pub fn parse_or<T: FromStr>(&self, or: T) -> T {
        self.parse().unwrap_or(or)
    }

    pub fn parse_or_default<T: FromStr + Default>(&self) -> T {
        self.parse().unwrap_or_default()
    }

    /// Returns `true` if the server DLL knows the key name.
    pub fn handled(&self) -> bool {
        self.raw.fHandled != 0
    }

    pub fn set_handled(&mut self, handled: bool) {
        self.raw.fHandled = handled.into();
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

impl ObjectCaps {
    pub fn is_continuous_use(&self) -> bool {
        self.intersects(Self::CONTINUOUS_USE)
    }

    pub fn is_impulse_use(&self) -> bool {
        self.intersects(Self::IMPULSE_USE)
    }

    pub fn is_on_off_use(&self) -> bool {
        self.intersects(Self::ONOFF_USE)
    }

    pub fn is_player_use(&self) -> bool {
        self.intersects(Self::IMPULSE_USE | Self::CONTINUOUS_USE | Self::ONOFF_USE)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum UseType {
    #[default]
    Off,
    On,
    Set(f32),
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Gib {
    /// Gib if entity was overkilled.
    #[default]
    Normal,
    /// Never gib, no matter how much death damage is done (freezing, etc).
    Never,
    /// Always gib (Houndeye Shock, Barnacle Bite).
    Always,
}

pub trait EntityCast: 'static {
    fn as_player(&self) -> Option<&dyn EntityPlayer>;
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
    pub trait Entity(delegate_entity): (EntitySaveRestore + EntityCast + AsEntityHandle) {
        fn private(&self) -> &::xash3d_server::entity::PrivateData;

        /// Returns a reference to the server engine.
        fn engine(&self) -> ::xash3d_server::engine::ServerEngineRef;

        fn global_state(&self) -> ::xash3d_server::global_state::GlobalStateRef;

        /// Returns a shared reference to entity variables.
        fn vars(&self) -> &::xash3d_server::entity::EntityVars;

        /// Returns an index of this entity.
        fn entity_index(&self) -> ::xash3d_server::entity::EntityIndex {
            self.engine().get_entity_index(self.vars())
        }

        /// Returns an offset of this entity.
        fn entity_offset(&self) -> ::xash3d_server::entity::EntityOffset {
            self.engine().get_entity_offset(self.vars())
        }

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

        fn make_dormant(&self) {
            let v = self.vars();
            v.with_flags(|f| f | EdictFlags::DORMANT);
            v.set_solid(Solid::Not);
            v.set_move_type(MoveType::None);
            v.with_effects(|f| f | Effects::NODRAW);
            v.stop_thinking();
        }

        fn is_dormant(&self) -> bool {
            self.vars().flags().intersects(EdictFlags::DORMANT)
        }

        fn key_value(&self, data: &mut ::xash3d_server::entity::KeyValue) {
            data.set_handled(false);
        }

        fn precache(&self) {}

        fn spawn(&self) {}

        fn think(&self) {}

        #[allow(unused_variables)]
        fn touched(&self, other: &dyn ::xash3d_server::entity::Entity) {}

        #[allow(unused_variables)]
        fn used(
            &self,
            use_type: ::xash3d_server::entity::UseType,
            activator: Option<&dyn ::xash3d_server::entity::Entity>,
            caller: &dyn ::xash3d_server::entity::Entity,
        ) {}

        #[allow(unused_variables)]
        fn blocked(&self, other: &dyn ::xash3d_server::entity::Entity) {}

        #[allow(unused_variables)]
        fn is_triggered(&self, activator: &dyn ::xash3d_server::entity::Entity) -> bool {
            true
        }

        #[allow(unused_variables)]
        fn take_health(
            &self,
            health: f32,
            damage_type: ::xash3d_server::entity::DamageFlags,
        ) -> bool {
            let v = self.vars();
            if v.take_damage() == TakeDamage::No {
                return false;
            }
            if v.health() == v.max_health() {
                return false;
            }
            v.set_health((v.health() + health).min(v.max_health()));
            true
        }

        #[allow(unused_variables)]
        fn take_damage(
            &self,
            damage: f32,
            damage_type: ::xash3d_server::entity::DamageFlags,
            inflictor: &::xash3d_server::entity::EntityVars,
            attacker: Option<&::xash3d_server::entity::EntityVars>,
        ) -> bool {
            let classname = self.classname();
            match (inflictor.classname(), attacker.and_then(|i| i.classname())) {
                (Some(from), None) => {
                    warn!("{classname}: take_damage from {from} is not implemented yet");
                }
                (Some(from), Some(from2)) => {
                    warn!("{classname}: take_damage from {from}({from2}) is not implemented yet");
                }
                _ => {
                    warn!("{classname}: take_damage is not implemented yet");
                }
            }
            false
        }

        #[allow(unused_variables)]
        fn killed(
            &self,
            attacker: &::xash3d_server::entity::EntityVars,
            gib: ::xash3d_server::entity::Gib,
        ) {
            let v = self.vars();
            v.set_take_damage(TakeDamage::No);
            v.set_dead(Dead::Yes);
            self.remove_from_world();
        }

        fn override_reset(&self) {}

        fn set_object_collision_box(&self) {
            set_object_collision_box(unsafe { &mut *self.vars().as_mut_ptr() });
        }

        fn intersects(&self, other: &dyn ::xash3d_server::entity::Entity) -> bool {
            let a = self.vars();
            let a_min = a.abs_min();
            let a_max = a.abs_max();
            let b = other.vars();
            let b_min = b.abs_min();
            let b_max = b.abs_max();
            !(     b_min.x > a_max.x
                || b_min.y > a_max.y
                || b_min.z > a_max.z
                || b_max.x < a_min.x
                || b_max.y < a_min.y
                || b_max.z < a_min.z)
        }

        /// Removes this entity from the world.
        fn remove_from_world(&self) {
            let v = self.vars();

            if v.flags().intersects(EdictFlags::KILLME) {
                warn!("{}: trying to remove dead entity", self.classname());
                return;
            }

            if v.flags().intersects(EdictFlags::GRAPHED) {
                // TODO: remove from the world graph
                warn!("Entity::update_on_remove(): remove from the world graph is not implemented");
            }

            if let Some(globalname) = self.globalname() {
                self.global_state().set_entity_state(globalname, EntityState::Dead);
            }

            if v.health() > 0.0 {
                v.set_health(0.0);
                warn!("Entity::remove_from_world(): called with health > 0");
            }

            v.delayed_remove();
        }
    }
}

impl dyn Entity {
    pub fn downcast_ref<U: Entity + ?Sized + 'static>(&self) -> Option<&U> {
        self.private().downcast_ref::<U>()
    }
}

/// Base type for all entities.
#[derive(Debug)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct BaseEntity {
    pub vars: EntityVars,
}

#[cfg(feature = "save")]
impl save::OnRestore for BaseEntity {
    fn on_restore(&self) {
        let v = self.vars();
        if let (true, Some(model)) = (v.model_index().is_some(), v.model_name()) {
            let mins = v.min_size();
            let maxs = v.max_size();
            let engine = self.engine();
            engine.precache_model(model);
            engine.set_model(self, model);
            engine.set_size(self, mins, maxs);
        }
    }
}

impl_entity_cast!(BaseEntity);

impl Entity for BaseEntity {
    fn private(&self) -> &PrivateData {
        let edict = unsafe { &*self.as_entity_handle() };
        PrivateData::from_edict(edict).unwrap()
    }

    fn engine(&self) -> ServerEngineRef {
        self.vars().engine()
    }

    fn global_state(&self) -> GlobalStateRef {
        self.vars().global_state()
    }

    fn vars(&self) -> &EntityVars {
        &self.vars
    }
}

define_entity_trait! {
    pub trait EntityPlayer(delegate_player): (Entity) {
        fn select_spawn_point(&self) -> ::xash3d_server::entity::EntityHandle;

        fn pre_think(&self);

        fn post_think(&self);

        #[allow(unused_variables)]
        fn set_geiger_range(&self, range: f32) {}

        fn is_observer(&self) -> bool {
            self.vars().iuser1() != 0
        }
    }
}

pub fn set_object_collision_box(ev: &mut entvars_s) {
    if ev.solid == Solid::Bsp.into() && ev.angles != vec3_t::ZERO {
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
        baseline.solid = Solid::SlideBox.into_raw() as c_short;
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

    fn key_value(&self, data: &mut KeyValue) {
        self.base.key_value(data);

        if self.dump_key_value && !data.handled() {
            let name = self.name();
            let key = data.key_name();
            let value = data.value();
            trace!("{name}: key={key} value={value}");
        }
    }

    fn spawn(&self) {
        let classname = self.classname();
        let name = self.vars().target_name();
        let target = self.vars().target();
        trace!("spawn {classname}({name:?}), target={target:?}");

        let v = self.vars();
        v.set_move_dir_from_angles();
        v.set_solid(Solid::Trigger);
        v.set_move_type(MoveType::Push);
        v.reload_model();
    }

    fn touched(&self, other: &dyn Entity) {
        let classname = self.classname();
        if let Some(name) = self.vars().target_name() {
            trace!("{classname}({name}) touched by {}", other.name());
        } else {
            trace!("{classname} touched by {}", other.name());
        }
    }

    fn used(&self, use_type: UseType, _activator: Option<&dyn Entity>, caller: &dyn Entity) {
        let classname = self.classname();
        if let Some(name) = self.vars().target_name() {
            trace!(
                "{classname}({name}) used({use_type:?}) by {}",
                caller.name()
            );
        } else {
            trace!("{classname} used({use_type:?} by {}", caller.name());
        }
    }

    fn blocked(&self, other: &dyn Entity) {
        let classname = self.classname();
        if let Some(name) = self.vars().target_name() {
            trace!("{classname}({name}) blocked by {}", other.name());
        } else {
            trace!("{classname} blocked by {}", other.name());
        }
    }
}
