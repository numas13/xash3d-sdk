#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_ushort, c_void, CStr},
    mem::MaybeUninit,
    slice,
    str::FromStr,
};

use csz::CStrArray;
use shared::{
    consts::MAX_PHYSENTS,
    raw::{
        byte, hull_s, model_s, movevars_s, msurface_s, physent_s, pmtrace_s, qboolean, trace_t,
        usercmd_s, vec3_t, EdictFlags, ModelType, MoveEnts, MoveType, SoundFlags,
    },
};

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

#[repr(C)]
pub struct playermove_s {
    pub player_index: c_int,
    pub server: qboolean,
    pub multiplayer: qboolean,
    pub time: f32,
    pub frametime: f32,
    pub forward: vec3_t,
    pub right: vec3_t,
    pub up: vec3_t,
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub oldangles: vec3_t,
    pub velocity: vec3_t,
    pub movedir: vec3_t,
    pub basevelocity: vec3_t,
    pub view_ofs: vec3_t,
    pub duck_time: f32,
    pub in_duck: qboolean,
    pub time_step_sound: c_int,
    pub step_left: c_int,
    pub flFallVelocity: f32,
    pub punchangle: vec3_t,
    pub swim_time: f32,
    pub flNextPrimaryAttack: f32,
    pub effects: c_int,
    pub flags: EdictFlags,
    pub usehull: c_int,
    pub gravity: f32,
    pub friction: f32,
    pub oldbuttons: c_int,
    pub waterjumptime: f32,
    pub dead: qboolean,
    pub deadflag: c_int,
    pub spectator: c_int,
    pub movetype: MoveType,
    pub onground: c_int,
    pub waterlevel: c_int,
    pub watertype: c_int,
    pub oldwaterlevel: c_int,
    pub sztexturename: CStrArray<256>,
    pub chtexturetype: c_char,
    pub maxspeed: f32,
    pub clientmaxspeed: f32,
    pub iuser1: c_int,
    pub iuser2: c_int,
    pub iuser3: c_int,
    pub iuser4: c_int,
    pub fuser1: f32,
    pub fuser2: f32,
    pub fuser3: f32,
    pub fuser4: f32,
    pub vuser1: vec3_t,
    pub vuser2: vec3_t,
    pub vuser3: vec3_t,
    pub vuser4: vec3_t,
    pub numphysent: c_int,
    pub physents: [physent_s; MAX_PHYSENTS],
    pub moveents: MoveEnts,
    pub numvisent: c_int,
    pub visents: [physent_s; MAX_PHYSENTS],
    pub cmd: usercmd_s,
    pub numtouch: c_int,
    pub touchindex: [pmtrace_s; MAX_PHYSENTS],
    pub physinfo: CStrArray<256>,
    pub movevars: *mut movevars_s,
    pub player_mins: [vec3_t; 4],
    pub player_maxs: [vec3_t; 4],
    pub PM_Info_ValueForKey:
        Option<unsafe extern "C" fn(s: *const c_char, key: *const c_char) -> *const c_char>,
    pub PM_Particle: Option<
        unsafe extern "C" fn(origin: *const f32, color: c_int, life: f32, zpos: c_int, zvel: c_int),
    >,
    pub PM_TestPlayerPosition:
        Option<unsafe extern "C" fn(pos: *const f32, ptrace: *mut pmtrace_s) -> c_int>,
    pub Con_NPrintf: Option<unsafe extern "C" fn(idx: c_int, fmt: *const c_char, ...)>,
    pub Con_DPrintf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Sys_FloatTime: Option<unsafe extern "C" fn() -> f64>,
    pub PM_StuckTouch: Option<unsafe extern "C" fn(hitent: c_int, ptraceresult: *mut pmtrace_s)>,
    pub PM_PointContents:
        Option<unsafe extern "C" fn(p: *const f32, truecontents: *mut c_int) -> c_int>,
    pub PM_TruePointContents: Option<unsafe extern "C" fn(p: *mut f32) -> c_int>,
    pub PM_HullPointContents:
        Option<unsafe extern "C" fn(hull: *const hull_s, num: c_int, p: *const f32) -> c_int>,
    pub PM_PlayerTrace: Option<
        unsafe extern "C" fn(
            start: *const f32,
            end: *const f32,
            traceFlags: c_int,
            ignore_pe: c_int,
        ) -> pmtrace_s,
    >,
    pub PM_TraceLine: Option<
        unsafe extern "C" fn(
            start: *mut f32,
            end: *mut f32,
            flags: c_int,
            usehulll: c_int,
            ignore_pe: c_int,
        ) -> *mut pmtrace_s,
    >,
    pub RandomLong: Option<unsafe extern "C" fn(lLow: c_int, lHigh: c_int) -> c_int>,
    pub RandomFloat: Option<unsafe extern "C" fn(flLow: f32, flHigh: f32) -> f32>,
    pub PM_GetModelType: Option<unsafe extern "C" fn(mod_: *const model_s) -> ModelType>,
    pub PM_GetModelBounds:
        Option<unsafe extern "C" fn(mod_: *const model_s, mins: *mut f32, maxs: *mut f32)>,
    pub PM_HullForBsp:
        Option<unsafe extern "C" fn(pe: *const physent_s, offset: *mut f32) -> *mut c_void>,
    pub PM_TraceModel: Option<
        unsafe extern "C" fn(
            pEnt: *const physent_s,
            start: *const f32,
            end: *const f32,
            trace: *mut trace_t,
        ) -> f32,
    >,
    pub COM_FileSize: Option<unsafe extern "C" fn(filename: *const c_char) -> c_int>,
    pub COM_LoadFile: Option<
        unsafe extern "C" fn(path: *const c_char, usehunk: c_int, pLength: *mut c_int) -> *mut byte,
    >,
    pub COM_FreeFile: Option<unsafe extern "C" fn(buffer: *mut c_void)>,
    pub memfgets: Option<
        unsafe extern "C" fn(
            pMemFile: *mut byte,
            fileSize: c_int,
            pFilePos: *mut c_int,
            pBuffer: *mut c_char,
            bufferSize: c_int,
        ) -> *mut c_char,
    >,
    pub runfuncs: qboolean,
    pub PM_PlaySound: Option<
        unsafe extern "C" fn(
            channel: c_int,
            sample: *const c_char,
            volume: f32,
            attenuation: f32,
            flags: SoundFlags,
            pitch: c_int,
        ),
    >,
    pub PM_TraceTexture: Option<
        unsafe extern "C" fn(ground: c_int, vstart: *const f32, vend: *const f32) -> *const c_char,
    >,
    pub PM_PlaybackEventFull: Option<
        unsafe extern "C" fn(
            flags: c_int,
            clientindex: c_int,
            eventindex: c_ushort,
            delay: f32,
            origin: *mut f32,
            angles: *mut f32,
            fparam1: f32,
            fparam2: f32,
            iparam1: c_int,
            iparam2: c_int,
            bparam1: c_int,
            bparam2: c_int,
        ),
    >,
    pub PM_PlayerTraceEx: Option<
        unsafe extern "C" fn(
            start: *mut f32,
            end: *mut f32,
            traceFlags: c_int,
            pfnIgnore: Option<unsafe extern "C" fn(pe: *mut physent_s) -> c_int>,
        ) -> pmtrace_s,
    >,
    pub PM_TestPlayerPositionEx: Option<
        unsafe extern "C" fn(
            pos: *mut f32,
            ptrace: *mut pmtrace_s,
            pfnIgnore: Option<unsafe extern "C" fn(pe: *mut physent_s) -> c_int>,
        ) -> c_int,
    >,
    pub PM_TraceLineEx: Option<
        unsafe extern "C" fn(
            start: *mut f32,
            end: *mut f32,
            flags: c_int,
            usehulll: c_int,
            pfnIgnore: Option<unsafe extern "C" fn(pe: *mut physent_s) -> c_int>,
        ) -> *mut pmtrace_s,
    >,
    pub PM_TraceSurface: Option<
        unsafe extern "C" fn(ground: c_int, vstart: *mut f32, vend: *mut f32) -> *mut msurface_s,
    >,
}

