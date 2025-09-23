mod private_data;

use core::{
    ffi::{c_int, c_short},
    mem,
};

use alloc::rc::Rc;
use bitflags::bitflags;
use csz::CStrThin;
use shared::{
    consts::{SOLID_BSP, SOLID_NOT, SOLID_SLIDEBOX},
    ffi::{
        common::{entity_state_s, vec3_t},
        server::{edict_s, entvars_s, KeyValueData, TYPEDESCRIPTION},
    },
    macros::const_assert_size_of_field_eq,
};

use crate::{
    engine::ServerEngineRef,
    game_rules::{GameRules, GameRulesRef},
    save::{self, KeyValueDataExt, SaveReader, SaveWriter},
    str::MapString,
};

pub use self::private_data::*;
pub use shared::entity::*;

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

    pub fn effects(&self) -> Effects {
        Effects::from_bits_retain(self.as_raw().effects)
    }

    pub fn effects_mut(&mut self) -> &mut Effects {
        const_assert_size_of_field_eq!(Effects, entvars_s, effects);
        unsafe { mem::transmute(&mut self.as_raw_mut().effects) }
    }
}

/// Common data for entities.
#[derive(Debug)]
pub struct BaseEntity {
    pub engine: ServerEngineRef,
    pub game_rules: GameRulesRef,
    pub vars: EntityVars,
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

pub trait EntityCast: 'static {
    /// Returns a shared reference to a base entity data.
    fn as_base(&self) -> &BaseEntity;

    /// Returns a mutable reference to a base entity data.
    fn as_base_mut(&mut self) -> &mut BaseEntity;

    fn as_player(&self) -> Option<&dyn EntityPlayer>;
    fn as_player_mut(&mut self) -> Option<&mut dyn EntityPlayer>;

    fn as_delay(&self) -> Option<&dyn EntityDelay>;
    fn as_delay_mut(&mut self) -> Option<&mut dyn EntityDelay>;

    fn as_animating(&self) -> Option<&dyn EntityAnimating>;
    fn as_animating_mut(&mut self) -> Option<&mut dyn EntityAnimating>;

    fn as_toggle(&self) -> Option<&dyn EntityToggle>;
    fn as_toggle_mut(&mut self) -> Option<&mut dyn EntityToggle>;

    fn as_monster(&self) -> Option<&dyn EntityMonster>;
    fn as_monster_mut(&mut self) -> Option<&mut dyn EntityMonster>;
}

/// The base trait for all entities.
pub trait Entity: EntityCast + AsEdict {
    fn private(&self) -> &PrivateData {
        PrivateData::from_edict(self.as_edict()).unwrap()
    }

    fn private_mut(&mut self) -> &mut PrivateData {
        PrivateData::from_edict_mut(self.as_edict_mut()).unwrap()
    }

    /// Returns a reference to the server engine.
    fn engine(&self) -> ServerEngineRef {
        self.as_base().engine
    }

    fn game_rules(&self) -> Option<Rc<dyn GameRules>> {
        self.as_base().game_rules.get()
    }

    /// Returns a shared reference to entity variables.
    fn vars(&self) -> &EntityVars {
        &self.as_base().vars
    }

    /// Returns a mutable reference to entity variables.
    fn vars_mut(&mut self) -> &mut EntityVars {
        &mut self.as_base_mut().vars
    }

    fn globalname(&self) -> MapString {
        self.vars().globalname().unwrap()
    }

    fn is_globalname(&self, name: &CStrThin) -> bool {
        name == self.globalname().as_thin()
    }

    fn classname(&self) -> MapString {
        self.vars().classname().unwrap()
    }

    fn is_classname(&self, name: &CStrThin) -> bool {
        name == self.classname().as_thin()
    }

    fn object_caps(&self) -> ObjectCaps {
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

    fn fields(&self) -> &'static [TYPEDESCRIPTION] {
        &[]
    }

    fn save(&mut self) {}

    fn restore(&mut self) {}

    fn save_fields(&mut self, _save: &mut SaveWriter) -> save::Result<()> {
        self.save();
        // TODO: Entity::save_fields
        debug!("TODO: save {:?}", self.classname());
        Ok(())
    }

    fn restore_fields(&mut self, restore: &mut SaveReader) -> save::Result<()> {
        restore.read_ent_vars(c"ENTVARS", self.vars_mut().as_raw_mut())?;

        let fields = self.fields();
        restore.read_fields(c"BASE", self as *mut _ as *mut _, fields)?;

        let ev = self.vars_mut().as_raw();
        if let (true, Some(model)) = (ev.modelindex != 0, ev.model()) {
            let mins = ev.mins;
            let maxs = ev.maxs;
            let engine = self.engine();
            engine.precache_model(&model);
            engine.set_model(self.as_edict_mut(), &model);
            engine.set_size(self.as_edict_mut(), mins, maxs);
        }

        self.restore();
        Ok(())
    }

