pub use xash3d_shared::macros::*;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_command {
    ($engine:expr, $name:expr, $expr:expr) => {{
        unsafe extern "C" fn command_hook() {
            use $crate::prelude::*;
            let engine = unsafe { ClientEngineRef::new() };
            let handler: fn(ClientEngineRef) = $expr;
            handler(engine);
        }

        if $engine.add_command($name, command_hook).is_err() {
            log::error!("failed to add console command {:?}", $name);
        }
    }};
}
#[doc(inline)]
pub use hook_command;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_command_key {
    ($engine:expr, $name:expr, $key:expr $(, down $down:block)? $(, up $up:block)?) => {
        $crate::macros::hook_command!($engine, concat!("+", $name), |_| {
            use $crate::input::KeyButtonExt;
            $key.key_down();
            $($down)?
        });
        $crate::macros::hook_command!($engine, concat!("-", $name), |_| {
            use $crate::input::KeyButtonExt;
            $key.key_up();
            $($up)?
        });
    };
}
#[doc(inline)]
pub use hook_command_key;

#[doc(hidden)]
#[macro_export]
macro_rules! spr_load {
    ($engine:expr, $($args:tt)+) => ({
        use core::fmt::Write;
        let buf = &mut $crate::csz::CStrArray::<256>::new();
        write!(buf.cursor(), $($args)+).ok();
        $engine.spr_load(buf.as_c_str())
    });
}
#[doc(inline)]
pub use spr_load;

#[doc(hidden)]
#[macro_export]
macro_rules! spr_get_list {
    ($engine:expr, $($args:tt)+) => ({
        use core::fmt::Write;
        let buf = &mut $crate::csz::CStrArray::<256>::new();
        write!(buf.cursor(), $($args)+).ok();
        $engine.spr_get_list(buf.as_c_str())
    });
}
#[doc(inline)]
pub use spr_get_list;
