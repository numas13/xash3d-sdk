use core::{
    ffi::{c_char, c_int, c_long, c_uchar, c_void, CStr},
    iter, ptr,
};

use csz::{CStrSlice, CStrThin};
use xash3d_shared::{
    entity::EntityIndex,
    export::impl_unsync_global,
    ffi::{
        common::{cvar_s, vec3_t},
        server::{edict_s, enginefuncs_s, globalvars_t, ALERT_TYPE, LEVELLIST},
    },
    sound::{Attenuation, Channel, Pitch, SoundFlags},
    str::{AsCStrPtr, ToEngineStr},
};

use crate::{
    cvar::CVarPtr,
    entity::EntityOffset,
    globals::ServerGlobals,
    str::MapString,
    svc::{Message, MessageDest},
};

pub use xash3d_shared::engine::{AddCmdError, EngineRef};

pub(crate) mod prelude {
    pub use xash3d_shared::engine::{
        EngineCmd, EngineCmdArgsRaw, EngineConsole, EngineCvar, EngineRng, EngineSystemTime,
    };
}

pub use self::prelude::*;

pub type ServerEngineRef = EngineRef<ServerEngine>;

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

    pub fn emit(self, sample: impl ToEngineStr, ent: &mut edict_s) {
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

    pub fn emit_dyn(self, sample: impl ToEngineStr, ent: &mut edict_s) {
        let sample = sample.to_engine_str();
        let sample = sample.as_ref();
        if let Some(b'!') = sample.to_bytes().first() {
            // TODO: find sound sample in sentences.txt
        } else {
            self.emit(sample, ent);
        }
    }

    pub fn ambient_emit(self, sample: impl ToEngineStr, pos: vec3_t, ent: &mut edict_s) {
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

    pub fn ambient_emit_dyn(self, sample: impl ToEngineStr, pos: vec3_t, ent: &mut edict_s) {
        let sample = sample.to_engine_str();
        let sample = sample.as_ref();
        if let Some(b'!') = sample.to_bytes().first() {
            // TODO: find sound sample in sentences.txt
        } else {
            self.ambient_emit(sample, pos, ent);
        }
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

    pub fn precache_model(&self, name: impl ToEngineStr) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnPrecacheModel)(name.as_ptr()) }
    }

    pub fn precache_sound(&self, name: impl ToEngineStr) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnPrecacheSound)(name.as_ptr()) }
    }

    pub fn set_model(&self, ent: &mut edict_s, model: impl ToEngineStr) {
        let model = model.to_engine_str();
        unsafe { unwrap!(self, pfnSetModel)(ent, model.as_ptr()) }
    }

    pub fn model_index(&self, m: impl ToEngineStr) -> c_int {
        let m = m.to_engine_str();
        unsafe { unwrap!(self, pfnModelIndex)(m.as_ptr()) }
    }

    // pub pfnModelFrames: Option<unsafe extern "C" fn(modelIndex: c_int) -> c_int>,

    pub fn set_size(&self, ent: &mut edict_s, min: vec3_t, max: vec3_t) {
        unsafe { unwrap!(self, pfnSetSize)(ent, min.as_ptr(), max.as_ptr()) }
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
    ) -> impl 'a + Iterator<Item = *mut edict_s> {
        let field = field.to_engine_str();
        let value = value.to_engine_str();
        let func = unwrap!(self, pfnFindEntityByString);
        let mut ent = unsafe { func(ptr::null_mut(), field.as_ptr(), value.as_ptr()) };
        iter::from_fn(move || {
            if !self.is_null_ent(ent) {
                let tmp = ent;
                ent = unsafe { func(ent, field.as_ptr(), value.as_ptr()) };
                Some(tmp)
            } else {
                None
            }
        })
    }

    pub fn find_ent_by_classname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = *mut edict_s> {
        self.find_ent_by_string_iter(c"classname", value)
    }

    pub fn find_ent_by_globalname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = *mut edict_s> {
        self.find_ent_by_string_iter(c"globalname", value)
    }

    pub fn find_ent_by_targetname_iter<'a>(
        &'a self,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = *mut edict_s> {
        self.find_ent_by_string_iter(c"targetname", value)
    }

    // pub pfnGetEntityIllum: Option<unsafe extern "C" fn(pEnt: *mut edict_t) -> c_int>,
    // pub pfnFindEntityInSphere: Option<
    //     unsafe extern "C" fn(
    //         pEdictStartSearchAfter: *mut edict_t,
    //         org: *const f32,
    //         rad: f32,
    //     ) -> *mut edict_t,
    // >,
    // pub pfnFindClientInPVS: Option<unsafe extern "C" fn(pEdict: *mut edict_t) -> *mut edict_t>,

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

    pub fn remove_entity(&self, ent: &mut edict_s) {
        unsafe { unwrap!(self, pfnRemoveEntity)(ent) }
    }

    // pub pfnCreateNamedEntity: Option<unsafe extern "C" fn(className: c_int) -> *mut edict_t>,
    // pub pfnMakeStatic: Option<unsafe extern "C" fn(ent: *mut edict_t)>,
    // pub pfnEntIsOnFloor: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_int>,
    // pub pfnDropToFloor: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_int>,
    // pub pfnWalkMove:
    //     Option<unsafe extern "C" fn(ent: *mut edict_t, yaw: f32, dist: f32, iMode: c_int) -> c_int>,

    pub fn set_origin(&self, ent: &mut edict_s, origin: vec3_t) {
        unsafe { unwrap!(self, pfnSetOrigin)(ent, origin.as_ptr()) }
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

    // pub pfnTraceLine: Option<
    //     unsafe extern "C" fn(
    //         v1: *const f32,
    //         v2: *const f32,
    //         fNoMonsters: c_int,
    //         pentToSkip: *mut edict_t,
    //         ptr: *mut TraceResult,
    //     ),
    // >,
    // pub pfnTraceToss: Option<
    //     unsafe extern "C" fn(pent: *mut edict_t, pentToIgnore: *mut edict_t, ptr: *mut TraceResult),
    // >,
    // pub pfnTraceMonsterHull: Option<
    //     unsafe extern "C" fn(
    //         pEdict: *mut edict_t,
    //         v1: *const f32,
    //         v2: *const f32,
    //         fNoMonsters: c_int,
    //         pentToSkip: *mut edict_t,
    //         ptr: *mut TraceResult,
    //     ) -> c_int,
    // >,
    // pub pfnTraceHull: Option<
    //     unsafe extern "C" fn(
    //         v1: *const f32,
    //         v2: *const f32,
    //         fNoMonsters: c_int,
    //         hullNumber: c_int,
    //         pentToSkip: *mut edict_t,
    //         ptr: *mut TraceResult,
    //     ),
    // >,
    // pub pfnTraceModel: Option<
    //     unsafe extern "C" fn(
    //         v1: *const f32,
    //         v2: *const f32,
    //         hullNumber: c_int,
    //         pent: *mut edict_t,
    //         ptr: *mut TraceResult,
    //     ),
    // >,
    // pub pfnTraceTexture: Option<
    //     unsafe extern "C" fn(
    //         pTextureEntity: *mut edict_t,
    //         v1: *const f32,
    //         v2: *const f32,
    //     ) -> *const c_char,
    // >,
    // pub pfnTraceSphere: Option<
    //     unsafe extern "C" fn(
    //         v1: *const f32,
    //         v2: *const f32,
    //         fNoMonsters: c_int,
    //         radius: f32,
    //         pentToSkip: *mut edict_t,
    //         ptr: *mut TraceResult,
    //     ),
    // >,
    // pub pfnGetAimVector:
    //     Option<unsafe extern "C" fn(ent: *mut edict_t, speed: f32, rgflReturn: *mut f32)>,

    pub fn server_command(&self, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnServerCommand)(cmd.as_ptr()) }
    }

    pub fn server_execute(&self) {
        unsafe { unwrap!(self, pfnServerExecute)() }
    }

    // pub pfnClientCommand:
    //     Option<unsafe extern "C" fn(pEdict: *mut edict_t, szFmt: *mut c_char, ...)>,
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

    fn msg_send<T: Message>(
        &self,
        dest: MessageDest,
        position: Option<vec3_t>,
        ent: Option<&mut edict_s>,
        msg: &T,
    ) {
        self.msg_begin(dest, T::MSG_TYPE, position, ent);
        msg.write_body(self);
        self.msg_end();
    }

    pub fn msg_broadcast<T: Message>(&self, msg: &T) {
        self.msg_send(MessageDest::Broadcast, None, None, msg);
    }

    pub fn msg_all<T: Message>(&self, msg: &T) {
        self.msg_send(MessageDest::All, None, None, msg);
    }

    pub fn msg_one<T: Message>(&self, ent: &mut edict_s, msg: &T) {
        self.msg_send(MessageDest::One, None, Some(ent), msg);
    }

    pub fn msg_one_reliable<T: Message>(&self, ent: &mut edict_s, msg: &T) {
        self.msg_send(MessageDest::OneReliable, None, Some(ent), msg);
    }

    pub fn msg_init<T: Message>(&self, msg: &T) {
        self.msg_send(MessageDest::Init, None, None, msg);
    }

    pub fn msg_pvs<T: Message>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::Pvs, Some(position), None, msg);
    }

    pub fn msg_pvs_reliable<T: Message>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::PvsReliable, Some(position), None, msg);
    }

    pub fn msg_pas<T: Message>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::Pas, Some(position), None, msg);
    }

    pub fn msg_reliable<T: Message>(&self, position: vec3_t, msg: &T) {
        self.msg_send(MessageDest::PasReliable, Some(position), None, msg);
    }

    pub fn msg_spec<T: Message>(&self, msg: &T) {
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
    // pub pfnRegUserMsg: Option<unsafe extern "C" fn(pszName: *const c_char, iSize: c_int) -> c_int>,
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
    // pub pfnLoadFileForMe:
    //     Option<unsafe extern "C" fn(filename: *const c_char, pLength: *mut c_int) -> *mut byte>,
    // pub pfnFreeFile: Option<unsafe extern "C" fn(buffer: *mut c_void)>,
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
    // pub pfnStaticDecal: Option<
    //     unsafe extern "C" fn(
    //         origin: *const f32,
    //         decalIndex: c_int,
    //         entityIndex: c_int,
    //         modelIndex: c_int,
    //     ),
    // >,
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
