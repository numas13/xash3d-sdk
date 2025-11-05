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
    ($ty:ty, $trait:path, $value:expr $(, $mut:ident)? $(,)?) => ({
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
/// use xash3d_server::entity::BaseEntity;
///
/// trait MyToggle {}
/// trait MyMonster {}
///
/// trait MyCast {
///     fn as_my_toggle(&self) -> Option<&dyn MyToggle>;
///     fn as_my_monster(&self) -> Option<&dyn MyMonster>;
/// }
///
/// macro_rules! impl_my_cast {
///     ($ty:ty) => {
///         impl MyCast for $ty {
///             xash3d_server::entity::impl_cast!{
///                 $ty {
///                     as_my_toggle -> MyToggle;
///                     as_my_monster -> MyMonster;
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
/// // impl MyCast for Zombie { ... }
/// impl_my_cast!(Zombie);
///
/// impl MyMonster for Zombie {}
///
/// // initialize to zeroes only for test purpose
/// let zombie: Zombie = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
///
/// assert!(zombie.as_my_toggle().is_none());
/// assert!(zombie.as_my_monster().is_some());
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! impl_cast {
    ($ty:ty {
        $( $(#[$attr:meta])* $as_ref:ident -> $to:path;)*
    }) => {
        $(
            $(#[$attr])*
            fn $as_ref(&self) -> Option<&dyn $to> {
                $crate::entity::static_trait_cast!($ty, $to, self)
            }
        )*
    };
}
#[doc(inline)]
pub use impl_cast;

#[doc(hidden)]
#[macro_export]
macro_rules! delegate_method {
    ($base:ident, $vis:vis fn $meth:ident(&self $(, $arg:ident: $ty:ty )* $(,)?) $(-> $ret:ty)?) => {
        $vis fn $meth(&self $(, $arg: $ty )*) $(-> $ret)? {
            self.$base.$meth($($arg),*)
        }
    };
    ($base:ident, $vis:vis fn $meth:ident(&mut self $(, $arg:ident: $ty:ty )* $(,)?) $(-> $ret:ty)?) => {
        $vis fn $meth(&mut self $(, $arg: $ty )*) $(-> $ret)? {
            self.$base.$meth($($arg),*)
        }
    };
}
pub use delegate_method;

#[doc(hidden)]
#[macro_export]
macro_rules! define_method_impl {
    ($( #[$attr:meta] )*
     $vis:vis fn $meth:ident($( $arg:tt )* ) $(-> $ret:ty)? $body:block $(;)?
    ) => {
        $( #[$attr] )*
        $vis fn $meth($( $arg )*) $(-> $ret)? $body
    };
    ($( #[$attr:meta] )*
     $vis:vis fn $meth:ident($( $arg:tt )* ) $(-> $ret:ty)? ;
    ) => {
        $( #[$attr] )*
        $vis fn $meth($( $arg )*) $(-> $ret)?;
    };
}
pub use define_method_impl;

// Hack for $ in nested macro.
// https://github.com/rust-lang/rust/issues/35853#issuecomment-415993963
#[doc(hidden)]
#[macro_export]
macro_rules! define_entity_trait_impl {
    ($($body:tt)*) => {
        macro_rules! __define_entity_trait_impl { $($body)* }
        __define_entity_trait_impl!(crate $);
    }
}
pub use define_entity_trait_impl;

/// Define an entity trait and a delegate macro for this trait.
///
/// # Examples
///
/// It is recommended to use full paths to types so there is no need to import types manually when
/// using the delegate macro.
///
/// ```
/// extern crate alloc;
///
/// mod entity {
///     use xash3d_server::entity::define_entity_trait;
///
///     define_entity_trait! {
///         pub trait Entity(delegate_entity) {
///             fn name(&self) -> &core::ffi::CStr;
///             fn set_name(&mut self, name: alloc::ffi::CString);
///             fn spawn(&mut self);
///             fn think(&mut self);
///         }
///     }
/// }
///
/// mod named {
///     use core::ffi::CStr;
///     use alloc::ffi::CString;
///
///     pub struct Named {
///         name: CString,
///     }
///
///     impl Named {
///         pub fn name(&self) -> &CStr {
///             &self.name
///         }
///
///         pub fn set_name(&mut self, name: CString) {
///             self.name = name;
///         }
///     }
/// }
///
/// use entity::{Entity, delegate_entity};
/// use named::Named;
///
/// struct Think;
///
/// impl Think {
///     fn think(&mut self) {
///         println!("thinking...");
///     }
/// }
///
/// struct MyEntity {
///     name: Named,
///     think: Think,
/// }
///
/// impl Entity for MyEntity {
///     delegate_entity!(name { name, set_name });
///     delegate_entity!(think { think });
///
///     fn spawn(&mut self) {
///         println!("spawn MyEntity");
///     }
/// }
/// ```
///
/// This example shows how to delegate all, selected or all but excluded methods.
///
/// ```
/// use xash3d_server::entity::define_entity_trait;
///
/// define_entity_trait! {
///     /// This is a test entity trait.
///     trait TestEntity(delegate_test) {
///         fn foo(&self) -> &str;
///         fn bar(&self) -> &str;
///     }
/// }
///
/// #[derive(Default)]
/// struct A;
///
/// impl TestEntity for A {
///     fn foo(&self) -> &str { "A::foo" }
///     fn bar(&self) -> &str { "A::bar" }
/// }
///
/// #[derive(Default)]
/// struct B {
///     base: A,
/// }
///
/// impl TestEntity for B {
///     // delegate all methods to the base field
///     delegate_test!(base);
/// }
///
/// assert_eq!(B::default().foo(), "A::foo");
/// assert_eq!(B::default().bar(), "A::bar");
///
/// #[derive(Default)]
/// struct C {
///     base: B,
/// }
///
/// impl TestEntity for C {
///     // delegate only foo method to the base field
///     delegate_test!(base { foo });
///     fn bar(&self) -> &str { "C::bar" }
/// }
///
/// assert_eq!(C::default().foo(), "A::foo");
/// assert_eq!(C::default().bar(), "C::bar");
///
/// #[derive(Default)]
/// struct D {
///     base: C,
/// }
///
/// impl TestEntity for D {
///     // delegate all methods except foo to the base field
///     delegate_test!(base not { foo });
///     fn foo(&self) -> &str { "D::foo" }
/// }
///
/// assert_eq!(D::default().foo(), "D::foo");
/// assert_eq!(D::default().bar(), "C::bar");
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! define_entity_trait {
    ($( #[$trait_attr:meta] )*
     $vis:vis trait $name:ident($delegate:ident) $(: ( $($sup:tt)* ))? {
        $(auto {
            $(
                $( #[$attr_auto:meta] )*
                fn $meth_auto:ident($( $arg_auto:tt )*) $(-> $ret_auto:ty)?
                $body_auto:block
            )*
        })?

        $(default {
            $(
                $( #[$attr_default:meta] )*
                fn $meth_default:ident($( $arg_default:tt )*) $(-> $ret_default:ty)?
                $body_default:block
            )*
        })?

        $(
            $( #[$attr:meta] )*
            fn $meth:ident($( $arg:tt )*) $(-> $ret:ty)? $( $body:block )? $(;)?
        )*
    }) => {
        $( #[$trait_attr] )*
        #[doc = concat!("\n\nDelegate macro is [", stringify!($delegate), "].")]
        $vis trait $name $(: $($sup)*)? {
            $(
                $(
                    $crate::entity::define_method_impl! {
                        $( #[$attr_auto] )*
                        /// # Delegation
                        ///
                        /// This method will be auto-implemented by
                        #[doc = concat!("[", stringify!($delegate), "]")]
                        /// macro.
                        fn $meth_auto($( $arg_auto )*) $(-> $ret_auto)? ;
                    }
                )*
            )?

            $(
                $(
                    $crate::entity::define_method_impl! {
                        $( #[$attr_default] )*
                        /// # Delegation
                        ///
                        /// This method will not be delegated by
                        #[doc = concat!("[", stringify!($delegate), "]")]
                        /// macro.
                        fn $meth_default($( $arg_default )*) $(-> $ret_default)?
                        $body_default ;
                    }
                )*
            )?

            $(
                $crate::entity::define_method_impl! {
                    $( #[$attr] )*
                    /// # Delegation
                    ///
                    /// This method will be delegated by
                    #[doc = concat!("[", stringify!($delegate), "]")]
                    /// macro.
                    fn $meth($( $arg )*) $(-> $ret)?
                    $( $body )? ;
                }
            )*
        }

        $crate::entity::define_entity_trait_impl! {
            ($krate:tt $d:tt) => {
                /// Delegate macro for
                #[doc = concat!("[", stringify!($name), "]")]
                /// trait.
                ///
                /// See [define_entity_trait] for examples.
                #[allow(clippy::crate_in_macro_def)]
                #[doc(hidden)]
                #[macro_export]
                macro_rules! $delegate {
                    ($base:ident $v:vis not { $d($d meth:ident),* $d(,)? }) => {
                        $delegate!(impl auto);
                        $( $delegate!($base, $v $meth, $d( $d meth ),*); )*
                    };
                    ($base:ident { $d($d v:vis $d meth:ident),* $d(,)? }) => {
                        $delegate!(impl auto);
                        $d( $delegate!($base, $d v $d meth); )*
                    };
                    ($base:ident $v:vis) => {
                        $delegate!($base { $( $v $meth ),* });
                    };
                    ($base:ident) => {
                        $delegate!($base { $( $meth ),* });
                    };
                    (impl auto) => {
                        $(
                            $(
                                fn $meth_auto($( $arg_auto )*) $(-> $ret_auto)? $body_auto
                            )*
                        )?
                    };
                    $(
                        ($base:ident, $v:vis $meth, $meth $d(, $d rest:ident)* $d(,)?) => {
                            // ignore
                        };
                        ($base:ident, $v:vis $meth, $other:ident $d(, $d rest:ident)* $d(,)?) => {
                            $delegate!($base, $v $meth $d(, $d rest)*);
                        };
                        ($base:ident, $v:vis $meth $d(,)?) => {
                            $crate::entity::delegate_method! {
                                $base, $v fn $meth($( $arg )*) $(-> $ret)?
                            }
                        };
                    )*
                }
                #[doc(inline)]
                $vis use $delegate;
            }
        }
    };
}
#[doc(inline)]
pub use define_entity_trait;
