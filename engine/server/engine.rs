use core::{
    ffi::{c_char, c_int, c_long, c_uchar, c_void, CStr},
    fmt, iter,
    mem::MaybeUninit,
    ptr::{self, NonNull},
    slice,
};

use bitflags::bitflags;
use csz::{CStrArray, CStrSlice, CStrThin};
use xash3d_shared::{
    entity::EntityIndex,
    export::impl_unsync_global,
    ffi::{
        self,
        common::{cvar_s, vec3_t},
        server::{edict_s, enginefuncs_s, globalvars_t, KeyValueData, ALERT_TYPE, LEVELLIST},
    },
    sound::{Attenuation, Channel, Pitch, SoundFlags},
    str::{AsCStrPtr, ToEngineStr},
    user_message::{Angle, Coord, UserMessageValue, UserMessageWrite},
};

use crate::{
    cvar::CVarPtr,
    entity::{
        AsEdict, BaseEntity, CreateEntity, Entity, EntityOffset, EntityVars, GetPrivateData,
        KeyValue, PrivateData, PrivateEntity,
    },
    global_state::GlobalStateRef,
    globals::ServerGlobals,
    str::MapString,
    user_message::{MessageDest, ServerMessage},
};

pub use xash3d_shared::engine::{AddCmdError, EngineRef};

pub(crate) mod prelude {
    pub use xash3d_shared::engine::{
        EngineCmd, EngineCmdArgsRaw, EngineConsole, EngineCvar, EngineRng, EngineSystemTime,
    };
}

pub use self::prelude::*;

pub type ServerEngineRef = EngineRef<ServerEngine>;

#[derive(Debug)]
pub struct RegisterUserMessageError;

impl fmt::Display for RegisterUserMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to register user message")
    }
}

pub trait LevelListExt {
    fn map_name(&self) -> &CStrThin;

    fn map_name_new(&mut self) -> &mut CStrSlice;

    fn landmark_name(&self) -> &CStrThin;

    fn landmark_name_new(&mut self) -> &mut CStrSlice;
}

impl LevelListExt for LEVELLIST {
    fn map_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.mapName.as_ptr()) }
    }

    fn map_name_new(&mut self) -> &mut CStrSlice {
        CStrSlice::new_in_slice(&mut self.mapName)
    }

    fn landmark_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.landmarkName.as_ptr()) }
    }

    fn landmark_name_new(&mut self) -> &mut CStrSlice {
        CStrSlice::new_in_slice(&mut self.landmarkName)
    }
}

bitflags! {
    pub struct TraceIgnore: u32 {
        const NONE      = 0;
        const MONSTERS  = 1 << 0;
        const MISSILE   = 1 << 1;
        const GLASS     = 1 << 8;
    }
}

pub struct SoundBuilder<'a> {
    engine: &'a ServerEngine,
    channel: Channel,
    volume: f32,
    attenuation: Attenuation,
    flags: SoundFlags,
    pitch: Pitch,
}

impl<'a> SoundBuilder<'a> {
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = channel;
        self
    }

    pub fn channel_weapon(mut self) -> Self {
        self.channel = Channel::Weapon;
        self
    }

    pub fn channel_voice(mut self) -> Self {
        self.channel = Channel::Voice;
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

    pub fn emit(self, sample: impl ToEngineStr, ent: &mut impl AsEdict) {
        self.engine.emit_sound(
            ent.as_edict_mut(),
            self.channel,
            sample,
            self.volume,
            self.attenuation,
            self.flags,
            self.pitch,
        );
    }

    pub fn emit_dyn(self, sample: impl ToEngineStr, ent: &mut impl AsEdict) {
        let sample = sample.to_engine_str();
        let sample = sample.as_ref();
        if let Some(b'!') = sample.bytes().next() {
            let global_state = self.engine.global_state_ref();
            let sentences = global_state.sentences();
            if let Some(name) = sentences.find_sentence(sample) {
                self.emit(&name, ent);
            } else {
                warn!("Unable to find {sample} in sentences.txt");
            }
        } else {
            self.emit(sample, ent);
        }
    }

    pub fn stop(self, sample: impl ToEngineStr, ent: &mut impl AsEdict) {
        self.flags(SoundFlags::STOP).emit_dyn(sample, ent)
    }

    pub fn ambient_emit(self, sample: impl ToEngineStr, pos: vec3_t, ent: &mut impl AsEdict) {
        self.engine.emit_ambient_sound(
            ent.as_edict_mut(),
            pos,
            sample,
            self.volume,
            self.attenuation,
            self.flags,
            self.pitch,
        );
    }

    pub fn ambient_emit_dyn(self, sample: impl ToEngineStr, pos: vec3_t, ent: &mut impl AsEdict) {
        let sample = sample.to_engine_str();
        let sample = sample.as_ref();
        if let Some(b'!') = sample.bytes().next() {
            let global_state = self.engine.global_state_ref();
            let sentences = global_state.sentences();
            if let Some(name) = sentences.find_sentence(sample) {
                self.ambient_emit(&name, pos, ent);
            } else {
                warn!("Unable to find {sample} in sentences.txt");
            }
        } else {
            self.ambient_emit(sample, pos, ent);
        }
    }

    pub fn emit_sequential(
        self,
        group: &CStrThin,
        pick: u16,
        reset: bool,
        ent: &mut impl AsEdict,
    ) -> Option<u16> {
        let global_state = self.engine.global_state_ref();
        let sentences = global_state.sentences();
        let mut buffer = CStrArray::<256>::new();
        if let Some((next, name)) = sentences.pick_sequential(group, pick, reset, &mut buffer) {
            self.channel_voice().emit_dyn(name, ent);
            Some(next)
        } else {
            None
        }
    }
}

pub struct EntityBuilder<'a, T: Entity> {
    engine: &'a ServerEngine,
    entity: &'a mut T,
    class_name: Option<MapString>,
}

impl<'a, T: Entity> EntityBuilder<'a, T> {
    fn new(engine: &'a ServerEngine, entity: &'a mut T) -> Self {
        Self {
            class_name: entity.vars().classname(),
            engine,
            entity,
        }
    }

    pub fn class_name(mut self, class_name: impl ToEngineStr) -> Self {
        let s = self.engine.new_map_string(class_name);
        self.entity.vars_mut().as_raw_mut().classname = s.index();
        self.class_name = Some(s);
        self
    }

