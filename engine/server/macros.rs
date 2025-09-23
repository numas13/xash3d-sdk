pub use xash3d_shared::macros::cstringify;

#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr, $flags:expr) => {
        $crate::ffi::server::TYPEDESCRIPTION {
            fieldType: $fieldtype as core::ffi::c_uint,
            fieldName: $crate::macros::cstringify!($name).as_ptr(),
            fieldOffset: core::mem::offset_of!($ty, $name) as core::ffi::c_int,
            fieldSize: $count as core::ffi::c_short,
            flags: $flags.bits(),
        }
    };
}
#[doc(inline)]
pub use field;

#[doc(hidden)]
#[macro_export]
macro_rules! define_field {
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr, global) => {
        $crate::macros::field!(
            $ty,
            $name,
            $fieldtype,
            $count,
            $crate::save::FtypeDesc::GLOBAL
        )
    };
    ($ty:ty, $name:ident, $fieldtype:expr, global) => {
        $crate::macros::field!($ty, $name, $fieldtype, 1, $crate::save::FtypeDesc::GLOBAL)
    };
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr) => {
        $crate::macros::field!(
            $ty,
            $name,
            $fieldtype,
            $count,
            $crate::save::FtypeDesc::NONE
        )
    };
    ($ty:ty, $name:ident, $fieldtype:expr) => {
        $crate::macros::field!($ty, $name, $fieldtype, 1, $crate::save::FtypeDesc::NONE)
    };
}
#[doc(inline)]
pub use define_field;

#[doc(hidden)]
#[macro_export]
macro_rules! define_entity_field {
    ($name:ident, $fieldtype:expr, $count:expr, global) => {
        $crate::macros::field!(
            $crate::raw::entvars_s,
            $name,
            $fieldtype,
            $count,
            $crate::save::FtypeDesc::GLOBAL
        )
    };
    ($name:ident, $fieldtype:expr, global) => {
        $crate::macros::field!(
            $crate::ffi::server::entvars_s,
            $name,
            $fieldtype,
            1,
            $crate::save::FtypeDesc::GLOBAL
        )
    };
    ($name:ident, $fieldtype:expr, $count:expr) => {
        $crate::macros::field!(
            $crate::ffi::server::entvars_s,
            $name,
            $fieldtype,
            $count,
            $crate::save::FtypeDesc::NONE
        )
    };
    ($name:ident, $fieldtype:expr) => {
        $crate::macros::field!(
            $crate::ffi::server::entvars_s,
            $name,
            $fieldtype,
            1,
            $crate::save::FtypeDesc::NONE
        )
    };
}
#[doc(inline)]
pub use define_entity_field;
