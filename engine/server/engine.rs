use core::{
    cmp,
    ffi::{c_char, c_int, c_long, c_uchar, c_void, CStr},
    fmt,
    hash::{BuildHasher, Hasher},
    iter,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    ptr::{self, NonNull},
    slice,
    time::Duration,
};

use bitflags::bitflags;
use csz::{CStrArray, CStrSlice, CStrThin};
use xash3d_shared::{
    consts::{Contents, MAX_SYSPATH},
    entity::{Buttons, EntityIndex},
    export::impl_unsync_global,
    ffi::{
        self,
        common::{cvar_s, cvar_t, entity_state_s, vec3_t},
        server::{
            edict_s, enginefuncs_s, entvars_s, globalvars_t, CRC32_t, KeyValueData, ALERT_TYPE,
            LEVELLIST,
        },
    },
    macros::define_enum_for_primitive,
    sound::{Attenuation, Channel, Pitch, SoundFlags},
    str::{AsCStrPtr, ToEngineStr},
    user_message::{Angle, Coord, UserMessageValue, UserMessageWrite},
    utils::cstr_or_none,
};

use crate::{
    cvar::CVarPtr,
    entity::{
        AsEntityHandle, BaseEntity, CreateEntity, Entity, EntityHandle, EntityHandleRef,
        EntityOffset, EntityVars, GetPrivateData, KeyValue,
    },
    global_state::GlobalStateRef,
    globals::ServerGlobals,
    private::{PrivateData, PrivateEntity},
    str::MapString,
    user_message::{MessageDest, ServerMessage},
};

#[cfg(feature = "save")]
use crate::save::{self, Restore, Save};

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

    pub fn emit(self, sample: impl ToEngineStr, ent: &impl AsEntityHandle) {
        self.engine.emit_sound(
            ent,
            self.channel,
            sample,
            self.volume,
            self.attenuation,
            self.flags,
            self.pitch,
        );
    }

    pub fn emit_dyn(self, sample: impl ToEngineStr, ent: &impl AsEntityHandle) {
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

    pub fn stop(self, sample: impl ToEngineStr, ent: &impl AsEntityHandle) {
        self.flags(SoundFlags::STOP).emit_dyn(sample, ent)
    }

    pub fn ambient_emit(self, sample: impl ToEngineStr, pos: vec3_t, ent: &impl AsEntityHandle) {
        self.engine.emit_ambient_sound(
            ent,
            pos,
            sample,
            self.volume,
            self.attenuation,
            self.flags,
            self.pitch,
        );
    }

    pub fn ambient_emit_dyn(
        self,
        sample: impl ToEngineStr,
        pos: vec3_t,
        ent: &impl AsEntityHandle,
    ) {
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
        ent: &impl AsEntityHandle,
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

    pub fn emit_random_sentence(self, group: &CStrThin, ent: &impl AsEntityHandle) -> Option<u16> {
        let global_state = self.engine.global_state_ref();
        let sentences = global_state.sentences();
        let mut buffer = CStrArray::<256>::new();
        if let Some((played, name)) = sentences.pick_random(group, &mut buffer) {
            trace!("play random sentence {name}");
            self.channel_voice().emit_dyn(name, ent);
            Some(played)
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
        self.entity.vars().set_classname(s);
        self.class_name = Some(s);
        self
    }

    pub fn target_name(self, target_name: impl ToEngineStr) -> Self {
        let s = self.engine.new_map_string(target_name);
        self.entity.vars().set_target_name(s);
        self
    }

    pub fn target(self, target: impl ToEngineStr) -> Self {
        let s = self.engine.new_map_string(target);
        self.entity.vars().set_target(s);
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

    pub fn vars(self, mut f: impl FnMut(&EntityVars)) -> Self {
        f(self.entity.vars());
        self
    }

    pub fn build(self) -> &'a mut T {
        self.entity
    }
}

pub struct TraceResult<'a> {
    engine: ServerEngineRef,
    raw: ffi::server::TraceResult,
    phantom: PhantomData<&'a ServerEngine>,
}

impl<'a> TraceResult<'a> {
    pub fn new(engine: &'a ServerEngine, raw: ffi::server::TraceResult) -> Self {
        debug_assert!(!raw.pHit.is_null());
        Self {
            engine: engine.engine_ref(),
            raw,
            phantom: PhantomData,
        }
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

    // TODO: return Option if fraction is 1.0?
    pub fn hit_entity(&self) -> EntityHandleRef<'a> {
        // SAFETY: the engine returns non-null pointer
        unsafe { EntityHandleRef::new_unchecked(self.engine, self.raw.pHit) }
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum EndSection {
    #[default]
    Logo,
    Demo,
    Training,
    Credits,
}

impl EndSection {
    pub fn as_c_str(&self) -> &CStr {
        match self {
            Self::Logo => c"_oem_end_logo",
            Self::Demo => c"_oem_end_demo",
            Self::Training => c"_oem_end_training",
            Self::Credits => c"oem_end_credits",
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum MoveToOriginType {
    /// Normal move in the direction monster is facing.
    #[default]
    Normal = 0,
    /// Moves in direction specified, no matter which way monster is facing.
    Strafe = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DropToFloorResult {
    AllSolid = -1,
    False = 0,
    True = 1,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum WalkMove {
    #[default]
    /// Normal walkmove.
    Normal = 0,
    /// Doesn't hit ANY entities, no matter what the solid type.
    WorldOnly = 1,
    /// Move, but don't touch triggers.
    CheckOnly = 2,
}

pub struct Crc32Hasher {
    engine: ServerEngineRef,
    state: CRC32_t,
}

impl Hasher for Crc32Hasher {
    fn finish(&self) -> u64 {
        self.engine.crc32_finish(self.state) as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.engine.crc32_process_bytes(&mut self.state, bytes);
    }

    fn write_u8(&mut self, i: u8) {
        self.engine.crc32_process_byte(&mut self.state, i);
    }

    fn write_i8(&mut self, i: i8) {
        self.engine.crc32_process_byte(&mut self.state, i as u8);
    }
}

pub struct BuildCrc32Hasher {
    engine: ServerEngineRef,
}

impl BuildCrc32Hasher {
    pub fn new(engine: &ServerEngine) -> Self {
        Self {
            engine: engine.engine_ref(),
        }
    }
}

impl BuildHasher for BuildCrc32Hasher {
    type Hasher = Crc32Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        self.engine.crc32_hasher()
    }
}

pub struct InfoBuffer<'a> {
    engine: ServerEngineRef,
    info_buffer: *mut c_char,
    phantom: PhantomData<&'a ServerEngine>,
}

impl<'a> InfoBuffer<'a> {
    /// Creates a new info buffer.
    ///
    /// # Safety
    ///
    /// The info buffer pointer must be non-null and received from the engine.
    pub unsafe fn new(engine: &'a ServerEngine, info_buffer: *mut c_char) -> Self {
        Self {
            engine: engine.engine_ref(),
            info_buffer,
            phantom: PhantomData,
        }
    }

    pub fn as_thin(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.info_buffer) }
    }

    pub fn get(&self, key: impl ToEngineStr) -> &CStrThin {
        self.engine.info_buffer_get(self.info_buffer, key)
    }

    pub fn set(&mut self, key: impl ToEngineStr, value: impl ToEngineStr) {
        self.engine.info_buffer_set(self.info_buffer, key, value);
    }

    pub fn remove(&mut self, key: impl ToEngineStr) {
        self.engine.info_buffer_remove(self.info_buffer, key);
    }
}

impl Deref for InfoBuffer<'_> {
    type Target = CStrThin;

    fn deref(&self) -> &Self::Target {
        self.as_thin()
    }
}

pub struct ClientInfoBuffer<'a> {
    engine: ServerEngineRef,
    entity: EntityHandle,
    info_buffer: *mut c_char,
    phantom: PhantomData<&'a ServerEngine>,
}

impl<'a> ClientInfoBuffer<'a> {
    /// Creates a new client info buffer.
    ///
    /// # Safety
    ///
    /// The pointer must be non-null and received from the engine.
    pub unsafe fn new(
        engine: ServerEngineRef,
        entity: EntityHandle,
        info_buffer: *mut c_char,
    ) -> Self {
        Self {
            engine: engine.engine_ref(),
            entity,
            info_buffer,
            phantom: PhantomData,
        }
    }

    pub fn entity(&self) -> EntityHandle {
        self.entity
    }

    pub fn as_thin(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.info_buffer) }
    }

    pub fn get(&self, key: impl ToEngineStr) -> &CStrThin {
        self.engine.info_buffer_get(self.info_buffer, key)
    }

    pub fn set(&mut self, key: impl ToEngineStr, value: impl ToEngineStr) {
        let index = self.engine.get_entity_index(&self.entity);
        self.engine
            .info_buffer_client_set(index, self.info_buffer, key, value);
    }

    pub fn remove(&mut self, key: impl ToEngineStr) {
        self.engine.info_buffer_remove(self.info_buffer, key);
    }
}