    pub fn target_name(self, target_name: impl ToEngineStr) -> Self {
        let s = self.engine.new_map_string(target_name);
        self.entity.vars_mut().as_raw_mut().targetname = s.index();
        self
    }

    pub fn target(self, target: impl ToEngineStr) -> Self {
        let s = self.engine.new_map_string(target);
        self.entity.vars_mut().as_raw_mut().target = s.index();
        self
    }

    pub fn key_value(self, key: &CStr, value: impl ToEngineStr) -> Self {
        let classname = self.class_name.as_deref().unwrap_or(c"null".into());
        let value = value.to_engine_str();
        let mut data = KeyValueData {
            szClassName: classname.as_ptr().cast_mut(),
            szKeyName: key.as_ptr().cast_mut(),
            szValue: value.as_ref().as_ptr().cast_mut(),
            fHandled: 0,
        };
        self.entity.key_value(KeyValue::new(&mut data));
        if data.fHandled == 0 {
            warn!("{classname}: key={key:?} is not handled");
        }
        self
    }

    pub fn vars(self, mut f: impl FnMut(&mut EntityVars)) -> Self {
        f(self.entity.vars_mut());
        self
    }

    pub fn build(self) -> &'a mut T {
        self.entity
    }
}

pub struct TraceResult {
    raw: ffi::server::TraceResult,
}

impl TraceResult {
    pub fn new(raw: ffi::server::TraceResult) -> Self {
        debug_assert!(!raw.pHit.is_null());
        Self { raw }
    }

    pub fn all_solid(&self) -> bool {
        self.raw.fAllSolid != 0
    }

    /// Returns `true` if the initial point was in a solid area.
    pub fn start_solid(&self) -> bool {
        self.raw.fStartSolid != 0
    }

    pub fn in_open(&self) -> bool {
        self.raw.fInOpen != 0
    }

    pub fn in_water(&self) -> bool {
        self.raw.fInWater != 0
    }

    /// Returns the trace completion fraction, `1.0` if the trace did not hit anything.
    pub fn fraction(&self) -> f32 {
        self.raw.flFraction
    }

    /// Returns the final trace position.
    pub fn end_position(&self) -> vec3_t {
        self.raw.vecEndPos
    }

    pub fn plane_dist(&self) -> f32 {
        self.raw.flPlaneDist
    }

    pub fn plane_normal(&self) -> vec3_t {
        self.raw.vecPlaneNormal
    }

    pub fn hit_entity(&self) -> &edict_s {
        // SAFETY: the engine returns non-null pointer
        unsafe { &*self.raw.pHit }
    }

    pub fn hit_entity_mut(&mut self) -> &mut edict_s {
        // SAFETY: the engine returns non-null pointer
        unsafe { &mut *self.raw.pHit }
    }

    /// Returns `0` for generic group and non-zero for a specific body part.
    pub fn hit_group(&self) -> u32 {
        self.raw.iHitgroup as u32
    }
}

pub struct LoadFileError(());

impl fmt::Debug for LoadFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FileError").finish()
    }
}

impl fmt::Display for LoadFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("failed to load a file")
    }
}

pub struct File {
    engine: ServerEngineRef,
    data: *mut u8,
    len: i32,
}

impl File {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.len as usize) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data, self.len as usize) }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { self.engine.free_file(self.data) }
    }
}

pub struct ServerEngine {
    raw: enginefuncs_s,
    pub globals: ServerGlobals,
}

