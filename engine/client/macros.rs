#[doc(hidden)]
#[macro_export]
macro_rules! hook_message {
    ($name:ident, $block:block) => {{
        $crate::macros::hook_message!($name, |_| $block);
    }};
    ($name:ident, $handle:expr) => {{
        use core::{
            ffi::{c_char, c_int, c_void, CStr},
            slice,
        };

        unsafe extern "C" fn message_hook(
            name: *const c_char,
            size: c_int,
            msg: *mut c_void,
        ) -> c_int {
            use shared::message::{Message, MessageResult};

            let name = unsafe { CStr::from_ptr(name) };
            let raw = unsafe { slice::from_raw_parts(msg as *const u8, size as usize) };
            let mut msg = Message::new(name, raw);
            // debug!("user message {name:?} = {msg:?}");
            let handle: fn(&mut Message) -> _ = $handle;
            handle(&mut msg).convert()
        }

        let name = shared::macros::cstringify!($name);
        $crate::engine().hook_user_msg(name, Some(message_hook));
    }};
}
#[doc(inline)]
pub use hook_message;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_message_flag {
    ($name:ident, $flag:expr) => {{
        $crate::macros::hook_message!($name, |msg| {
            let value = msg.read_u8().map_or(false, |i| i != 0);
            $flag = value;
            true
        });
    }};
}
#[doc(inline)]
pub use hook_message_flag;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_command {
    ($name:expr, $block:block) => ({
        unsafe extern "C" fn command_hook() $block
        $crate::engine().add_command($name, command_hook);
    });
    ($name:expr, $expr:expr) => ({
        unsafe extern "C" fn command_hook() {
            $expr;
        }
        $crate::engine().add_command($name, command_hook);
    });
}
#[doc(inline)]
pub use hook_command;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_command_key {
    ($name:expr, $expr:expr $(, down $down:block)? $(, up $up:block)?) => {{
        use $crate::KeyButtonExt;

        unsafe extern "C" fn on_key_down() {
            $expr.key_down();
            $($down)?
        }

        unsafe extern "C" fn on_key_up() {
            $expr.key_up();
            $($up)?
        }

        let engine = engine();
        engine.add_command(concat!("+", $name), on_key_down);
        engine.add_command(concat!("-", $name), on_key_up);
    }};
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
        use $crate::raw::event_args_s;

        unsafe extern "C" fn event_hook(args: *mut event_args_s) {
            let handle: fn(&mut event_args_s) -> _ = $handle;
            handle(unsafe { &mut *args });
        }

        $crate::engine().hook_event($name, Some(event_hook));
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
        $crate::engine().spr_load(buf.as_c_str())
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
        $crate::engine().spr_get_list(buf.as_c_str())
    });
}
#[doc(inline)]
pub use spr_get_list;
