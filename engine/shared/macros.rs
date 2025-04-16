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
