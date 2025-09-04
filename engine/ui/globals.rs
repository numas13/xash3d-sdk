use core::{ffi::c_int, time::Duration};

use csz::{CStrArray, CStrThin};
use shared::export::impl_unsync_global;

use crate::Size;

#[allow(non_camel_case_types)]
pub type ui_globalvars_s = UiGlobalsRaw;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UiGlobalsRaw {
    pub time: f32,
    pub frametime: f32,
    pub screen_width: c_int,
    pub screen_height: c_int,
    pub max_clients: c_int,
    pub developer: c_int,
    pub demo_playback: c_int,
    pub demo_recording: c_int,
    pub demo_name: CStrArray<64>,
    pub map_title: CStrArray<64>,
}

pub struct UiGlobals {
    raw: *mut UiGlobalsRaw,
}

impl_unsync_global!(UiGlobals);

impl UiGlobals {
    pub(crate) fn new(raw: *mut UiGlobalsRaw) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> *const UiGlobalsRaw {
        self.raw
    }

    pub fn raw_mut(&self) -> *mut UiGlobalsRaw {
        self.raw
    }

    pub fn system_time_f32(&self) -> f32 {
        unsafe { (*self.raw).time }
    }

    pub fn system_time(&self) -> Duration {
        Duration::from_secs_f32(self.system_time_f32())
    }

    pub fn frame_time_f32(&self) -> f32 {
        unsafe { (*self.raw).frametime }
    }

    pub fn frame_time(&self) -> Duration {
        Duration::from_secs_f32(self.frame_time_f32())
    }

    pub fn screen_width(&self) -> c_int {
        unsafe { (*self.raw).screen_width }
    }

    pub fn screen_height(&self) -> c_int {
        unsafe { (*self.raw).screen_height }
    }

    pub fn screen_size(&self) -> Size {
        Size::new(self.screen_width(), self.screen_height())
    }

    pub fn max_clients(&self) -> c_int {
        unsafe { (*self.raw).max_clients }
    }

    pub fn developer(&self) -> c_int {
        unsafe { (*self.raw).developer }
    }

    // TODO: ui global demoplayback

    // TODO: ui global demorecording

    pub fn demo_name(&self) -> &CStrThin {
        unsafe { (*self.raw).demo_name.as_thin() }
    }

    pub fn map_title(&self) -> &CStrThin {
        unsafe { (*self.raw).map_title.as_thin() }
    }
}
