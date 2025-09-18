use core::{
    any::{Any, TypeId},
    ffi::c_void,
    mem,
    ops::{Deref, DerefMut},
    ptr,
};

use sv::{
    ffi::server::{edict_s, entvars_s},
    prelude::*,
};

use crate::entity::Entity;

/// Wrapper for `dyn Trait` fat pointers.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct DynPtr {
    #[allow(dead_code)]
    data: *mut (),
    vtable: *const fn(*mut ()),
}

/// Private data for [edict_s](sv::ffi::server::edict_s).
#[derive(Debug)]
#[repr(C)]
struct PrivateData<T = ()> {
    /// Entity vtable for data.
    vtable: *const fn(*mut T),
    /// Data type id function.
    type_id: fn(&T) -> TypeId,
    /// The drop function must be called on free.
    drop_fn: unsafe fn(*mut T),
    /// Offset to the `inner` for unknown data type.
    offset: u16,
    /// Inner data.
    inner: T,
}

impl<T> PrivateData<T> {
    unsafe fn init(data: &mut Self, inner: T)
    where
        T: Entity,
    {
        data.vtable = unsafe { mem::transmute::<&dyn Entity, DynPtr>(&inner).vtable.cast() };
        data.type_id = <T as Any>::type_id;
        data.drop_fn = ptr::drop_in_place::<T>;
        data.offset = mem::offset_of!(PrivateData<T>, inner) as u16;
        unsafe {
            ptr::write(&mut data.inner, inner);
        }
    }

    const fn as_ptr(&self) -> *const T {
        unsafe { (self as *const Self).byte_add(self.offset as usize).cast() }
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.as_ptr() as *mut T
    }

    const fn as_ref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }

    // const fn as_mut(&mut self) -> &mut T {
    //     unsafe { &mut *self.as_mut_ptr() }
    // }

    fn as_dyn_entity(&self) -> DynPtr {
        DynPtr {
            data: self.as_ptr() as *mut (),
            vtable: self.vtable.cast(),
        }
    }

    fn inner_type_id(&self) -> TypeId {
        (self.type_id)(self.as_ref())
    }

    fn downcast<A: Any>(&self) -> Option<&A> {
        if TypeId::of::<A>() == self.inner_type_id() {
            Some(unsafe { &*self.as_ptr().cast() })
        } else {
            None
        }
    }

    fn downcast_mut<A: Any>(&mut self) -> Option<&mut A> {
        if TypeId::of::<A>() == self.inner_type_id() {
            Some(unsafe { &mut *self.as_mut_ptr().cast() })
        } else {
            None
        }
    }
}

/// Wrapper for pointer to entity private data.
#[repr(transparent)]
pub struct PrivateDataRef {
    raw: *mut PrivateData,
}

impl PrivateDataRef {
    pub fn new<T: Entity>(engine: ServerEngineRef, ent: &mut edict_s, data: T) -> Self {
        let len = mem::size_of::<PrivateData<T>>();
        let raw = engine.alloc_ent_private_data(ent, len);
        unsafe {
            PrivateData::init(&mut *raw.cast(), data);
            Self::from_raw(raw)
        }
    }

    unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self { raw: ptr.cast() }
    }

    unsafe fn into_raw(self) -> *mut c_void {
        self.raw.cast()
    }

    fn as_ref(&self) -> &PrivateData {
        unsafe { &*self.raw }
    }

    fn as_mut(&mut self) -> &mut PrivateData {
        unsafe { &mut *self.raw }
    }

    pub unsafe fn free(ent: *mut edict_s) {
        if ent.is_null() {
            return;
        }
        let ent = unsafe { &mut *ent };
        if ent.pvPrivateData.is_null() {
            return;
        }
        let mut private = unsafe { PrivateDataRef::from_raw(ent.pvPrivateData) };
        let data = private.as_mut();
        unsafe {
            (data.drop_fn)(data.as_mut_ptr());
        }
    }
}

impl Deref for PrivateDataRef {
    type Target = dyn Entity;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self.as_ref().as_dyn_entity()) }
    }
}

impl DerefMut for PrivateDataRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { mem::transmute(self.as_mut().as_dyn_entity()) }
    }
}

/// Helper trait to initialize and retrieve an entity's private data.
pub trait Private {
    /// Get a reference to a containing entity.
    fn as_ent(&self) -> &edict_s;

    /// Get a mutable reference to a containing entity.
    fn as_ent_mut(&mut self) -> &mut edict_s;

    /// Initialize private data.
    ///
    /// # Panics
    ///
    /// Panics if private data is initialized already.
    fn private_init<T, F>(&mut self, engine: ServerEngineRef, init: F) -> &mut T
    where
        T: Entity,
        F: FnOnce(ServerEngineRef, *mut entvars_s) -> T,
    {
        let ent = self.as_ent_mut();
        assert!(ent.pvPrivateData.is_null());
        let data = init(engine, &mut ent.v);
        let mut private = PrivateDataRef::new(engine, ent, data);
        let data = private.as_mut().as_mut_ptr();
        ent.pvPrivateData = unsafe { private.into_raw().cast() };
        unsafe { &mut *data.cast() }
    }

    /// Attempts to retrieve a reference to private data.
    fn private(&self) -> Option<&PrivateDataRef> {
        let ptr = &self.as_ent().pvPrivateData;
        if !ptr.is_null() {
            Some(unsafe { &*(ptr as *const _ as *const PrivateDataRef) })
        } else {
            None
        }
    }

    /// Attempts to retrieve a mutable reference to private data.
    fn private_mut(&mut self) -> Option<&mut PrivateDataRef> {
        let ptr = &mut self.as_ent_mut().pvPrivateData;
        if !ptr.is_null() {
            Some(unsafe { &mut *(ptr as *mut _ as *mut PrivateDataRef) })
        } else {
            None
        }
    }

    // /// Attempts to retrieve a reference to an [Entity] trait.
    // fn as_entity(&self) -> Option<&dyn Entity> {
    //     self.private().map(|p| &**p)
    // }

    // /// Attempts to retrieve a mutable reference to an [Entity] trait.
    // fn as_entity_mut(&mut self) -> Option<&mut dyn Entity> {
    //     self.private_mut().map(|p| &mut **p)
    // }

    /// Attempts to downcast to a concrete type reference.
    fn downcast<T: Any>(&self) -> Option<&T> {
        self.private()?.as_ref().downcast()
    }

    /// Attempts to downcast to a concrete type mutable reference.
    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.private_mut()?.as_mut().downcast_mut()
    }
}

impl Private for edict_s {
    fn as_ent(&self) -> &edict_s {
        self
    }

    fn as_ent_mut(&mut self) -> &mut edict_s {
        self
    }
}

impl Private for entvars_s {
    fn as_ent(&self) -> &edict_s {
        unsafe { &*self.pContainingEntity }
    }

    fn as_ent_mut(&mut self) -> &mut edict_s {
        unsafe { &mut *self.pContainingEntity }
    }
}

impl Private for dyn Entity {
    fn as_ent(&self) -> &edict_s {
        self.ent()
    }

    fn as_ent_mut(&mut self) -> &mut edict_s {
        self.ent_mut()
    }
}
