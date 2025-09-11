use core::{ffi::c_int, mem::MaybeUninit};

use csz::CStrThin;
use shared::{
    ffi::{
        api::event::event_api_s,
        common::{pmtrace_s, vec3_t},
        player_move::physent_s,
    },
    str::{AsCStrPtr, ToEngineStr},
};

use crate::raw::SoundFlags;

pub use shared::ffi::common::event_args_s;

pub trait EventArgsExt {
    fn origin(&self) -> vec3_t;

    fn angles(&self) -> vec3_t;

    fn velocity(&self) -> vec3_t;
}

impl EventArgsExt for event_args_s {
    fn origin(&self) -> vec3_t {
        self.origin.into()
    }

    fn angles(&self) -> vec3_t {
        self.angles.into()
    }

    fn velocity(&self) -> vec3_t {
        self.velocity.into()
    }
}

pub struct PmStates<'a>(&'a EventApi);

impl PmStates<'_> {
    pub fn pop(self) {}
}

impl Drop for PmStates<'_> {
    fn drop(&mut self) {
        self.0.pop_pm_states();
    }
}

pub struct EventApi {
    raw: *mut event_api_s,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw().$name {
            Some(func) => func,
            None => panic!("event_api_s.{} is null", stringify!($name)),
        }
    };
}

#[allow(dead_code)]
impl EventApi {
    pub(super) fn new(raw: *mut event_api_s) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &event_api_s {
        unsafe { self.raw.as_ref().unwrap() }
    }

    pub fn version(&self) -> c_int {
        self.raw().version
    }

    #[allow(clippy::too_many_arguments)]
    pub fn play_sound(
        &self,
        ent: c_int,
        mut origin: vec3_t,
        channel: c_int,
        sample: impl ToEngineStr,
        volume: f32,
        attenuation: f32,
        flags: SoundFlags,
        pitch: c_int,
    ) {
        let sample = sample.to_engine_str();
        unsafe {
            // FIXME: ffi: why origin is mutable?
            unwrap!(self, EV_PlaySound)(
                ent,
                origin.as_mut_ptr(),
                channel,
                sample.as_ptr(),
                volume,
                attenuation,
                flags.bits(),
                pitch,
            )
        }
    }

    pub fn stop_sound(&self, ent: c_int, channel: c_int, sample: impl ToEngineStr) {
        let sample = sample.to_engine_str();
        unsafe { unwrap!(self, EV_StopSound)(ent, channel, sample.as_ptr()) }
    }

    pub fn find_model_index(&self, modelname: impl ToEngineStr) -> c_int {
        let modelname = modelname.to_engine_str();
        unsafe { unwrap!(self, EV_FindModelIndex)(modelname.as_ptr()) }
    }

    pub fn is_local(&self, player_num: c_int) -> bool {
        unsafe { unwrap!(self, EV_IsLocal)(player_num) != 0 }
    }

    // pub EV_LocalPlayerDucking: Option<unsafe extern "C" fn() -> c_int>,

    pub fn local_player_view_height(&self) -> vec3_t {
        unsafe {
            let mut ret = MaybeUninit::<vec3_t>::uninit();
            unwrap!(self, EV_LocalPlayerViewheight)(ret.as_mut_ptr().cast());
            ret.assume_init()
        }
    }

    // pub EV_LocalPlayerBounds:
    //     Option<unsafe extern "C" fn(hull: c_int, mins: *mut f32, maxs: *mut f32)>,

    pub fn index_from_trace(&self, tr: &pmtrace_s) -> c_int {
        let tr = tr as *const pmtrace_s as *mut pmtrace_s;
        // FIXME: ffi: why pTrace is mutable?
        unsafe { unwrap!(self, EV_IndexFromTrace)(tr) }
    }

    pub fn get_phys_ent(&self, idx: c_int) -> Option<&physent_s> {
        unsafe {
            let ptr = unwrap!(self, EV_GetPhysent)(idx);
            if !ptr.is_null() {
                Some(&*ptr)
            } else {
                None
            }
        }
    }

    pub fn setup_player_predication(&self, do_pred: bool, include_local_client: bool) {
        unsafe {
            unwrap!(self, EV_SetUpPlayerPrediction)(do_pred as c_int, include_local_client as c_int)
        }
    }

