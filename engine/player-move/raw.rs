#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_void, CStr},
    mem::{self, MaybeUninit},
    slice,
    str::FromStr,
};

use csz::{CStrSlice, CStrThin};
use xash3d_shared::{
    entity::{EdictFlags, MoveType},
    ffi::{
        common::{hull_s, model_s, movevars_s, pmtrace_s, trace_t, vec3_t},
        player_move::{physent_s, playermove_s},
    },
    macros::const_assert_size_of_field_eq,
    model::ModelType,
    sound::{Attenuation, Channel, Pitch, SoundFlags},
    str::{AsCStrPtr, ToEngineStr},
};

use crate::PlayerMove;

pub struct MemFile {
    data: *mut u8,
    len: usize,
    free: unsafe extern "C" fn(buffer: *mut c_void),
}

impl MemFile {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

impl Drop for MemFile {
    fn drop(&mut self) {
        unsafe {
            (self.free)(self.data.cast());
        }
    }
}

impl PlayerMove<'_> {
    pub fn flags(&self) -> EdictFlags {
        EdictFlags::from_bits_retain(self.raw.flags)
    }

    pub fn flags_mut(&mut self) -> &mut EdictFlags {
        const_assert_size_of_field_eq!(EdictFlags, playermove_s, flags);
        unsafe { mem::transmute(&mut self.raw.flags) }
    }

    pub fn movevars(&self) -> &movevars_s {
        unsafe { &*self.raw.movevars }
    }

    pub fn usehull(&self) -> usize {
        self.raw.usehull as usize
    }

    pub fn physents(&self) -> &[physent_s] {
        &self.raw.physents[..self.raw.numphysent as usize]
    }

    pub fn physinfo(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.raw.physinfo.as_ptr()) }
    }

    pub fn is_server(&self) -> bool {
        self.raw.server != 0
    }

    pub fn is_client(&self) -> bool {
        !self.is_server()
    }

    pub fn is_multiplayer(&self) -> bool {
        self.raw.multiplayer != 0
    }

    pub fn is_singleplayer(&self) -> bool {
        !self.is_multiplayer()
    }

    pub fn is_dead(&self) -> bool {
        self.raw.dead != 0
    }

    pub fn is_alive(&self) -> bool {
        !self.is_dead()
    }

    pub fn is_spectator(&self) -> bool {
        self.raw.spectator != 0
    }

    pub fn in_water(&self) -> bool {
        self.raw.waterlevel > 1
    }

    pub fn height(&self) -> f32 {
        let usehull = self.usehull();
        self.raw.player_mins[usehull][2] + self.raw.player_maxs[usehull][2]
    }

    pub fn movetype(&self) -> MoveType {
        MoveType::from_raw(self.raw.movetype).unwrap()
    }

    pub fn texture_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.raw.sztexturename.as_ptr()) }
    }

    pub fn texture_name_clear(&mut self) -> &mut CStrSlice {
        CStrSlice::new_in_slice(&mut self.raw.sztexturename)
    }

    pub fn move_vector(&self) -> vec3_t {
        let cmd = &self.raw.cmd;
        vec3_t::new(cmd.forwardmove, cmd.sidemove, cmd.upmove)
    }

    pub fn set_move_vector(&mut self, v: vec3_t) {
        self.raw.cmd.forwardmove = v[0];
        self.raw.cmd.sidemove = v[1];
        self.raw.cmd.upmove = v[2];
    }

    pub fn is_button(&self, button: c_int) -> bool {
        self.raw.cmd.buttons as c_int & button != 0
    }
}

macro_rules! pm_unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("playermove_s.{} is null", stringify!($name)),
        }
    };
}

