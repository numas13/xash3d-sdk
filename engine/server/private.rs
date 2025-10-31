use core::{
    any::TypeId,
    ffi::c_void,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr,
};

use xash3d_shared::ffi::server::{edict_s, entvars_s};

use crate::{
    entity::{BaseEntity, EntityCast, EntityHandle, EntityItem, EntityPlayer, EntityVars},
    global_state::GlobalStateRef,
    prelude::*,
};

/// A virtual table for a private data type.
#[derive(Copy, Clone)]
#[repr(C)]
struct PrivateDataVtable<T> {
    drop_in_place: unsafe fn(*mut T),
    as_entity: fn(&T) -> &dyn Entity,
    downcast: unsafe fn(&T, TypeId, *mut ()) -> bool,
}

impl<T: Entity> PrivateDataVtable<T> {
    /// Returns a private data `vtable` for this type.
    fn new<P: PrivateEntity<Entity = T>>() -> &'static PrivateDataVtable<T> {
        fn as_entity<T: Entity>(value: &T) -> &dyn Entity {
            value
        }

        unsafe fn downcast<P: PrivateEntity>(
            value: &P::Entity,
            type_id: TypeId,
            ret: *mut (),
        ) -> bool {
            let t = unsafe { Downcast::new(value, type_id, ret) };
            t.downcast::<P::Entity>(|i| Some(i))
                || t.downcast::<dyn Entity>(|i| Some(i))
                || t.downcast::<dyn EntityPlayer>(|i| i.as_player())
                || t.downcast::<dyn EntityItem>(|i| i.as_item())
                || P::downcast(&t)
        }

        &PrivateDataVtable {
            drop_in_place: ptr::drop_in_place::<T>,
            as_entity: as_entity::<P::Entity>,
            downcast: downcast::<P>,
        }
    }
}

pub struct Downcast<'a, T> {
    value: &'a T,
    type_id: TypeId,
    ret: *mut (),
}

impl<'a, T> Downcast<'a, T> {
    unsafe fn new(value: &'a T, type_id: TypeId, ret: *mut ()) -> Self {
        Self {
            value,
            type_id,
            ret,
        }
    }

    #[must_use]
    pub fn downcast<U: ?Sized + 'static>(&self, cast: impl Fn(&'a T) -> Option<&'a U>) -> bool {
        self.type_id == TypeId::of::<U>()
            && cast(self.value)
                .map(|i| unsafe {
                    self.ret.cast::<&U>().write(i);
                })
                .is_some()
    }
}

/// A type can be stored in an entity's private data.
///
/// Use [impl_private] or [export_entity](crate::export::export_entity) macros to auto-implement
/// this trait.
pub trait PrivateEntity: Sized + 'static {
    type Entity: Entity;

    /// Downcast this entity to a custom entity trait.
    ///
    /// Downcast to the type and [Entity] trait is not needed because it will be done by
    /// [PrivateData].
    ///
    /// # Examples
    ///
    /// ```
    /// use xash3d_server::{
    ///     prelude::*,
    ///     entity::{impl_entity_cast, delegate_entity, BaseEntity, EntityItem},
    ///     private::{Downcast, PrivateEntity},
    /// };
    /// use log::info;
    ///
    /// #[derive(Save, Restore)]
    /// struct Pen {
    ///     base: BaseEntity,
    /// }
    ///
    /// impl_entity_cast!(Pen);
    ///
    /// impl Pen {
    ///     fn draw(&self) {
    ///         let name = self.pretty_name();
    ///         info!("{name}: draw");
    ///     }
    /// }
    ///
    /// // Pen is an entity
    /// impl Entity for Pen {
    ///     delegate_entity!(base);
    /// }
    ///
    /// // Pen is an item entity
    /// impl EntityItem for Pen {
    ///     fn try_give(&self, other: &dyn Entity) -> bool {
    ///         let name = self.pretty_name();
    ///         info!("{name}: can not give to {}", other.pretty_name());
    ///         false
    ///     }
    /// }
    ///
    /// // define a custom entity trait
    /// trait EntityTool: EntityItem {
    ///     fn repair(&self);
    /// }
    ///
    /// // Pen is a tool entity
    /// impl EntityTool for Pen {
    ///     fn repair(&self) {
    ///         let name = self.pretty_name();
    ///         info!("{name}: you can draw now!");
    ///     }
    /// }
    ///
    /// // can be implemented by impl_private macro
    /// impl PrivateEntity for Pen {
    ///     type Entity = Self;
    ///
    ///     fn downcast(t: &Downcast<Self::Entity>) -> bool {
    ///         false
    ///             || t.downcast::<dyn EntityItem>(|i| Some(i))
    ///             || t.downcast::<dyn EntityTool>(|i| Some(i))
    ///     }
    /// }
    ///
    /// fn repair_tool(entity: &dyn Entity) {
    ///     if let Some(tool) = entity.downcast_ref::<dyn EntityTool>() {
    ///         tool.repair();
    ///     }
    /// }
    ///
    /// fn draw_something_with_pen(entity: &dyn Entity) {
    ///     if let Some(pen) = entity.downcast_ref::<Pen>() {
    ///         pen.draw();
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn downcast(t: &Downcast<'_, Self::Entity>) -> bool {
        false
    }
}