macro_rules! pm_unwrap {
    ($self:expr, $name:ident) => {
        match $self.$name {
            Some(func) => func,
            None => panic!("playermove_s.{} is null", stringify!($name)),
        }
    };
}

impl playermove_s {
    pub fn movevars(&self) -> &movevars_s {
        unsafe { &*self.movevars }
    }

    pub fn usehull(&self) -> usize {
        self.usehull as usize
    }

    pub fn physents(&self) -> &[physent_s] {
        &self.physents[..self.numphysent as usize]
    }

    pub fn physinfo(&self) -> &CStr {
        self.physinfo.as_c_str()
    }

    pub fn is_server(&self) -> bool {
        self.server != 0
    }

    pub fn is_client(&self) -> bool {
        !self.is_server()
    }

    pub fn is_multiplayer(&self) -> bool {
        self.multiplayer != 0
    }

    pub fn is_singleplayer(&self) -> bool {
        !self.is_multiplayer()
    }

    pub fn is_dead(&self) -> bool {
        self.dead != 0
    }

    pub fn is_alive(&self) -> bool {
        !self.is_dead()
    }

    pub fn is_spectator(&self) -> bool {
        self.spectator != 0
    }

    pub fn in_water(&self) -> bool {
        self.waterlevel > 1
    }