impl_unsync_global!(ServerEngine);

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("enginefuncs_s.{} is null", stringify!($name)),
        }
    };
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl ServerEngine {
    pub(crate) fn new(raw: &enginefuncs_s, globals: *mut globalvars_t) -> Self {
        let engine = unsafe { ServerEngineRef::new() };
        Self {
            raw: *raw,
            globals: ServerGlobals::new(engine, globals),
        }
    }

    pub fn raw(&self) -> &enginefuncs_s {
        &self.raw
    }

    pub fn engine_ref(&self) -> ServerEngineRef {
        // SAFETY: we are in the game thread
        unsafe { ServerEngineRef::new() }
    }

    pub fn global_state_ref(&self) -> GlobalStateRef {
        // SAFETY: we are in the game thread
        unsafe { GlobalStateRef::new() }
    }

    pub fn precache_model(&self, name: impl ToEngineStr) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnPrecacheModel)(name.as_ptr()) }
    }

    pub fn precache_sound(&self, name: impl ToEngineStr) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnPrecacheSound)(name.as_ptr()) }
    }

    pub fn set_model(&self, ent: &mut impl AsEdict, model: impl ToEngineStr) {
        let model = model.to_engine_str();
        unsafe { unwrap!(self, pfnSetModel)(ent.as_edict_mut(), model.as_ptr()) }
    }

    pub fn model_index(&self, m: impl ToEngineStr) -> c_int {
        let m = m.to_engine_str();
        unsafe { unwrap!(self, pfnModelIndex)(m.as_ptr()) }
    }

    pub fn model_frames(&self, model_index: c_int) -> c_int {
        unsafe { unwrap!(self, pfnModelFrames)(model_index) }
    }

    pub fn set_size(&self, ent: &mut impl AsEdict, min: vec3_t, max: vec3_t) {
        unsafe { unwrap!(self, pfnSetSize)(ent.as_edict_mut(), min.as_ptr(), max.as_ptr()) }
    }

    pub fn change_level(&self, map: impl ToEngineStr, spot: impl ToEngineStr) {
        let map = map.to_engine_str();
        let spot = spot.to_engine_str();
        unsafe { unwrap!(self, pfnChangeLevel)(map.as_ptr(), spot.as_ptr()) }
    }

    // pub pfnGetSpawnParms: Option<unsafe extern "C" fn(ent: *mut edict_t)>,
    // pub pfnSaveSpawnParms: Option<unsafe extern "C" fn(ent: *mut edict_t)>,
    // pub pfnVecToYaw: Option<unsafe extern "C" fn(rgflVector: *const f32) -> f32>,
    // pub pfnVecToAngles:
    //     Option<unsafe extern "C" fn(rgflVectorIn: *const f32, rgflVectorOut: *mut f32)>,
    // pub pfnMoveToOrigin: Option<
    //     unsafe extern "C" fn(ent: *mut edict_t, pflGoal: *const f32, dist: f32, iMoveType: c_int),
    // >,
    // pub pfnChangeYaw: Option<unsafe extern "C" fn(ent: *mut edict_t)>,
    // pub pfnChangePitch: Option<unsafe extern "C" fn(ent: *mut edict_t)>,

    pub fn find_ent_by_string(
        &self,
        start_search_after: *mut edict_s,
        field: impl ToEngineStr,
        value: impl ToEngineStr,
    ) -> *mut edict_s {
        let start = start_search_after;
        let field = field.to_engine_str();
        let value = value.to_engine_str();
        unsafe { unwrap!(self, pfnFindEntityByString)(start, field.as_ptr(), value.as_ptr()) }
    }

    pub fn find_ent_by_classname(
        &self,
        start_search_after: *mut edict_s,
        name: impl ToEngineStr,
    ) -> *mut edict_s {
        self.find_ent_by_string(start_search_after, c"classname", name)
    }

    pub fn find_ent_by_target_name(
        &self,
        start_search_after: *mut edict_s,
        name: impl ToEngineStr,
    ) -> *mut edict_s {
        self.find_ent_by_string(start_search_after, c"targetname", name)
    }

    pub fn is_null_ent(&self, ent: *const edict_s) -> bool {
        ent.is_null() || self.ent_offset_of_entity(unsafe { &*ent }).is_first()
    }

    pub fn find_ent_by_string_iter<'a>(
        &'a self,
        field: impl ToEngineStr + 'a,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = NonNull<edict_s>> {
        let field = field.to_engine_str();
        let value = value.to_engine_str();
        let func = unwrap!(self, pfnFindEntityByString);
        let mut ent = unsafe { func(ptr::null_mut(), field.as_ptr(), value.as_ptr()) };
        iter::from_fn(move || {
            if !self.is_null_ent(ent) {
                let tmp = ent;
                ent = unsafe { func(ent, field.as_ptr(), value.as_ptr()) };
                Some(unsafe { NonNull::new_unchecked(tmp) })
            } else {
                None
            }
        })
    }

    pub fn find_ent_by_classname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = NonNull<edict_s>> {
        self.find_ent_by_string_iter(c"classname", value)
    }

    pub fn find_ent_by_globalname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = NonNull<edict_s>> {
        self.find_ent_by_string_iter(c"globalname", value)
    }

    pub fn find_ent_by_targetname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = NonNull<edict_s>> {
        self.find_ent_by_string_iter(c"targetname", value)
    }

    pub fn find_global_entity(
        &self,
        class_name: MapString,
        global_name: MapString,
    ) -> Option<NonNull<edict_s>> {
        self.find_ent_by_globalname_iter(&global_name).find(|&ent| {
            if let Some(entity) = unsafe { ent.as_ref() }.get_entity() {
                if entity.is_classname(&class_name) {
                    return true;
                } else {
                    debug!("Global entity found \"{global_name}\", wrong class \"{class_name}\"");
                }
            }
            false
        })
    }

    // pub pfnGetEntityIllum: Option<unsafe extern "C" fn(pEnt: *mut edict_t) -> c_int>,
    // pub pfnFindEntityInSphere: Option<
    //     unsafe extern "C" fn(
    //         pEdictStartSearchAfter: *mut edict_t,
    //         org: *const f32,
    //         rad: f32,
    //     ) -> *mut edict_t,
    // >,

    pub fn find_client_in_pvs(&self, ent: &mut impl AsEdict) -> Option<NonNull<edict_s>> {
        let ret = unsafe { unwrap!(self, pfnFindClientInPVS)(ent.as_edict_mut()) };
        NonNull::new(ret)
    }

    pub fn entities_in_pvs(&self, player: *mut edict_s) -> *mut edict_s {
        unsafe { unwrap!(self, pfnEntitiesInPVS)(player) }
    }

    /// Write results to globals().{v_forward, v_right, v_up}
    pub fn make_vectors(&self, angles: vec3_t) {
        unsafe { unwrap!(self, pfnMakeVectors)(angles.as_ptr()) }
    }

    // pub pfnAngleVectors: Option<
    //     unsafe extern "C" fn(
    //         rgflVector: *const f32,
    //         forward: *mut f32,
    //         right: *mut f32,
    //         up: *mut f32,
    //     ),
    // >,

    pub fn create_entity(&self) -> *mut edict_s {
        unsafe { unwrap!(self, pfnCreateEntity)() }
    }

    /// Call the private data destructor and immediately delete the entity.
    ///
    /// Use [EntityVars::delayed_remove](crate::entity::EntityVars::delayed_remove) instead.
    ///
    /// # Safety
    ///
    /// <div class="warning">
    ///
    /// **VERY DANGEROUS**
    ///
    /// Any access to [edict_s], [entvars_s](crate::ffi::server::entvars_s) or the associated
    /// private data after this function will result in an undefined behaviour.
    ///
    /// </div>
    pub unsafe fn remove_entity_now(&self, ent: &mut impl AsEdict) {
        unsafe { unwrap!(self, pfnRemoveEntity)(ent.as_edict_mut()) }
    }

    // pub pfnCreateNamedEntity: Option<unsafe extern "C" fn(className: c_int) -> *mut edict_t>,
    // pub pfnMakeStatic: Option<unsafe extern "C" fn(ent: *mut edict_t)>,
    // pub pfnEntIsOnFloor: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_int>,
    // pub pfnDropToFloor: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_int>,
    // pub pfnWalkMove:
    //     Option<unsafe extern "C" fn(ent: *mut edict_t, yaw: f32, dist: f32, iMode: c_int) -> c_int>,

    pub fn set_origin(&self, origin: vec3_t, ent: &mut impl AsEdict) {
        unsafe { unwrap!(self, pfnSetOrigin)(ent.as_edict_mut(), origin.as_ptr()) }
    }

    pub fn build_sound<'a>(&'a self) -> SoundBuilder<'a> {
        SoundBuilder {
            engine: self,
            channel: Channel::Auto,
            volume: 1.0,
            attenuation: Attenuation::NORM,
            flags: SoundFlags::NONE,
            pitch: Pitch::NORM,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_sound(
        &self,
        entity: &mut edict_s,
        channel: Channel,
        sample: impl ToEngineStr,
        volume: f32,
        attenuation: Attenuation,
        flags: SoundFlags,
        pitch: Pitch,
    ) {
        let sample = sample.to_engine_str();
        unsafe {
            unwrap!(self, pfnEmitSound)(
                entity,
                channel.into(),
                sample.as_ptr(),
                volume,
                attenuation.into(),
                flags.bits(),
                pitch.into(),
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_ambient_sound(
        &self,
        entity: &mut edict_s,
        mut pos: vec3_t,
        sample: impl ToEngineStr,
        volume: f32,
        attenuation: Attenuation,
        flags: SoundFlags,
        pitch: Pitch,
    ) {
        let sample = sample.to_engine_str();
        // FIXME: ffi: why pos is mutable?
        unsafe {
            unwrap!(self, pfnEmitAmbientSound)(
                entity,
                pos.as_mut_ptr(),
                sample.as_ptr(),
                volume,
                attenuation.into(),
                flags.bits(),
                pitch.into(),
            )
        }
    }

    pub fn trace_line(
        &self,
        start: vec3_t,
        end: vec3_t,
        ignore: TraceIgnore,
        ignore_ent: Option<&mut impl AsEdict>,
    ) -> TraceResult {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceLine)(
                start.as_ptr(),
                end.as_ptr(),
                ignore.bits() as c_int,
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_edict_mut()),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(unsafe { trace.assume_init() })
    }

    pub fn trace_toss(
        &self,
        ent: &mut impl AsEdict,
        ignore_ent: Option<&mut impl AsEdict>,
    ) -> TraceResult {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceToss)(
                ent.as_edict_mut(),
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_edict_mut()),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(unsafe { trace.assume_init() })
    }

    pub fn trace_monster_hull(
        &self,
        start: vec3_t,
        end: vec3_t,
        ent: &mut impl AsEdict,
        ignore: TraceIgnore,
        ignore_ent: Option<&mut impl AsEdict>,
    ) -> Option<TraceResult> {
        let mut trace = MaybeUninit::uninit();
        let result = unsafe {
            unwrap!(self, pfnTraceMonsterHull)(
                ent.as_edict_mut(),
                start.as_ptr(),
                end.as_ptr(),
                ignore.bits() as c_int,
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_edict_mut()),
                trace.as_mut_ptr(),
            )
        };
        if result != 0 {
            Some(TraceResult::new(unsafe { trace.assume_init() }))
        } else {
            None
        }
    }

    pub fn trace_hull(
        &self,
        start: vec3_t,
        end: vec3_t,
        hull_number: i32,
        ignore: TraceIgnore,
        ignore_ent: Option<&mut impl AsEdict>,
    ) -> TraceResult {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceHull)(
                start.as_ptr(),
                end.as_ptr(),
                ignore.bits() as c_int,
                hull_number,
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_edict_mut()),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(unsafe { trace.assume_init() })
    }

    pub fn trace_model(
        &self,
        start: vec3_t,
        end: vec3_t,
        hull_number: i32,
        ent: &mut impl AsEdict,
    ) -> TraceResult {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceModel)(
                start.as_ptr(),
                end.as_ptr(),
                hull_number,
                ent.as_edict_mut(),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(unsafe { trace.assume_init() })
    }

    pub fn trace_texture(
        &self,
        start: vec3_t,
        end: vec3_t,
        ent: &mut impl AsEdict,
    ) -> Option<&CStrThin> {
        let result = unsafe {
            unwrap!(self, pfnTraceTexture)(ent.as_edict_mut(), start.as_ptr(), end.as_ptr())
        };
        if !result.is_null() {
            Some(unsafe { CStrThin::from_ptr(result) })
        } else {
            None
        }
    }

    // pfnTraceSphere is obsolete and not implemented in the engine

    // pub pfnGetAimVector:
    //     Option<unsafe extern "C" fn(ent: *mut edict_t, speed: f32, rgflReturn: *mut f32)>,

    pub fn server_command(&self, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnServerCommand)(cmd.as_ptr()) }
    }

    pub fn server_execute(&self) {
        unsafe { unwrap!(self, pfnServerExecute)() }
    }

    pub fn client_command(&self, ent: &mut impl AsEdict, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        // FIXME: ffi: why szFmt is mutable?
        unsafe { unwrap!(self, pfnClientCommand)(ent.as_edict_mut(), cmd.as_ptr().cast_mut()) }
    }

    // pub pfnParticleEffect:
    //     Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, color: f32, count: f32)>,

    pub fn light_style(&self, style: c_int, value: impl ToEngineStr) {
        let value = value.to_engine_str();
        unsafe { unwrap!(self, pfnLightStyle)(style, value.as_ptr()) }
    }

    pub fn decal_index(&self, name: impl ToEngineStr) -> Option<u16> {
        let name = name.to_engine_str();
        let index = unsafe { unwrap!(self, pfnDecalIndex)(name.as_ptr()) };
        if index >= 0 {
            // TODO: use NonZeroU16 for decal index?
            index.try_into().ok()
        } else {
            None
        }
    }

    // pub pfnPointContents: Option<unsafe extern "C" fn(rgflVector: *const f32) -> c_int>,

    fn msg_send<T: ServerMessage>(
        &self,
        dest: MessageDest,
        position: Option<vec3_t>,
        ent: Option<&mut edict_s>,
        msg: &T,
    ) {
        self.msg_begin(dest, T::msg_type(None), position, ent);
        msg.msg_write_body(&mut MsgWriter { engine: self });
        self.msg_end();
    }

    pub fn msg_broadcast<T: ServerMessage>(&self, msg: &T) {
        self.msg_send(MessageDest::Broadcast, None, None, msg);
    }

    pub fn msg_all<T: ServerMessage>(&self, msg: &T) {
        self.msg_send(MessageDest::All, None, None, msg);
    }

    pub fn msg_one<T: ServerMessage>(&self, ent: &mut impl AsEdict, msg: &T) {
        self.msg_send(MessageDest::One, None, Some(ent.as_edict_mut()), msg);
    }

    pub fn msg_one_reliable<T: ServerMessage>(&self, ent: &mut impl AsEdict, msg: &T) {
        self.msg_send(
            MessageDest::OneReliable,
            None,
            Some(ent.as_edict_mut()),
            msg,
        );
    }

    pub fn msg_init<T: ServerMessage>(&self, msg: &T) {
        self.msg_send(MessageDest::Init, None, None, msg);
    }

    pub fn msg_pvs<T: ServerMessage>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::Pvs, Some(position), None, msg);
    }

    pub fn msg_pvs_reliable<T: ServerMessage>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::PvsReliable, Some(position), None, msg);
    }

    pub fn msg_pas<T: ServerMessage>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::Pas, Some(position), None, msg);
    }

    pub fn msg_reliable<T: ServerMessage>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::PasReliable, Some(position), None, msg);
    }

    pub fn msg_spec<T: ServerMessage>(&self, msg: &T) {
        self.msg_send(MessageDest::Spec, None, None, msg);
    }

    pub fn msg_begin(
        &self,
        dest: MessageDest,
        msg_type: c_int,
        origin: Option<vec3_t>,
        ent: Option<&mut edict_s>,
    ) {
        unsafe {
            unwrap!(self, pfnMessageBegin)(
                dest.into(),
                msg_type,
                origin.as_ref().map_or(ptr::null(), |v| v.as_ptr()),
                ent.map_or(ptr::null_mut(), |e| e as *mut edict_s),
            )
        }
    }

    pub fn msg_end(&self) {
        unsafe { unwrap!(self, pfnMessageEnd)() }
    }

    pub fn msg_write_u8(&self, value: u8) {
        unsafe { unwrap!(self, pfnWriteByte)(value as c_int) }
    }

    pub fn msg_write_i8(&self, value: i8) {
        unsafe { unwrap!(self, pfnWriteChar)(value as c_int) }
    }

    pub fn msg_write_u16(&self, value: u16) {
        unsafe { unwrap!(self, pfnWriteShort)(value as c_int) }
    }

    pub fn msg_write_i16(&self, value: i16) {
        unsafe { unwrap!(self, pfnWriteShort)(value as c_int) }
    }

    pub fn msg_write_u32(&self, value: u32) {
        unsafe { unwrap!(self, pfnWriteLong)(value as c_int) }
    }

    pub fn msg_write_i32(&self, value: i32) {
        unsafe { unwrap!(self, pfnWriteLong)(value) }
    }

    pub fn msg_write_angle(&self, value: f32) {
        unsafe { unwrap!(self, pfnWriteAngle)(value) }
    }

    pub fn msg_write_coord(&self, value: f32) {
        unsafe { unwrap!(self, pfnWriteCoord)(value) }
    }

    pub fn msg_write_coord_vec3(&self, v: vec3_t) {
        self.msg_write_coord(v.x());
        self.msg_write_coord(v.y());
        self.msg_write_coord(v.z());
    }

    pub fn msg_write_string(&self, value: impl ToEngineStr) {
        let value = value.to_engine_str();
        unsafe { unwrap!(self, pfnWriteString)(value.as_ptr()) }
    }

    pub fn msg_write_entity(&self, index: EntityIndex) {
        unsafe { unwrap!(self, pfnWriteEntity)(index.to_i32()) }
    }

    pub fn cvar_register(&self, cvar: &'static mut cvar_s) {
        unsafe { unwrap!(self, pfnCVarRegister)(cvar) }
    }

    pub fn alert_message(&self, atype: ALERT_TYPE, msg: impl ToEngineStr) {
        let fmt = c"%s\n".as_ptr().cast_mut();
        let msg = msg.to_engine_str();
        unsafe {
            // FIXME: ffi: why szFmt is mutable?
            unwrap!(self, pfnAlertMessage)(atype, fmt, msg.as_ptr());
        }
    }

    // pub pfnEngineFprintf: Option<unsafe extern "C" fn(pfile: *mut FILE, szFmt: *mut c_char, ...)>,

    pub fn alloc_ent_private_data(&self, edict: *mut edict_s, cb: usize) -> *mut c_void {
        let ptr = unsafe { unwrap!(self, pfnPvAllocEntPrivateData)(edict, cb as c_long) };
        assert!(!ptr.is_null());
        ptr
    }

    pub fn ent_private_data(&self, edict: &mut edict_s) -> *mut c_void {
        unsafe { unwrap!(self, pfnPvEntPrivateData)(edict) }
    }

    pub fn free_ent_private_data(&self, edict: &mut edict_s) {
        unsafe { unwrap!(self, pfnFreeEntPrivateData)(edict) }
    }

    /// Tries to create a new map string from a given `string`.
    pub fn try_alloc_map_string(&self, string: impl ToEngineStr) -> Option<MapString> {
        let string = string.to_engine_str();
        let index = unsafe { unwrap!(self, pfnAllocString)(string.as_ptr()) };
        MapString::from_index(unsafe { ServerEngineRef::new() }, index)
    }

    /// Creates a new map string from a given `string`.
    pub fn new_map_string(&self, string: impl ToEngineStr) -> MapString {
        self.try_alloc_map_string(string)
            .expect("failed to allocate a map string")
    }

    pub(crate) fn find_map_string<'a>(&self, string: &'a MapString) -> Option<&'a CStrThin> {
        let p = unsafe { unwrap!(self, pfnSzFromIndex)(string.index()) };
        if p.is_null() {
            None
        } else {
            Some(unsafe { CStrThin::from_ptr(p) })
        }
    }

    // pub pfnGetVarsOfEnt: Option<unsafe extern "C" fn(pEdict: *mut edict_t) -> *mut entvars_s>,

    pub fn entity_of_ent_offset(&self, offset: EntityOffset) -> *mut edict_s {
        let offset = offset.to_u32() as c_int;
        unsafe { unwrap!(self, pfnPEntityOfEntOffset)(offset) }
    }

    pub fn ent_offset_of_entity(&self, edict: &edict_s) -> EntityOffset {
        let offset = unsafe { unwrap!(self, pfnEntOffsetOfPEntity)(edict) };
        unsafe { EntityOffset::new_unchecked(offset.try_into().unwrap()) }
    }

    pub fn ent_index(&self, edict: &edict_s) -> EntityIndex {
        let index = unsafe { unwrap!(self, pfnIndexOfEdict)(edict) };
        unsafe { EntityIndex::new_unchecked(index.try_into().unwrap()) }
    }

    pub fn entity_of_ent_index(&self, ent: EntityIndex) -> *mut edict_s {
        unsafe { unwrap!(self, pfnPEntityOfEntIndex)(ent.to_i32()) }
    }

    // pub pfnFindEntityByVars: Option<unsafe extern "C" fn(pvars: *mut entvars_s) -> *mut edict_t>,
    // pub pfnGetModelPtr: Option<unsafe extern "C" fn(pEdict: *mut edict_t) -> *mut c_void>,

    pub fn register_user_message<'a, T>(
        &self,
        name: impl ToEngineStr,
    ) -> Result<i32, RegisterUserMessageError>
    where
        T: ServerMessage + UserMessageValue<'a>,
    {
        let id = self.register_user_message_raw(name, T::msg_size())?;
        T::msg_type(Some(id));
        Ok(id)
    }

    pub fn register_user_message_raw(
        &self,
        name: impl ToEngineStr,
        size: Option<usize>,
    ) -> Result<i32, RegisterUserMessageError> {
        let name = name.to_engine_str();
        let size = size.map_or(-1, |i| i as c_int);
        let id = unsafe { unwrap!(self, pfnRegUserMsg)(name.as_ptr(), size) };
        if id != ffi::common::svc_bad {
            debug!("register user message {id} {} (size {size})", name.as_ref());
            Ok(id)
        } else {
            error!("failed to register user message {}", name.as_ref());
            Err(RegisterUserMessageError)
        }
    }

    // pub pfnAnimationAutomove: Option<unsafe extern "C" fn(pEdict: *const edict_t, flTime: f32)>,
    // pub pfnGetBonePosition: Option<
    //     unsafe extern "C" fn(
    //         pEdict: *const edict_t,
    //         iBone: c_int,
    //         rgflOrigin: *mut f32,
    //         rgflAngles: *mut f32,
    //     ),
    // >,
    // pub pfnFunctionFromName: Option<unsafe extern "C" fn(pName: *const c_char) -> c_ulong>,
    // pub pfnNameForFunction: Option<unsafe extern "C" fn(function: c_ulong) -> *const c_char>,
    // pub pfnClientPrintf:
    //     Option<unsafe extern "C" fn(pEdict: *mut edict_t, ptype: PRINT_TYPE, szMsg: *const c_char)>,

    // pub pfnGetAttachment: Option<
    //     unsafe extern "C" fn(
    //         pEdict: *const edict_t,
    //         iAttachment: c_int,
    //         rgflOrigin: *mut f32,
    //         rgflAngles: *mut f32,
    //     ),
    // >,
    // pub pfnCRC32_Init: Option<unsafe extern "C" fn(pulCRC: *mut CRC32_t)>,
    // pub pfnCRC32_ProcessBuffer:
    //     Option<unsafe extern "C" fn(pulCRC: *mut CRC32_t, p: *const c_void, len: c_int)>,
    // pub pfnCRC32_ProcessByte: Option<unsafe extern "C" fn(pulCRC: *mut CRC32_t, ch: c_uchar)>,
    // pub pfnCRC32_Final: Option<unsafe extern "C" fn(pulCRC: CRC32_t) -> CRC32_t>,

    // pub pfnSetView: Option<unsafe extern "C" fn(pClient: *const edict_t, pViewent: *const edict_t)>,
    // pub pfnCrosshairAngle:
    //     Option<unsafe extern "C" fn(pClient: *const edict_t, pitch: f32, yaw: f32)>,

    pub fn load_file(&self, filename: impl ToEngineStr) -> Result<File, LoadFileError> {
        let filename = filename.to_engine_str();
        let mut len = 0;
        let data = unsafe { unwrap!(self, pfnLoadFileForMe)(filename.as_ptr(), &mut len) };
        if !data.is_null() {
            Ok(File {
                engine: self.engine_ref(),
                data,
                len,
            })
        } else {
            Err(LoadFileError(()))
        }
    }

    unsafe fn free_file(&self, buffer: *mut u8) {
        unsafe { unwrap!(self, pfnFreeFile)(buffer.cast()) }
    }

    // pub pfnEndSection: Option<unsafe extern "C" fn(pszSectionName: *const c_char)>,
    // pub pfnCompareFileTime: Option<
    //     unsafe extern "C" fn(
    //         filename1: *const c_char,
    //         filename2: *const c_char,
    //         iCompare: *mut c_int,
    //     ) -> c_int,
    // >,
    // pub pfnGetGameDir: Option<unsafe extern "C" fn(szGetGameDir: *mut c_char)>,
    // pub pfnCvar_RegisterVariable: Option<unsafe extern "C" fn(variable: *mut cvar_t)>,
    // pub pfnFadeClientVolume: Option<
    //     unsafe extern "C" fn(
    //         pEdict: *const edict_t,
    //         fadePercent: c_int,
    //         fadeOutSeconds: c_int,
    //         holdTime: c_int,
    //         fadeInSeconds: c_int,
    //     ),
    // >,
    // pub pfnSetClientMaxspeed:
    //     Option<unsafe extern "C" fn(pEdict: *const edict_t, fNewMaxspeed: f32)>,
    // pub pfnCreateFakeClient: Option<unsafe extern "C" fn(netname: *const c_char) -> *mut edict_t>,
    // pub pfnRunPlayerMove: Option<
    //     unsafe extern "C" fn(
    //         fakeclient: *mut edict_t,
    //         viewangles: *const f32,
    //         forwardmove: f32,
    //         sidemove: f32,
    //         upmove: f32,
    //         buttons: c_ushort,
    //         impulse: byte,
    //         msec: byte,
    //     ),
    // >,
    // pub pfnNumberOfEntities: Option<unsafe extern "C" fn() -> c_int>,
    // pub pfnGetInfoKeyBuffer: Option<unsafe extern "C" fn(e: *mut edict_t) -> *mut c_char>,
    // pub pfnInfoKeyValue: Option<
    //     unsafe extern "C" fn(infobuffer: *const c_char, key: *const c_char) -> *const c_char,
    // >,
    // pub pfnSetKeyValue:
    //     Option<unsafe extern "C" fn(infobuffer: *mut c_char, key: *mut c_char, value: *mut c_char)>,
    // pub pfnSetClientKeyValue: Option<
    //     unsafe extern "C" fn(
    //         clientIndex: c_int,
    //         infobuffer: *mut c_char,
    //         key: *mut c_char,
    //         value: *mut c_char,
    //     ),
    // >,
    // pub pfnIsMapValid: Option<unsafe extern "C" fn(filename: *mut c_char) -> c_int>,

    pub fn static_decal(
        &self,
        origin: vec3_t,
        decal_index: u16,
        entity: EntityIndex,
        model_index: u16,
    ) {
        unsafe {
            unwrap!(self, pfnStaticDecal)(
                origin.as_ptr(),
                decal_index as c_int,
                entity.to_i32(),
                model_index as c_int,
            )
        }
    }

    // pub pfnPrecacheGeneric: Option<unsafe extern "C" fn(s: *const c_char) -> c_int>,
    // pub pfnGetPlayerUserId: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_int>,
    // pub pfnBuildSoundMsg: Option<
    //     unsafe extern "C" fn(
    //         entity: *mut edict_t,
    //         channel: c_int,
    //         sample: *const c_char,
    //         volume: f32,
    //         attenuation: f32,
    //         fFlags: c_int,
    //         pitch: c_int,
    //         msg_dest: c_int,
    //         msg_type: c_int,
    //         pOrigin: *const f32,
    //         ed: *mut edict_t,
    //     ),
    // >,

    pub fn is_dedicated_server(&self) -> bool {
        unsafe { unwrap!(self, pfnIsDedicatedServer)() != 0 }
    }

    pub fn get_cvar_ptr(&self, name: impl ToEngineStr) -> CVarPtr {
        let name = name.to_engine_str();
        let ptr = unsafe { unwrap!(self, pfnCVarGetPointer)(name.as_ptr()) };
        CVarPtr::from_ptr(ptr.cast())
    }

    // pub pfnGetPlayerWONId: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_uint>,
    // pub pfnInfo_RemoveKey: Option<unsafe extern "C" fn(s: *mut c_char, key: *const c_char)>,

    pub fn get_physics_key_value(&self, client: &edict_s, key: impl ToEngineStr) -> &CStr {
        let key = key.to_engine_str();
        let ptr = unsafe { unwrap!(self, pfnGetPhysicsKeyValue)(client, key.as_ptr()) };
        assert!(!ptr.is_null());
        unsafe { CStr::from_ptr(ptr) }
    }

    pub fn set_physics_key_value(
        &self,
        client: *mut edict_s,
        key: impl ToEngineStr,
        value: impl ToEngineStr,
    ) {
        let key = key.to_engine_str();
        let value = value.to_engine_str();
        unsafe { unwrap!(self, pfnSetPhysicsKeyValue)(client, key.as_ptr(), value.as_ptr()) }
    }

    pub fn get_physics_info_string(&self, client: &edict_s) -> &CStr {
        let info = unsafe { unwrap!(self, pfnGetPhysicsInfoString)(client) };
        assert!(!info.is_null());
        unsafe { CStr::from_ptr(info) }
    }

    // pub pfnPrecacheEvent:
    //     Option<unsafe extern "C" fn(type_: c_int, psz: *const c_char) -> c_ushort>,
    // pub pfnPlaybackEvent: Option<
    //     unsafe extern "C" fn(
    //         flags: c_int,
    //         pInvoker: *const edict_t,
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

    pub fn set_pvs(&self, org: vec3_t) -> *mut c_uchar {
        unsafe { unwrap!(self, pfnSetFatPVS)(org.as_ptr()) }
    }

    pub fn set_pas(&self, org: vec3_t) -> *mut c_uchar {
        unsafe { unwrap!(self, pfnSetFatPAS)(org.as_ptr()) }
    }

    pub fn check_visibility(&self, entity: &edict_s, set: *mut c_uchar) -> bool {
        unsafe { unwrap!(self, pfnCheckVisibility)(entity, set) != 0 }
    }

    // pub pfnDeltaSetField:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldname: *const c_char)>,
    // pub pfnDeltaUnsetField:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldname: *const c_char)>,
    // pub pfnDeltaAddEncoder: Option<
    //     unsafe extern "C" fn(
    //         name: *mut c_char,
    //         conditionalencode: Option<
    //             unsafe extern "C" fn(
    //                 pFields: *mut delta_s,
    //                 from: *const c_uchar,
    //                 to: *const c_uchar,
    //             ),
    //         >,
    //     ),
    // >,
    // pub pfnGetCurrentPlayer: Option<unsafe extern "C" fn() -> c_int>,
    // pub pfnCanSkipPlayer: Option<unsafe extern "C" fn(player: *const edict_t) -> c_int>,
    // pub pfnDeltaFindField:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldname: *const c_char) -> c_int>,
    // pub pfnDeltaSetFieldByIndex:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldNumber: c_int)>,
    // pub pfnDeltaUnsetFieldByIndex:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldNumber: c_int)>,
    // pub pfnSetGroupMask: Option<unsafe extern "C" fn(mask: c_int, op: c_int)>,
    // pub pfnCreateInstancedBaseline:
    //     Option<unsafe extern "C" fn(classname: c_int, baseline: *mut entity_state_s) -> c_int>,
    // pub pfnCvar_DirectSet: Option<unsafe extern "C" fn(var: *mut cvar_s, value: *const c_char)>,
    // pub pfnForceUnmodified: Option<
    //     unsafe extern "C" fn(
    //         type_: FORCE_TYPE,
    //         mins: *mut f32,
    //         maxs: *mut f32,
    //         filename: *const c_char,
    //     ),
    // >,
    // pub pfnGetPlayerStats: Option<
    //     unsafe extern "C" fn(pClient: *const edict_t, ping: *mut c_int, packet_loss: *mut c_int),
    // >,
    // pub pfnVoice_GetClientListening:
    //     Option<unsafe extern "C" fn(iReceiver: c_int, iSender: c_int) -> qboolean>,
    // pub pfnVoice_SetClientListening: Option<
    //     unsafe extern "C" fn(iReceiver: c_int, iSender: c_int, bListen: qboolean) -> qboolean,
    // >,
    // pub pfnGetPlayerAuthId: Option<unsafe extern "C" fn(e: *mut edict_t) -> *const c_char>,
    // pub pfnSequenceGet: Option<
    //     unsafe extern "C" fn(fileName: *const c_char, entryName: *const c_char) -> *mut c_void,
    // >,
    // pub pfnSequencePickSentence: Option<
    //     unsafe extern "C" fn(
    //         groupName: *const c_char,
    //         pickMethod: c_int,
    //         picked: *mut c_int,
    //     ) -> *mut c_void,
    // >,
    // pub pfnGetFileSize: Option<unsafe extern "C" fn(filename: *const c_char) -> c_int>,
    // pub pfnGetApproxWavePlayLen: Option<unsafe extern "C" fn(filepath: *const c_char) -> c_uint>,
    // pub pfnIsCareerMatch: Option<unsafe extern "C" fn() -> c_int>,
    // pub pfnGetLocalizedStringLength: Option<unsafe extern "C" fn(label: *const c_char) -> c_int>,
    // pub pfnRegisterTutorMessageShown: Option<unsafe extern "C" fn(mid: c_int)>,
    // pub pfnGetTimesTutorMessageShown: Option<unsafe extern "C" fn(mid: c_int) -> c_int>,
    // pub pfnProcessTutorMessageDecayBuffer:
    //     Option<unsafe extern "C" fn(buffer: *mut c_int, bufferLength: c_int)>,
    // pub pfnConstructTutorMessageDecayBuffer:
    //     Option<unsafe extern "C" fn(buffer: *mut c_int, bufferLength: c_int)>,
    // pub pfnResetTutorMessageDecayData: Option<unsafe extern "C" fn()>,
    // pub pfnQueryClientCvarValue:
    //     Option<unsafe extern "C" fn(player: *const edict_t, cvarName: *const c_char)>,
    // pub pfnQueryClientCvarValue2: Option<
    //     unsafe extern "C" fn(player: *const edict_t, cvarName: *const c_char, requestID: c_int),
    // >,
    // pub pfnCheckParm:
    //     Option<unsafe extern "C" fn(parm: *mut c_char, ppnext: *mut *mut c_char) -> c_int>,
    // pub pfnPEntityOfEntIndexAllEntities:
    //     Option<unsafe extern "C" fn(iEntIndex: c_int) -> *mut edict_t>,

    pub fn new_entity_with<'a, P: PrivateEntity>(
        &'a self,
        init: impl FnMut(BaseEntity) -> P::Entity,
    ) -> EntityBuilder<'a, P::Entity> {
        let entity = unsafe {
            let engine = ServerEngineRef::new();
            let global_state = GlobalStateRef::new();
            PrivateData::create_with::<P>(engine, global_state, ptr::null_mut(), init)
        };
        EntityBuilder::new(self, entity)
    }

    pub fn new_entity<'a, P>(&'a self) -> EntityBuilder<'a, P::Entity>
    where
        P: PrivateEntity,
        P::Entity: CreateEntity,
    {
        self.new_entity_with::<P>(P::Entity::create)
    }
}