    pub fn push_pm_states(&self) -> PmStates<'_> {
        unsafe { unwrap!(self, EV_PushPMStates)() }
        PmStates(self)
    }

    fn pop_pm_states(&self) {
        unsafe { unwrap!(self, EV_PopPMStates)() }
    }

    pub fn set_solid_players(&self, player_num: c_int) {
        unsafe { unwrap!(self, EV_SetSolidPlayers)(player_num) }
    }

    pub fn set_trace_hull(&self, hull: c_int) {
        unsafe { unwrap!(self, EV_SetTraceHull)(hull) }
    }

    pub fn player_trace(
        &self,
        mut start: vec3_t,
        mut end: vec3_t,
        trace_flags: c_int,
        ignore_pe: c_int,
    ) -> pmtrace_s {
        unsafe {
            let mut pm = MaybeUninit::uninit();
            // FIXME: ffi why start and ent are mutable?
            unwrap!(self, EV_PlayerTrace)(
                start.as_mut_ptr(),
                end.as_mut_ptr(),
                trace_flags,
                ignore_pe,
                pm.as_mut_ptr(),
            );
            pm.assume_init()
        }
    }

    pub fn weapon_animation(&self, sequence: c_int, body: c_int) {
        unsafe { unwrap!(self, EV_WeaponAnimation)(sequence, body) }
    }

    // pub EV_PrecacheEvent:
    //     Option<unsafe extern "C" fn(type_: c_int, psz: *const c_char) -> c_ushort>,
    // pub EV_PlaybackEvent: Option<
    //     unsafe extern "C" fn(
    //         flags: c_int,
    //         pInvoker: *const edict_s,
    //         eventindex: c_ushort,
    //         delay: f32,
    //         origin: *mut f32,
    //         angles: *mut f32,
    //         fparam1: f32,
    //         fparam2: f32,
    //         iparam1: c_int,
    //         iparam2: c_int,
    //         bparam1: c_int,
    //         bparam2: c_int,
    //     ),
    // >,

    pub fn trace_texture(
        &self,
        ground: c_int,
        mut start: vec3_t,
        mut end: vec3_t,
    ) -> Option<&CStrThin> {
        unsafe {
            // FIXME: ffi: why start and end are mutable?
            let ptr = unwrap!(self, EV_TraceTexture)(ground, start.as_mut_ptr(), end.as_mut_ptr());
            if !ptr.is_null() {
                Some(CStrThin::from_ptr(ptr))
            } else {
                None
            }
        }
    }

    // pub EV_StopAllSounds: Option<unsafe extern "C" fn(entnum: c_int, entchannel: c_int)>,

    pub fn kill_events(&self, ent: c_int, event_name: impl ToEngineStr) {
        let event_name = event_name.to_engine_str();
        unsafe { unwrap!(self, EV_KillEvents)(ent, event_name.as_ptr()) }
    }

    // pub EV_PlayerTraceExt: Option<
    //     unsafe extern "C" fn(
    //         start: *mut f32,
    //         end: *mut f32,
    //         traceFlags: c_int,
    //         pfnIgnore: Option<unsafe extern "C" fn(pe: *mut physent_s) -> c_int>,
    //         tr: *mut pmtrace_s,
    //     ),
    // >,
    // pub EV_SoundForIndex: Option<unsafe extern "C" fn(index: c_int) -> *const c_char>,
    // pub EV_TraceSurface: Option<
    //     unsafe extern "C" fn(ground: c_int, vstart: *mut f32, vend: *mut f32) -> *mut msurface_s,
    // >,
    // pub EV_GetMovevars: Option<unsafe extern "C" fn() -> *mut movevars_s>,
    // pub EV_VisTraceLine: Option<
    //     unsafe extern "C" fn(start: *mut f32, end: *mut f32, flags: c_int) -> *mut pmtrace_s,
    // >,
    // pub EV_GetVisent: Option<unsafe extern "C" fn(idx: c_int) -> *mut physent_s>,
    // pub EV_TestLine:
    //     Option<unsafe extern "C" fn(start: *mut vec3_t, end: *mut vec3_t, flags: c_int) -> c_int>,
    // pub EV_PushTraceBounds:
    //     Option<unsafe extern "C" fn(hullnum: c_int, mins: *const f32, maxs: *const f32)>,
    // pub EV_PopTraceBounds: Option<unsafe extern "C" fn()>,
}