    pub fn height(&self) -> f32 {
        let usehull = self.usehull as usize;
        self.player_mins[usehull][2] + self.player_maxs[usehull][2]
    }

    pub fn float_time(&self) -> f64 {
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
        channel: c_int,
        sample: &CStr,
        volume: f32,
        attenuation: f32,
        flags: SoundFlags,
        pitch: c_int,
    ) {
        unsafe {
            pm_unwrap!(self, PM_PlaySound)(
                channel,
                sample.as_ptr(),
                volume,
                attenuation,
                flags,
                pitch,
            );
        }
    }

    pub fn trace_texture(&self, ground: bool, start: vec3_t, end: vec3_t) -> Option<&'static CStr> {
        unsafe {
            let p = pm_unwrap!(self, PM_TraceTexture)(ground.into(), start.as_ptr(), end.as_ptr());
            if !p.is_null() {
                Some(CStr::from_ptr(p))
            } else {
                None
            }
        }
    }

    pub fn point_contents(&self, point: vec3_t) -> (c_int, c_int) {
        unsafe {
            let mut truecont = MaybeUninit::uninit();
            let cont = pm_unwrap!(self, PM_PointContents)(point.as_ptr(), truecont.as_mut_ptr());
            (cont, truecont.assume_init())
        }
    }

    pub fn hull_point_contents(&self, hull: &hull_s, num: c_int, test: vec3_t) -> c_int {
        unsafe { pm_unwrap!(self, PM_HullPointContents)(hull, num, test.as_ptr()) }
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
                    free: self.COM_FreeFile.unwrap(),
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
        flags: u32,
        ignore_pe: c_int,
    ) -> pmtrace_s {
        unsafe {
            pm_unwrap!(self, PM_PlayerTrace)(
                start.as_ptr(),
                end.as_ptr(),
                flags as c_int,
                ignore_pe,
            )
        }
    }

    pub fn test_player_position(&self, point: vec3_t) -> (c_int, pmtrace_s) {
        unsafe {
            let mut trace = MaybeUninit::uninit();
            let hitent =
                pm_unwrap!(self, PM_TestPlayerPosition)(point.as_ptr(), trace.as_mut_ptr());
            (hitent, trace.assume_init())
        }
    }

    pub fn get_model_type(&self, model: &model_s) -> ModelType {
        unsafe { pm_unwrap!(self, PM_GetModelType)(model) }
    }

    pub fn get_model_bounds(&self, model: &model_s) -> (vec3_t, vec3_t) {
        unsafe {
            let mut min = MaybeUninit::<vec3_t>::uninit();
            let mut max = MaybeUninit::<vec3_t>::uninit();
            pm_unwrap!(self, PM_GetModelBounds)(
                model,
                min.as_mut_ptr().cast(),
                max.as_mut_ptr().cast(),
            );
            (min.assume_init(), max.assume_init())
        }
    }

    pub fn hull_for_bsp(&self, pe: &physent_s) -> (*mut hull_s, vec3_t) {
        unsafe {
            let mut vec = MaybeUninit::<vec3_t>::uninit();
            let hull = pm_unwrap!(self, PM_HullForBsp)(pe, vec.as_mut_ptr().cast());
            (hull.cast(), vec.assume_init())
        }
    }

    pub fn trace_model(&self, pe: &physent_s, start: vec3_t, end: vec3_t) -> (trace_t, f32) {
        unsafe {
            let mut trace = MaybeUninit::uninit();
            let ret = pm_unwrap!(self, PM_TraceModel)(
                pe,
                start.as_ptr(),
                end.as_ptr(),
                trace.as_mut_ptr(),
            );
            (trace.assume_init(), ret)
        }
    }

    pub fn info_value_for_key_raw(&self, physinfo: &CStr, key: &CStr) -> *const c_char {
        unsafe { pm_unwrap!(self, PM_Info_ValueForKey)(physinfo.as_ptr(), key.as_ptr()) }
    }

    pub fn info_value_for_key<T: FromStr>(&self, physinfo: &CStr, key: &CStr) -> Option<T> {
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
            pm_unwrap!(self, PM_Particle)(origin.as_ptr(), color, life, zpos, zvel);
        }
    }
}