impl EngineCvar for ServerEngine {
    fn fn_get_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char) -> f32 {
        unwrap!(self, pfnCVarGetFloat)
    }

    fn fn_set_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char, value: f32) {
        unwrap!(self, pfnCVarSetFloat)
    }

    fn fn_get_cvar_string(&self) -> unsafe extern "C" fn(name: *const c_char) -> *const c_char {
        unwrap!(self, pfnCVarGetString)
    }

    fn fn_set_cvar_string(
        &self,
    ) -> unsafe extern "C" fn(name: *const c_char, value: *const c_char) {
        unwrap!(self, pfnCVarSetString)
    }
}

impl EngineRng for ServerEngine {
    fn fn_random_float(&self) -> unsafe extern "C" fn(min: f32, max: f32) -> f32 {
        unwrap!(self, pfnRandomFloat)
    }

    fn fn_random_int(&self) -> unsafe extern "C" fn(min: c_int, max: c_int) -> c_int {
        unwrap!(self, pfnRandomLong)
    }
}

impl EngineConsole for ServerEngine {
    fn console_print(&self, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe { unwrap!(self, pfnServerPrint)(msg.as_ptr()) }
    }
}

impl EngineCmd for ServerEngine {
    fn fn_cmd_argc(&self) -> unsafe extern "C" fn() -> c_int {
        unwrap!(self, pfnCmd_Argc)
    }