impl Deref for ClientInfoBuffer<'_> {
    type Target = CStrThin;

    fn deref(&self) -> &Self::Target {
        self.as_thin()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventIndex(u16);

impl EventIndex {
    pub fn to_u16(self) -> u16 {
        self.0
    }
}

#[cfg(feature = "save")]
impl Save for EventIndex {
    fn save(&self, state: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        self.0.save(state, cur)
    }
}

#[cfg(feature = "save")]
impl Restore for EventIndex {
    fn restore(
        &mut self,
        state: &save::RestoreState,
        cur: &mut save::Cursor,
    ) -> save::SaveResult<()> {
        self.0.restore(state, cur)
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    struct PlaybackEventFlags: u32 {
        const NONE      = 0;
        const NOT_HOST  = 1 << 0;
        const RELIABLE  = 1 << 1;
        const GLOBAL    = 1 << 2;
        const UPDATE    = 1 << 3;
        const HOST_ONLY = 1 << 4;
        const SERVER    = 1 << 5;
        const CLIENT    = 1 << 6;
    }
}

pub struct PlaybackEventBuilder {
    engine: ServerEngineRef,
    flags: PlaybackEventFlags,
    delay: f32,
    origin: vec3_t,
    angles: vec3_t,
    fparam1: f32,
    fparam2: f32,
    iparam1: c_int,
    iparam2: c_int,
    bparam1: c_int,
    bparam2: c_int,
}

impl PlaybackEventBuilder {
    fn add_flags(mut self, flags: PlaybackEventFlags) -> Self {
        self.flags = self.flags.union(flags);
        self
    }

    pub fn not_host(self) -> Self {
        self.add_flags(PlaybackEventFlags::NOT_HOST)
    }

    pub fn reliable(self) -> Self {
        self.add_flags(PlaybackEventFlags::RELIABLE)
    }

    pub fn global(self) -> Self {
        self.add_flags(PlaybackEventFlags::GLOBAL)
    }

    pub fn update(self) -> Self {
        self.add_flags(PlaybackEventFlags::UPDATE)
    }

    pub fn host_only(self) -> Self {
        self.add_flags(PlaybackEventFlags::HOST_ONLY)
    }

    pub fn server(self) -> Self {
        self.add_flags(PlaybackEventFlags::SERVER)
    }

    pub fn client(self) -> Self {
        self.add_flags(PlaybackEventFlags::CLIENT)
    }

    pub fn delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }

    pub fn origin(mut self, origin: vec3_t) -> Self {
        self.origin = origin;
        self
    }

    pub fn angles(mut self, angles: vec3_t) -> Self {
        self.angles = angles;
        self
    }

    pub fn fparam1(mut self, value: f32) -> Self {
        self.fparam1 = value;
        self
    }

    pub fn fparam2(mut self, value: f32) -> Self {
        self.fparam2 = value;
        self
    }

    pub fn iparam1(mut self, value: c_int) -> Self {
        self.iparam1 = value;
        self
    }

    pub fn iparam2(mut self, value: c_int) -> Self {
        self.iparam2 = value;
        self
    }

    pub fn bparam1(mut self, value: bool) -> Self {
        self.bparam1 = value as i32;
        self
    }

    pub fn bparam2(mut self, value: bool) -> Self {
        self.bparam2 = value as i32;
        self
    }

    pub fn bparam1_raw(mut self, value: c_int) -> Self {
        self.bparam1 = value;
        self
    }

    pub fn bparam2_raw(mut self, value: c_int) -> Self {
        self.bparam2 = value;
        self
    }

    pub fn build(self, event_index: EventIndex, invoker: &impl AsEntityHandle) {
        self.engine.playback_event(
            self.flags.bits() as i32,
            invoker,
            event_index,
            self.delay,
            self.origin,
            self.angles,
            self.fparam1,
            self.fparam2,
            self.iparam1,
            self.iparam2,
            self.bparam1,
            self.bparam2,
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GroupOp {
    And = 0,
    Nand = 1,
}

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum ForceType: ffi::server::FORCE_TYPE {
        ExactFile(ffi::server::FORCE_TYPE_force_exactfile),
        ModelSameBounds(ffi::server::FORCE_TYPE_force_model_samebounds),
        ModelSpecifyBounds(ffi::server::FORCE_TYPE_force_model_specifybounds),
        ModelSpecifyBoundsIfAvailable(ffi::server::FORCE_TYPE_force_model_specifybounds_if_avail),
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerStats {
    pub ping: i32,
    pub packet_loss: i32,
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

    // TODO: create newtype wrapper for model index
    pub fn precache_model(&self, name: impl ToEngineStr) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnPrecacheModel)(name.as_ptr()) }
    }

    pub fn precache_sound(&self, name: impl ToEngineStr) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnPrecacheSound)(name.as_ptr()) }
    }

    pub fn set_model(&self, ent: &impl AsEntityHandle, model: impl ToEngineStr) {
        let model = model.to_engine_str();
        unsafe { unwrap!(self, pfnSetModel)(ent.as_entity_handle(), model.as_ptr()) }
    }

    pub fn set_model_with_precache(&self, model: impl ToEngineStr, ent: &impl AsEntityHandle) {
        let model = model.to_engine_str();
        self.precache_model(model.as_ref());
        self.set_model(ent, model.as_ref());
    }

    #[deprecated(note = "use vars().reload_model() instead")]
    pub fn reload_model<T>(&self, model: Option<T>, ent: &impl AsEntityHandle)
    where
        T: ToEngineStr,
    {
        if let Some(model) = model {
            self.set_model(ent, model);
        }
    }

    #[deprecated(note = "use vars().reload_model_with_precache() instead")]
    pub fn reload_model_with_precache<T>(&self, model: Option<T>, ent: &impl AsEntityHandle)
    where
        T: ToEngineStr,
    {
        if let Some(model) = model {
            self.set_model_with_precache(model, ent);
        }
    }

    pub fn model_index(&self, m: impl ToEngineStr) -> c_int {
        let m = m.to_engine_str();
        unsafe { unwrap!(self, pfnModelIndex)(m.as_ptr()) }
    }

    pub fn model_frames(&self, model_index: c_int) -> c_int {
        unsafe { unwrap!(self, pfnModelFrames)(model_index) }
    }

    pub fn set_size(&self, ent: &impl AsEntityHandle, min: vec3_t, max: vec3_t) {
        unsafe {
            unwrap!(self, pfnSetSize)(
                ent.as_entity_handle(),
                min.as_ref().as_ptr(),
                max.as_ref().as_ptr(),
            )
        }
    }

    pub fn change_level(&self, map: impl ToEngineStr, spot: impl ToEngineStr) {
        let map = map.to_engine_str();
        let spot = spot.to_engine_str();
        unsafe { unwrap!(self, pfnChangeLevel)(map.as_ptr(), spot.as_ptr()) }
    }

    // pfnGetSpawnParmsis is obsolete and not implemented in the engine
    // pfnSaveSpawnParms is obsolete and not implemented in the engine

    pub fn vec_to_yaw(&self, direction: vec3_t) -> f32 {
        unsafe { unwrap!(self, pfnVecToYaw)(direction.as_ref().as_ptr()) }
    }

    pub fn vec_to_angles(&self, forward: vec3_t) -> vec3_t {
        unsafe {
            let mut angles = MaybeUninit::<[f32; 3]>::uninit();
            unwrap!(self, pfnVecToAngles)(forward.as_ref().as_ptr(), angles.as_mut_ptr().cast());
            angles.assume_init().into()
        }
    }

    pub fn move_to_origin_with_type(
        &self,
        ent: &impl AsEntityHandle,
        goal: vec3_t,
        dist: f32,
        move_type: MoveToOriginType,
    ) {
        unsafe {
            unwrap!(self, pfnMoveToOrigin)(
                ent.as_entity_handle(),
                goal.as_ref().as_ptr(),
                dist,
                move_type as i32,
            );
        }
    }

    pub fn move_to_origin(&self, ent: &impl AsEntityHandle, goal: vec3_t, dist: f32) {
        self.move_to_origin_with_type(ent, goal, dist, MoveToOriginType::Normal);
    }

    pub fn change_yaw(&self, ent: &impl AsEntityHandle) {
        unsafe { unwrap!(self, pfnChangeYaw)(ent.as_entity_handle()) }
    }

    pub fn change_pitch(&self, ent: &impl AsEntityHandle) {
        unsafe { unwrap!(self, pfnChangePitch)(ent.as_entity_handle()) }
    }

    fn find_entity_by_string_impl<'a>(
        &'a self,
        start: *mut edict_s,
        field: &CStrThin,
        value: &CStrThin,
    ) -> Option<EntityHandleRef<'a>> {
        let ret =
            unsafe { unwrap!(self, pfnFindEntityByString)(start, field.as_ptr(), value.as_ptr()) };
        unsafe { EntityHandleRef::new_not_world_spawn(self.engine_ref(), ret) }
    }

    pub fn entities(&self) -> Entities<'_> {
        Entities::new(self)
    }

    #[deprecated(note = "use entities instead")]
    #[allow(deprecated)]
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

    #[deprecated(note = "use entities instead")]
    #[allow(deprecated)]
    pub fn find_ent_by_classname(
        &self,
        start_search_after: *mut edict_s,
        name: impl ToEngineStr,
    ) -> *mut edict_s {
        self.find_ent_by_string(start_search_after, c"classname", name)
    }

    #[deprecated(note = "use entities instead")]
    #[allow(deprecated)]
    pub fn find_ent_by_target_name(
        &self,
        start_search_after: *mut edict_s,
        name: impl ToEngineStr,
    ) -> *mut edict_s {
        self.find_ent_by_string(start_search_after, c"targetname", name)
    }

    pub fn is_null_ent(&self, ent: *const edict_s) -> bool {
        ent.is_null() || self.get_entity_offset(unsafe { &*ent }).is_world_spawn()
    }

    #[deprecated(note = "use entities instead")]
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

    #[deprecated(note = "use entities instead")]
    #[allow(deprecated)]
    pub fn find_ent_by_classname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = NonNull<edict_s>> {
        self.find_ent_by_string_iter(c"classname", value)
    }

    #[deprecated(note = "use entities instead")]
    #[allow(deprecated)]
    pub fn find_ent_by_globalname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = NonNull<edict_s>> {
        self.find_ent_by_string_iter(c"globalname", value)
    }

    #[deprecated(note = "use entities instead")]
    #[allow(deprecated)]
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
    ) -> Option<EntityHandle> {
        self.entities()
            .by_global_name(&global_name)
            .find(|&i| {
                if let Some(entity) = i.get_entity() {
                    if entity.is_classname(&class_name) {
                        return true;
                    } else {
                        debug!(
                            "Global entity found \"{global_name}\", wrong class \"{class_name}\""
                        );
                    }
                }
                false
            })
            .map(|i| i.into())
    }

    /// Returns an iterator over spawned and connected players.
    pub fn players(&self) -> PlayerIter<'_> {
        PlayerIter::new(self)
    }

    pub fn get_entity_illum(&self, ent: &impl AsEntityHandle) -> c_int {
        unsafe { unwrap!(self, pfnGetEntityIllum)(ent.as_entity_handle()) }
    }

    fn find_entity_in_sphere_impl<'a>(
        &'a self,
        start_search_after: *mut edict_s,
        origin: vec3_t,
        radius: f32,
    ) -> Option<EntityHandleRef<'a>> {
        let ret = unsafe {
            unwrap!(self, pfnFindEntityInSphere)(
                start_search_after,
                origin.as_ref().as_ptr(),
                radius,
            )
        };
        unsafe { EntityHandleRef::new_not_world_spawn(self.engine_ref(), ret) }
    }

    #[deprecated(note = "use entites().in_sphere instead")]
    pub fn find_entity_in_sphere<'a>(
        &'a self,
        start_search_after: Option<&impl AsEntityHandle>,
        origin: vec3_t,
        radius: f32,
    ) -> Option<EntityHandleRef<'a>> {
        let start = start_search_after.map_or(ptr::null_mut(), |e| e.as_entity_handle());
        self.find_entity_in_sphere_impl(start, origin, radius)
    }

    #[deprecated(note = "use entites().in_sphere instead")]
    pub fn find_entity_in_sphere_iter<'a>(
        &'a self,
        start_search_after: Option<&impl AsEntityHandle>,
        origin: vec3_t,
        radius: f32,
    ) -> impl 'a + Iterator<Item = EntityHandleRef<'a>> {
        let mut start = start_search_after.map_or(ptr::null_mut(), |i| i.as_entity_handle());
        iter::from_fn(move || {
            let ret = self.find_entity_in_sphere_impl(start, origin, radius);
            start = ret.map_or(ptr::null_mut(), |i| i.as_ptr());
            ret
        })
    }

    pub fn find_client_in_pvs<'a>(
        &'a self,
        ent: &impl AsEntityHandle,
    ) -> Option<EntityHandleRef<'a>> {
        let ret = unsafe { unwrap!(self, pfnFindClientInPVS)(ent.as_entity_handle()) };
        unsafe { EntityHandleRef::new_not_world_spawn(self.engine_ref(), ret) }
    }

    fn entities_in_pvs_impl<'a>(
        &'a self,
        player: &impl AsEntityHandle,
    ) -> Option<EntityHandleRef<'a>> {
        let raw = unsafe { unwrap!(self, pfnEntitiesInPVS)(player.as_entity_handle()) };
        unsafe { EntityHandleRef::new_not_world_spawn(self.engine_ref(), raw) }
    }

    /// Write results to globals().{v_forward, v_right, v_up}
    pub fn make_vectors(&self, angles: vec3_t) {
        unsafe { unwrap!(self, pfnMakeVectors)(angles.as_ref().as_ptr()) }
    }

    // pub pfnAngleVectors: Option<
    //     unsafe extern "C" fn(
    //         rgflVector: *const f32,
    //         forward: *mut f32,
    //         right: *mut f32,
    //         up: *mut f32,
    //     ),
    // >,

    pub fn create_entity(&self) -> Option<EntityHandle> {
        let ret = unsafe { unwrap!(self, pfnCreateEntity)() };
        unsafe { EntityHandle::new(self.engine_ref(), ret) }
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
    /// Any access to [edict_s], [entvars_s] or the associated private data after this function
    /// will result in an undefined behaviour.
    ///
    /// </div>
    pub unsafe fn remove_entity_now(&self, ent: &impl AsEntityHandle) {
        unsafe { unwrap!(self, pfnRemoveEntity)(ent.as_entity_handle()) }
    }

    pub fn create_named_entity(&self, class_name: impl ToEngineStr) -> Option<EntityHandle> {
        let class_name = self.new_map_string(class_name);
        let ent = unsafe { unwrap!(self, pfnCreateNamedEntity)(class_name.index()) };
        unsafe { EntityHandle::new(self.engine_ref(), ent) }
    }

    pub fn make_static(&self, ent: &impl AsEntityHandle) {
        unsafe { unwrap!(self, pfnMakeStatic)(ent.as_entity_handle()) }
    }

    pub fn is_on_floor(&self, ent: &impl AsEntityHandle) -> bool {
        unsafe { unwrap!(self, pfnEntIsOnFloor)(ent.as_entity_handle()) != 0 }
    }

    pub fn drop_to_floor(&self, ent: &impl AsEntityHandle) -> DropToFloorResult {
        let result = unsafe { unwrap!(self, pfnDropToFloor)(ent.as_entity_handle()) };
        match result {
            -1 => DropToFloorResult::AllSolid,
            0 => DropToFloorResult::False,
            1 => DropToFloorResult::True,
            _ => {
                error!("drop_to_floor: unexpected result={result} from the engine");
                DropToFloorResult::False
            }
        }
    }

    pub fn walk_move(
        &self,
        ent: &impl AsEntityHandle,
        yaw: f32,
        dist: f32,
        mode: WalkMove,
    ) -> bool {
        let mode = mode as i32;
        unsafe { unwrap!(self, pfnWalkMove)(ent.as_entity_handle(), yaw, dist, mode) != 0 }
    }

    /// Links the entity to the world at specified position.
    pub fn set_origin_and_link(&self, origin: vec3_t, ent: &impl AsEntityHandle) {
        unsafe { unwrap!(self, pfnSetOrigin)(ent.as_entity_handle(), origin.as_ref().as_ptr()) }
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
        entity: &impl AsEntityHandle,
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
                entity.as_entity_handle(),
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
        entity: &impl AsEntityHandle,
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
                entity.as_entity_handle(),
                pos.as_mut().as_mut_ptr(),
                sample.as_ptr(),
                volume,
                attenuation.into(),
                flags.bits(),
                pitch.into(),
            )
        }
    }

    pub fn trace_line<'a>(
        &'a self,
        start: vec3_t,
        end: vec3_t,
        ignore: TraceIgnore,
        ignore_ent: Option<&impl AsEntityHandle>,
    ) -> TraceResult<'a> {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceLine)(
                start.as_ref().as_ptr(),
                end.as_ref().as_ptr(),
                ignore.bits() as c_int,
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_entity_handle()),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(self, unsafe { trace.assume_init() })
    }

    pub fn trace_toss<'a>(
        &'a self,
        ent: &impl AsEntityHandle,
        ignore_ent: Option<&impl AsEntityHandle>,
    ) -> TraceResult<'a> {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceToss)(
                ent.as_entity_handle(),
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_entity_handle()),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(self, unsafe { trace.assume_init() })
    }

    pub fn trace_monster_hull<'a>(
        &'a self,
        start: vec3_t,
        end: vec3_t,
        ent: &impl AsEntityHandle,
        ignore: TraceIgnore,
        ignore_ent: Option<&impl AsEntityHandle>,
    ) -> Option<TraceResult<'a>> {
        let mut trace = MaybeUninit::uninit();
        let result = unsafe {
            unwrap!(self, pfnTraceMonsterHull)(
                ent.as_entity_handle(),
                start.as_ref().as_ptr(),
                end.as_ref().as_ptr(),
                ignore.bits() as c_int,
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_entity_handle()),
                trace.as_mut_ptr(),
            )
        };
        if result != 0 {
            Some(TraceResult::new(self, unsafe { trace.assume_init() }))
        } else {
            None
        }
    }

    pub fn trace_hull<'a>(
        &'a self,
        start: vec3_t,
        end: vec3_t,
        hull_number: i32,
        ignore: TraceIgnore,
        ignore_ent: Option<&impl AsEntityHandle>,
    ) -> TraceResult<'a> {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceHull)(
                start.as_ref().as_ptr(),
                end.as_ref().as_ptr(),
                ignore.bits() as c_int,
                hull_number,
                ignore_ent.map_or(ptr::null_mut(), |e| e.as_entity_handle()),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(self, unsafe { trace.assume_init() })
    }

    pub fn trace_model<'a>(
        &'a self,
        start: vec3_t,
        end: vec3_t,
        hull_number: i32,
        ent: &impl AsEntityHandle,
    ) -> TraceResult<'a> {
        let mut trace = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnTraceModel)(
                start.as_ref().as_ptr(),
                end.as_ref().as_ptr(),
                hull_number,
                ent.as_entity_handle(),
                trace.as_mut_ptr(),
            );
        }
        TraceResult::new(self, unsafe { trace.assume_init() })
    }

    pub fn trace_texture(
        &self,
        start: vec3_t,
        end: vec3_t,
        ent: &impl AsEntityHandle,
    ) -> Option<&CStrThin> {
        let result = unsafe {
            unwrap!(self, pfnTraceTexture)(
                ent.as_entity_handle(),
                start.as_ref().as_ptr(),
                end.as_ref().as_ptr(),
            )
        };
        if !result.is_null() {
            Some(unsafe { CStrThin::from_ptr(result) })
        } else {
            None
        }
    }

    // pfnTraceSphere is obsolete and not implemented in the engine

    pub fn get_aim_vector(&self, ent: &impl AsEntityHandle, speed: f32) -> vec3_t {
        let ent = ent.as_entity_handle();
        let mut ret = vec3_t::ZERO;
        unsafe {
            unwrap!(self, pfnGetAimVector)(ent, speed, ret.as_mut().as_mut_ptr());
        }
        ret
    }

    pub fn server_command(&self, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnServerCommand)(cmd.as_ptr()) }
    }

    pub fn server_execute(&self) {
        unsafe { unwrap!(self, pfnServerExecute)() }
    }

    pub fn client_command(&self, ent: &impl AsEntityHandle, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        // FIXME: ffi: why szFmt is mutable?
        unsafe { unwrap!(self, pfnClientCommand)(ent.as_entity_handle(), cmd.as_ptr().cast_mut()) }
    }

    pub fn particle_effect(&self, origin: vec3_t, direction: vec3_t, color: f32, count: f32) {
        unsafe {
            unwrap!(self, pfnParticleEffect)(
                origin.as_ref().as_ptr(),
                direction.as_ref().as_ptr(),
                color,
                count,
            )
        }
    }

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

    pub fn point_contents(&self, origin: vec3_t) -> Contents {
        let raw = unsafe { unwrap!(self, pfnPointContents)(origin.as_ref().as_ptr()) };
        match Contents::from_raw(raw) {
            Some(i) => i,
            None => panic!("point_contents: unexpected contents({raw}) from the engine"),
        }
    }

    fn msg_send<T: ServerMessage>(
        &self,
        dest: MessageDest,
        position: Option<vec3_t>,
        ent: Option<*mut edict_s>,
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

    pub fn msg_one<T: ServerMessage>(&self, ent: &impl AsEntityHandle, msg: &T) {
        self.msg_send(MessageDest::One, None, Some(ent.as_entity_handle()), msg);
    }

    pub fn msg_one_reliable<T: ServerMessage>(&self, ent: &impl AsEntityHandle, msg: &T) {
        self.msg_send(
            MessageDest::OneReliable,
            None,
            Some(ent.as_entity_handle()),
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
        ent: Option<*mut edict_s>,
    ) {
        unsafe {
            unwrap!(self, pfnMessageBegin)(
                dest.into(),
                msg_type,
                origin.as_ref().map_or(ptr::null(), |v| v.as_ref().as_ptr()),
                ent.unwrap_or(ptr::null_mut()),
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
        self.msg_write_coord(v.x);
        self.msg_write_coord(v.y);
        self.msg_write_coord(v.z);
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

    pub fn alloc_ent_private_data(&self, ent: &impl AsEntityHandle, cb: usize) -> *mut c_void {
        let edict = ent.as_entity_handle();
        let ptr = unsafe { unwrap!(self, pfnPvAllocEntPrivateData)(edict, cb as c_long) };
        assert!(!ptr.is_null());
        ptr
    }

    // pub fn ent_private_data(&self, edict: &impl AsEntityHandle) -> *mut c_void {
    //     unsafe { unwrap!(self, pfnPvEntPrivateData)(edict.as_entity_handle()) }
    // }

    // pub unsafe fn free_ent_private_data(&self, ent: &impl AsEntityHandle) {
    //     unsafe { unwrap!(self, pfnFreeEntPrivateData)(ent.as_entity_handle()) }
    // }

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

    // pub fn get_entity_vars(&self, ent: &impl AsEntityHandle) -> *mut entvars_s {
    //     unsafe { unwrap!(self, pfnGetVarsOfEnt)(ent.as_entity_handle()) }
    // }

    #[deprecated(note = "use get_entity_by_offset instead")]
    pub fn entity_of_ent_offset(&self, offset: EntityOffset) -> *mut edict_s {
        let offset = offset.to_u32() as c_int;
        unsafe { unwrap!(self, pfnPEntityOfEntOffset)(offset) }
    }

    #[deprecated(note = "use get_entity_offset instead")]
    pub fn ent_offset_of_entity(&self, ent: &impl AsEntityHandle) -> EntityOffset {
        let offset = unsafe { unwrap!(self, pfnEntOffsetOfPEntity)(ent.as_entity_handle()) };
        unsafe { EntityOffset::new_unchecked(offset.try_into().unwrap()) }
    }

    #[deprecated(note = "use get_entity_index instead")]
    pub fn ent_index(&self, edict: &(impl AsEntityHandle + ?Sized)) -> EntityIndex {
        let index = unsafe { unwrap!(self, pfnIndexOfEdict)(edict.as_entity_handle()) };
        unsafe { EntityIndex::new_unchecked(index.try_into().unwrap()) }
    }

    #[deprecated(note = "use get_entity_by_index instead")]
    pub fn entity_of_ent_index(&self, ent: EntityIndex) -> *mut edict_s {
        self.get_entity_by_index(ent)
            .map_or(ptr::null_mut(), |i| i.as_ptr())
    }

    pub fn get_entity_offset(&self, ent: &impl AsEntityHandle) -> EntityOffset {
        let offset = unsafe { unwrap!(self, pfnEntOffsetOfPEntity)(ent.as_entity_handle()) };
        unsafe { EntityOffset::new_unchecked(offset.try_into().unwrap()) }
    }

    pub fn get_entity_by_offset(&self, offset: EntityOffset) -> Option<EntityHandle> {
        let offset = offset.to_u32() as c_int;
        let ret = unsafe { unwrap!(self, pfnPEntityOfEntOffset)(offset) };
        unsafe { EntityHandle::new(self.engine_ref(), ret) }
    }

    pub fn get_entity_index(&self, ent: &impl AsEntityHandle) -> EntityIndex {
        let index = unsafe { unwrap!(self, pfnIndexOfEdict)(ent.as_entity_handle()) };
        unsafe { EntityIndex::new_unchecked(index.try_into().unwrap()) }
    }

    pub fn get_entity_by_index(&self, index: EntityIndex) -> Option<EntityHandle> {
        let ret = unsafe { unwrap!(self, pfnPEntityOfEntIndex)(index.to_i32()) };
        unsafe { EntityHandle::new(self.engine_ref(), ret) }
    }

    pub fn get_world_spawn_entity(&self) -> EntityHandle {
        self.get_entity_by_offset(EntityOffset::WORLD_SPAWN)
            .expect("world spawn entity")
    }

    pub fn get_single_player(&self) -> Option<EntityHandle> {
        self.get_entity_by_index(EntityIndex::SINGLE_PLAYER)
    }

    pub fn find_entity_by_vars(&self, vars: *mut entvars_s) -> *mut edict_s {
        unsafe { unwrap!(self, pfnFindEntityByVars)(vars) }
    }

    pub fn get_model_ptr(&self, ent: &impl AsEntityHandle) -> *mut c_void {
        unsafe { unwrap!(self, pfnGetModelPtr)(ent.as_entity_handle()) }
    }

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

    // pfnAnimationAutomove is obsolete and not implemented in the engine

    fn get_bone_position_impl(
        &self,
        ent: &impl AsEntityHandle,
        bone: i32,
        origin: Option<&mut vec3_t>,
        angles: Option<&mut vec3_t>,
    ) {
        unsafe {
            unwrap!(self, pfnGetBonePosition)(
                ent.as_entity_handle(),
                bone,
                origin.map_or(ptr::null_mut(), |v| v.as_mut().as_mut_ptr()),
                angles.map_or(ptr::null_mut(), |v| v.as_mut().as_mut_ptr()),
            );
        }
    }

    pub fn get_bone_position_and_angles(
        &self,
        ent: &impl AsEntityHandle,
        bone: i32,
    ) -> (vec3_t, vec3_t) {
        let mut origin = MaybeUninit::uninit();
        let mut angles = MaybeUninit::uninit();
        unsafe {
            self.get_bone_position_impl(
                ent,
                bone,
                Some(origin.assume_init_mut()),
                Some(angles.assume_init_mut()),
            );
            (origin.assume_init(), angles.assume_init())
        }
    }

    pub fn get_bone_position(&self, ent: &impl AsEntityHandle, bone: i32) -> vec3_t {
        let mut origin = MaybeUninit::uninit();
        unsafe {
            self.get_bone_position_impl(ent, bone, Some(origin.assume_init_mut()), None);
            origin.assume_init()
        }
    }

    pub fn get_bone_angles(&self, ent: &impl AsEntityHandle, bone: i32) -> vec3_t {
        let mut angles = MaybeUninit::uninit();
        unsafe {
            self.get_bone_position_impl(ent, bone, None, Some(angles.assume_init_mut()));
            angles.assume_init()
        }
    }

    // pub fn function_from_name(&self, name: impl ToEngineStr) -> c_ulong {
    //     let name = name.to_engine_str();
    //     unsafe { unwrap!(self, pfnFunctionFromName)(name.as_ptr()) }
    // }

    // pub fn name_for_function(&self, func: c_ulong) -> Option<&CStrThin> {
    //     unsafe {
    //         let name = unwrap!(self, pfnNameForFunction)(func);
    //         cstr_or_none(name)
    //     }
    // }

    // pub pfnClientPrintf:
    //     Option<unsafe extern "C" fn(pEdict: *mut edict_t, ptype: PRINT_TYPE, szMsg: *const c_char)>,

    fn get_attachment_impl(
        &self,
        ent: &impl AsEntityHandle,
        attachment: i32,
        origin: Option<&mut vec3_t>,
        angles: Option<&mut vec3_t>,
    ) {
        unsafe {
            unwrap!(self, pfnGetAttachment)(
                ent.as_entity_handle(),
                attachment,
                origin.map_or(ptr::null_mut(), |v| v.as_mut().as_mut_ptr()),
                angles.map_or(ptr::null_mut(), |v| v.as_mut().as_mut_ptr()),
            );
        }
    }

    pub fn get_attachment_position_and_angles(
        &self,
        ent: &impl AsEntityHandle,
        attachment: i32,
    ) -> (vec3_t, vec3_t) {
        let mut origin = MaybeUninit::uninit();
        let mut angles = MaybeUninit::uninit();
        unsafe {
            self.get_attachment_impl(
                ent,
                attachment,
                Some(origin.assume_init_mut()),
                Some(angles.assume_init_mut()),
            );
            (origin.assume_init(), angles.assume_init())
        }
    }

    pub fn get_attachment_position(&self, ent: &impl AsEntityHandle, attachment: i32) -> vec3_t {
        let mut origin = MaybeUninit::uninit();
        unsafe {
            self.get_attachment_impl(ent, attachment, Some(origin.assume_init_mut()), None);
            origin.assume_init()
        }
    }

    pub fn get_attachment_angles(&self, ent: &impl AsEntityHandle, attachment: i32) -> vec3_t {
        let mut angles = MaybeUninit::uninit();
        unsafe {
            self.get_attachment_impl(ent, attachment, None, Some(angles.assume_init_mut()));
            angles.assume_init()
        }
    }

    pub fn crc32_new(&self) -> CRC32_t {
        unsafe {
            let mut ret = MaybeUninit::uninit();
            unwrap!(self, pfnCRC32_Init)(ret.as_mut_ptr());
            ret.assume_init()
        }
    }

    pub fn crc32_process_bytes(&self, state: &mut CRC32_t, bytes: &[u8]) {
        unsafe {
            unwrap!(self, pfnCRC32_ProcessBuffer)(state, bytes.as_ptr().cast(), bytes.len() as i32)
        }
    }

    pub fn crc32_process_byte(&self, state: &mut CRC32_t, byte: u8) {
        unsafe { unwrap!(self, pfnCRC32_ProcessByte)(state, byte) }
    }

    pub fn crc32_finish(&self, state: CRC32_t) -> CRC32_t {
        unsafe { unwrap!(self, pfnCRC32_Final)(state) }
    }

    pub fn crc32_hasher(&self) -> Crc32Hasher {
        Crc32Hasher {
            engine: self.engine_ref(),
            state: self.crc32_new(),
        }
    }

    pub fn set_view(&self, client: &impl AsEntityHandle, view_entity: &impl AsEntityHandle) {
        unsafe {
            unwrap!(self, pfnSetView)(client.as_entity_handle(), view_entity.as_entity_handle())
        }
    }

    pub fn crosshair_angle(&self, ent: &impl AsEntityHandle, pitch: f32, yaw: f32) {
        unsafe { unwrap!(self, pfnCrosshairAngle)(ent.as_entity_handle(), pitch, yaw) }
    }

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

    pub fn end_section_by_name(&self, name: impl ToEngineStr) {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnEndSection)(name.as_ptr()) }
    }

    pub fn end_section(&self, section: EndSection) {
        self.end_section_by_name(section.as_c_str());
    }

    pub fn compare_file_time(
        &self,
        path1: impl ToEngineStr,
        path2: impl ToEngineStr,
    ) -> Option<cmp::Ordering> {
        let path1 = path1.to_engine_str();
        let path2 = path2.to_engine_str();
        let mut compare = MaybeUninit::uninit();
        let result = unsafe {
            unwrap!(self, pfnCompareFileTime)(path1.as_ptr(), path2.as_ptr(), compare.as_mut_ptr())
        };
        if result == 0 {
            return None;
        }
        match unsafe { compare.assume_init() } {
            -1 => Some(cmp::Ordering::Less),
            0 => Some(cmp::Ordering::Equal),
            1 => Some(cmp::Ordering::Greater),
            compare => unreachable!("compare_file_time: unexpected compare {compare}"),
        }
    }

    pub fn get_game_dir(&self) -> CStrArray<MAX_SYSPATH> {
        // FIXME: limit game dir buffer size to 256 bytes???
        let mut buffer = CStrArray::new();
        unsafe { unwrap!(self, pfnGetGameDir)(buffer.as_mut_ptr()) }
        buffer
    }

    // TODO: add safety docs
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn register_cvar_raw(&self, cvar: *mut cvar_t) {
        unsafe { unwrap!(self, pfnCvar_RegisterVariable)(cvar) }
    }

    pub fn fade_client_volume(
        &self,
        ent: &impl AsEntityHandle,
        fade_precent: u8,
        fade_out_seconds: u8,
        hold_time: u8,
        fade_in_seconds: u8,
    ) {
        unsafe {
            unwrap!(self, pfnFadeClientVolume)(
                ent.as_entity_handle(),
                fade_precent as i32,
                fade_out_seconds as i32,
                hold_time as i32,
                fade_in_seconds as i32,
            );
        }
    }

    pub fn set_client_maxspeed(&self, ent: &impl AsEntityHandle, new_maxspeed: f32) {
        unsafe { unwrap!(self, pfnSetClientMaxspeed)(ent.as_entity_handle(), new_maxspeed) }
    }

    pub fn create_fake_client(&self, net_name: impl ToEngineStr) -> Option<EntityHandle> {
        let net_name = net_name.to_engine_str();
        unsafe {
            let ent = unwrap!(self, pfnCreateFakeClient)(net_name.as_ptr());
            EntityHandle::new(self.engine_ref(), ent)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_player_move(
        &self,
        fake_client: &impl AsEntityHandle,
        view_angles: vec3_t,
        forward_move: f32,
        side_move: f32,
        up_move: f32,
        buttons: Buttons,
        impulse: u8,
        msec: u8,
    ) {
        unsafe {
            unwrap!(self, pfnRunPlayerMove)(
                fake_client.as_entity_handle(),
                view_angles.as_ref().as_ptr(),
                forward_move,
                side_move,
                up_move,
                buttons.bits() as u16,
                impulse,
                msec,
            )
        }
    }

    fn number_of_entities(&self) -> usize {
        unsafe { unwrap!(self, pfnNumberOfEntities)() as usize }
    }

    pub fn get_info_buffer_raw(&self, ent: &impl AsEntityHandle) -> *mut c_char {
        let ent = ent.as_entity_handle();
        unsafe { unwrap!(self, pfnGetInfoKeyBuffer)(ent) }
    }

    pub fn get_info_buffer<'a>(&'a self, ent: &impl AsEntityHandle) -> ClientInfoBuffer<'a> {
        ClientInfoBuffer {
            engine: self.engine_ref(),
            info_buffer: self.get_info_buffer_raw(ent),
            entity: unsafe {
                EntityHandle::new_unchecked(self.engine_ref(), ent.as_entity_handle())
            },
            phantom: PhantomData,
        }
    }

    pub fn info_buffer_get(&self, info_buffer: *const c_char, key: impl ToEngineStr) -> &CStrThin {
        let key = key.to_engine_str();
        let value = unsafe { unwrap!(self, pfnInfoKeyValue)(info_buffer, key.as_ptr()) };
        // SAFETY: the engine never return a null pointer
        unsafe { CStrThin::from_ptr(value) }
    }

    pub fn info_buffer_set(
        &self,
        info_buffer: *mut c_char,
        key: impl ToEngineStr,
        value: impl ToEngineStr,
    ) {
        let key = key.to_engine_str();
        let value = value.to_engine_str();
        unsafe {
            // FIXME: ffi: why key and value are mutable?
            unwrap!(self, pfnSetKeyValue)(
                info_buffer,
                key.as_ptr().cast_mut(),
                value.as_ptr().cast_mut(),
            )
        }
    }

    pub fn info_buffer_client_set(
        &self,
        client: EntityIndex,
        info_buffer: *mut c_char,
        key: impl ToEngineStr,
        value: impl ToEngineStr,
    ) {
        let key = key.to_engine_str();
        let value = value.to_engine_str();
        unsafe {
            // FIXME: ffi: why key and value are mutable?
            unwrap!(self, pfnSetClientKeyValue)(
                client.to_i32(),
                info_buffer,
                key.as_ptr().cast_mut(),
                value.as_ptr().cast_mut(),
            )
        }
    }

    pub fn info_buffer_remove(&self, info_buffer: *mut c_char, key: impl ToEngineStr) {
        let key = key.to_engine_str();
        unsafe { unwrap!(self, pfnInfo_RemoveKey)(info_buffer, key.as_ptr()) }
    }

    pub fn is_map_valid(&self, filename: impl ToEngineStr) -> bool {
        let filename = filename.to_engine_str();
        // FIXME: ffi: why filename is mutable?
        unsafe { unwrap!(self, pfnIsMapValid)(filename.as_ptr().cast_mut()) != 0 }
    }

    pub fn static_decal(
        &self,
        origin: vec3_t,
        decal_index: u16,
        entity: EntityIndex,
        model_index: u16,
    ) {
        unsafe {
            unwrap!(self, pfnStaticDecal)(
                origin.as_ref().as_ptr(),
                decal_index as c_int,
                entity.to_i32(),
                model_index as c_int,
            )
        }
    }

    pub fn precache_generic(&self, filename: impl ToEngineStr) -> i32 {
        let filename = filename.to_engine_str();
        unsafe { unwrap!(self, pfnPrecacheGeneric)(filename.as_ptr()) }
    }

    pub fn get_player_user_id(&self, ent: &impl AsEntityHandle) -> Option<i32> {
        let ent = ent.as_entity_handle();
        let id = unsafe { unwrap!(self, pfnGetPlayerUserId)(ent) };
        if id != -1 {
            Some(id)
        } else {
            None
        }
    }

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

    // pfnGetPlayerWONId is obsolete and not implemented in the engine

    pub fn get_physics_key_value(
        &self,
        client: &impl AsEntityHandle,
        key: impl ToEngineStr,
    ) -> &CStr {
        let ent = client.as_entity_handle();
        let key = key.to_engine_str();
        let ptr = unsafe { unwrap!(self, pfnGetPhysicsKeyValue)(ent, key.as_ptr()) };
        assert!(!ptr.is_null());
        unsafe { CStr::from_ptr(ptr) }
    }

    pub fn set_physics_key_value(
        &self,
        client: &impl AsEntityHandle,
        key: impl ToEngineStr,
        value: impl ToEngineStr,
    ) {
        let ent = client.as_entity_handle();
        let key = key.to_engine_str();
        let value = value.to_engine_str();
        unsafe { unwrap!(self, pfnSetPhysicsKeyValue)(ent, key.as_ptr(), value.as_ptr()) }
    }

    pub fn get_physics_info_string(&self, client: &impl AsEntityHandle) -> &CStr {
        let ent = client.as_entity_handle();
        let info = unsafe { unwrap!(self, pfnGetPhysicsInfoString)(ent) };
        assert!(!info.is_null());
        unsafe { CStr::from_ptr(info) }
    }

    pub fn precache_event(&self, filename: impl ToEngineStr) -> EventIndex {
        let filename = filename.to_engine_str();
        let index = unsafe { unwrap!(self, pfnPrecacheEvent)(1, filename.as_ptr()) };
        EventIndex(index)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn playback_event(
        &self,
        flags: c_int,
        invoker: &impl AsEntityHandle,
        event_index: EventIndex,
        delay: f32,
        origin: vec3_t,
        angles: vec3_t,
        fparam1: f32,
        fparam2: f32,
        iparam1: c_int,
        iparam2: c_int,
        bparam1: c_int,
        bparam2: c_int,
    ) {
        let mut origin = origin;
        let mut angles = angles;
        unsafe {
            // FIXME: ffi: why origin and angles are mutable?
            unwrap!(self, pfnPlaybackEvent)(
                flags,
                invoker.as_entity_handle(),
                event_index.to_u16(),
                delay,
                origin.as_mut().as_mut_ptr(),
                angles.as_mut().as_mut_ptr(),
                fparam1,
                fparam2,
                iparam1,
                iparam2,
                bparam1,
                bparam2,
            );
        }
    }

    pub fn build_playback_event(&self) -> PlaybackEventBuilder {
        PlaybackEventBuilder {
            engine: self.engine_ref(),
            flags: PlaybackEventFlags::NONE,
            delay: 0.0,
            origin: vec3_t::ZERO,
            angles: vec3_t::ZERO,
            fparam1: 0.0,
            fparam2: 0.0,
            iparam1: 0,
            iparam2: 0,
            bparam1: 0,
            bparam2: 0,
        }
    }

    pub fn set_pvs(&self, org: vec3_t) -> *mut c_uchar {
        unsafe { unwrap!(self, pfnSetFatPVS)(org.as_ref().as_ptr()) }
    }

    pub fn set_pas(&self, org: vec3_t) -> *mut c_uchar {
        unsafe { unwrap!(self, pfnSetFatPAS)(org.as_ref().as_ptr()) }
    }

    pub fn check_visibility(&self, ent: &impl AsEntityHandle, set: *mut c_uchar) -> bool {
        let ent = ent.as_entity_handle();
        unsafe { unwrap!(self, pfnCheckVisibility)(ent, set) != 0 }
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
    // pub pfnDeltaFindField:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldname: *const c_char) -> c_int>,
    // pub pfnDeltaSetFieldByIndex:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldNumber: c_int)>,
    // pub pfnDeltaUnsetFieldByIndex:
    //     Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldNumber: c_int)>,

    pub fn get_current_player(&self) -> Option<i32> {
        let index = unsafe { unwrap!(self, pfnGetCurrentPlayer)() };
        if index != -1 {
            Some(index)
        } else {
            None
        }
    }

    pub fn can_skip_player(&self, ent: &impl AsEntityHandle) -> bool {
        unsafe { unwrap!(self, pfnCanSkipPlayer)(ent.as_entity_handle()) != 0 }
    }

    pub fn set_group_mask(&self, mask: i32, op: GroupOp) {
        unsafe { unwrap!(self, pfnSetGroupMask)(mask, op as i32) }
    }

    pub fn set_group_mask_and(&self, mask: i32) {
        self.set_group_mask(mask, GroupOp::And)
    }

    pub fn set_group_mask_nand(&self, mask: i32) {
        self.set_group_mask(mask, GroupOp::Nand)
    }

    pub fn create_instanced_baseline(
        &self,
        classname: MapString,
        baseline: &entity_state_s,
    ) -> Option<i32> {
        let index = unsafe {
            // FIXME: ffi: why baseline is mutable?
            unwrap!(self, pfnCreateInstancedBaseline)(
                classname.index(),
                (baseline as *const entity_state_s).cast_mut(),
            )
        };
        if index != 0 {
            Some(index)
        } else {
            None
        }
    }

    // pub pfnCvar_DirectSet: Option<unsafe extern "C" fn(var: *mut cvar_s, value: *const c_char)>,

    pub fn force_unmodified(
        &self,
        ty: ForceType,
        mins: Option<vec3_t>,
        maxs: Option<vec3_t>,
        filename: impl ToEngineStr,
    ) {
        let mut mins = mins;
        let mut maxs = maxs;
        let filename = filename.to_engine_str();
        unsafe {
            // FIXME: ffi: why mins and maxs are mutable?
            unwrap!(self, pfnForceUnmodified)(
                ty as u32,
                mins.as_mut()
                    .map_or(ptr::null_mut(), |v| v.as_mut().as_mut_ptr()),
                maxs.as_mut()
                    .map_or(ptr::null_mut(), |v| v.as_mut().as_mut_ptr()),
                filename.as_ptr(),
            );
        }
    }

    pub fn get_player_stats(&self, client: &impl AsEntityHandle) -> PlayerStats {
        let client = client.as_entity_handle();
        let mut ping = MaybeUninit::uninit();
        let mut packet_loss = MaybeUninit::uninit();
        unsafe {
            unwrap!(self, pfnGetPlayerStats)(client, ping.as_mut_ptr(), packet_loss.as_mut_ptr());
            PlayerStats {
                ping: ping.assume_init(),
                packet_loss: packet_loss.assume_init(),
            }
        }
    }

    pub fn voice_get_client_listening(&self, receiver: i32, sender: i32) -> bool {
        unsafe { unwrap!(self, pfnVoice_GetClientListening)(receiver, sender) != 0 }
    }

    pub fn voice_set_client_listening(&self, receiver: i32, sender: i32, listen: bool) -> bool {
        unsafe { unwrap!(self, pfnVoice_SetClientListening)(receiver, sender, listen as i32) != 0 }
    }

    pub fn get_player_auth_id(&self, ent: &impl AsEntityHandle) -> &CStrThin {
        let ent = ent.as_entity_handle();
        let id = unsafe { unwrap!(self, pfnGetPlayerAuthId)(ent) };
        // SAFETY: the engine never returns a null pointer
        unsafe { CStrThin::from_ptr(id) }
    }

    pub fn get_file_size(&self, filename: impl ToEngineStr) -> Option<i32> {
        let filename = filename.to_engine_str();
        let size = unsafe { unwrap!(self, pfnGetFileSize)(filename.as_ptr()) };
        if size != -1 {
            Some(size)
        } else {
            None
        }
    }

    pub fn get_approx_wav_duration(&self, filepath: impl ToEngineStr) -> Duration {
        let filepath = filepath.to_engine_str();
        let msec = unsafe { unwrap!(self, pfnGetApproxWavePlayLen)(filepath.as_ptr()) };
        Duration::from_millis(msec as u64)
    }

    // used by CS:CZ (client stub)
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
    // pub pfnIsCareerMatch: Option<unsafe extern "C" fn() -> c_int>,

    // extended iface stubs
    // pub pfnGetLocalizedStringLength: Option<unsafe extern "C" fn(label: *const c_char) -> c_int>,

    // only exists in PlayStation version
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

    pub fn check_parm(&self, parm: impl ToEngineStr) -> bool {
        let parm = parm.to_engine_str();
        // FIXME: ffi: why parm is mutable?
        unsafe { unwrap!(self, pfnCheckParm)(parm.as_ptr().cast_mut(), ptr::null_mut()) != 0 }
    }

    pub fn get_parm(&self, parm: impl ToEngineStr) -> Option<&CStrThin> {
        let parm = parm.to_engine_str();
        let mut next = ptr::null_mut();
        unsafe {
            // FIXME: ffi: why parm is mutable?
            unwrap!(self, pfnCheckParm)(parm.as_ptr().cast_mut(), &mut next);
            cstr_or_none(next)
        }
    }

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

pub struct Entities<'a> {
    engine: &'a ServerEngine,
}

impl<'a> Entities<'a> {
    fn new(engine: &'a ServerEngine) -> Self {
        Self { engine }
    }

    pub fn count(&self) -> usize {
        self.engine.number_of_entities()
    }

    pub fn by_string<F: ToEngineStr, V: ToEngineStr>(
        &self,
        field: F,
        value: V,
    ) -> EntitiesByString<'a, F, V> {
        EntitiesByString {
            engine: self.engine,
            last: Some(ptr::null_mut()),
            field: field.to_engine_str(),
            value: value.to_engine_str(),
        }
    }

    pub fn by_class_name<V: ToEngineStr>(
        &self,
        value: V,
    ) -> EntitiesByString<'a, &'static CStr, V> {
        self.by_string(c"classname", value)
    }

    pub fn by_global_name<V: ToEngineStr>(
        &self,
        value: V,
    ) -> EntitiesByString<'a, &'static CStr, V> {
        self.by_string(c"globalname", value)
    }

    pub fn by_target_name<V: ToEngineStr>(
        &self,
        value: V,
    ) -> EntitiesByString<'a, &'static CStr, V> {
        self.by_string(c"targetname", value)
    }

    pub fn by_target<V: ToEngineStr>(&self, value: V) -> EntitiesByString<'a, &'static CStr, V> {
        self.by_string(c"target", value)
    }

    pub fn in_pvs(&self, player: &impl AsEntityHandle) -> EntitiesInPvs<'a> {
        EntitiesInPvs {
            last: self.engine.entities_in_pvs_impl(player),
        }
    }

    pub fn in_sphere(&self, origin: vec3_t, radius: f32) -> EntitiesInSphere<'a> {
        EntitiesInSphere {
            engine: self.engine,
            last: Some(ptr::null_mut()),
            origin,
            radius,
        }
    }
}

pub struct EntitiesByString<'a, F: ToEngineStr, V: ToEngineStr> {
    engine: &'a ServerEngine,
    last: Option<*mut edict_s>,
    field: F::Output,
    value: V::Output,
}