/// Private data memory representation.
#[repr(C)]
struct Data<T = ()> {
    vtable: *const PrivateDataVtable<T>,
    offset: u16,
    data: T,
}

impl<T: Entity> Data<T> {
    unsafe fn new_in<P: PrivateEntity>(raw: *mut Self, data: T) {
        let raw = unsafe { &mut *raw };
        raw.vtable = PrivateDataVtable::new::<P>() as *const _ as *const _;
        raw.offset = mem::offset_of!(Self, data) as u16;
        unsafe {
            ptr::write(&mut raw.data, data);
        }
    }
}

impl<T> Data<T> {
    fn as_ptr(&self) -> *mut T {
        (self as *const Self as *mut u8)
            .wrapping_add(self.offset as usize)
            .cast()
    }

    fn vtable(&self) -> &PrivateDataVtable<T> {
        unsafe { &*self.vtable }
    }

    unsafe fn drop_in_place(raw: *mut Self) {
        unsafe {
            let raw = &mut *raw;
            (raw.vtable().drop_in_place)(raw.as_ptr());
        }
    }
}

/// A private data of an entity.
#[repr(transparent)]
pub struct PrivateData {
    data: *mut Data,
}

impl PrivateData {
    fn alloc<P>(engine: ServerEngineRef, ent: &EntityHandle, value: P::Entity) -> Self
    where
        P: PrivateEntity,
    {
        let data = engine.alloc_ent_private_data(ent, mem::size_of::<Data<P::Entity>>());
        unsafe {
            Data::new_in::<P>(&mut *data.cast(), value);
        }
        Self { data: data.cast() }
    }

