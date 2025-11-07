use crate::save::{FieldType, TypeDescription};

#[doc(hidden)]
#[macro_export]
macro_rules! field {
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr, global $(,)?) => {
        $crate::ffi::server::TYPEDESCRIPTION {
            fieldType: $fieldtype as core::ffi::c_uint,
            fieldName: $crate::macros::cstringify!($name).as_ptr(),
            fieldOffset: core::mem::offset_of!($ty, $name) as core::ffi::c_int,
            fieldSize: $count as core::ffi::c_short,
            flags: $crate::save::FtypeDesc::GLOBAL.bits(),
        }
    };
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr $(,)?) => {
        $crate::ffi::server::TYPEDESCRIPTION {
            fieldType: $fieldtype as core::ffi::c_uint,
            fieldName: $crate::macros::cstringify!($name).as_ptr(),
            fieldOffset: core::mem::offset_of!($ty, $name) as core::ffi::c_int,
            fieldSize: $count as core::ffi::c_short,
            flags: 0,
        }
    };
}
#[doc(inline)]
pub use field;

#[doc(hidden)]
#[macro_export]
macro_rules! define_field_for {
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr, global) => {
        $crate::save::field!($ty, $name, $fieldtype, $count, global)
    };
    ($ty:ty, $name:ident, $fieldtype:expr, global) => {
        $crate::save::field!($ty, $name, $fieldtype, 1, global)
    };
    ($ty:ty, $name:ident, $fieldtype:expr, $count:expr) => {
        $crate::save::field!($ty, $name, $fieldtype, $count,)
    };
    ($ty:ty, $name:ident, $fieldtype:expr) => {
        $crate::save::field!($ty, $name, $fieldtype, 1)
    };
}
#[doc(inline)]
pub use define_field_for;

#[doc(hidden)]
pub const fn type_of_return_value<T, U: TypeDescription>(_: &impl FnOnce(&T) -> &U) -> FieldType {
    U::TYPE
}

#[doc(hidden)]
#[macro_export]
macro_rules! type_of_field {
    ($ty:ty, $($fields:tt)+ $(,)?) => ({
        $crate::save::type_of_return_value(&|t: &$ty| &t.$($fields)+)
    });
}
pub use type_of_field;

#[doc(hidden)]
pub const fn host_count_of_return_value<T, U>(_: &impl FnOnce(&T) -> &U, ftype: FieldType) -> i16 {
    let count = core::mem::size_of::<U>() / ftype.host_size();
    count as i16
}

#[doc(hidden)]
#[macro_export]
macro_rules! host_count_of_field {
    ($ftype:expr, $ty:ty, $($fields:tt)+ $(,)?) => ({
        $crate::save::host_count_of_return_value(&|t: &$ty| &t.$($fields)+, $ftype)
    });
}
pub use host_count_of_field;

#[doc(hidden)]
#[macro_export]
macro_rules! define_field {
    ($(:$global:ident)? $name:ident $(.$rest:expr)*) => ({
        let ftype = $crate::save::type_of_field!(Self, $name $(.$rest)*);
        $crate::save::define_field!($(:$global)? $name $(.$rest)* => unsafe ftype)
    });
    ($(:$global:ident)? $name:ident $(.$rest:expr)* => unsafe $type:expr ) => {
        $crate::save::field!(
            Self,
            $name $(.$rest)*,
            $type,
            $crate::save::host_count_of_field!($type, Self, $name $(.$rest)*)
            $(, $global)?
        )
    };
}
#[doc(inline)]
pub use define_field;

/// Generate an array of field descriptions.
///
/// # Examples
///
/// ```
/// use core::ffi::CStr;
///
/// use xash3d_server::{
///     csz::CStrArray,
///     ffi::server::TYPEDESCRIPTION,
///     save::{FieldType, SaveFields, define_fields},
/// };
///
/// struct Item {
///     name: CStrArray<32>,
///     count: u32,
///     max: u32,
/// }
///
/// unsafe impl SaveFields for Item {
///     const SAVE_NAME: &'static CStr = c"Item";
///
///     const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![
///         name,
///         count,
///         // manually set the field type
///         max => unsafe FieldType::INTEGER,
///     ];
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! define_fields {
    ($( $(:$global:ident)? $name:ident $(.$rest:expr)* $(=> unsafe $type:expr)? ),* $(,)?) => {
        [$( $crate::save::define_field!($(:$global)? $name $(.$rest)* $(=> unsafe $type)?) ),*]
    };
}
#[doc(inline)]
pub use define_fields;