    fn fn_cmd_argv(&self) -> unsafe extern "C" fn(argc: c_int) -> *const c_char {
        unwrap!(self, pfnCmd_Argv)
    }

    fn add_command(
        &self,
        name: impl ToEngineStr,
        func: unsafe extern "C" fn(),
    ) -> Result<(), AddCmdError> {
        let name = name.to_engine_str();
        unsafe {
            unwrap!(self, pfnAddServerCommand)(name.as_ptr(), Some(func));
        }
        Ok(())
    }
}

impl EngineCmdArgsRaw for ServerEngine {
    fn fn_cmd_args_raw(&self) -> unsafe extern "C" fn() -> *const c_char {
        unwrap!(self, pfnCmd_Args)
    }
}

impl EngineSystemTime for ServerEngine {
    fn system_time_f64(&self) -> f64 {
        // XXX: server dll has only f32 system time
        unsafe { unwrap!(self, pfnTime)() as f64 }
    }
}

pub struct MsgWriter<'a> {
    engine: &'a ServerEngine,
}

impl UserMessageWrite for MsgWriter<'_> {
    fn write_u8(&mut self, value: u8) {
        self.engine.msg_write_u8(value);
    }

    fn write_i8(&mut self, value: i8) {
        self.engine.msg_write_i8(value);
    }

    fn write_u16(&mut self, value: u16) {
        self.engine.msg_write_u16(value);
    }

    fn write_i16(&mut self, value: i16) {
        self.engine.msg_write_i16(value);
    }

    fn write_u32(&mut self, value: u32) {
        self.engine.msg_write_u32(value);
    }

    fn write_i32(&mut self, value: i32) {
        self.engine.msg_write_i32(value);
    }

    fn write_f32(&mut self, value: f32) {
        self.engine.msg_write_u32(value.to_bits());
    }

    fn write_coord(&mut self, coord: Coord<f32>) {
        self.engine.msg_write_coord(coord.into());
    }

    fn write_angle(&mut self, angle: Angle) {
        self.engine.msg_write_angle(angle.to_degrees());
    }

    fn write_entity(&mut self, entity: EntityIndex) {
        self.engine.msg_write_entity(entity);
    }

    fn write_str(&mut self, str: impl ToEngineStr) {
        self.engine.msg_write_string(str);
    }
}