    /// Initialize a private data for the given entity variables.
    ///
    /// The function will create a new entity if `ev` is null.
    ///
    /// # Safety
    ///
    /// * `ev` must be received from the engine.
    ///
    /// # Panics
    ///
    /// Panics if the entity already has a private data.
    pub unsafe fn create<'a, P>(
        engine: ServerEngineRef,
        global_state: GlobalStateRef,
        ev: *mut entvars_s,
    ) -> &'a mut P::Entity
    where
        P: PrivateEntity,
        P::Entity: CreateEntity,
    {
        unsafe { Self::create_with::<P>(engine, global_state, ev, P::Entity::create) }
    }

    /// Initialize a private data for the given entity variables with a value returned from `init`
    /// function.
    ///
    /// # Safety
    ///
    /// See [create](Self::create).
    pub unsafe fn create_with<'a, P>(
        engine: ServerEngineRef,
        global_state: GlobalStateRef,
        ev: *mut entvars_s,
        mut init: impl FnMut(BaseEntity) -> P::Entity,
    ) -> &'a mut P::Entity
    where
        P: PrivateEntity,
    {
        let mut ent = unsafe { ev.as_ref() }
            .and_then(|ev| unsafe { EntityHandle::new(engine, ev.pContainingEntity) })
            .or_else(|| engine.create_entity())
            .expect("failed to allocate new entity");

        if ent.get_private().is_some() {
            panic!("The entity already has a private data.");
        }

        let vars = unsafe { EntityVars::from_raw(engine, global_state, &mut (*ent.as_ptr()).v) };
        let base = BaseEntity { vars };
        let private = Self::alloc::<P>(engine, &ent, init(base));

        unsafe {
            ent.set_private_data(private.data.cast());
            &mut *(&*private.data).as_ptr().cast()
        }
    }

    /// Executes the private data destructor of the given entity.
    ///
    /// The pointer can be null.
    ///
    /// # Safety
    ///
    /// The pointer must be received from the engine.
    pub unsafe fn drop_in_place(ent: *mut edict_s) {
        unsafe {
            if let Some(ent) = ent.as_mut() {
                if let Some(private) = ent.get_private_mut() {
                    Data::drop_in_place(private.data);
                }
            }
        }
    }

    pub fn from_edict(ent: &edict_s) -> Option<&PrivateData> {
        if !ent.pvPrivateData.is_null() {
            let data = &ent.pvPrivateData as *const *mut c_void;
            Some(unsafe { &*(data as *const PrivateData) })
        } else {
            None
        }
    }

    pub fn from_edict_mut(ent: &mut edict_s) -> Option<&mut PrivateData> {
        if !ent.pvPrivateData.is_null() {
            let data = &mut ent.pvPrivateData as *mut *mut c_void;
            Some(unsafe { &mut *(data as *mut PrivateData) })
        } else {
            None
        }
    }

    fn as_entity_ptr(&self) -> *const dyn Entity {
        let data = unsafe { &*self.data };
        unsafe { ((*data.vtable).as_entity)(&*data.as_ptr()) }
    }

    /// Converts this private data to a shared [Entity] reference.
    pub fn as_entity(&self) -> &dyn Entity {
        unsafe { &*self.as_entity_ptr() }
    }

    /// Converts this private data to a mutable [Entity] reference.
    pub fn as_entity_mut(&mut self) -> &mut dyn Entity {
        unsafe { &mut *self.as_entity_ptr().cast_mut() }
    }

    fn downcast<U: Entity + ?Sized>(&self) -> Option<*mut U> {
        unsafe {
            let mut result = MaybeUninit::<*mut U>::uninit();
            let data = &*self.data;
            let type_id = TypeId::of::<U>();
            if ((*data.vtable).downcast)(&*data.as_ptr(), type_id, result.as_mut_ptr().cast()) {
                Some(result.assume_init())
            } else {
                None
            }
        }
    }

    pub fn downcast_ref<U: Entity + ?Sized>(&self) -> Option<&U> {
        self.downcast::<U>().map(|i| unsafe { &*i })
    }

    pub fn downcast_mut<U: Entity + ?Sized>(&mut self) -> Option<&mut U> {
        self.downcast::<U>().map(|i| unsafe { &mut *i })
    }
}

#[deprecated(note = "impl PrivateEntity trait instead")]
pub struct Private<T>(PhantomData<T>);

#[allow(deprecated)]
impl<T: Entity> PrivateEntity for Private<T> {
    type Entity = T;
}

