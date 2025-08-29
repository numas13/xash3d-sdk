use core::{
    cell::{Ref, RefCell, RefMut},
    ffi::{c_int, c_long, c_uchar, c_void, CStr},
    fmt, iter,
    ops::{Deref, DerefMut},
    ptr,
};

use shared::engine::AsCStrPtr;

use crate::{
    cell::SyncOnceCell,
    cvar::{cvar_s, CVarPtr},
    raw::{self, edict_s, string_t, vec3_t},
};

pub use shared::engine::ToEngineStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EntOffset(pub c_int);

pub struct Engine {
    raw: raw::enginefuncs_s,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("enginefuncs_s.{} is null", stringify!($name)),
        }
    };
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[allow(dead_code)]
impl Engine {
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
        start_search_after: *const edict_s,
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
        start_search_after: *const edict_s,
        name: impl ToEngineStr,
    ) -> *mut edict_s {
        self.find_ent_by_string(start_search_after, c"classname", name)
    }

    pub fn find_ent_by_target_name(
        &self,
        start_search_after: *const edict_s,
        name: impl ToEngineStr,
    ) -> *mut edict_s {
        self.find_ent_by_string(start_search_after, c"targetname", name)
    }

    pub fn is_null_ent(&self, ent: *const edict_s) -> bool {
        ent.is_null() || self.ent_offset_of_entity(ent) == 0
    }

    pub fn find_ent_by_string_iter<'a>(
        &'a self,
        field: impl ToEngineStr + 'a,
        value: impl ToEngineStr + 'a,
    ) -> impl 'a + Iterator<Item = *mut edict_s> {
        let field = field.to_engine_str();
        let value = value.to_engine_str();
        let func = unwrap!(self, pfnFindEntityByString);
        let mut ent = unsafe { func(ptr::null(), field.as_ptr(), value.as_ptr()) };
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

    // pub pfnRemoveEntity: Option<unsafe extern "C" fn(e: *mut edict_t)>,
    // pub pfnCreateNamedEntity: Option<unsafe extern "C" fn(className: c_int) -> *mut edict_t>,
    // pub pfnMakeStatic: Option<unsafe extern "C" fn(ent: *mut edict_t)>,
    // pub pfnEntIsOnFloor: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_int>,
    // pub pfnDropToFloor: Option<unsafe extern "C" fn(e: *mut edict_t) -> c_int>,
    // pub pfnWalkMove:
    //     Option<unsafe extern "C" fn(ent: *mut edict_t, yaw: f32, dist: f32, iMode: c_int) -> c_int>,

    pub fn set_origin(&self, ent: &mut edict_s, origin: vec3_t) {
        unsafe { unwrap!(self, pfnSetOrigin)(ent, origin.as_ptr()) }
    }

    // pub pfnEmitSound: Option<
    //     unsafe extern "C" fn(
    //         entity: *mut edict_t,
    //         channel: c_int,
    //         sample: *const c_char,
    //         volume: f32,
    //         attenuation: f32,
    //         fFlags: c_int,
    //         pitch: c_int,
    //     ),
    // >,
    // pub pfnEmitAmbientSound: Option<
    //     unsafe extern "C" fn(
    //         entity: *mut edict_t,
    //         pos: *mut f32,
    //         samp: *const c_char,
    //         vol: f32,
    //         attenuation: f32,
    //         fFlags: c_int,
    //         pitch: c_int,
    //     ),
    // >,
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
    // pub pfnLightStyle: Option<unsafe extern "C" fn(style: c_int, val: *const c_char)>,
    // pub pfnDecalIndex: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    // pub pfnPointContents: Option<unsafe extern "C" fn(rgflVector: *const f32) -> c_int>,
    // pub pfnMessageBegin: Option<
    //     unsafe extern "C" fn(
    //         msg_dest: c_int,
    //         msg_type: c_int,
    //         pOrigin: *const f32,
    //         ed: *mut edict_t,
    //     ),
    // >,
    // pub pfnMessageEnd: Option<unsafe extern "C" fn()>,
    // pub pfnWriteByte: Option<unsafe extern "C" fn(iValue: c_int)>,
    // pub pfnWriteChar: Option<unsafe extern "C" fn(iValue: c_int)>,
    // pub pfnWriteShort: Option<unsafe extern "C" fn(iValue: c_int)>,
    // pub pfnWriteLong: Option<unsafe extern "C" fn(iValue: c_int)>,
    // pub pfnWriteAngle: Option<unsafe extern "C" fn(flValue: f32)>,
    // pub pfnWriteCoord: Option<unsafe extern "C" fn(flValue: f32)>,
    // pub pfnWriteString: Option<unsafe extern "C" fn(sz: *const c_char)>,
    // pub pfnWriteEntity: Option<unsafe extern "C" fn(iValue: c_int)>,

    pub fn cvar_register(&self, cvar: &'static mut cvar_s) {
        unsafe { unwrap!(self, pfnCVarRegister)(cvar) }
    }

    pub fn cvar_get_float(&self, name: impl ToEngineStr) -> f32 {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnCVarGetFloat)(name.as_ptr()) }
    }

    // pub pfnCVarGetString: Option<unsafe extern "C" fn(szVarName: *const c_char) -> *const c_char>,
    // pub pfnCVarSetFloat: Option<unsafe extern "C" fn(szVarName: *const c_char, flValue: f32)>,

    pub fn cvar_set_string(&self, name: impl ToEngineStr, value: impl ToEngineStr) {
        let name = name.to_engine_str();
        let value = value.to_engine_str();
        unsafe { unwrap!(self, pfnCVarSetString)(name.as_ptr(), value.as_ptr()) }
    }

    pub fn alert_message(&self, atype: raw::ALERT_TYPE, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, pfnAlertMessage)(atype, c"%s\n".as_ptr(), msg.as_ptr());
        }
    }

    #[deprecated(note = "use alert_message instead")]
    pub fn alert_message_fmt(&self, atype: raw::ALERT_TYPE, args: fmt::Arguments) {
        self.alert_message(atype, args);
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

    // pub pfnSzFromIndex: Option<unsafe extern "C" fn(iString: c_int) -> *const c_char>,

    pub fn alloc_string(&self, value: impl ToEngineStr) -> string_t {
        let value = value.to_engine_str();
        let n = unsafe { unwrap!(self, pfnAllocString)(value.as_ptr()) };
        string_t(n)
    }

    // pub pfnGetVarsOfEnt: Option<unsafe extern "C" fn(pEdict: *mut edict_t) -> *mut entvars_s>,

    pub fn entity_of_ent_offset(&self, ent_offset: EntOffset) -> *mut edict_s {
        unsafe { unwrap!(self, pfnPEntityOfEntOffset)(ent_offset.0) }
    }

    pub fn ent_offset_of_entity(&self, edict: *const edict_s) -> c_int {
        unsafe { unwrap!(self, pfnEntOffsetOfPEntity)(edict) }
    }

    pub fn ent_index(&self, edict: &edict_s) -> c_int {
        unsafe { unwrap!(self, pfnIndexOfEdict)(edict) }
    }

    pub fn entity_of_ent_index(&self, ent_index: c_int) -> *mut edict_s {
        unsafe { unwrap!(self, pfnPEntityOfEntIndex)(ent_index) }
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

    pub fn server_print(&self, s: impl ToEngineStr) {
        let s = s.to_engine_str();
        unsafe { unwrap!(self, pfnServerPrint)(s.as_ptr()) }
    }

    // pub pfnCmd_Args: Option<unsafe extern "C" fn() -> *const c_char>,

    pub fn cmd_argv(&self, argc: c_int) -> &CStr {
        let ptr = unsafe { unwrap!(self, pfnCmd_Argv)(argc) };
        assert!(!ptr.is_null());
        unsafe { CStr::from_ptr(ptr) }
    }

    // pub pfnCmd_Argc: Option<unsafe extern "C" fn() -> c_int>,
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

    pub fn random_int(&self, min: c_int, max: c_int) -> c_int {
        assert!(min >= 0, "min must be greater than or equal to zero");
        assert!(min <= max, "min must be less than or equal to max");
        unsafe { unwrap!(self, pfnRandomLong)(min, max) }
    }

    pub fn random_float(&self, min: f32, max: f32) -> f32 {
        unsafe { unwrap!(self, pfnRandomFloat)(min, max) }
    }

    // pub pfnSetView: Option<unsafe extern "C" fn(pClient: *const edict_t, pViewent: *const edict_t)>,
    // pub pfnTime: Option<unsafe extern "C" fn() -> f32>,
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

    pub fn get_cvar(&self, name: impl ToEngineStr) -> CVarPtr {
        let name = name.to_engine_str();
        let ptr = unsafe { unwrap!(self, pfnCVarGetPointer)(name.as_ptr()) };
        CVarPtr::from_ptr(ptr)
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
    // pub pfnAddServerCommand: Option<
    //     unsafe extern "C" fn(cmd_name: *const c_char, function: Option<unsafe extern "C" fn()>),
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

impl From<raw::enginefuncs_s> for Engine {
    fn from(raw: raw::enginefuncs_s) -> Self {
        Self { raw }
    }
}

pub struct Globals {
    raw: *mut raw::globalvars_t,
}

impl Globals {
    fn new(raw: *mut raw::globalvars_t) -> Self {
        Self { raw }
    }

    pub fn string(&self, string: string_t) -> &'static CStr {
        unsafe { CStr::from_ptr(self.pStringBase.wrapping_byte_add(string.0 as usize)) }
    }

    #[deprecated = "use Engine::alloc_string"]
    pub fn make_string(&self, s: &CStr) -> string_t {
        let base = self.string(string_t(0)).as_ptr() as usize;
        string_t(s.as_ptr().wrapping_byte_sub(base) as c_int)
    }
}

impl Deref for Globals {
    type Target = raw::globalvars_t;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for Globals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}

static ENGINE: SyncOnceCell<Engine> = unsafe { SyncOnceCell::new() };

static GLOBALS: SyncOnceCell<RefCell<Globals>> = unsafe { SyncOnceCell::new() };

pub fn engine<'a>() -> &'a Engine {
    ENGINE.get().unwrap()
}

pub fn globals() -> Ref<'static, Globals> {
    GLOBALS.get().unwrap().borrow()
}

pub fn globals_mut() -> RefMut<'static, Globals> {
    GLOBALS.get().unwrap().borrow_mut()
}

pub fn engine_set(funcs: raw::enginefuncs_s, globals: *mut raw::globalvars_t) {
    if ENGINE.set(funcs.into()).is_err()
        || GLOBALS.set(RefCell::new(Globals::new(globals))).is_err()
    {
        warn!("server engine initialized multiple times");
        return;
    }
    crate::logger::init_console_logger();
}
