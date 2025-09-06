use core::ffi::{c_int, c_uchar};

#[allow(non_camel_case_types)]
pub type demo_api_s = DemoApiFunctions;

#[allow(non_snake_case)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DemoApiFunctions {
    pub IsRecording: Option<unsafe extern "C" fn() -> c_int>,
    pub IsPlayingback: Option<unsafe extern "C" fn() -> c_int>,
    pub IsTimeDemo: Option<unsafe extern "C" fn() -> c_int>,
    pub WriteBuffer: Option<unsafe extern "C" fn(size: c_int, buffer: *const c_uchar)>,
}

pub struct DemoApi<'a> {
    raw: &'a DemoApiFunctions,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("demo_api_s.{} is null", stringify!($name)),
        }
    };
}

impl<'a> DemoApi<'a> {
    pub(super) fn new(raw: &'a DemoApiFunctions) -> Self {
        Self { raw }
    }

    pub fn raw(&'a self) -> &'a DemoApiFunctions {
        self.raw
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
        unsafe { unwrap!(self, WriteBuffer)(buffer.len() as c_int, buffer.as_ptr()) }
    }
}
