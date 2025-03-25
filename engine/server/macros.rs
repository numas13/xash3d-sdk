#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr, $flags:expr) => {
        $crate::raw::TYPEDESCRIPTION {
            fieldType: $fieldtype,
            fieldName: shared::macros::cstringify!($name).as_ptr(),
            fieldOffset: core::mem::offset_of!($ty, $name) as core::ffi::c_int,
            fieldSize: $count as core::ffi::c_short,
            flags: $flags,
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
            $crate::raw::FtypeDesc::GLOBAL
        )
    };
    ($ty:ty, $name:ident, $fieldtype:expr, global) => {
        $crate::macros::field!($ty, $name, $fieldtype, 1, $crate::raw::FtypeDesc::GLOBAL)
    };
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr) => {
        $crate::macros::field!($ty, $name, $fieldtype, $count, $crate::raw::FtypeDesc::NONE)
    };
    ($ty:ty, $name:ident, $fieldtype:expr) => {
        $crate::macros::field!($ty, $name, $fieldtype, 1, $crate::raw::FtypeDesc::NONE)
    };
}
#[doc(inline)]
pub use define_field;

#[doc(hidden)]
#[macro_export]
macro_rules! define_entity_field {
    ($name:ident, $fieldtype:expr, $count:expr, global) => {
        $crate::macros::field!(
            shared::raw::entvars_s,
            $name,
            $fieldtype,
            $count,
            $crate::raw::FtypeDesc::GLOBAL
        )
    };
    ($name:ident, $fieldtype:expr, global) => {
        $crate::macros::field!(
            shared::raw::entvars_s,
            $name,
            $fieldtype,
            1,
            $crate::raw::FtypeDesc::GLOBAL
        )
    };
    ($name:ident, $fieldtype:expr, $count:expr) => {
        $crate::macros::field!(
            shared::raw::entvars_s,
            $name,
            $fieldtype,
            $count,
            $crate::raw::FtypeDesc::NONE
        )
    };
    ($name:ident, $fieldtype:expr) => {
        $crate::macros::field!(
            shared::raw::entvars_s,
            $name,
            $fieldtype,
            1,
            $crate::raw::FtypeDesc::NONE
        )
    };
}
#[doc(inline)]
pub use define_entity_field;