#[allow(dead_code)]
impl PlayerMove<'_> {
    pub fn system_time_f64(&self) -> f64 {
        unsafe { pm_unwrap!(self, Sys_FloatTime)() }
    }

    pub fn random_int(&self, min: c_int, max: c_int) -> c_int {
        assert!(min >= 0, "min must be greater than or equal to zero");
        assert!(min <= max, "min must be less than or equal to max");
        unsafe { pm_unwrap!(self, RandomLong)(min, max) }
    }

    pub fn random_float(&self, min: f32, max: f32) -> f32 {
        unsafe { pm_unwrap!(self, RandomFloat)(min, max) }
    }

    pub fn play_sound(
        &self,
        channel: Channel,
        sample: &CStr,
        volume: f32,
        attenuation: Attenuation,
        flags: SoundFlags,
        pitch: Pitch,
    ) {
        unsafe {
            pm_unwrap!(self, PM_PlaySound)(
                channel.into(),
                sample.as_ptr(),
                volume,
                attenuation.into(),
                flags.bits(),
                pitch.into(),
            );
        }
    }

    pub fn trace_texture(&self, ground: bool, start: vec3_t, end: vec3_t) -> Option<&'static CStr> {
        let mut start = start;
        let mut end = end;
        unsafe {
            // FIXME: ffi: why start and end are mutable?
            let p = pm_unwrap!(self, PM_TraceTexture)(
                ground.into(),
                start.as_mut().as_mut_ptr(),
                end.as_mut().as_mut_ptr(),
            );
            if !p.is_null() {
                Some(CStr::from_ptr(p))
            } else {
                None
            }
        }
    }

    pub fn point_contents(&self, point: vec3_t) -> (c_int, c_int) {
        let mut point = point;
        unsafe {
            let mut truecont = MaybeUninit::uninit();
            // FIXME: ffi: why point is mutable?
            let cont = pm_unwrap!(self, PM_PointContents)(
                point.as_mut().as_mut_ptr(),
                truecont.as_mut_ptr(),
            );
            (cont, truecont.assume_init())
        }
    }

    pub fn hull_point_contents(&self, hull: &hull_s, num: c_int, test: vec3_t) -> c_int {
        let mut hull = *hull;
        let mut test = test;
        // FIXME: ffi: why hull and test are mutable?
        unsafe {
            pm_unwrap!(self, PM_HullPointContents)(&mut hull, num, test.as_mut().as_mut_ptr())
        }
    }

    pub fn file_size(&self, path: &CStr) -> c_int {
        unsafe { pm_unwrap!(self, COM_FileSize)(path.as_ptr()) }
    }

    pub fn load_file(&self, path: &CStr, usehunk: c_int) -> Option<MemFile> {
        unsafe {
            let mut len = MaybeUninit::uninit();
            let data = pm_unwrap!(self, COM_LoadFile)(path.as_ptr(), usehunk, len.as_mut_ptr());
            if !data.is_null() {
                Some(MemFile {
                    data,
                    len: len.assume_init() as usize,
                    free: self.raw.COM_FreeFile.unwrap(),
                })
            } else {
                None
            }
        }
    }

    pub fn player_trace(
        &self,
        start: vec3_t,
        end: vec3_t,
        flags: c_int,
        ignore_pe: c_int,
    ) -> pmtrace_s {
        let mut start = start;
        let mut end = end;
        // FIXME: ffi: why start and end are mutable?
        unsafe {
            pm_unwrap!(self, PM_PlayerTrace)(
                start.as_mut().as_mut_ptr(),
                end.as_mut().as_mut_ptr(),
                flags,
                ignore_pe,
            )
        }
    }

    pub fn test_player_position(&self, point: vec3_t) -> (c_int, pmtrace_s) {
        let mut point = point;
        // FIXME: ffi: why point is mutable?
        unsafe {
            let mut trace = MaybeUninit::uninit();
            let hitent = pm_unwrap!(self, PM_TestPlayerPosition)(
                point.as_mut().as_mut_ptr(),
                trace.as_mut_ptr(),
            );
            (hitent, trace.assume_init())
        }
    }

    pub fn get_model_type(&self, model: &model_s) -> ModelType {
        let model = model as *const model_s;
        // FIXME: ffi: why model is mutable?
        let raw = unsafe { pm_unwrap!(self, PM_GetModelType)(model.cast_mut()) };
        ModelType::from_raw(raw).unwrap()
    }

    pub fn get_model_bounds(&self, model: &model_s) -> (vec3_t, vec3_t) {
        let model = model as *const model_s;
        // FIXME: ffi: why model is mutable?
        unsafe {
            let mut min = MaybeUninit::<vec3_t>::uninit();
            let mut max = MaybeUninit::<vec3_t>::uninit();
            pm_unwrap!(self, PM_GetModelBounds)(
                model.cast_mut(),
                min.as_mut_ptr().cast(),
                max.as_mut_ptr().cast(),
            );
            (min.assume_init(), max.assume_init())
        }
    }

    pub fn hull_for_bsp(&self, pe: &physent_s) -> (*mut hull_s, vec3_t) {
        let pe = pe as *const physent_s;
        // FIXME: ffi: why pe is mutable?
        unsafe {
            let mut vec = MaybeUninit::<vec3_t>::uninit();
            let hull = pm_unwrap!(self, PM_HullForBsp)(pe.cast_mut(), vec.as_mut_ptr().cast());
            (hull.cast(), vec.assume_init())
        }
    }

    pub fn trace_model(&self, pe: &physent_s, start: vec3_t, end: vec3_t) -> (trace_t, f32) {
        let pe = pe as *const physent_s;
        let mut start = start;
        let mut end = end;
        // FIXME: ffi: why pe, start and end are mutable?
        unsafe {
            let mut trace = MaybeUninit::uninit();
            let ret = pm_unwrap!(self, PM_TraceModel)(
                pe.cast_mut(),
                start.as_mut().as_mut_ptr(),
                end.as_mut().as_mut_ptr(),
                trace.as_mut_ptr(),
            );
            (trace.assume_init(), ret)
        }
    }

    pub fn info_value_for_key_raw(
        &self,
        physinfo: impl ToEngineStr,
        key: impl ToEngineStr,
    ) -> *const c_char {
        let physinfo = physinfo.to_engine_str();
        let key = key.to_engine_str();
        unsafe { pm_unwrap!(self, PM_Info_ValueForKey)(physinfo.as_ptr(), key.as_ptr()) }
    }

    pub fn info_value_for_key<T: FromStr>(
        &self,
        physinfo: impl ToEngineStr,
        key: impl ToEngineStr,
    ) -> Option<T> {
        let raw = self.info_value_for_key_raw(physinfo, key);
        let s = unsafe { CStr::from_ptr(raw) }.to_str().ok()?;
        s.parse::<T>().ok()
    }

    pub fn stuck_touch(&self, hitent: c_int, trace_result: &mut pmtrace_s) {
        unsafe {
            pm_unwrap!(self, PM_StuckTouch)(hitent, trace_result);
        }
    }

    pub fn particle(&self, origin: vec3_t, color: c_int, life: f32, zpos: c_int, zvel: c_int) {
        unsafe {
            pm_unwrap!(self, PM_Particle)(origin.as_ref().as_ptr(), color, life, zpos, zvel);
        }
    }
}
