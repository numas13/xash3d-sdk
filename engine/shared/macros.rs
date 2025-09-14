use core::mem;

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
pub const fn size_of_return_value<T, U>(_: &impl FnOnce(&T) -> &U) -> usize {
    core::mem::size_of::<U>()
}

/// Returns the size of a field in bytes of the given type.
///
/// # Examples
///
/// ```
/// use xash3d_shared::macros::size_of_field;
///
/// struct Foo {
///     f1: i8,
///     f2: i16,
///     bar: Bar,
/// }
///
/// struct Bar {
///     f4: i32,
///     f8: i64,
/// }
///
/// assert_eq!(size_of_field!(Foo, f1), 1);
/// assert_eq!(size_of_field!(Foo, f2), 2);
/// assert_eq!(size_of_field!(Foo, bar), core::mem::size_of::<Bar>());
/// assert_eq!(size_of_field!(Foo, bar.f4), 4);
/// assert_eq!(size_of_field!(Foo, bar.f8), 8);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! size_of_field {
    ($ty:ty, $($fields:tt)+ $(,)?) => ({
        $crate::macros::size_of_return_value(&|t: &$ty| &t.$($fields)+)
    });
}
#[doc(inline)]
pub use size_of_field;

/// Asserts that the size in bytes of `lhs` is equal to the size in bytes of a field of the `rhs.`
///
/// # Examples
///
/// ```
/// use xash3d_shared::macros::const_assert_size_of_field_eq;
///
/// struct Foo {
///     a: u16,
///     b: u32,
/// }
///
/// const_assert_size_of_field_eq!(u16, Foo, a);
/// const_assert_size_of_field_eq!(u32, Foo, b);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! const_assert_size_of_field_eq {
    ($lhs:ty, $($rhs:tt)+) => {
        $crate::macros::const_assert_eq!(
            core::mem::size_of::<$lhs>(),
            $crate::macros::size_of_field!($($rhs)+),
        );
    };
}
#[doc(inline)]
pub use const_assert_size_of_field_eq;

const_assert_eq!(size_of_field!((u32, u64), 0), 4);
const_assert_eq!(size_of_field!((u32, u64), 1), 8);

#[allow(dead_code)]
struct Foo {
    a: u16,
    b: u32,
    c: u64,
}

const_assert_eq!(size_of_field!(Foo, a), 2);
const_assert_eq!(size_of_field!(Foo, b), 4);
const_assert_eq!(size_of_field!(Foo, c), 8);

const_assert_size_of_field_eq!(u16, Foo, a);
const_assert_size_of_field_eq!(u32, Foo, b);
const_assert_size_of_field_eq!(u64, Foo, c);

#[allow(dead_code)]
struct Bar {
    a: i8,
    b: i64,
    c: i128,
    foo: Foo,
}

const_assert_eq!(size_of_field!(Bar, a), 1);
const_assert_eq!(size_of_field!(Bar, b), 8);
const_assert_eq!(size_of_field!(Bar, c), 16);

const_assert_eq!(size_of_field!(Bar, foo), mem::size_of::<Foo>());
const_assert_eq!(size_of_field!(Bar, foo.a), 2);
const_assert_eq!(size_of_field!(Bar, foo.b), 4);
const_assert_eq!(size_of_field!(Bar, foo.c), 8);

const_assert_size_of_field_eq!(Foo, Bar, foo);
const_assert_size_of_field_eq!(u16, Bar, foo.a);
const_assert_size_of_field_eq!(u32, Bar, foo.b);
const_assert_size_of_field_eq!(u64, Bar, foo.c);

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
        $vis:vis enum $name:ident: $ty:ty $(as $cast:ty)? {
            $($(#[$variant_attr:meta])* $variant:ident($($value:tt)+)),+ $(,)?
        }
    ) => {
        $(#[$enum_attr])*
        #[repr(C)]
        $vis enum $name {
            $($(#[$variant_attr])* $variant = $($value)+ as isize,)+
        }

        impl $name {
            /// Creates an enum if the given raw value is valid.
            pub const fn from_raw(value: $ty) -> Option<Self> {
                match value $(as $cast)? {
                    $($($value)+ => Some(Self::$variant),)+
                    _ => None,
                }
            }

            /// Converts this enum to a raw value.
            pub const fn into_raw(self) -> $ty {
                match self {
                    $(Self::$variant => $($value)+ as $ty,)+
                }
            }
        }

        impl From<$name> for $ty {
            fn from(value: $name) -> Self {
                value.into_raw()
            }
        }
    };
}
#[doc(inline)]
pub use define_enum_for_primitive;
