#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void},
    slice,
};

use bitflags::bitflags;
use csz::{CStrArray, CStrThin};

use crate::{consts::MAX_LEVEL_CONNECTIONS, cvar::cvar_s, str::MapString};

pub use shared::raw::*;

pub type FILE = c_void;
pub type CRC32_t = u32;

pub const INTERFACE_VERSION: c_int = 140;
pub const NEW_DLL_FUNCTIONS_VERSION: c_int = 1;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct link_s {
    pub prev: *mut link_s,
    pub next: *mut link_s,
}

pub const MAX_ENT_LEAFS_32: usize = 24; // originally was 16
pub const MAX_ENT_LEAFS_16: usize = 48;

#[derive(Copy, Clone)]
#[repr(C)]
pub union edits_s_leafnums {
    pub leafnums32: [c_int; MAX_ENT_LEAFS_32],
    pub leafnums16: [c_short; MAX_ENT_LEAFS_16],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct edict_s {
    pub free: qboolean,
    pub serialnumber: c_int,
    pub area: link_s,
    pub headnode: c_int,
    pub num_leafs: c_int,
    pub leafnums: edits_s_leafnums,
    pub freetime: f32,
    pub pvPrivateData: *mut c_void,
    pub v: entvars_s,
}

// pub type trace_t = shared::raw::trace_t<edict_s>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct entvars_s {
    pub classname: Option<MapString>,
    pub globalname: Option<MapString>,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub velocity: vec3_t,
    pub basevelocity: vec3_t,
    pub clbasevelocity: vec3_t,
    pub movedir: vec3_t,
    pub angles: vec3_t,
    pub avelocity: vec3_t,
    pub punchangle: vec3_t,
    pub v_angle: vec3_t,
    pub endpos: vec3_t,
    pub startpos: vec3_t,
    pub impacttime: f32,
    pub starttime: f32,
    pub fixangle: c_int,
    pub idealpitch: f32,
    pub pitch_speed: f32,
    pub ideal_yaw: f32,
    pub yaw_speed: f32,
    pub modelindex: c_int,
    pub model: Option<MapString>,
    pub viewmodel: Option<MapString>,
    pub weaponmodel: Option<MapString>,
    pub absmin: vec3_t,
    pub absmax: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub size: vec3_t,
    pub ltime: f32,
    pub nextthink: f32,
    pub movetype: MoveType,
    pub solid: c_int,
    pub skin: c_int,
    pub body: c_int,
    pub effects: Effects,
    pub gravity: f32,
    pub friction: f32,
    pub light_level: c_int,
    pub sequence: c_int,
    pub gaitsequence: c_int,
    pub frame: f32,
    pub animtime: f32,
    pub framerate: f32,
    pub controller: [byte; 4],
    pub blending: [byte; 2],
    pub scale: f32,
    pub rendermode: RenderMode,
    pub renderamt: f32,
    pub rendercolor: vec3_t,
    pub renderfx: RenderFx,
    pub health: f32,
    pub frags: f32,
    pub weapons: c_int,
    pub takedamage: f32,
    pub deadflag: c_int,
    pub view_ofs: vec3_t,
    pub button: c_int,
    pub impulse: c_int,
    pub chain: *mut edict_s,
    pub dmg_inflictor: *mut edict_s,
    pub enemy: *mut edict_s,
    pub aiment: *mut edict_s,
    pub owner: *mut edict_s,
    pub groundentity: *mut edict_s,
    pub spawnflags: c_int,
    pub flags: EdictFlags,
    pub colormap: c_int,
    pub team: c_int,
    pub max_health: f32,
    pub teleport_time: f32,
    pub armortype: f32,
    pub armorvalue: f32,
    pub waterlevel: c_int,
    pub watertype: c_int,
    pub target: Option<MapString>,
    pub targetname: Option<MapString>,
    pub netname: Option<MapString>,
    pub message: Option<MapString>,
    pub dmg_take: f32,
    pub dmg_save: f32,
    pub dmg: f32,
    pub dmgtime: f32,
    pub noise: Option<MapString>,
    pub noise1: Option<MapString>,
    pub noise2: Option<MapString>,
    pub noise3: Option<MapString>,
    pub speed: f32,
    pub air_finished: f32,
    pub pain_finished: f32,
    pub radsuit_finished: f32,
    pub pContainingEntity: *mut edict_s,
    pub playerclass: c_int,
    pub maxspeed: f32,
    pub fov: f32,
    pub weaponanim: c_int,
    pub pushmsec: c_int,
    pub bInDuck: c_int,
    pub flTimeStepSound: c_int,
    pub flSwimTime: c_int,
    pub flDuckTime: c_int,
    pub iStepLeft: c_int,
    pub flFallVelocity: f32,
    pub gamestate: c_int,
    pub oldbuttons: c_int,
    pub groupinfo: c_int,
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
    pub euser1: *mut edict_s,
    pub euser2: *mut edict_s,
    pub euser3: *mut edict_s,
    pub euser4: *mut edict_s,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct delta_s {
    _unused: [u8; 0],
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum PRINT_TYPE {
    Console = 0,
    Center = 1,
    Chat = 2,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum ALERT_TYPE {
    Notice = 0,
    Console = 1,
    AiConsole = 2,
    Warning = 3,
    Error = 4,
    Logged = 5,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum FORCE_TYPE {
    exactfile = 0,
    model_samebounds = 1,
    model_specifybounds = 2,
}

#[doc(hidden)]
#[deprecated]
pub type string_t = Option<crate::str::MapString>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TraceResult {
    pub fAllSolid: c_int,
    pub fStartSolid: c_int,
    pub fInOpen: c_int,
    pub fInWater: c_int,
    pub flFraction: f32,
    pub vecEndPos: vec3_t,
    pub flPlaneDist: f32,
    pub vecPlaneNormal: vec3_t,
    pub pHit: *mut edict_s,
    pub iHitgroup: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct enginefuncs_s {
    pub pfnPrecacheModel: Option<unsafe extern "C" fn(s: *const c_char) -> c_int>,
    pub pfnPrecacheSound: Option<unsafe extern "C" fn(s: *const c_char) -> c_int>,
    pub pfnSetModel: Option<unsafe extern "C" fn(e: *mut edict_s, m: *const c_char)>,
    pub pfnModelIndex: Option<unsafe extern "C" fn(m: *const c_char) -> c_int>,
    pub pfnModelFrames: Option<unsafe extern "C" fn(modelIndex: c_int) -> c_int>,
    pub pfnSetSize:
        Option<unsafe extern "C" fn(e: *mut edict_s, rgflMin: *const f32, rgflMax: *const f32)>,
    pub pfnChangeLevel: Option<unsafe extern "C" fn(s1: *const c_char, s2: *const c_char)>,
    pub pfnGetSpawnParms: Option<unsafe extern "C" fn(ent: *mut edict_s)>,
    pub pfnSaveSpawnParms: Option<unsafe extern "C" fn(ent: *mut edict_s)>,
    pub pfnVecToYaw: Option<unsafe extern "C" fn(rgflVector: *const f32) -> f32>,
    pub pfnVecToAngles:
        Option<unsafe extern "C" fn(rgflVectorIn: *const f32, rgflVectorOut: *mut f32)>,
    pub pfnMoveToOrigin: Option<
        unsafe extern "C" fn(ent: *mut edict_s, pflGoal: *const f32, dist: f32, iMoveType: c_int),
    >,
    pub pfnChangeYaw: Option<unsafe extern "C" fn(ent: *mut edict_s)>,
    pub pfnChangePitch: Option<unsafe extern "C" fn(ent: *mut edict_s)>,
    pub pfnFindEntityByString: Option<
        unsafe extern "C" fn(
            pEdictStartSearchAfter: *const edict_s,
            pszField: *const c_char,
            pszValue: *const c_char,
        ) -> *mut edict_s,
    >,
    pub pfnGetEntityIllum: Option<unsafe extern "C" fn(pEnt: *mut edict_s) -> c_int>,
    pub pfnFindEntityInSphere: Option<
        unsafe extern "C" fn(
            pEdictStartSearchAfter: *mut edict_s,
            org: *const f32,
            rad: f32,
        ) -> *mut edict_s,
    >,
    pub pfnFindClientInPVS: Option<unsafe extern "C" fn(pEdict: *mut edict_s) -> *mut edict_s>,
    pub pfnEntitiesInPVS: Option<unsafe extern "C" fn(pplayer: *mut edict_s) -> *mut edict_s>,
    pub pfnMakeVectors: Option<unsafe extern "C" fn(rgflVector: *const f32)>,
    pub pfnAngleVectors: Option<
        unsafe extern "C" fn(
            rgflVector: *const f32,
            forward: *mut f32,
            right: *mut f32,
            up: *mut f32,
        ),
    >,
    pub pfnCreateEntity: Option<unsafe extern "C" fn() -> *mut edict_s>,
    pub pfnRemoveEntity: Option<unsafe extern "C" fn(e: *mut edict_s)>,
    pub pfnCreateNamedEntity: Option<unsafe extern "C" fn(className: c_int) -> *mut edict_s>,
    pub pfnMakeStatic: Option<unsafe extern "C" fn(ent: *mut edict_s)>,
    pub pfnEntIsOnFloor: Option<unsafe extern "C" fn(e: *mut edict_s) -> c_int>,
    pub pfnDropToFloor: Option<unsafe extern "C" fn(e: *mut edict_s) -> c_int>,
    pub pfnWalkMove:
        Option<unsafe extern "C" fn(ent: *mut edict_s, yaw: f32, dist: f32, iMode: c_int) -> c_int>,
    pub pfnSetOrigin: Option<unsafe extern "C" fn(e: *mut edict_s, rgflOrigin: *const f32)>,
    pub pfnEmitSound: Option<
        unsafe extern "C" fn(
            entity: *mut edict_s,
            channel: c_int,
            sample: *const c_char,
            volume: f32,
            attenuation: f32,
            fFlags: c_int,
            pitch: c_int,
        ),
    >,
    pub pfnEmitAmbientSound: Option<
        unsafe extern "C" fn(
            entity: *mut edict_s,
            pos: *mut f32,
            samp: *const c_char,
            vol: f32,
            attenuation: f32,
            fFlags: c_int,
            pitch: c_int,
        ),
    >,
    pub pfnTraceLine: Option<
        unsafe extern "C" fn(
            v1: *const f32,
            v2: *const f32,
            fNoMonsters: c_int,
            pentToSkip: *mut edict_s,
            ptr: *mut TraceResult,
        ),
    >,
    pub pfnTraceToss: Option<
        unsafe extern "C" fn(pent: *mut edict_s, pentToIgnore: *mut edict_s, ptr: *mut TraceResult),
    >,
    pub pfnTraceMonsterHull: Option<
        unsafe extern "C" fn(
            pEdict: *mut edict_s,
            v1: *const f32,
            v2: *const f32,
            fNoMonsters: c_int,
            pentToSkip: *mut edict_s,
            ptr: *mut TraceResult,
        ) -> c_int,
    >,
    pub pfnTraceHull: Option<
        unsafe extern "C" fn(
            v1: *const f32,
            v2: *const f32,
            fNoMonsters: c_int,
            hullNumber: c_int,
            pentToSkip: *mut edict_s,
            ptr: *mut TraceResult,
        ),
    >,
    pub pfnTraceModel: Option<
        unsafe extern "C" fn(
            v1: *const f32,
            v2: *const f32,
            hullNumber: c_int,
            pent: *mut edict_s,
            ptr: *mut TraceResult,
        ),
    >,
    pub pfnTraceTexture: Option<
        unsafe extern "C" fn(
            pTextureEntity: *mut edict_s,
            v1: *const f32,
            v2: *const f32,
        ) -> *const c_char,
    >,
    pub pfnTraceSphere: Option<
        unsafe extern "C" fn(
            v1: *const f32,
            v2: *const f32,
            fNoMonsters: c_int,
            radius: f32,
            pentToSkip: *mut edict_s,
            ptr: *mut TraceResult,
        ),
    >,
    pub pfnGetAimVector:
        Option<unsafe extern "C" fn(ent: *mut edict_s, speed: f32, rgflReturn: *mut f32)>,
    pub pfnServerCommand: Option<unsafe extern "C" fn(str_: *const c_char)>,
    pub pfnServerExecute: Option<unsafe extern "C" fn()>,
    pub pfnClientCommand:
        Option<unsafe extern "C" fn(pEdict: *mut edict_s, szFmt: *mut c_char, ...)>,
    pub pfnParticleEffect:
        Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, color: f32, count: f32)>,
    pub pfnLightStyle: Option<unsafe extern "C" fn(style: c_int, val: *const c_char)>,
    pub pfnDecalIndex: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    pub pfnPointContents: Option<unsafe extern "C" fn(rgflVector: *const f32) -> c_int>,
    pub pfnMessageBegin: Option<
        unsafe extern "C" fn(
            msg_dest: c_int,
            msg_type: c_int,
            pOrigin: *const f32,
            ed: *mut edict_s,
        ),
    >,
    pub pfnMessageEnd: Option<unsafe extern "C" fn()>,
    pub pfnWriteByte: Option<unsafe extern "C" fn(iValue: c_int)>,
    pub pfnWriteChar: Option<unsafe extern "C" fn(iValue: c_int)>,
    pub pfnWriteShort: Option<unsafe extern "C" fn(iValue: c_int)>,
    pub pfnWriteLong: Option<unsafe extern "C" fn(iValue: c_int)>,
    pub pfnWriteAngle: Option<unsafe extern "C" fn(flValue: f32)>,
    pub pfnWriteCoord: Option<unsafe extern "C" fn(flValue: f32)>,
    pub pfnWriteString: Option<unsafe extern "C" fn(sz: *const c_char)>,
    pub pfnWriteEntity: Option<unsafe extern "C" fn(iValue: c_int)>,
    pub pfnCVarRegister: Option<unsafe extern "C" fn(pCvar: *mut cvar_s)>,
    pub pfnCVarGetFloat: Option<unsafe extern "C" fn(szVarName: *const c_char) -> f32>,
    pub pfnCVarGetString: Option<unsafe extern "C" fn(szVarName: *const c_char) -> *const c_char>,
    pub pfnCVarSetFloat: Option<unsafe extern "C" fn(szVarName: *const c_char, flValue: f32)>,
    pub pfnCVarSetString:
        Option<unsafe extern "C" fn(szVarName: *const c_char, szValue: *const c_char)>,
    pub pfnAlertMessage: Option<unsafe extern "C" fn(atype: ALERT_TYPE, szFmt: *const c_char, ...)>,
    pub pfnEngineFprintf: Option<unsafe extern "C" fn(pfile: *mut FILE, szFmt: *const c_char, ...)>,
    pub pfnPvAllocEntPrivateData:
        Option<unsafe extern "C" fn(pEdict: *mut edict_s, cb: c_long) -> *mut c_void>,
    pub pfnPvEntPrivateData: Option<unsafe extern "C" fn(pEdict: *mut edict_s) -> *mut c_void>,
    pub pfnFreeEntPrivateData: Option<unsafe extern "C" fn(pEdict: *mut edict_s)>,
    pub pfnSzFromIndex: Option<unsafe extern "C" fn(iString: c_int) -> *const c_char>,
    pub pfnAllocString: Option<unsafe extern "C" fn(szValue: *const c_char) -> c_int>,
    pub pfnGetVarsOfEnt: Option<unsafe extern "C" fn(pEdict: *mut edict_s) -> *mut entvars_s>,
    pub pfnPEntityOfEntOffset: Option<unsafe extern "C" fn(iEntOffset: c_int) -> *mut edict_s>,
    pub pfnEntOffsetOfPEntity: Option<unsafe extern "C" fn(pEdict: *const edict_s) -> c_int>,
    pub pfnIndexOfEdict: Option<unsafe extern "C" fn(pEdict: *const edict_s) -> c_int>,
    pub pfnPEntityOfEntIndex: Option<unsafe extern "C" fn(iEntIndex: c_int) -> *mut edict_s>,
    pub pfnFindEntityByVars: Option<unsafe extern "C" fn(pvars: *mut entvars_s) -> *mut edict_s>,
    pub pfnGetModelPtr: Option<unsafe extern "C" fn(pEdict: *mut edict_s) -> *mut c_void>,
    pub pfnRegUserMsg: Option<unsafe extern "C" fn(pszName: *const c_char, iSize: c_int) -> c_int>,
    pub pfnAnimationAutomove: Option<unsafe extern "C" fn(pEdict: *const edict_s, flTime: f32)>,
    pub pfnGetBonePosition: Option<
        unsafe extern "C" fn(
            pEdict: *const edict_s,
            iBone: c_int,
            rgflOrigin: *mut f32,
            rgflAngles: *mut f32,
        ),
    >,
    pub pfnFunctionFromName: Option<unsafe extern "C" fn(pName: *const c_char) -> c_ulong>,
    pub pfnNameForFunction: Option<unsafe extern "C" fn(function: c_ulong) -> *const c_char>,
    pub pfnClientPrintf:
        Option<unsafe extern "C" fn(pEdict: *mut edict_s, ptype: PRINT_TYPE, szMsg: *const c_char)>,
    pub pfnServerPrint: Option<unsafe extern "C" fn(szMsg: *const c_char)>,
    pub pfnCmd_Args: Option<unsafe extern "C" fn() -> *const c_char>,
    pub pfnCmd_Argv: Option<unsafe extern "C" fn(argc: c_int) -> *const c_char>,
    pub pfnCmd_Argc: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetAttachment: Option<
        unsafe extern "C" fn(
            pEdict: *const edict_s,
            iAttachment: c_int,
            rgflOrigin: *mut f32,
            rgflAngles: *mut f32,
        ),
    >,
    pub pfnCRC32_Init: Option<unsafe extern "C" fn(pulCRC: *mut CRC32_t)>,
    pub pfnCRC32_ProcessBuffer:
        Option<unsafe extern "C" fn(pulCRC: *mut CRC32_t, p: *const c_void, len: c_int)>,
    pub pfnCRC32_ProcessByte: Option<unsafe extern "C" fn(pulCRC: *mut CRC32_t, ch: c_uchar)>,
    pub pfnCRC32_Final: Option<unsafe extern "C" fn(pulCRC: CRC32_t) -> CRC32_t>,
    pub pfnRandomLong: Option<unsafe extern "C" fn(lLow: c_int, lHigh: c_int) -> c_int>,
    pub pfnRandomFloat: Option<unsafe extern "C" fn(flLow: f32, flHigh: f32) -> f32>,
    pub pfnSetView: Option<unsafe extern "C" fn(pClient: *const edict_s, pViewent: *const edict_s)>,
    pub pfnTime: Option<unsafe extern "C" fn() -> f32>,
    pub pfnCrosshairAngle:
        Option<unsafe extern "C" fn(pClient: *const edict_s, pitch: f32, yaw: f32)>,
    pub pfnLoadFileForMe:
        Option<unsafe extern "C" fn(filename: *const c_char, pLength: *mut c_int) -> *mut byte>,
    pub pfnFreeFile: Option<unsafe extern "C" fn(buffer: *mut c_void)>,
    pub pfnEndSection: Option<unsafe extern "C" fn(pszSectionName: *const c_char)>,
    pub pfnCompareFileTime: Option<
        unsafe extern "C" fn(
            filename1: *const c_char,
            filename2: *const c_char,
            iCompare: *mut c_int,
        ) -> c_int,
    >,
    pub pfnGetGameDir: Option<unsafe extern "C" fn(szGetGameDir: *mut c_char)>,
    pub pfnCvar_RegisterVariable: Option<unsafe extern "C" fn(variable: *mut cvar_s)>,
    pub pfnFadeClientVolume: Option<
        unsafe extern "C" fn(
            pEdict: *const edict_s,
            fadePercent: c_int,
            fadeOutSeconds: c_int,
            holdTime: c_int,
            fadeInSeconds: c_int,
        ),
    >,
    pub pfnSetClientMaxspeed:
        Option<unsafe extern "C" fn(pEdict: *const edict_s, fNewMaxspeed: f32)>,
    pub pfnCreateFakeClient: Option<unsafe extern "C" fn(netname: *const c_char) -> *mut edict_s>,
    pub pfnRunPlayerMove: Option<
        unsafe extern "C" fn(
            fakeclient: *mut edict_s,
            viewangles: *const f32,
            forwardmove: f32,
            sidemove: f32,
            upmove: f32,
            buttons: c_ushort,
            impulse: byte,
            msec: byte,
        ),
    >,
    pub pfnNumberOfEntities: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetInfoKeyBuffer: Option<unsafe extern "C" fn(e: *mut edict_s) -> *mut c_char>,
    pub pfnInfoKeyValue: Option<
        unsafe extern "C" fn(infobuffer: *const c_char, key: *const c_char) -> *const c_char,
    >,
    pub pfnSetKeyValue:
        Option<unsafe extern "C" fn(infobuffer: *mut c_char, key: *mut c_char, value: *mut c_char)>,
    pub pfnSetClientKeyValue: Option<
        unsafe extern "C" fn(
            clientIndex: c_int,
            infobuffer: *mut c_char,
            key: *mut c_char,
            value: *mut c_char,
        ),
    >,
    pub pfnIsMapValid: Option<unsafe extern "C" fn(filename: *const c_char) -> c_int>,
    pub pfnStaticDecal: Option<
        unsafe extern "C" fn(
            origin: *const f32,
            decalIndex: c_int,
            entityIndex: c_int,
            modelIndex: c_int,
        ),
    >,
    pub pfnPrecacheGeneric: Option<unsafe extern "C" fn(s: *const c_char) -> c_int>,
    pub pfnGetPlayerUserId: Option<unsafe extern "C" fn(e: *mut edict_s) -> c_int>,
    pub pfnBuildSoundMsg: Option<
        unsafe extern "C" fn(
            entity: *mut edict_s,
            channel: c_int,
            sample: *const c_char,
            volume: f32,
            attenuation: f32,
            fFlags: c_int,
            pitch: c_int,
            msg_dest: c_int,
            msg_type: c_int,
            pOrigin: *const f32,
            ed: *mut edict_s,
        ),
    >,
    pub pfnIsDedicatedServer: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnCVarGetPointer: Option<unsafe extern "C" fn(szVarName: *const c_char) -> *mut cvar_s>,
    pub pfnGetPlayerWONId: Option<unsafe extern "C" fn(e: *mut edict_s) -> c_uint>,
    pub pfnInfo_RemoveKey: Option<unsafe extern "C" fn(s: *mut c_char, key: *const c_char)>,
    pub pfnGetPhysicsKeyValue:
        Option<unsafe extern "C" fn(pClient: *const edict_s, key: *const c_char) -> *const c_char>,
    pub pfnSetPhysicsKeyValue: Option<
        unsafe extern "C" fn(pClient: *const edict_s, key: *const c_char, value: *const c_char),
    >,
    pub pfnGetPhysicsInfoString:
        Option<unsafe extern "C" fn(pClient: *const edict_s) -> *const c_char>,
    pub pfnPrecacheEvent:
        Option<unsafe extern "C" fn(type_: c_int, psz: *const c_char) -> c_ushort>,
    pub pfnPlaybackEvent: Option<
        unsafe extern "C" fn(
            flags: c_int,
            pInvoker: *const edict_s,
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
    pub pfnSetFatPVS: Option<unsafe extern "C" fn(org: *const f32) -> *mut c_uchar>,
    pub pfnSetFatPAS: Option<unsafe extern "C" fn(org: *const f32) -> *mut c_uchar>,
    pub pfnCheckVisibility:
        Option<unsafe extern "C" fn(entity: *const edict_s, pset: *mut c_uchar) -> c_int>,
    pub pfnDeltaSetField:
        Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldname: *const c_char)>,
    pub pfnDeltaUnsetField:
        Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldname: *const c_char)>,
    pub pfnDeltaAddEncoder: Option<
        unsafe extern "C" fn(
            name: *mut c_char,
            conditionalencode: Option<
                unsafe extern "C" fn(
                    pFields: *mut delta_s,
                    from: *const c_uchar,
                    to: *const c_uchar,
                ),
            >,
        ),
    >,
    pub pfnGetCurrentPlayer: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnCanSkipPlayer: Option<unsafe extern "C" fn(player: *const edict_s) -> c_int>,
    pub pfnDeltaFindField:
        Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldname: *const c_char) -> c_int>,
    pub pfnDeltaSetFieldByIndex:
        Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldNumber: c_int)>,
    pub pfnDeltaUnsetFieldByIndex:
        Option<unsafe extern "C" fn(pFields: *mut delta_s, fieldNumber: c_int)>,
    pub pfnSetGroupMask: Option<unsafe extern "C" fn(mask: c_int, op: c_int)>,
    pub pfnCreateInstancedBaseline:
        Option<unsafe extern "C" fn(classname: c_int, baseline: *mut entity_state_s) -> c_int>,
    pub pfnCvar_DirectSet: Option<unsafe extern "C" fn(var: *mut cvar_s, value: *const c_char)>,
    pub pfnForceUnmodified: Option<
        unsafe extern "C" fn(
            type_: FORCE_TYPE,
            mins: *mut f32,
            maxs: *mut f32,
            filename: *const c_char,
        ),
    >,
    pub pfnGetPlayerStats: Option<
        unsafe extern "C" fn(pClient: *const edict_s, ping: *mut c_int, packet_loss: *mut c_int),
    >,
    pub pfnAddServerCommand:
        Option<unsafe extern "C" fn(cmd_name: *const c_char, function: unsafe extern "C" fn())>,
    pub pfnVoice_GetClientListening:
        Option<unsafe extern "C" fn(iReceiver: c_int, iSender: c_int) -> qboolean>,
    pub pfnVoice_SetClientListening: Option<
        unsafe extern "C" fn(iReceiver: c_int, iSender: c_int, bListen: qboolean) -> qboolean,
    >,
    pub pfnGetPlayerAuthId: Option<unsafe extern "C" fn(e: *mut edict_s) -> *const c_char>,
    pub pfnSequenceGet: Option<
        unsafe extern "C" fn(fileName: *const c_char, entryName: *const c_char) -> *mut c_void,
    >,
    pub pfnSequencePickSentence: Option<
        unsafe extern "C" fn(
            groupName: *const c_char,
            pickMethod: c_int,
            picked: *mut c_int,
        ) -> *mut c_void,
    >,
    pub pfnGetFileSize: Option<unsafe extern "C" fn(filename: *const c_char) -> c_int>,
    pub pfnGetApproxWavePlayLen: Option<unsafe extern "C" fn(filepath: *const c_char) -> c_uint>,
    pub pfnIsCareerMatch: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetLocalizedStringLength: Option<unsafe extern "C" fn(label: *const c_char) -> c_int>,
    pub pfnRegisterTutorMessageShown: Option<unsafe extern "C" fn(mid: c_int)>,
    pub pfnGetTimesTutorMessageShown: Option<unsafe extern "C" fn(mid: c_int) -> c_int>,
    pub pfnProcessTutorMessageDecayBuffer:
        Option<unsafe extern "C" fn(buffer: *mut c_int, bufferLength: c_int)>,
    pub pfnConstructTutorMessageDecayBuffer:
        Option<unsafe extern "C" fn(buffer: *mut c_int, bufferLength: c_int)>,
    pub pfnResetTutorMessageDecayData: Option<unsafe extern "C" fn()>,
    pub pfnQueryClientCvarValue:
        Option<unsafe extern "C" fn(player: *const edict_s, cvarName: *const c_char)>,
    pub pfnQueryClientCvarValue2: Option<
        unsafe extern "C" fn(player: *const edict_s, cvarName: *const c_char, requestID: c_int),
    >,
    pub pfnCheckParm:
        Option<unsafe extern "C" fn(parm: *mut c_char, ppnext: *mut *mut c_char) -> c_int>,
    pub pfnPEntityOfEntIndexAllEntities:
        Option<unsafe extern "C" fn(iEntIndex: c_int) -> *mut edict_s>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct globalvars_t {
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
    pub changelevel: c_int,
    pub cdAudioTrack: c_int,
    pub maxClients: c_int,
    pub maxEntities: c_int,
    pub pStringBase: *const c_char,
    pub pSaveData: *mut c_void,
    pub vecLandmarkOffset: vec3_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyValueData {
    pub szClassName: *mut c_char,
    pub szKeyName: *mut c_char,
    pub szValue: *mut c_char,
    pub fHandled: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ENTITYTABLE {
    pub id: c_int,
    pub pent: *mut edict_s,
    pub location: c_int,
    pub size: c_int,
    pub flags: c_int,
    pub classname: Option<MapString>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LEVELLIST {
    pub mapName: CStrArray<32>,
    pub landmarkName: CStrArray<32>,
    pub pentLandmark: *mut edict_s,
    pub vecLandmarkOrigin: vec3_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct saverestore_s {
    pub base_data: *mut c_char,
    pub current_data: *mut c_char,
    pub size: c_int,
    pub buffer_size: c_int,
    pub token_size: c_int,
    token_count: c_int,
    tokens: *mut *mut c_char,
    pub current_index: c_int,
    table_count: c_int,
    pub connection_count: c_int,
    table: *mut ENTITYTABLE,
    pub level_list: [LEVELLIST; MAX_LEVEL_CONNECTIONS],
    pub use_landmark: c_int,
    pub landmark_name: CStrArray<20>,
    pub landmark_offset: vec3_t,
    pub time: f32,
    pub current_map_name: CStrArray<32>,
}
pub type SAVERESTOREDATA = saverestore_s;

impl saverestore_s {
    pub fn table(&self) -> &[ENTITYTABLE] {
        if !self.table.is_null() {
            unsafe { slice::from_raw_parts(self.table, self.table_count as usize) }
        } else {
            &[]
        }
    }

    pub fn table_mut(&mut self) -> &mut [ENTITYTABLE] {
        if !self.table.is_null() {
            unsafe { slice::from_raw_parts_mut(self.table, self.table_count as usize) }
        } else {
            &mut []
        }
    }

    pub fn tokens(&mut self) -> &[*mut c_char] {
        if !self.tokens.is_null() {
            unsafe { slice::from_raw_parts(self.tokens, self.token_count as usize) }
        } else {
            &mut []
        }
    }

    pub fn tokens_mut(&mut self) -> &mut [*mut c_char] {
        if !self.tokens.is_null() {
            unsafe { slice::from_raw_parts_mut(self.tokens, self.token_count as usize) }
        } else {
            &mut []
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum FieldType {
    FLOAT = 0,
    STRING = 1,
    ENTITY = 2,
    CLASSPTR = 3,
    EHANDLE = 4,
    EVARS = 5,
    EDICT = 6,
    VECTOR = 7,
    POSITION_VECTOR = 8,
    POINTER = 9,
    INTEGER = 10,
    FUNCTION = 11,
    BOOLEAN = 12,
    SHORT = 13,
    CHARACTER = 14,
    TIME = 15,
    MODELNAME = 16,
    SOUNDNAME = 17,
    TYPECOUNT = 18,
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    #[repr(transparent)]
    pub struct FtypeDesc: c_short {
        const NONE = 0;
        const GLOBAL = 1;
        const SAVE = 2;
        const KEY = 4;
        const FUNCTIONTABLE = 8;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TYPEDESCRIPTION {
    pub fieldType: FieldType,
    pub fieldName: *const c_char,
    pub fieldOffset: c_int,
    pub fieldSize: c_short,
    pub flags: FtypeDesc,
}

impl TYPEDESCRIPTION {
    pub fn name(&self) -> Option<&CStrThin> {
        if self.fieldName.is_null() {
            None
        } else {
            Some(unsafe { CStrThin::from_ptr(self.fieldName) })
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DLL_FUNCTIONS {
    pub pfnGameInit: Option<unsafe extern "C" fn()>,
    pub pfnSpawn: Option<unsafe extern "C" fn(pent: *mut edict_s) -> c_int>,
    pub pfnThink: Option<unsafe extern "C" fn(pent: *mut edict_s)>,
    pub pfnUse: Option<unsafe extern "C" fn(pentUsed: *mut edict_s, pentOther: *mut edict_s)>,
    pub pfnTouch: Option<unsafe extern "C" fn(pentTouched: *mut edict_s, pentOther: *mut edict_s)>,
    pub pfnBlocked:
        Option<unsafe extern "C" fn(pentBlocked: *mut edict_s, pentOther: *mut edict_s)>,
    pub pfnKeyValue:
        Option<unsafe extern "C" fn(pentKeyvalue: *mut edict_s, pkvd: *mut KeyValueData)>,
    pub pfnSave: Option<unsafe extern "C" fn(pent: *mut edict_s, pSaveData: *mut SAVERESTOREDATA)>,
    pub pfnRestore: Option<
        unsafe extern "C" fn(
            pent: *mut edict_s,
            pSaveData: *mut SAVERESTOREDATA,
            globalEntity: c_int,
        ) -> c_int,
    >,
    pub pfnSetAbsBox: Option<unsafe extern "C" fn(pent: *mut edict_s)>,
    pub pfnSaveWriteFields: Option<
        unsafe extern "C" fn(
            save_data: *mut SAVERESTOREDATA,
            name: *const c_char,
            base_data: *mut c_void,
            fields: *mut TYPEDESCRIPTION,
            fields_count: c_int,
        ),
    >,
    pub pfnSaveReadFields: Option<
        unsafe extern "C" fn(
            save_data: *mut SAVERESTOREDATA,
            name: *const c_char,
            base_data: *mut c_void,
            fields: *mut TYPEDESCRIPTION,
            fields_count: c_int,
        ),
    >,
    pub pfnSaveGlobalState: Option<unsafe extern "C" fn(save_data: *mut SAVERESTOREDATA)>,
    pub pfnRestoreGlobalState: Option<unsafe extern "C" fn(save_data: *mut SAVERESTOREDATA)>,
    pub pfnResetGlobalState: Option<unsafe extern "C" fn()>,
    pub pfnClientConnect: Option<
        unsafe extern "C" fn(
            pEntity: *mut edict_s,
            pszName: *const c_char,
            pszAddress: *const c_char,
            szRejectReason: *mut [c_char; 128usize],
        ) -> qboolean,
    >,
    pub pfnClientDisconnect: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientKill: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientPutInServer: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientCommand: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientUserInfoChanged:
        Option<unsafe extern "C" fn(pEntity: *mut edict_s, infobuffer: *mut c_char)>,
    pub pfnServerActivate:
        Option<unsafe extern "C" fn(pEdictList: *mut edict_s, edictCount: c_int, clientMax: c_int)>,
    pub pfnServerDeactivate: Option<unsafe extern "C" fn()>,
    pub pfnPlayerPreThink: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnPlayerPostThink: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnStartFrame: Option<unsafe extern "C" fn()>,
    pub pfnParmsNewLevel: Option<unsafe extern "C" fn()>,
    pub pfnParmsChangeLevel: Option<unsafe extern "C" fn()>,
    pub pfnGetGameDescription: Option<unsafe extern "C" fn() -> *const c_char>,
    pub pfnPlayerCustomization:
        Option<unsafe extern "C" fn(pEntity: *mut edict_s, pCustom: *mut customization_s)>,
    pub pfnSpectatorConnect: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnSpectatorDisconnect: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnSpectatorThink: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnSys_Error: Option<unsafe extern "C" fn(error_string: *const c_char)>,
    pub pfnPM_Move: Option<unsafe extern "C" fn(ppmove: *mut playermove_s, server: qboolean)>,
    pub pfnPM_Init: Option<unsafe extern "C" fn(ppmove: *mut playermove_s)>,
    pub pfnPM_FindTextureType: Option<unsafe extern "C" fn(name: *const c_char) -> c_char>,
    pub pfnSetupVisibility: Option<
        unsafe extern "C" fn(
            pViewEntity: *mut edict_s,
            pClient: *mut edict_s,
            pvs: *mut *mut c_uchar,
            pas: *mut *mut c_uchar,
        ),
    >,
    pub pfnUpdateClientData: Option<
        unsafe extern "C" fn(ent: *const edict_s, sendweapons: c_int, cd: *mut clientdata_s),
    >,
    pub pfnAddToFullPack: Option<
        unsafe extern "C" fn(
            state: *mut entity_state_s,
            e: c_int,
            ent: *mut edict_s,
            host: *mut edict_s,
            hostflags: c_int,
            player: c_int,
            pSet: *mut c_uchar,
        ) -> c_int,
    >,
    pub pfnCreateBaseline: Option<
        unsafe extern "C" fn(
            player: c_int,
            eindex: c_int,
            baseline: *mut entity_state_s,
            entity: *mut edict_s,
            playermodelindex: c_int,
            player_mins: *mut vec3_t,
            player_maxs: *mut vec3_t,
        ),
    >,
    pub pfnRegisterEncoders: Option<unsafe extern "C" fn()>,
    pub pfnGetWeaponData:
        Option<unsafe extern "C" fn(player: *mut edict_s, info: *mut weapon_data_s) -> c_int>,
    pub pfnCmdStart: Option<
        unsafe extern "C" fn(player: *const edict_s, cmd: *const usercmd_s, random_seed: c_uint),
    >,
    pub pfnCmdEnd: Option<unsafe extern "C" fn(player: *const edict_s)>,
    pub pfnConnectionlessPacket: Option<
        unsafe extern "C" fn(
            net_from: *const netadr_s,
            args: *const c_char,
            response_buffer: *mut c_char,
            response_buffer_size: *mut c_int,
        ) -> c_int,
    >,
    pub pfnGetHullBounds: Option<
        unsafe extern "C" fn(hullnumber: c_int, mins: *mut vec3_t, maxs: *mut vec3_t) -> c_int,
    >,
    pub pfnCreateInstancedBaselines: Option<unsafe extern "C" fn()>,
    pub pfnInconsistentFile: Option<
        unsafe extern "C" fn(
            player: *const edict_s,
            filename: *const c_char,
            disconnect_message: *mut c_char,
        ) -> c_int,
    >,
    pub pfnAllowLagCompensation: Option<unsafe extern "C" fn() -> c_int>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NEW_DLL_FUNCTIONS {
    pub pfnOnFreeEntPrivateData: Option<unsafe extern "C" fn(pEnt: *mut edict_s)>,
    pub pfnGameShutdown: Option<unsafe extern "C" fn()>,
    pub pfnShouldCollide:
        Option<unsafe extern "C" fn(pentTouched: *mut edict_s, pentOther: *mut edict_s) -> c_int>,
    pub pfnCvarValue: Option<unsafe extern "C" fn(pEnt: *const edict_s, value: *const c_char)>,
    pub pfnCvarValue2: Option<
        unsafe extern "C" fn(
            pEnt: *const edict_s,
            requestID: c_int,
            cvarName: *const c_char,
            value: *const c_char,
        ),
    >,
}

pub type NEW_DLL_FUNCTIONS_FN = unsafe extern "C" fn(
    pFunctionTable: *mut NEW_DLL_FUNCTIONS,
    interfaceVersion: *mut c_int,
) -> c_int;

pub type APIFUNCTION =
    unsafe extern "C" fn(pFunctionTable: *mut DLL_FUNCTIONS, interfaceVersion: c_int) -> c_int;

pub type APIFUNCTION2 =
    unsafe extern "C" fn(pFunctionTable: *mut DLL_FUNCTIONS, interfaceVersion: *mut c_int) -> c_int;
