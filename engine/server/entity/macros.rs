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

/// Implement the [EntityCast](super::EntityCast) trait for given types.
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
