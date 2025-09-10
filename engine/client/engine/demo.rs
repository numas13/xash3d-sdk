use core::ffi::c_int;

use shared::ffi::client::demo_api_s;

pub struct DemoApi {
    raw: *mut demo_api_s,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw().$name {
            Some(func) => func,
            None => panic!("demo_api_s.{} is null", stringify!($name)),
        }
    };
}

impl DemoApi {
    pub(super) fn new(raw: *mut demo_api_s) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &demo_api_s {
        unsafe { self.raw.as_ref().unwrap() }
    }

    pub fn is_recording(&self) -> bool {
        unsafe { unwrap!(self, IsRecording)() != 0 }
    }

    pub fn is_playingback(&self) -> bool {
        unsafe { unwrap!(self, IsPlayingback)() != 0 }
    }

    pub fn is_time_demo(&self) -> bool {
        unsafe { unwrap!(self, IsTimeDemo)() != 0 }
    }

    pub fn write_buffer(&self, buffer: &[u8]) {
        // FIXME: ffi: why buffer is mutable?
        unsafe { unwrap!(self, WriteBuffer)(buffer.len() as c_int, buffer.as_ptr().cast_mut()) }
    }
}