impl<'a, F: ToEngineStr, V: ToEngineStr> EntitiesByString<'a, F, V> {
    pub fn first(mut self) -> Option<EntityHandle> {
        self.next().map(|i| i.into())
    }
}

impl<'a, F: ToEngineStr, V: ToEngineStr> Iterator for EntitiesByString<'a, F, V> {
    type Item = EntityHandleRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.last?;
        let field = self.field.as_ref();
        let value = self.value.as_ref();
        let result = self.engine.find_entity_by_string_impl(start, field, value);
        self.last = result.map(|i| i.as_ptr());
        result
    }
}

pub struct EntitiesInPvs<'a> {
    last: Option<EntityHandleRef<'a>>,
}

impl<'a> Iterator for EntitiesInPvs<'a> {
    type Item = EntityHandleRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.last.inspect(|i| self.last = i.next())
    }
}

pub struct EntitiesInSphere<'a> {
    engine: &'a ServerEngine,
    origin: vec3_t,
    radius: f32,
    last: Option<*mut edict_s>,
}

impl<'a> Iterator for EntitiesInSphere<'a> {
    type Item = EntityHandleRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.last?;
        let result = self
            .engine
            .find_entity_in_sphere_impl(start, self.origin, self.radius);
        self.last = result.map(|i| i.as_ptr());
        result
    }
}

