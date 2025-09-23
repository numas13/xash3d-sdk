use core::marker::PhantomData;

use xash3d_server::entity::{Downcast, Entity, EntityCast, PrivateEntity};

pub struct Private<T>(PhantomData<T>);

impl<T: Entity + CustomCast> PrivateEntity for Private<T> {
    type Entity = T;

    fn downcast(t: &Downcast<'_, Self::Entity>) -> bool {
        t.downcast(|ent| ent.as_test_entity())
    }
}

#[allow(dead_code)]
pub trait CustomCast: EntityCast {
    fn as_test_entity(&self) -> Option<&dyn EntityTest>;
    fn as_test_entity_mut(&mut self) -> Option<&mut dyn EntityTest>;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_custom_cast {
    ($(#[$attr:meta])* $ty:ty) => {
        $(#[$attr])*
        impl $crate::entity::CustomCast for $ty {
            xash3d_server::entity::impl_cast! {
                $ty {
                    as_test_entity, as_test_entity_mut -> $crate::entity::EntityTest;
                }
            }
        }
    };
}
#[doc(inline)]
pub use impl_custom_cast;

#[doc(hidden)]
#[macro_export]
macro_rules! impl_cast {
    ($(#[$attr:meta])* $ty:ty) => {
        xash3d_server::entity::impl_entity_cast!($(#[$attr])* $ty);
        $crate::entity::impl_custom_cast!($(#[$attr])* $ty);
    };
}
#[doc(inline)]
pub use impl_cast;

#[allow(dead_code)]
pub trait EntityTest: Entity + CustomCast {
    fn do_test_work(&self) {}
}
