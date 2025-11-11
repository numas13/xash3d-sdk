use core::{
    any::{Any, type_name},
    cell::{Ref, RefCell},
};

use alloc::{boxed::Box, vec::Vec};

/// A list of custom global objects.
#[derive(Default)]
pub struct CustomGlobals {
    objects: RefCell<Vec<Box<dyn Any>>>,
}

impl CustomGlobals {
    /// Clears global custom objects, removing all values.
    pub fn clear(&self) {
        self.objects.borrow_mut().clear();
    }

    /// Returns a custom global state with type `T`.
    pub fn try_get<T: Any>(&self) -> Option<Ref<'_, T>> {
        Ref::filter_map(self.objects.borrow(), |list| {
            list.iter().find_map(|i| i.as_ref().downcast_ref::<T>())
        })
        .ok()
    }

    /// Returns a custom global object with the type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the custom global object is not added.
    pub fn get<T: Any>(&self) -> Ref<'_, T> {
        match self.try_get() {
            Some(custom) => custom,
            None => {
                panic!("custom({}) is not added to CustomGlobals", type_name::<T>());
            }
        }
    }

    /// Returns a custom global object or insert the value with the given function if it is not
    /// added.
    pub fn get_or_insert_with<T: Any>(&self, with: impl FnOnce() -> T) -> Ref<'_, T> {
        match self.try_get::<T>() {
            Some(i) => i,
            None => {
                let custom = Box::new(with());
                trace!("add custom({})", type_name::<T>());
                self.objects.borrow_mut().push(custom);
                Ref::map(self.objects.borrow(), |list| {
                    list.last().unwrap().as_ref().downcast_ref::<T>().unwrap()
                })
            }
        }
    }

    /// Returns a custom global object or insert the value if it is not added.
    pub fn get_or_insert<T: Any>(&self, default: T) -> Ref<'_, T> {
        self.get_or_insert_with::<T>(|| default)
    }

    /// Add a custom global object with the type `T`.
    pub fn add<T: Any>(&self, custom: T) {
        let mut list = self.objects.borrow_mut();
        match list.iter_mut().find_map(|i| i.as_mut().downcast_mut::<T>()) {
            Some(i) => {
                trace!("replace custom({})", type_name::<T>());
                *i = custom;
            }
            None => {
                trace!("add custom({})", type_name::<T>());
                list.push(Box::new(custom));
            }
        }
    }
}
