pub use shared::macros::*;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_command {
    ($name:expr, $block:block) => ({
        unsafe extern "C" fn command_hook() $block
        let engine = $crate::instance::engine();
        if engine.add_command($name, command_hook).is_err() {
            log::error!("failed to add console command {:?}", $name);
        }
    });
    ($name:expr, $expr:expr) => {
        $crate::macros::hook_command!($name, { $expr; });
    };
}
#[doc(inline)]
pub use hook_command;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_command_key {
    ($name:expr, $key:expr $(, down $down:block)? $(, up $up:block)?) => {
        $crate::macros::hook_command!(concat!("+", $name), {
            use $crate::input::KeyButtonExt;
            $key.key_down();
            $($down)?
        });
        $crate::macros::hook_command!(concat!("-", $name), {
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
macro_rules! hook_event {
    ($name:expr, $block:block) => {{
        $crate::macros::hook_event!($name, |_| $block);
    }};
    ($name:expr, $handle:expr) => {{
        use $crate::engine::event::event_args_s;

        unsafe extern "C" fn event_hook(args: *mut event_args_s) {
            let handle: fn(&mut event_args_s) -> _ = $handle;
            handle(unsafe { &mut *args });
        }

        $crate::instance::engine().hook_event($name, Some(event_hook));
    }};
}
#[doc(inline)]
pub use hook_event;

#[doc(hidden)]
#[macro_export]
macro_rules! spr_load {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let buf = &mut csz::CStrArray::<256>::new();
        write!(buf.cursor(), $($args)+).ok();
        $crate::instance::engine().spr_load(buf.as_c_str())
    });
}
#[doc(inline)]
pub use spr_load;

#[doc(hidden)]
#[macro_export]
macro_rules! spr_get_list {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let buf = &mut csz::CStrArray::<256>::new();
        write!(buf.cursor(), $($args)+).ok();
        $crate::instance::engine().spr_get_list(buf.as_c_str())
    });
}
#[doc(inline)]
pub use spr_get_list;
