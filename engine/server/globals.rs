use core::{
    ffi::{c_char, c_int, c_void},
    ptr::NonNull,
    time::Duration,
};

use shared::export::impl_unsync_global;

use crate::{
    raw::{self, edict_s, vec3_t},
    str::MapString,
};

#[allow(non_camel_case_types)]
pub type globalvars_t = ServerGlobalsRaw;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ServerGlobalsRaw {
    pub time: f32,
    pub frametime: f32,
    pub force_retouch: f32,
    pub mapname: Option<MapString>,
    pub startspot: Option<MapString>,
    pub deathmatch: f32,
    pub coop: f32,
    pub teamplay: f32,
    pub serverflags: f32,
    pub found_secrets: f32,
    pub v_forward: vec3_t,
    pub v_up: vec3_t,
    pub v_right: vec3_t,
    pub trace_allsolid: f32,
    pub trace_startsolid: f32,
    pub trace_fraction: f32,
    pub trace_endpos: vec3_t,
    pub trace_plane_normal: vec3_t,
    pub trace_plane_dist: f32,
    pub trace_ent: *mut edict_s,
    pub trace_inopen: f32,
    pub trace_inwater: f32,
    pub trace_hitgroup: c_int,
    pub trace_flags: c_int,
    pub change_level: c_int,
    pub cd_audio_track: c_int,
    pub max_clients: c_int,
    pub max_entities: c_int,
    pub string_base: *const c_char,
    pub save_data: *mut c_void,
    pub landmark_offset: vec3_t,
}

pub struct ServerGlobals {
    raw: *mut ServerGlobalsRaw,
}

impl_unsync_global!(ServerGlobals);

impl ServerGlobals {
    pub(crate) fn new(raw: *mut ServerGlobalsRaw) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> *const ServerGlobalsRaw {
        self.raw
    }

    pub fn raw_mut(&self) -> *mut ServerGlobalsRaw {
        self.raw
    }

    pub fn map_time_f32(&self) -> f32 {
        unsafe { (*self.raw).time }
    }

    pub fn map_time(&self) -> Duration {
        Duration::from_secs_f32(self.map_time_f32())
    }

    pub fn map_name(&self) -> Option<MapString> {
        unsafe { (*self.raw).mapname }
    }

    pub fn start_spot(&self) -> Option<MapString> {
        unsafe { (*self.raw).startspot }
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
        unsafe { (*self.raw).max_clients }
    }

    pub fn save_data(&self) -> Option<NonNull<raw::SAVERESTOREDATA>> {
        NonNull::new(unsafe { (*self.raw).save_data.cast() })
    }

    pub fn set_landmark_offset(&self, landmark_offset: vec3_t) {
        unsafe {
            (*self.raw).landmark_offset = landmark_offset;
        }
    }
}
