use core::{any::Any, cell::Ref};

use xash3d_shared::{engine::EngineRef, export::impl_unsync_global, global_state::CustomGlobals};

use crate::prelude::*;

/// Used to create a new global objects.
pub trait DefaultGlobal {
    /// Constructs a new global object.
    fn default_global(engine: ClientEngineRef) -> Self;
}

pub struct GlobalState {
    engine: ClientEngineRef,
    customs: CustomGlobals,
}

impl_unsync_global!(GlobalState);

impl GlobalState {
    pub fn new(engine: ClientEngineRef) -> Self {
        Self {
            engine,
            customs: CustomGlobals::default(),
        }
    }

    pub fn reset(&self) {
        self.customs.clear();
    }

    pub fn try_get<T: Any>(&self) -> Option<Ref<'_, T>> {
        self.customs.try_get::<T>()
    }

    pub fn get<T: Any>(&self) -> Ref<'_, T> {
        self.customs.get::<T>()
    }

    pub fn get_or_insert_with<T: Any>(&self, with: impl FnOnce() -> T) -> Ref<'_, T> {
        self.customs.get_or_insert_with::<T>(with)
    }

    pub fn get_or_insert<T: Any>(&self, value: T) -> Ref<'_, T> {
        self.customs.get_or_insert(value)
    }

    pub fn get_or_default<T: Any + DefaultGlobal>(&self) -> Ref<'_, T> {
        self.customs
            .get_or_insert_with(|| T::default_global(self.engine))
    }

    pub fn add<T: Any>(&self, custom: T) {
        self.customs.add::<T>(custom)
    }
}

pub type GlobalStateRef = EngineRef<GlobalState>;