/// Used to get a reference to a private data of entity.
pub trait GetPrivateData<'a> {
    /// Returns a shared reference to a private data of this entity.
    fn get_private(&self) -> Option<&'a PrivateData>;

    /// Returns a mutable reference to a private data of this entity.
    ///
    /// # Safety
    ///
    /// Must follow Rust aliasing rules.
    unsafe fn get_private_mut(&mut self) -> Option<&'a mut PrivateData>;

    /// Returns a shared dyn reference if the entity has a private data.
    fn get_entity(&self) -> Option<&'a dyn Entity> {
        self.get_private().map(|i| i.as_entity())
    }

    /// Returns a mutable dyn reference if the entity has a private data.
    ///
    /// # Safety
    ///
    /// Must follow Rust aliasing rules.
    unsafe fn get_entity_mut(&mut self) -> Option<&'a mut dyn Entity> {
        unsafe { self.get_private_mut() }.map(|i| i.as_entity_mut())
    }

    fn downcast_ref<U: Entity + ?Sized>(&self) -> Option<&'a U> {
        self.get_private().and_then(|i| i.downcast_ref())
    }

    /// Tries to downcast this entity to some mutable reference.
    ///
    /// # Safety
    ///
    /// Undefined behaviour if Rust aliasing rules are violated.
    unsafe fn downcast_mut<U: Entity + ?Sized>(&mut self) -> Option<&'a mut U> {
        let private = unsafe { self.get_private_mut()? };
        private.downcast_mut()
    }
}

impl<'a, T: 'a + AsEntityHandle> GetPrivateData<'a> for T {
    fn get_private(&self) -> Option<&'a PrivateData> {
        let edict = unsafe { &*self.as_entity_handle() };
        PrivateData::from_edict(edict)
    }

    unsafe fn get_private_mut(&mut self) -> Option<&'a mut PrivateData> {
        let edict = unsafe { &mut *self.as_entity_handle() };
        PrivateData::from_edict_mut(edict)
    }
}

impl<'a, T: 'a + AsEntityHandle> GetPrivateData<'a> for Option<T> {
    fn get_private(&self) -> Option<&'a PrivateData> {
        self.as_ref().and_then(|i| i.get_private())
    }

    unsafe fn get_private_mut(&mut self) -> Option<&'a mut PrivateData> {
        self.as_mut().and_then(|i| unsafe { i.get_private_mut() })
    }
}

/// Implements [PrivateEntity] trait for the given type.
///
/// # Examples
///
/// ```
/// use xash3d_server::{
///     prelude::*,
///     entity::{impl_entity_cast, delegate_entity, BaseEntity, EntityItem, EntityPlayer},
///     private::impl_private,
/// };
///
/// #[derive(Save, Restore)]
/// struct Crab {
///     base: BaseEntity,
/// }
///
/// impl_entity_cast!(Crab);
///
/// impl Entity for Crab {
///     delegate_entity!(base);
/// }
///
/// trait EntityMonster: Entity {}
///
/// impl EntityMonster for Crab {}
///
/// impl_private!(Crab {
///     // downcast to Crab type and Entity trait will be done by PrivateData
///
///     // optional downcasts
///     // useful to define a custom macro to export entities with predefined traits
///     ?EntityItem,
///     ?EntityPlayer,
///
///     // required downcasts (compile error if not implemented)
///     EntityMonster,
/// });
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! impl_private {
    ($entity:ty { $( ?$opt:path ),+ $(, $trait:path )* $(,)? }) => {
        impl $crate::private::PrivateEntity for $entity {
            type Entity = Self;

            fn downcast(_t: &$crate::private::Downcast<Self::Entity>) -> bool {
                false
                $( || _t.downcast::<dyn $opt>(|i| {
                    $crate::entity::static_trait_cast!(Self, $opt, i)
                }) )+
                $( || _t.downcast::<dyn $trait>(|i| Some(i)) )*
            }
        }
    };
    ($entity:ty { $( $trait:path ),* $(,)? }) => {
        impl $crate::private::PrivateEntity for $entity {
            type Entity = Self;

            fn downcast(_t: &$crate::private::Downcast<Self::Entity>) -> bool {
                false $( || _t.downcast::<dyn $trait>(|i| Some(i)) )*
            }
        }
    };
}
#[doc(inline)]
pub use impl_private;