pub struct PlayerIter<'a> {
    engine: &'a ServerEngine,
    index: u16,
}

impl<'a> PlayerIter<'a> {
    fn new(engine: &'a ServerEngine) -> Self {
        Self { engine, index: 0 }
    }
}

impl<'a> Iterator for PlayerIter<'a> {
    type Item = &'a dyn Entity;

    fn next(&mut self) -> Option<Self::Item> {
        while (self.index as i32) < self.engine.globals.max_clients() {
            self.index += 1;
            let index = unsafe { EntityIndex::new_unchecked(self.index) };
            if let Some(entity) = self.engine.get_entity_by_index(index) {
                if entity.is_free() {
                    continue;
                }
                if let Some(player) = entity.get_entity() {
                    return Some(player);
                }
            }
        }
        None
    }
}

/// Add server command.
///
/// # Examples
///
/// ```
/// use xash3d_server::{engine::add_command, prelude::*};
///
/// fn add_my_command(engine: &ServerEngine) {
///     add_command!(engine, c"my_command", |engine| {
///         // first argument is the command name
///         let comand_name = engine.cmd_argv(0);
///         log::trace!("execute server command \"{comand_name}\"",);
///
///         // print command arguments
///         log::trace!("command arguments:");
///         for (i, arg) in engine.cmd_args().skip(1).enumerate() {
///             log::trace!("  {i}: {arg}");
///         }
///     });
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! add_command {
    ($engine:expr, $name:expr, $expr:expr) => {{
        unsafe extern "C" fn __command_entry() {
            let engine = unsafe { $crate::engine::ServerEngineRef::new() };
            let handler: fn($crate::engine::ServerEngineRef) = $expr;
            handler(engine);
        }

        use $crate::engine::EngineCmd;
        if $engine.add_command($name, __command_entry).is_err() {
            log::error!("failed to add server command {:?}", $name);
        }
    }};
}
#[doc(inline)]
pub use add_command;
