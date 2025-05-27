pub use shared::message::*;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_message {
    ($name:ident, $block:block) => {{
        $crate::message::hook_message!($name, |_| $block);
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
            use $crate::message::{Message, MessageResult};

            let name = unsafe { CStr::from_ptr(name) };
            let raw = unsafe { slice::from_raw_parts(msg as *const u8, size as usize) };
            let mut msg = Message::new(name, raw);
            // debug!("user message {name:?} = {msg:?}");
            let handle: fn(&mut Message) -> _ = $handle;
            handle(&mut msg).convert()
        }

        let name = $crate::macros::cstringify!($name);
        $crate::engine().hook_user_msg(name, Some(message_hook));
    }};
}
#[doc(inline)]
pub use hook_message;

#[doc(hidden)]
#[macro_export]
macro_rules! hook_message_flag {
    ($name:ident, $flag:expr) => {{
        $crate::message::hook_message!($name, |msg| {
            let value = msg.read_u8().map_or(false, |i| i != 0);
            $flag = value;
            true
        });
    }};
}
#[doc(inline)]
pub use hook_message_flag;
