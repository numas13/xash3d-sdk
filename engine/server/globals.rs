use core::{ffi::c_int, ptr::NonNull, time::Duration};

use xash3d_shared::ffi::{
    common::vec3_t,
    server::{globalvars_t, SAVERESTOREDATA},
};

use crate::{prelude::*, str::MapString};

pub struct ServerGlobals {
    engine: ServerEngineRef,
    raw: *mut globalvars_t,
}

impl ServerGlobals {
    pub(crate) fn new(engine: ServerEngineRef, raw: *mut globalvars_t) -> Self {
        Self { engine, raw }
    }

    pub fn raw(&self) -> *const globalvars_t {
        self.raw
    }

    pub fn raw_mut(&self) -> *mut globalvars_t {
        self.raw
    }

    pub fn map_time_f32(&self) -> f32 {
        unsafe { (*self.raw).time }
    }

    pub fn map_time(&self) -> Duration {
        Duration::from_secs_f32(self.map_time_f32())
    }

    pub fn map_name(&self) -> Option<MapString> {
        MapString::from_index(self.engine, unsafe { &*self.raw }.mapname)
    }

    pub fn start_spot(&self) -> Option<MapString> {
        MapString::from_index(self.engine, unsafe { &*self.raw }.startspot)
    }

    pub fn is_deathmatch(&self) -> bool {
        unsafe { (*self.raw).deathmatch != 0.0 }
    }

    pub fn forward(&self) -> vec3_t {
        unsafe { (*self.raw).v_forward }
    }

    pub fn right(&self) -> vec3_t {
        unsafe { (*self.raw).v_right }
    }

    pub fn up(&self) -> vec3_t {
        unsafe { (*self.raw).v_up }
    }

    pub fn max_clients(&self) -> c_int {
        unsafe { (*self.raw).maxClients }
    }

    pub fn save_data(&self) -> Option<NonNull<SAVERESTOREDATA>> {
        NonNull::new(unsafe { &*self.raw }.pSaveData.cast())
    }

    pub fn set_landmark_offset(&self, landmark_offset: vec3_t) {
        unsafe {
            (*self.raw).vecLandmarkOffset = landmark_offset;
        }
    }
}
