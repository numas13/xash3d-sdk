use core::{ffi::c_int, time::Duration};

use csz::CStrThin;
use shared::{
    export::impl_unsync_global,
    ffi::menu::ui_globalvars_s,
    misc::{Rect, Size},
};

pub struct UiGlobals {
    raw: *mut ui_globalvars_s,
}

impl_unsync_global!(UiGlobals);

impl UiGlobals {
    pub(crate) fn new(raw: *mut ui_globalvars_s) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> *const ui_globalvars_s {
        self.raw
    }

    pub fn raw_mut(&self) -> *mut ui_globalvars_s {
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

    pub fn screen_width(&self) -> u32 {
        unsafe { (*self.raw).scrWidth as u32 }
    }

    pub fn screen_height(&self) -> u32 {
        unsafe { (*self.raw).scrHeight as u32 }
    }

    pub fn screen_size(&self) -> Size {
        Size::new(self.screen_width(), self.screen_height())
    }

    pub fn screen_area(&self) -> Rect {
        self.screen_size().into()
    }

    pub fn max_clients(&self) -> c_int {
        unsafe { (*self.raw).maxClients }
    }

    pub fn developer(&self) -> c_int {
        unsafe { (*self.raw).developer }
    }

    // TODO: ui global demoplayback

    // TODO: ui global demorecording

    pub fn demo_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr((*self.raw).demoname.as_ptr()) }
    }

    pub fn map_title(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr((*self.raw).maptitle.as_ptr()) }
    }
}