    fn key_value(&mut self, data: &mut KeyValueData) {
        let class_name = data.class_name();
        let key_name = data.key_name();
        let value = data.value();
        debug!(
            "{}::key_value({class_name:?}, {key_name}, {value})",
            self.classname()
        );
        data.set_handled(true);
    }

    fn precache(&mut self) {}

    fn spawn(&mut self) {}

    fn think(&mut self) {}

    fn touched(&mut self, other: &mut dyn Entity) {
        let touched = self.classname();
        let other = other.classname();
        trace!("touched {touched} by {other}");
    }

    fn used(&mut self, other: &mut dyn Entity) {
        let touched = self.classname();
        let other = other.classname();
        trace!("used {touched} by {other}");
    }

    fn blocked(&mut self, other: &mut dyn Entity) {
        let touched = self.classname();
        let other = other.classname();
        trace!("blocked {touched} by {other}");
    }

    fn override_reset(&mut self) {}

    fn set_object_collision_box(&mut self) {
        set_object_collision_box(self.vars_mut().as_raw_mut());
    }

    fn intersects(&self, other: &dyn Entity) -> bool {
        let a = self.vars().as_raw();
        let b = other.vars().as_raw();
        !(b.absmin.x() > a.absmax.x()
            || b.absmin.y() > a.absmax.y()
            || b.absmin.z() > a.absmax.z()
            || b.absmax.x() < a.absmin.x()
            || b.absmax.y() < a.absmin.y()
            || b.absmax.z() < a.absmin.z())
    }

    // TODO: BaseEntity::classify

    // TODO: BaseEntity::death_notice
}

impl dyn Entity {
    pub fn downcast_ref<U: Entity + ?Sized + 'static>(&self) -> Option<&U> {
        self.private().downcast_ref::<U>()
    }

    pub fn downcast_mut<U: Entity + ?Sized + 'static>(&mut self) -> Option<&mut U> {
        self.private_mut().downcast_mut::<U>()
    }
}

pub trait EntityPlayer: Entity {
    fn select_spawn_point(&self) -> *mut edict_s;

    fn pre_think(&mut self) {}

    fn post_think(&mut self) {}
}

pub trait EntityDelay: Entity {}
pub trait EntityAnimating: EntityDelay {}
pub trait EntityToggle: EntityAnimating {}
pub trait EntityMonster: EntityToggle {}

