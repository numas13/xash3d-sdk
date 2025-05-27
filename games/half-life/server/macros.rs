// #[doc(hidden)]
// macro_rules! alert {
//     ($atype:ident, $($args:tt)+) => (
//         $crate::engine::engine().alert_message_fmt(sdk::ffi::ALERT_TYPE::$atype, format_args!($($args)+))
//     );
// }
// #[doc(inline)]
// pub(super) use alert;

#[doc(hidden)]
macro_rules! link_entity {
    ($name:ident, $create:expr) => {
        #[no_mangle]
        unsafe extern "C" fn $name(ev: *mut sv::raw::entvars_s) {
            use $crate::private_data::Private;
            let ent = if !ev.is_null() {
                unsafe { (*ev).pContainingEntity }
            } else {
                sv::engine().create_entity()
            };
            let ent = unsafe { &mut *ent };
            ent.private_init($create);
        }
    };
}
#[doc(inline)]
pub(super) use link_entity;
