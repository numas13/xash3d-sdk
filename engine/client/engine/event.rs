use core::{ffi::c_int, mem::MaybeUninit};

use bitflags::bitflags;
use csz::CStrThin;
use xash3d_shared::{
    entity::EntityIndex,
    ffi::{
        self,
        api::event::event_api_s,
        common::{event_args_s, pmtrace_s, vec3_t},
        player_move::physent_s,
    },
    sound::{Attenuation, Channel, Pitch, SoundFlags},
    str::{AsCStrPtr, ToEngineStr},
};

bitflags! {
    /// Event flags.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct EventFlags: c_int {
        /// Event was invoked with stated origin.
        const ORIGIN = ffi::common::FEVENT_ORIGIN;
        /// Event was invoked with stated angles.
        const ANGLES = ffi::common::FEVENT_ANGLES;
    }
}

#[repr(transparent)]
pub struct EventArgs {
    args: event_args_s,
}

impl EventArgs {
    pub fn flags(&self) -> EventFlags {
        EventFlags::from_bits_retain(self.args.flags)
    }

    pub fn entindex(&self) -> EntityIndex {
        EntityIndex::new(self.args.entindex as u16)
            .expect("invalid entity index in event arguments")
    }

    pub fn origin(&self) -> vec3_t {
        self.args.origin.into()
    }

    pub fn angles(&self) -> vec3_t {
        self.args.angles.into()
    }

    pub fn velocity(&self) -> vec3_t {
        self.args.velocity.into()
    }

    pub fn ducking(&self) -> bool {
        self.args.ducking != 0
    }

    pub fn fparam1(&self) -> f32 {
        self.args.fparam1
    }

    pub fn fparam2(&self) -> f32 {
        self.args.fparam2
    }

    pub fn iparam1(&self) -> c_int {
        self.args.iparam1
    }

    pub fn iparam2(&self) -> c_int {
        self.args.iparam2
    }

    pub fn bparam1(&self) -> bool {
        self.args.bparam1 != 0
    }

    pub fn bparam2(&self) -> bool {
        self.args.bparam2 != 0
    }
}

pub struct SoundBuilder<'a> {
    ev: &'a EventApi,
    ent: Option<EntityIndex>,
    origin: vec3_t,
    channel: Channel,
    volume: f32,
    attenuation: Attenuation,
    flags: SoundFlags,
    pitch: Pitch,
}

impl<'a> SoundBuilder<'a> {
    pub fn entity(mut self, ent: EntityIndex) -> Self {
        self.ent = Some(ent);
        self
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = channel;
        self
    }

    pub fn channel_weapon(mut self) -> Self {
        self.channel = Channel::Weapon;
        self
    }

    pub fn channel_item(mut self) -> Self {
        self.channel = Channel::Item;
        self
    }

    pub fn channel_body(mut self) -> Self {
        self.channel = Channel::Body;
        self
    }

    pub fn channel_static(mut self) -> Self {
        self.channel = Channel::Static;
        self
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn attenuation(mut self, attn: impl Into<Attenuation>) -> Self {
        self.attenuation = attn.into();
        self
    }

    pub fn flags(mut self, flags: SoundFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn change_pitch(self) -> Self {
        self.flags(SoundFlags::CHANGE_PITCH)
    }

    pub fn pitch(mut self, pitch: impl Into<Pitch>) -> Self {
        self.pitch = pitch.into();
        self
    }

    pub fn play(self, sample: impl ToEngineStr) {
        self.ev.play_sound(
            self.ent,
            self.origin,
            self.channel,
            sample,
            self.volume,
            self.attenuation,
            self.flags,
            self.pitch,
        );
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

    /// Returns a sound builder at given origin.
    ///
    /// The defaults are:
    ///
    /// * entity index is -1 (none).
    /// * channel is [Channel::Auto].
    /// * volume is 1.0 (100%).
    /// * attenuation is [Attenuation::NORM].
    /// * flags are cleared.
    /// * pitch is [Pitch::NORM].
    pub fn build_sound_at(&self, origin: vec3_t) -> SoundBuilder<'_> {
        SoundBuilder {
            ev: self,
            ent: None,
            origin,
            channel: Channel::Auto,
            volume: 1.0,
            attenuation: Attenuation::NORM,
            flags: SoundFlags::NONE,
            pitch: Pitch::NORM,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn play_sound(
        &self,
        ent: Option<EntityIndex>,
        mut origin: vec3_t,
        channel: Channel,
        sample: impl ToEngineStr,
        volume: f32,
        attenuation: impl Into<Attenuation>,
        flags: SoundFlags,
        pitch: impl Into<Pitch>,
    ) {
        let sample = sample.to_engine_str();
        unsafe {
            // FIXME: ffi: why origin is mutable?
            unwrap!(self, EV_PlaySound)(
                ent.map_or(-1, |i| i.to_i32()),
                origin.as_mut().as_mut_ptr(),
                channel.into(),
                sample.as_ptr(),
                volume,
                attenuation.into().into(),
                flags.bits(),
                pitch.into().into(),
            )
        }
    }

    pub fn stop_sound(&self, ent: EntityIndex, channel: Channel, sample: impl ToEngineStr) {
        let ent = ent.to_i32();
        let sample = sample.to_engine_str();
        unsafe { unwrap!(self, EV_StopSound)(ent, channel.into(), sample.as_ptr()) }
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
                start.as_mut().as_mut_ptr(),
                end.as_mut().as_mut_ptr(),
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
            let ptr = unwrap!(self, EV_TraceTexture)(
                ground,
                start.as_mut().as_mut_ptr(),
                end.as_mut().as_mut_ptr(),
            );
            if !ptr.is_null() {
                Some(CStrThin::from_ptr(ptr))
            } else {
                None
            }
        }
    }

    // pub EV_StopAllSounds: Option<unsafe extern "C" fn(entnum: c_int, entchannel: c_int)>,

    pub fn kill_events(&self, ent: EntityIndex, event_name: impl ToEngineStr) {
        let event_name = event_name.to_engine_str();
        unsafe { unwrap!(self, EV_KillEvents)(ent.to_i32(), event_name.as_ptr()) }
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

#[doc(hidden)]
#[macro_export]
macro_rules! hook_event {
    ($engine:expr, $name:expr, $block:block) => {{
        $crate::macros::hook_event!($engine, $name, |_| $block);
    }};
    ($engine:expr, $name:expr, $handle:expr) => {{
        use $crate::{engine::event::EventArgs, ffi::common::event_args_s};

        unsafe extern "C" fn event_hook(args: *mut event_args_s) {
            let handle: fn(&mut EventArgs) -> _ = $handle;
            handle(unsafe { &mut *args.cast::<EventArgs>() });
        }

        $engine.hook_event($name, Some(event_hook));
    }};
}
#[doc(inline)]
pub use hook_event;