pub fn set_object_collision_box(ev: &mut entvars_s) {
    if ev.solid == SOLID_BSP && ev.angles != vec3_t::ZERO {
        let mut max = 0.0;
        for i in 0..3 {
            let v = ev.mins[i].abs();
            if v > max {
                max = v;
            }
            let v = ev.maxs[i].abs();
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

/// Return `Some(&dyn Trait)` if the given type implements the trait.
///
/// # Examples
///
/// ```
/// use xash3d_server::entity::static_trait_cast;
///
/// trait Armor {}
/// trait Weapon {}
///
/// struct Crowbar;
/// impl Weapon for Crowbar {}
///
/// let crowbar = Crowbar;
/// assert!(static_trait_cast!(Crowbar, Armor, &crowbar).is_none());
/// assert!(static_trait_cast!(Crowbar, Weapon, &crowbar).is_some());
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! static_trait_cast {
    ($ty:ty, $trait:path, $value:expr $(, $mut:ident)?) => ({
        #[allow(dead_code)]
        trait NoImpl {
            // called if trait is not implemented
            fn cast<V>(_: &$($mut)? V) -> Option<&$($mut)? dyn $trait> { None }
        }
        impl<T> NoImpl for T {}

        struct MaybeImpl<V>(core::marker::PhantomData<V>);
        #[allow(dead_code)]
        impl<V: $trait> MaybeImpl<V> {
            // called if trait is implemented
            fn cast(value: &$($mut)? V) -> Option<&$($mut)? dyn $trait> { Some(value) }
        }

        MaybeImpl::<$ty>::cast($value)
    });
}
pub use static_trait_cast;

/// Auto-implement a cast trait for a given type.
///
/// # Examples
///
/// ```
/// use xash3d_server::entity::{BaseEntity, EntityCast, impl_entity_cast};
///
/// trait MyToggle {}
/// trait MyMonster {}
///
/// trait MyCast: EntityCast {
///     fn as_my_toggle(&self) -> Option<&dyn MyToggle>;
///     fn as_my_toggle_mut(&mut self) -> Option<&mut dyn MyToggle>;
///
///     fn as_my_monster(&self) -> Option<&dyn MyMonster>;
///     fn as_my_monster_mut(&mut self) -> Option<&mut dyn MyMonster>;
/// }
///
/// macro_rules! impl_my_cast {
///     ($ty:ty) => {
///         impl MyCast for $ty {
///             xash3d_server::entity::impl_cast!{
///                 $ty {
///                     as_my_toggle, as_my_toggle_mut -> MyToggle;
///                     as_my_monster, as_my_monster_mut -> MyMonster;
///                 }
///             }
///         }
///     };
/// }
///
/// struct Zombie {
///     base: BaseEntity,
/// }
///
/// // impl EntityCast for Zombie { ... }
/// impl_entity_cast!(Zombie);
///
/// // impl MyCast for Zombie { ... }
/// impl_my_cast!(Zombie);
///
/// impl MyMonster for Zombie {}
///
/// // initialize to zeroes only for test purpose
/// let zombie: Zombie = unsafe { core::mem::zeroed() };
///
/// assert!(zombie.as_my_toggle().is_none());
/// assert!(zombie.as_my_monster().is_some());
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! impl_cast {
    ($ty:ty {
        $( $(#[$attr:meta])* $as_ref:ident, $as_mut:ident -> $to:path;)*
    }) => {
        $(
            $(#[$attr])*
            fn $as_ref(&self) -> Option<&dyn $to> {
                $crate::entity::static_trait_cast!($ty, $to, self)
            }

            $(#[$attr])*
            fn $as_mut(&mut self) -> Option<&mut dyn $to> {
                $crate::entity::static_trait_cast!($ty, $to, self, mut)
            }
        )*
    };
}
#[doc(inline)]
pub use impl_cast;

/// Implement the [EntityCast] trait for given types.
///
/// # Examples
///
/// ```
/// use xash3d_server::entity::{BaseEntity, EntityCast, impl_entity_cast};
///
/// struct Item {
///     base: BaseEntity,
/// }
///
/// // impl EntityCast for Item {
/// //      impl_entity_cast!(base Item);
/// //      impl_entity_cast!(cast Item);
/// // }
/// impl_entity_cast!(Item);
///
/// struct Battery {
///     item: Item,
/// }
///
/// // implement as_base/as_base_mut manually
/// impl EntityCast for Battery {
///     impl_entity_cast!(cast Battery);
///
///     fn as_base(&self) -> &BaseEntity {
///         self.item.as_base()
///     }
///
///     fn as_base_mut(&mut self) -> &mut BaseEntity {
///         self.item.as_base_mut()
///     }
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! impl_entity_cast {
    (base $ty:ty) => {
        fn as_base(&self) -> &$crate::entity::BaseEntity {
            &self.base
        }

        fn as_base_mut(&mut self) -> &mut $crate::entity::BaseEntity {
            &mut self.base
        }
    };
    (cast $ty:ty) => {
        $crate::entity::impl_cast! {
            $ty {
                as_player, as_player_mut -> $crate::entity::EntityPlayer;
                as_delay, as_delay_mut -> $crate::entity::EntityDelay;
                as_animating, as_animating_mut -> $crate::entity::EntityAnimating;
                as_toggle, as_toggle_mut -> $crate::entity::EntityToggle;
                as_monster, as_monster_mut -> $crate::entity::EntityMonster;
            }
        }
    };
    ($(#[$attr:meta])* $ty:ty) => {
        $(#[$attr])*
        impl $crate::entity::EntityCast for $ty {
            $crate::entity::impl_entity_cast!(base $ty);
            $crate::entity::impl_entity_cast!(cast $ty);
        }
    };
}
#[doc(inline)]
pub use impl_entity_cast;

/// Links an entity with the given name to the engine.
///
/// # Examples
///
/// ```
/// use core::marker::PhantomData;
/// use xash3d_server::entity::{
///     Entity, BaseEntity, CreateEntity, PrivateEntity,
///     impl_entity_cast, link_entity,
/// };
///
/// // define a private wrapper for our entities
/// struct Private<T>(PhantomData<T>);
///
/// impl<T: Entity> PrivateEntity for Private<T> {
///     type Entity = T;
/// }
///
/// // define a player entity
/// struct Player {
///     base: BaseEntity,
/// }
///
/// impl_entity_cast!(Player);
///
/// impl CreateEntity for Player {
///     fn create(base: BaseEntity) -> Self {
///         Self { base }
///     }
/// }
///
/// impl Entity for Player {}
///
/// // link the player entity to the engine
/// link_entity!(player, Private<Player>);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! link_entity {
    ($name:ident, $private:ty $(,)?) => {
        $crate::entity::link_entity!(
            $name,
            $private,
            <$private as $crate::entity::PrivateEntity>::Entity::create,
        );
    };
    ($name:ident, $private:ty, $init:expr $(,)?) => {
        #[no_mangle]
        unsafe extern "C" fn $name(ev: *mut $crate::ffi::server::entvars_s) {
            use $crate::{
                engine::ServerEngineRef,
                entity::{CreateEntity, PrivateData, PrivateEntity},
            };
            unsafe {
                let engine = ServerEngineRef::new();
                PrivateData::create_with::<$private, _>(engine, ev, $init);
            }
        }
    };
}
#[doc(inline)]
pub use link_entity;
