#[doc(hidden)]
#[macro_export]
macro_rules! const_assert {
    ($expr:expr $(,)?) => {
        const _: [(); 0 / {
            const C: bool = $expr as bool;
            C as usize
        }] = [];
    };
}
#[doc(inline)]
pub use const_assert;

#[doc(hidden)]
#[macro_export]
macro_rules! const_assert_eq {
    ($lhs:expr, $rhs:expr $(,)?) => {
        $crate::macros::const_assert!($lhs == $rhs);
    };
}
#[doc(inline)]
pub use const_assert_eq;

#[doc(hidden)]
#[macro_export]
macro_rules! const_assert_ne {
    ($lhs:expr, $rhs:expr $(,)?) => {
        $crate::macros::const_assert!($lhs != $rhs);
    };
}
#[doc(inline)]
pub use const_assert_ne;

#[doc(hidden)]
#[macro_export]
macro_rules! const_assert_size_eq {
    ($lhs:ty, $rhs:ty $(,)?) => {
        $crate::macros::const_assert_eq!(
            core::mem::size_of::<$lhs>(),
            core::mem::size_of::<$rhs>(),
        );
    };
}
#[doc(inline)]
pub use const_assert_size_eq;

#[doc(hidden)]
#[macro_export]
macro_rules! cstringify {
    ($x:ident) => {
        unsafe {
            core::ffi::CStr::from_bytes_with_nul_unchecked(concat!(stringify!($x), "\0").as_bytes())
        }
    };
}
#[doc(inline)]
pub use cstringify;

/// Helper macro to define enums for primitive types.
///
/// # Examples
///
/// ```
/// use xash3d_shared::macros::define_enum_for_primitive;
///
/// mod bindings {
///     pub const TYPE_A: u8 = 0;
///     pub const TYPE_B: u8 = 1;
/// }
///
/// define_enum_for_primitive! {
///     #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
///     pub enum Type: u8 {
///         A(bindings::TYPE_A),
///         B(bindings::TYPE_B),
///         #[default]
///         C(4),
///     }
/// }
///
/// assert_eq!(Type::A.into_raw(), bindings::TYPE_A);
/// assert_eq!(Type::B.into_raw(), bindings::TYPE_B);
/// assert_eq!(Type::C.into_raw(), 4);
///
/// assert_eq!(Type::from_raw(0), Some(Type::A));
/// assert_eq!(Type::from_raw(1), Some(Type::B));
/// assert_eq!(Type::from_raw(4), Some(Type::C));
///
/// assert_eq!(Type::from_raw(2), None);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! define_enum_for_primitive {
    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $name:ident: $ty:ty {
            $($(#[$variant_attr:meta])* $variant:ident($($value:tt)+)),+ $(,)?
        }
    ) => {
        $(#[$enum_attr])*
        $vis enum $name {
            $($(#[$variant_attr])* $variant,)+
        }

        impl $name {
            /// Creates an enum if the given raw value is valid.
            pub fn from_raw(value: $ty) -> Option<Self> {
                match value {
                    $($($value)+ => Some(Self::$variant),)+
                    _ => None,
                }
            }

            /// Converts this enum to a raw value.
            pub fn into_raw(self) -> $ty {
                match self {
                    $(Self::$variant => $($value)+ as $ty,)+
                }
            }
        }
    };
}
#[doc(inline)]
pub use define_enum_for_primitive;
