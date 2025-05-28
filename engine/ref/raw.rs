#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::type_complexity)]

pub mod filesystem;
pub mod vgui;

use core::ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_ushort, c_void, CStr};

use bitflags::bitflags;
use shared::{
    consts::{RefParm, MAX_MODELS, MAX_SKINS},
    cvar::cvar_s,
    raw::bsp::word,
};

pub use shared::raw::*;

pub type vec_t = f32;
pub type rgba_t = [byte; 4];

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct FContext: c_int {
        const CORE_PROFILE  = 1 << 0;
        const DEBUG_ARB     = 1 << 1;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct ScreenshotType(c_int);

impl ScreenshotType {
    pub const VID_SCREENSHOT: Self = ScreenshotType(0);
    pub const VID_LEVELSHOT: Self = ScreenshotType(1);
    pub const VID_MINISHOT: Self = ScreenshotType(2);
    /// Special case for overview layer.
    pub const VID_MAPSHOT: Self = ScreenshotType(3);
    /// Save screenshot into root dir and no gamma correction.
    pub const VID_SNAPSHOT: Self = ScreenshotType(4);
}

bitflags! {
    /// goes into world.flags
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    #[repr(transparent)]
    pub struct WorldFlags: c_int {
        const SKYSPHERE         = 1 << 0;
        const CUSTOM_SKYBOX     = 1 << 1;
        const WATERALPHA        = 1 << 2;
        const HAS_DELUXEMAP     = 1 << 3;
    }
}

pub const SKYBOX_MAX_SIDES: usize = 6;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum demo_mode {
    INACTIVE = 0,
    XASH3D = 1,
    QUAKE1 = 2,
}

// pub const DXT_ENCODE_DEFAULT: u32 = 0;
// pub const DXT_ENCODE_COLOR_YCoCg: u32 = 6657;
// pub const DXT_ENCODE_ALPHA_1BIT: u32 = 6658;
// pub const DXT_ENCODE_ALPHA_8BIT: u32 = 6659;
// pub const DXT_ENCODE_ALPHA_SDF: u32 = 6660;
// pub const DXT_ENCODE_NORMAL_AG_ORTHO: u32 = 6661;
// pub const DXT_ENCODE_NORMAL_AG_STEREO: u32 = 6662;
// pub const DXT_ENCODE_NORMAL_AG_PARABOLOID: u32 = 6663;
// pub const DXT_ENCODE_NORMAL_AG_QUARTIC: u32 = 6664;
// pub const DXT_ENCODE_NORMAL_AG_AZIMUTHAL: u32 = 6665;
// pub const STUDIO_VERSION: u32 = 10;
// pub const MAXSTUDIOVERTS: u32 = 16384;
// pub const MAXSTUDIOSEQUENCES: u32 = 256;
// pub const MAXSTUDIOSKINS: u32 = 256;
// pub const MAXSTUDIOSRCBONES: u32 = 512;
// pub const MAXSTUDIOBONES: u32 = 128;
// pub const MAXSTUDIOMODELS: u32 = 32;
// pub const MAXSTUDIOBODYPARTS: u32 = 32;
// pub const MAXSTUDIOGROUPS: u32 = 16;
// pub const MAXSTUDIOMESHES: u32 = 256;
// pub const MAXSTUDIOCONTROLLERS: u32 = 32;
// pub const MAXSTUDIOATTACHMENTS: u32 = 64;
// pub const MAXSTUDIOBONEWEIGHTS: u32 = 4;
// pub const MAXSTUDIONAME: u32 = 32;
// pub const MAXSTUDIOPOSEPARAM: u32 = 24;
// pub const MAX_STUDIO_LIGHTMAP_SIZE: u32 = 256;
// pub const STUDIO_ROCKET: u32 = 1;
// pub const STUDIO_GRENADE: u32 = 2;
// pub const STUDIO_GIB: u32 = 4;
// pub const STUDIO_ROTATE: u32 = 8;
// pub const STUDIO_TRACER: u32 = 16;
// pub const STUDIO_ZOMGIB: u32 = 32;
// pub const STUDIO_TRACER2: u32 = 64;
// pub const STUDIO_TRACER3: u32 = 128;
// pub const STUDIO_AMBIENT_LIGHT: u32 = 256;
// pub const STUDIO_TRACE_HITBOX: u32 = 512;
// pub const STUDIO_FORCE_SKYLIGHT: u32 = 1024;
// pub const STUDIO_HAS_BUMP: u32 = 65536;
// pub const STUDIO_STATIC_PROP: u32 = 536870912;
// pub const STUDIO_HAS_BONEINFO: u32 = 1073741824;
// pub const STUDIO_HAS_BONEWEIGHTS: u32 = 2147483648;
// pub const STUDIO_NF_FLATSHADE: u32 = 1;
// pub const STUDIO_NF_CHROME: u32 = 2;
// pub const STUDIO_NF_FULLBRIGHT: u32 = 4;
// pub const STUDIO_NF_NOMIPS: u32 = 8;
// pub const STUDIO_NF_SMOOTH: u32 = 16;
// pub const STUDIO_NF_ADDITIVE: u32 = 32;
// pub const STUDIO_NF_MASKED: u32 = 64;
// pub const STUDIO_NF_NORMALMAP: u32 = 128;
// pub const STUDIO_NF_GLOSSMAP: u32 = 256;
// pub const STUDIO_NF_GLOSSPOWER: u32 = 512;
// pub const STUDIO_NF_LUMA: u32 = 1024;
// pub const STUDIO_NF_ALPHASOLID: u32 = 2048;
// pub const STUDIO_NF_TWOSIDE: u32 = 4096;
// pub const STUDIO_NF_HEIGHTMAP: u32 = 8192;
// pub const STUDIO_NF_NODRAW: u32 = 65536;
// pub const STUDIO_NF_NODLIGHT: u32 = 131072;
// pub const STUDIO_NF_NOSUNLIGHT: u32 = 262144;
// pub const STUDIO_NF_HAS_ALPHA: u32 = 1048576;
// pub const STUDIO_NF_HAS_DETAIL: u32 = 2097152;
// pub const STUDIO_NF_COLORMAP: u32 = 1073741824;
// pub const STUDIO_NF_UV_COORDS: u32 = 2147483648;
// pub const STUDIO_X: u32 = 1;
// pub const STUDIO_Y: u32 = 2;
// pub const STUDIO_Z: u32 = 4;
// pub const STUDIO_XR: u32 = 8;
// pub const STUDIO_YR: u32 = 16;
// pub const STUDIO_ZR: u32 = 32;
// pub const STUDIO_LX: u32 = 64;
// pub const STUDIO_LY: u32 = 128;
// pub const STUDIO_LZ: u32 = 256;
// pub const STUDIO_LXR: u32 = 512;
// pub const STUDIO_LYR: u32 = 1024;
// pub const STUDIO_LZR: u32 = 2048;
// pub const STUDIO_LINEAR: u32 = 4096;
// pub const STUDIO_QUADRATIC_MOTION: u32 = 8192;
// pub const STUDIO_RESERVED: u32 = 16384;
// pub const STUDIO_TYPES: u32 = 32767;
// pub const STUDIO_RLOOP: u32 = 32768;
// pub const STUDIO_MOUTH: u32 = 4;
// pub const STUDIO_LOOPING: u32 = 1;
// pub const STUDIO_SNAP: u32 = 2;
// pub const STUDIO_DELTA: u32 = 4;
// pub const STUDIO_AUTOPLAY: u32 = 8;
// pub const STUDIO_POST: u32 = 16;
// pub const STUDIO_ALLZEROS: u32 = 32;
// pub const STUDIO_BLENDPOSE: u32 = 64;
// pub const STUDIO_CYCLEPOSE: u32 = 128;
// pub const STUDIO_REALTIME: u32 = 256;
// pub const STUDIO_LOCAL: u32 = 512;
// pub const STUDIO_HIDDEN: u32 = 1024;
// pub const STUDIO_IKRULES: u32 = 2048;
// pub const STUDIO_ACTIVITY: u32 = 4096;
// pub const STUDIO_EVENT: u32 = 8192;
// pub const STUDIO_WORLD: u32 = 16384;
// pub const STUDIO_LIGHT_FROM_ROOT: u32 = 32768;
// pub const STUDIO_AL_POST: u32 = 1;
// pub const STUDIO_AL_SPLINE: u32 = 2;
// pub const STUDIO_AL_XFADE: u32 = 4;
// pub const STUDIO_AL_NOBLEND: u32 = 8;
// pub const STUDIO_AL_LOCAL: u32 = 16;
// pub const STUDIO_AL_POSE: u32 = 32;
// pub const BONE_ALWAYS_PROCEDURAL: u32 = 1;
// pub const BONE_SCREEN_ALIGN_SPHERE: u32 = 2;
// pub const BONE_SCREEN_ALIGN_CYLINDER: u32 = 4;
// pub const BONE_JIGGLE_PROCEDURAL: u32 = 8;
// pub const BONE_FIXED_ALIGNMENT: u32 = 16;
// pub const BONE_USED_BY_HITBOX: u32 = 256;
// pub const BONE_USED_BY_ATTACHMENT: u32 = 512;
// pub const BONE_USED_BY_VERTEX: u32 = 1024;
// pub const BONE_USED_BY_BONE_MERGE: u32 = 2048;
// pub const STUDIO_PROC_AXISINTERP: u32 = 1;
// pub const STUDIO_PROC_QUATINTERP: u32 = 2;
// pub const STUDIO_PROC_AIMATBONE: u32 = 3;
// pub const STUDIO_PROC_AIMATATTACH: u32 = 4;
// pub const STUDIO_PROC_JIGGLE: u32 = 5;
// pub const JIGGLE_IS_FLEXIBLE: u32 = 1;
// pub const JIGGLE_IS_RIGID: u32 = 2;
// pub const JIGGLE_HAS_YAW_CONSTRAINT: u32 = 4;
// pub const JIGGLE_HAS_PITCH_CONSTRAINT: u32 = 8;
// pub const JIGGLE_HAS_ANGLE_CONSTRAINT: u32 = 16;
// pub const JIGGLE_HAS_LENGTH_CONSTRAINT: u32 = 32;
// pub const JIGGLE_HAS_BASE_SPRING: u32 = 64;
// pub const JIGGLE_IS_BOING: u32 = 128;
// pub const STUDIO_ATTACHMENT_LOCAL: u32 = 1;
// pub const IK_SELF: u32 = 1;
// pub const IK_WORLD: u32 = 2;
// pub const IK_GROUND: u32 = 3;
// pub const IK_RELEASE: u32 = 4;
// pub const IK_ATTACHMENT: u32 = 5;
// pub const IK_UNLATCH: u32 = 6;

// pub const svc_bad: u32 = 0;
// pub const svc_nop: u32 = 1;
// pub const svc_disconnect: u32 = 2;
// pub const svc_event: u32 = 3;
// pub const svc_changing: u32 = 4;
// pub const svc_setview: u32 = 5;
// pub const svc_sound: u32 = 6;
// pub const svc_time: u32 = 7;
// pub const svc_print: u32 = 8;
// pub const svc_stufftext: u32 = 9;
// pub const svc_setangle: u32 = 10;
// pub const svc_serverdata: u32 = 11;
// pub const svc_lightstyle: u32 = 12;
// pub const svc_updateuserinfo: u32 = 13;
// pub const svc_deltatable: u32 = 14;
// pub const svc_clientdata: u32 = 15;
// pub const svc_resource: u32 = 16;
// pub const svc_pings: u32 = 17;
// pub const svc_particle: u32 = 18;
// pub const svc_restoresound: u32 = 19;
// pub const svc_spawnstatic: u32 = 20;
// pub const svc_event_reliable: u32 = 21;
// pub const svc_spawnbaseline: u32 = 22;
// pub const svc_temp_entity: u32 = 23;
// pub const svc_setpause: u32 = 24;
// pub const svc_signonnum: u32 = 25;
// pub const svc_centerprint: u32 = 26;
// pub const svc_intermission: u32 = 30;
// pub const svc_finale: u32 = 31;
// pub const svc_cdtrack: u32 = 32;
// pub const svc_restore: u32 = 33;
// pub const svc_cutscene: u32 = 34;
// pub const svc_weaponanim: u32 = 35;
// pub const svc_bspdecal: u32 = 36;
// pub const svc_roomtype: u32 = 37;
// pub const svc_addangle: u32 = 38;
// pub const svc_usermessage: u32 = 39;
// pub const svc_packetentities: u32 = 40;
// pub const svc_deltapacketentities: u32 = 41;
// pub const svc_choke: u32 = 42;
// pub const svc_resourcelist: u32 = 43;
// pub const svc_deltamovevars: u32 = 44;
// pub const svc_resourcerequest: u32 = 45;
// pub const svc_customization: u32 = 46;
// pub const svc_crosshairangle: u32 = 47;
// pub const svc_soundfade: u32 = 48;
// pub const svc_filetxferfailed: u32 = 49;
// pub const svc_hltv: u32 = 50;
// pub const svc_director: u32 = 51;
// pub const svc_voiceinit: u32 = 52;
// pub const svc_voicedata: u32 = 53;
// pub const svc_resourcelocation: u32 = 56;
// pub const svc_querycvarvalue: u32 = 57;
// pub const svc_querycvarvalue2: u32 = 58;
// pub const svc_exec: u32 = 59;
// pub const svc_lastmsg: u32 = 59;

// pub const clc_bad: u32 = 0;
// pub const clc_nop: u32 = 1;
// pub const clc_move: u32 = 2;
// pub const clc_stringcmd: u32 = 3;
// pub const clc_delta: u32 = 4;
// pub const clc_resourcelist: u32 = 5;
// pub const clc_fileconsistency: u32 = 7;
// pub const clc_voicedata: u32 = 8;
// pub const clc_requestcvarvalue: u32 = 9;
// pub const clc_requestcvarvalue2: u32 = 10;
// pub const clc_lastmsg: u32 = 11;

// pub const SND_VOLUME: u32 = 1;
// pub const SND_ATTENUATION: u32 = 2;
// pub const SND_SEQUENCE: u32 = 4;
// pub const SND_PITCH: u32 = 8;
// pub const SND_SENTENCE: u32 = 16;
// pub const SND_STOP: u32 = 32;
// pub const SND_CHANGE_VOL: u32 = 64;
// pub const SND_CHANGE_PITCH: u32 = 128;
// pub const SND_SPAWNING: u32 = 256;
// pub const SND_LOCALSOUND: u32 = 512;
// pub const SND_STOP_LOOPING: u32 = 1024;
// pub const SND_FILTER_CLIENT: u32 = 2048;
// pub const SND_RESTORE_POSITION: u32 = 4096;
// pub const FDECAL_PERMANENT: u32 = 1;
// pub const FDECAL_USE_LANDMARK: u32 = 2;
// pub const FDECAL_CUSTOM: u32 = 4;
// pub const FDECAL_DONTSAVE: u32 = 32;
// pub const FDECAL_STUDIO: u32 = 64;
// pub const FDECAL_LOCAL_SPACE: u32 = 128;
// pub const GAME_SINGLEPLAYER: u32 = 0;
// pub const GAME_DEATHMATCH: u32 = 1;
// pub const GAME_COOP: u32 = 2;
// pub const GAME_TEAMPLAY: u32 = 4;
// pub const NUM_BACKUP_COMMAND_BITS: u32 = 4;
// pub const MAX_TOTAL_CMDS: u32 = 32;
// pub const MAX_RESOURCES: u32 = 8192;
// pub const MAX_RESOURCE_BITS: u32 = 13;
// pub const FRAGMENT_MIN_SIZE: u32 = 508;
// pub const FRAGMENT_DEFAULT_SIZE: u32 = 1200;
// pub const FRAGMENT_MAX_SIZE: u32 = 64000;
// pub const FRAGMENT_LOCAL_SIZE: u32 = 64000;

// pub const svc_updatestat: u32 = 3;
// pub const svc_version: u32 = 4;
// pub const svc_updatename: u32 = 13;
// pub const svc_updatefrags: u32 = 14;
// pub const svc_stopsound: u32 = 16;
// pub const svc_updatecolors: u32 = 17;
// pub const svc_damage: u32 = 19;
// pub const svc_spawnbinary: u32 = 21;
// pub const svc_killedmonster: u32 = 27;
// pub const svc_foundsecret: u32 = 28;
// pub const svc_spawnstaticsound: u32 = 29;
// pub const svc_sellscreen: u32 = 33;
// pub const svc_showlmp: u32 = 35;
// pub const svc_hidelmp: u32 = 36;
// pub const svc_skybox: u32 = 37;
// pub const svc_skyboxsize: u32 = 50;
// pub const svc_fog: u32 = 51;

// pub const U_MOREBITS: u32 = 1;
// pub const U_ORIGIN1: u32 = 2;
// pub const U_ORIGIN2: u32 = 4;
// pub const U_ORIGIN3: u32 = 8;
// pub const U_ANGLE2: u32 = 16;
// pub const U_NOLERP: u32 = 32;
// pub const U_FRAME: u32 = 64;
// pub const U_SIGNAL: u32 = 128;
// pub const U_ANGLE1: u32 = 256;
// pub const U_ANGLE3: u32 = 512;
// pub const U_MODEL: u32 = 1024;
// pub const U_COLORMAP: u32 = 2048;
// pub const U_SKIN: u32 = 4096;
// pub const U_EFFECTS: u32 = 8192;
// pub const U_LONGENTITY: u32 = 16384;
// pub const U_TRANS: u32 = 32768;

// pub const SU_VIEWHEIGHT: u32 = 1;
// pub const SU_IDEALPITCH: u32 = 2;
// pub const SU_PUNCH1: u32 = 4;
// pub const SU_PUNCH2: u32 = 8;
// pub const SU_PUNCH3: u32 = 16;
// pub const SU_VELOCITY1: u32 = 32;
// pub const SU_VELOCITY2: u32 = 64;
// pub const SU_VELOCITY3: u32 = 128;
// pub const SU_ITEMS: u32 = 512;
// pub const SU_ONGROUND: u32 = 1024;
// pub const SU_INWATER: u32 = 2048;
// pub const SU_WEAPONFRAME: u32 = 4096;
// pub const SU_ARMOR: u32 = 8192;
// pub const SU_WEAPON: u32 = 16384;

// pub const NET_EXT_SPLITSIZE: u32 = 1;
// pub const PROTOCOL_LEGACY_VERSION: u32 = 48;

// pub const svc_legacy_modelindex: u32 = 31;
// pub const svc_legacy_soundindex: u32 = 28;
// pub const svc_legacy_eventindex: u32 = 34;
// pub const svc_legacy_ambientsound: u32 = 29;
// pub const svc_legacy_chokecount: u32 = 42;
// pub const svc_legacy_event: u32 = 27;
// pub const svc_legacy_changing: u32 = 3;
// pub const clc_legacy_userinfo: u32 = 6;

// pub const SND_LEGACY_LARGE_INDEX: u32 = 4;
// pub const MAX_LEGACY_ENTITY_BITS: u32 = 12;
// pub const MAX_LEGACY_WEAPON_BITS: u32 = 5;
// pub const MAX_LEGACY_MODEL_BITS: u32 = 11;
// pub const MAX_LEGACY_TOTAL_CMDS: u32 = 16;
// pub const MAX_LEGACY_BACKUP_CMDS: u32 = 12;
// pub const MAX_LEGACY_EDICTS: u32 = 4096;
// pub const MIN_LEGACY_EDICTS: u32 = 30;
// pub const MS_SCAN_REQUEST: &[u8; 13] = b"1\xFF0.0.0.0:0\0\0";

// pub const PROTOCOL_GOLDSRC_VERSION: u32 = 48;

// pub const svc_goldsrc_version: u32 = 4;
// pub const svc_goldsrc_stopsound: u32 = 16;
// pub const svc_goldsrc_damage: u32 = 19;
// pub const svc_goldsrc_killedmonster: u32 = 27;
// pub const svc_goldsrc_foundsecret: u32 = 28;
// pub const svc_goldsrc_spawnstaticsound: u32 = 29;
// pub const svc_goldsrc_decalname: u32 = 36;
// pub const svc_goldsrc_sendextrainfo: u32 = 54;
// pub const svc_goldsrc_timescale: u32 = 55;
// pub const clc_goldsrc_hltv: u32 = 9;
// pub const clc_goldsrc_requestcvarvalue: u32 = 10;
// pub const clc_goldsrc_requestcvarvalue2: u32 = 11;
// pub const clc_goldsrc_lastmsg: u32 = 11;
// pub const MAX_GOLDSRC_BACKUP_CMDS: u32 = 8;
// pub const MAX_GOLDSRC_TOTAL_CMDS: u32 = 16;
// pub const MAX_GOLDSRC_EXTENDED_TOTAL_CMDS: u32 = 62;
// pub const MAX_GOLDSRC_MODEL_BITS: u32 = 10;
// pub const MAX_GOLDSRC_RESOURCE_BITS: u32 = 12;
// pub const MAX_GOLDSRC_ENTITY_BITS: u32 = 11;
// pub const A2A_PING: &CStr = c"ping";
// pub const A2A_ACK: &CStr = c"ack";
// pub const A2A_INFO: &CStr = c"info";
// pub const A2A_NETINFO: &CStr = c"netinfo";
// pub const A2A_GOLDSRC_PING: &CStr = c"i";
// pub const A2A_GOLDSRC_ACK: &CStr = c"j";
// pub const A2S_GOLDSRC_INFO: &CStr = c"TSource Engine Query";
// pub const A2S_GOLDSRC_RULES: u8 = 86u8;
// pub const A2S_GOLDSRC_PLAYERS: u8 = 85u8;
// pub const S2A_GOLDSRC_INFO: u8 = 73u8;
// pub const S2A_GOLDSRC_RULES: u8 = 69u8;
// pub const S2A_GOLDSRC_PLAYERS: u8 = 68u8;
// pub const M2S_CHALLENGE: &CStr = c"s";
// pub const M2S_NAT_CONNECT: &CStr = c"c";
// pub const S2M_INFO: &CStr = c"0\n";
// pub const C2S_BANDWIDTHTEST: &CStr = c"bandwidth";
// pub const C2S_GETCHALLENGE: &CStr = c"getchallenge";
// pub const C2S_CONNECT: &CStr = c"connect";
// pub const C2S_RCON: &CStr = c"rcon";
// pub const S2C_BANDWIDTHTEST: &CStr = c"testpacket";
// pub const S2C_CHALLENGE: &CStr = c"challenge";
// pub const S2C_CONNECTION: &CStr = c"client_connect";
// pub const S2C_ERRORMSG: &CStr = c"errormsg";
// pub const S2C_REJECT: &CStr = c"disconnect";
// pub const S2C_GOLDSRC_REJECT_BADPASSWORD: u8 = 56u8;
// pub const S2C_GOLDSRC_REJECT: u8 = 57u8;
// pub const S2C_GOLDSRC_CHALLENGE: &CStr = c"A00000000";
// pub const S2C_GOLDSRC_CONNECTION: &CStr = c"B";
// pub const A2C_PRINT: &CStr = c"print";
// pub const A2C_GOLDSRC_PRINT: u8 = 108u8;
// pub const M2A_SERVERSLIST: &CStr = c"f";

// pub type pfnCreateInterface_t = Option<
//     unsafe extern "C" fn(
//         arg1: *const c_char,
//         arg2: *mut c_int,
//     ) -> *mut c_void,
// >;

// #[repr(C)]
// pub struct color24 {
//     pub r: byte,
//     pub g: byte,
//     pub b: byte,
// }

#[repr(C)]
pub struct colorVec {
    pub r: c_uint,
    pub g: c_uint,
    pub b: c_uint,
    pub a: c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct server_studio_api_s {
    pub Mem_Calloc: Option<unsafe extern "C" fn(number: c_int, size: usize) -> *mut c_void>,
    pub Cache_Check: Option<unsafe extern "C" fn(c: *mut cache_user_s) -> *mut c_void>,
    pub LoadCacheFile: Option<unsafe extern "C" fn(path: *const c_char, cu: *mut cache_user_s)>,
    pub Mod_Extradata: Option<unsafe extern "C" fn(mod_: *mut model_s) -> *mut c_void>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sv_blending_interface_s {
    pub version: c_int,
    pub SV_StudioSetupBones: Option<
        unsafe extern "C" fn(
            pModel: *mut model_s,
            frame: f32,
            sequence: c_int,
            angles: *mut vec3_t,
            origin: *mut vec3_t,
            pcontroller: *const byte,
            pblending: *const byte,
            iBone: c_int,
            pEdict: *const edict_s,
        ),
    >,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct render_api_s {
    pub RenderGetParm: Option<unsafe extern "C" fn(parm: c_int, arg: c_int) -> isize>,
    pub GetDetailScaleForTexture:
        Option<unsafe extern "C" fn(texture: c_int, xScale: *mut f32, yScale: *mut f32)>,
    pub GetExtraParmsForTexture: Option<
        unsafe extern "C" fn(
            texture: c_int,
            red: *mut byte,
            green: *mut byte,
            blue: *mut byte,
            alpha: *mut byte,
        ),
    >,
    pub GetLightStyle: Option<unsafe extern "C" fn(number: c_int) -> *mut lightstyle_t>,
    pub GetDynamicLight: Option<unsafe extern "C" fn(number: c_int) -> *mut dlight_s>,
    pub GetEntityLight: Option<unsafe extern "C" fn(number: c_int) -> *mut dlight_s>,
    pub LightToTexGamma: Option<unsafe extern "C" fn(color: byte) -> byte>,
    pub GetFrameTime: Option<unsafe extern "C" fn() -> f32>,
    pub R_SetCurrentEntity: Option<unsafe extern "C" fn(ent: *mut cl_entity_s)>,
    pub R_SetCurrentModel: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    pub R_FatPVS: Option<
        unsafe extern "C" fn(
            org: *const f32,
            radius: f32,
            visbuffer: *mut byte,
            merge: qboolean,
            fullvis: qboolean,
        ) -> c_int,
    >,
    pub R_StoreEfrags: Option<unsafe extern "C" fn(ppefrag: *mut *mut efrag_s, framecount: c_int)>,
    pub GL_FindTexture: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    pub GL_TextureName: Option<unsafe extern "C" fn(texnum: c_uint) -> *const c_char>,
    pub GL_TextureData: Option<unsafe extern "C" fn(texnum: c_uint) -> *const byte>,
    pub GL_LoadTexture: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            buf: *const byte,
            size: usize,
            flags: c_int,
        ) -> c_int,
    >,
    pub GL_CreateTexture: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            width: c_int,
            height: c_int,
            buffer: *const c_void,
            flags: texFlags_t,
        ) -> c_int,
    >,
    pub GL_LoadTextureArray:
        Option<unsafe extern "C" fn(names: *mut *const c_char, flags: c_int) -> c_int>,
    pub GL_CreateTextureArray: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            width: c_int,
            height: c_int,
            depth: c_int,
            buffer: *const c_void,
            flags: texFlags_t,
        ) -> c_int,
    >,
    pub GL_FreeTexture: Option<unsafe extern "C" fn(texnum: c_uint)>,
    pub DrawSingleDecal: Option<unsafe extern "C" fn(pDecal: *mut decal_s, fa: *mut msurface_s)>,
    pub R_DecalSetupVerts: Option<
        unsafe extern "C" fn(
            pDecal: *mut decal_s,
            surf: *mut msurface_s,
            texture: c_int,
            outCount: *mut c_int,
        ) -> *mut f32,
    >,
    pub R_EntityRemoveDecals: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    pub AVI_LoadVideo:
        Option<unsafe extern "C" fn(filename: *const c_char, load_audio: qboolean) -> *mut c_void>,
    pub AVI_GetVideoInfo: Option<
        unsafe extern "C" fn(
            Avi: *mut c_void,
            xres: *mut c_int,
            yres: *mut c_int,
            duration: *mut f32,
        ) -> c_int,
    >,
    pub AVI_GetVideoFrameNumber: Option<unsafe extern "C" fn(Avi: *mut c_void, time: f32) -> c_int>,
    pub AVI_GetVideoFrame:
        Option<unsafe extern "C" fn(Avi: *mut c_void, frame: c_int) -> *mut byte>,
    pub AVI_UploadRawFrame: Option<
        unsafe extern "C" fn(
            texture: c_int,
            cols: c_int,
            rows: c_int,
            width: c_int,
            height: c_int,
            data: *const byte,
        ),
    >,
    pub AVI_FreeVideo: Option<unsafe extern "C" fn(Avi: *mut c_void)>,
    pub AVI_IsActive: Option<unsafe extern "C" fn(Avi: *mut c_void) -> c_int>,
    pub AVI_StreamSound: Option<
        unsafe extern "C" fn(Avi: *mut c_void, entnum: c_int, fvol: f32, attn: f32, synctime: f32),
    >,
    pub AVI_Reserved0: Option<unsafe extern "C" fn()>,
    pub AVI_Reserved1: Option<unsafe extern "C" fn()>,
    pub GL_Bind: Option<unsafe extern "C" fn(tmu: c_int, texnum: c_uint)>,
    pub GL_SelectTexture: Option<unsafe extern "C" fn(tmu: c_int)>,
    pub GL_LoadTextureMatrix: Option<unsafe extern "C" fn(glmatrix: *const f32)>,
    pub GL_TexMatrixIdentity: Option<unsafe extern "C" fn()>,
    pub GL_CleanUpTextureUnits: Option<unsafe extern "C" fn(last: c_int)>,
    pub GL_TexGen: Option<unsafe extern "C" fn(coord: c_uint, mode: c_uint)>,
    pub GL_TextureTarget: Option<unsafe extern "C" fn(target: c_uint)>,
    pub GL_TexCoordArrayMode: Option<unsafe extern "C" fn(texmode: c_uint)>,
    pub GL_GetProcAddress: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,
    pub GL_UpdateTexSize:
        Option<unsafe extern "C" fn(texnum: c_int, width: c_int, height: c_int, depth: c_int)>,
    pub GL_Reserved0: Option<unsafe extern "C" fn()>,
    pub GL_Reserved1: Option<unsafe extern "C" fn()>,
    pub GL_DrawParticles: Option<
        unsafe extern "C" fn(rvp: *const ref_viewpass_s, trans_pass: qboolean, frametime: f32),
    >,
    pub EnvShot: Option<
        unsafe extern "C" fn(
            vieworg: *const f32,
            name: *const c_char,
            skyshot: qboolean,
            shotsize: c_int,
        ),
    >,
    pub SPR_LoadExt:
        Option<unsafe extern "C" fn(szPicName: *const c_char, texFlags: c_uint) -> c_int>,
    pub LightVec: Option<
        unsafe extern "C" fn(
            start: *const f32,
            end: *const f32,
            lightspot: *mut f32,
            lightvec: *mut f32,
        ) -> colorVec,
    >,
    pub StudioGetTexture: Option<unsafe extern "C" fn(e: *mut cl_entity_s) -> *mut mstudiotex_s>,
    pub GetOverviewParms: Option<unsafe extern "C" fn() -> *const ref_overview_s>,
    pub GetFileByIndex: Option<unsafe extern "C" fn(fileindex: c_int) -> *const c_char>,
    pub pfnSaveFile: Option<
        unsafe extern "C" fn(filename: *const c_char, data: *const c_void, len: c_int) -> c_int,
    >,
    pub R_Reserved0: Option<unsafe extern "C" fn()>,
    pub pfnMemAlloc: Option<
        unsafe extern "C" fn(cb: usize, filename: *const c_char, fileline: c_int) -> *mut c_void,
    >,
    pub pfnMemFree:
        Option<unsafe extern "C" fn(mem: *mut c_void, filename: *const c_char, fileline: c_int)>,
    pub pfnGetFilesList: Option<
        unsafe extern "C" fn(
            pattern: *const c_char,
            numFiles: *mut c_int,
            gamedironly: c_int,
        ) -> *mut *mut c_char,
    >,
    pub pfnFileBufferCRC32:
        Option<unsafe extern "C" fn(buffer: *const c_void, length: c_int) -> c_uint>,
    pub COM_CompareFileTime: Option<
        unsafe extern "C" fn(
            filename1: *const c_char,
            filename2: *const c_char,
            iCompare: *mut c_int,
        ) -> c_int,
    >,
    pub Host_Error: Option<unsafe extern "C" fn(error: *const c_char, ...)>,
    pub pfnGetModel: Option<unsafe extern "C" fn(modelindex: c_int) -> *mut c_void>,
    pub pfnTime: Option<unsafe extern "C" fn() -> f32>,
    pub Cvar_Set: Option<unsafe extern "C" fn(name: *const c_char, value: *const c_char)>,
    pub S_FadeMusicVolume: Option<unsafe extern "C" fn(fadePercent: f32)>,
    pub SetRandomSeed: Option<unsafe extern "C" fn(lSeed: c_int)>,
}

#[repr(C)]
pub struct render_interface_s {
    pub version: c_int,
    pub GL_RenderFrame: Option<unsafe extern "C" fn(rvp: *const ref_viewpass_s) -> c_int>,
    pub GL_BuildLightmaps: Option<unsafe extern "C" fn()>,
    pub GL_OrthoBounds: Option<unsafe extern "C" fn(mins: *const f32, maxs: *const f32)>,
    pub R_CreateStudioDecalList:
        Option<unsafe extern "C" fn(pList: *mut decallist_s, count: c_int) -> c_int>,
    pub R_ClearStudioDecals: Option<unsafe extern "C" fn()>,
    pub R_SpeedsMessage: Option<unsafe extern "C" fn(out: *mut c_char, size: usize) -> qboolean>,
    pub Mod_ProcessUserData:
        Option<unsafe extern "C" fn(mod_: *mut model_s, create: qboolean, buffer: *const byte)>,
    pub R_ProcessEntData: Option<unsafe extern "C" fn(allocate: qboolean)>,
    pub Mod_GetCurrentVis: Option<unsafe extern "C" fn() -> *mut byte>,
    pub R_NewMap: Option<unsafe extern "C" fn()>,
    pub R_ClearScene: Option<unsafe extern "C" fn()>,
    pub CL_UpdateLatchedVars: Option<unsafe extern "C" fn(e: *mut cl_entity_s, reset: qboolean)>,
}
pub type render_interface_t = render_interface_s;

pub const pixformat_t_PF_UNKNOWN: pixformat_t = 0;
pub const pixformat_t_PF_INDEXED_24: pixformat_t = 1;
pub const pixformat_t_PF_INDEXED_32: pixformat_t = 2;
pub const pixformat_t_PF_RGBA_32: pixformat_t = 3;
pub const pixformat_t_PF_BGRA_32: pixformat_t = 4;
pub const pixformat_t_PF_RGB_24: pixformat_t = 5;
pub const pixformat_t_PF_BGR_24: pixformat_t = 6;
pub const pixformat_t_PF_LUMINANCE: pixformat_t = 7;
pub const pixformat_t_PF_DXT1: pixformat_t = 8;
pub const pixformat_t_PF_DXT3: pixformat_t = 9;
pub const pixformat_t_PF_DXT5: pixformat_t = 10;
pub const pixformat_t_PF_ATI2: pixformat_t = 11;
pub const pixformat_t_PF_BC4_SIGNED: pixformat_t = 12;
pub const pixformat_t_PF_BC4_UNSIGNED: pixformat_t = 13;
pub const pixformat_t_PF_BC5_SIGNED: pixformat_t = 14;
pub const pixformat_t_PF_BC5_UNSIGNED: pixformat_t = 15;
pub const pixformat_t_PF_BC6H_SIGNED: pixformat_t = 16;
pub const pixformat_t_PF_BC6H_UNSIGNED: pixformat_t = 17;
pub const pixformat_t_PF_BC7_UNORM: pixformat_t = 18;
pub const pixformat_t_PF_BC7_SRGB: pixformat_t = 19;
pub const pixformat_t_PF_KTX2_RAW: pixformat_t = 20;
pub const pixformat_t_PF_TOTALCOUNT: pixformat_t = 21;
pub type pixformat_t = c_uint;

#[repr(C)]
pub struct bpc_desc_s {
    pub format: c_int,
    pub name: [c_char; 16],
    pub glFormat: c_uint,
    pub bpp: c_int,
}
pub type bpc_desc_t = bpc_desc_s;

pub const ilFlags_t_IL_USE_LERPING: ilFlags_t = 1;
pub const ilFlags_t_IL_KEEP_8BIT: ilFlags_t = 2;
pub const ilFlags_t_IL_ALLOW_OVERWRITE: ilFlags_t = 4;
pub const ilFlags_t_IL_DONTFLIP_TGA: ilFlags_t = 8;
pub const ilFlags_t_IL_DDS_HARDWARE: ilFlags_t = 16;
pub const ilFlags_t_IL_LOAD_DECAL: ilFlags_t = 32;
pub const ilFlags_t_IL_OVERVIEW: ilFlags_t = 64;
pub const ilFlags_t_IL_LOAD_PLAYER_DECAL: ilFlags_t = 128;
pub const ilFlags_t_IL_KTX2_RAW: ilFlags_t = 256;
pub type ilFlags_t = c_uint;

pub const imgFlags_t_IMAGE_CUBEMAP: imgFlags_t = 1;
pub const imgFlags_t_IMAGE_HAS_ALPHA: imgFlags_t = 2;
pub const imgFlags_t_IMAGE_HAS_COLOR: imgFlags_t = 4;
pub const imgFlags_t_IMAGE_COLORINDEX: imgFlags_t = 8;
pub const imgFlags_t_IMAGE_HAS_LUMA: imgFlags_t = 16;
pub const imgFlags_t_IMAGE_SKYBOX: imgFlags_t = 32;
pub const imgFlags_t_IMAGE_QUAKESKY: imgFlags_t = 64;
pub const imgFlags_t_IMAGE_DDS_FORMAT: imgFlags_t = 128;
pub const imgFlags_t_IMAGE_MULTILAYER: imgFlags_t = 256;
pub const imgFlags_t_IMAGE_ONEBIT_ALPHA: imgFlags_t = 512;
pub const imgFlags_t_IMAGE_QUAKEPAL: imgFlags_t = 1024;
pub const imgFlags_t_IMAGE_FLIP_X: imgFlags_t = 65536;
pub const imgFlags_t_IMAGE_FLIP_Y: imgFlags_t = 131072;
pub const imgFlags_t_IMAGE_ROT_90: imgFlags_t = 262144;
pub const imgFlags_t_IMAGE_ROT180: imgFlags_t = 196608;
pub const imgFlags_t_IMAGE_ROT270: imgFlags_t = 458752;
pub const imgFlags_t_IMAGE_RESAMPLE: imgFlags_t = 1048576;
pub const imgFlags_t_IMAGE_FORCE_RGBA: imgFlags_t = 8388608;
pub const imgFlags_t_IMAGE_MAKE_LUMA: imgFlags_t = 16777216;
pub const imgFlags_t_IMAGE_QUANTIZE: imgFlags_t = 33554432;
pub const imgFlags_t_IMAGE_LIGHTGAMMA: imgFlags_t = 67108864;
pub const imgFlags_t_IMAGE_REMAP: imgFlags_t = 134217728;
pub type imgFlags_t = c_uint;

#[repr(C)]
pub struct rgbdata_s {
    pub width: word,
    pub height: word,
    pub depth: word,
    pub type_: c_uint,
    pub flags: c_uint,
    pub encode: word,
    pub numMips: byte,
    pub palette: *mut byte,
    pub buffer: *mut byte,
    pub fogParams: rgba_t,
    pub size: usize,
}
pub type rgbdata_t = rgbdata_s;

#[repr(C)]
pub struct studiohdr_s {
    pub ident: i32,
    pub version: i32,
    pub name: [c_char; 64],
    pub length: i32,
    pub eyeposition: vec3_t,
    pub min: vec3_t,
    pub max: vec3_t,
    pub bbmin: vec3_t,
    pub bbmax: vec3_t,
    pub flags: i32,
    pub numbones: i32,
    pub boneindex: i32,
    pub numbonecontrollers: i32,
    pub bonecontrollerindex: i32,
    pub numhitboxes: i32,
    pub hitboxindex: i32,
    pub numseq: i32,
    pub seqindex: i32,
    pub numseqgroups: i32,
    pub seqgroupindex: i32,
    pub numtextures: i32,
    pub textureindex: i32,
    pub texturedataindex: i32,
    pub numskinref: i32,
    pub numskinfamilies: i32,
    pub skinindex: i32,
    pub numbodyparts: i32,
    pub bodypartindex: i32,
    pub numattachments: i32,
    pub attachmentindex: i32,
    pub studiohdr2index: i32,
    pub unused: i32,
    pub unused2: i32,
    pub unused3: i32,
    pub numtransitions: i32,
    pub transitionindex: i32,
}
pub type studiohdr_t = studiohdr_s;

#[repr(C)]
pub struct studiohdr2_t {
    pub numposeparameters: i32,
    pub poseparamindex: i32,
    pub numikautoplaylocks: i32,
    pub ikautoplaylockindex: i32,
    pub numikchains: i32,
    pub ikchainindex: i32,
    pub keyvalueindex: i32,
    pub keyvaluesize: i32,
    pub numhitboxsets: i32,
    pub hitboxsetindex: i32,
    pub unused: [i32; 6],
}

#[repr(C)]
pub struct studioseqhdr_t {
    pub id: i32,
    pub version: i32,
    pub name: [c_char; 64],
    pub length: i32,
}

#[repr(C)]
pub struct mstudiobone_s {
    pub name: [c_char; 32],
    pub parent: i32,
    pub unused: i32,
    pub bonecontroller: [i32; 6],
    pub value: [vec_t; 6],
    pub scale: [vec_t; 6],
}
pub type mstudiobone_t = mstudiobone_s;

#[repr(C)]
pub struct mstudioaxisinterpbone_t {
    pub control: i32,
    pub axis: i32,
    pub pos: [vec3_t; 6],
    pub quat: [vec4_t; 6],
}

#[repr(C)]
pub struct mstudioquatinterpinfo_t {
    pub inv_tolerance: vec_t,
    pub trigger: vec4_t,
    pub pos: vec3_t,
    pub quat: vec4_t,
}

#[repr(C)]
pub struct mstudioquatinterpbone_t {
    pub control: i32,
    pub numtriggers: i32,
    pub triggerindex: i32,
}

#[repr(C)]
pub struct mstudioboneinfo_t {
    pub poseToBone: [[vec_t; 4]; 3],
    pub qAlignment: vec4_t,
    pub proctype: i32,
    pub procindex: i32,
    pub quat: vec4_t,
    pub reserved: [i32; 10],
}

#[repr(C)]
pub struct mstudiojigglebone_t {
    pub flags: i32,
    pub length: vec_t,
    pub tipMass: vec_t,
    pub yawStiffness: vec_t,
    pub yawDamping: vec_t,
    pub pitchStiffness: vec_t,
    pub pitchDamping: vec_t,
    pub alongStiffness: vec_t,
    pub alongDamping: vec_t,
    pub angleLimit: vec_t,
    pub minYaw: vec_t,
    pub maxYaw: vec_t,
    pub yawFriction: vec_t,
    pub yawBounce: vec_t,
    pub minPitch: vec_t,
    pub maxPitch: vec_t,
    pub pitchFriction: vec_t,
    pub pitchBounce: vec_t,
    pub baseMass: vec_t,
    pub baseStiffness: vec_t,
    pub baseDamping: vec_t,
    pub baseMinLeft: vec_t,
    pub baseMaxLeft: vec_t,
    pub baseLeftFriction: vec_t,
    pub baseMinUp: vec_t,
    pub baseMaxUp: vec_t,
    pub baseUpFriction: vec_t,
    pub baseMinForward: vec_t,
    pub baseMaxForward: vec_t,
    pub baseForwardFriction: vec_t,
    pub boingImpactSpeed: vec_t,
    pub boingImpactAngle: vec_t,
    pub boingDampingRate: vec_t,
    pub boingFrequency: vec_t,
    pub boingAmplitude: vec_t,
}

#[repr(C)]
pub struct mstudioaimatbone_t {
    pub parent: i32,
    pub aim: i32,
    pub aimvector: vec3_t,
    pub upvector: vec3_t,
    pub basepos: vec3_t,
}

#[repr(C)]
pub struct mstudiobonecontroller_t {
    pub bone: i32,
    pub type_: i32,
    pub start: vec_t,
    pub end: vec_t,
    pub unused: i32,
    pub index: i32,
}

#[repr(C)]
pub struct mstudiobbox_t {
    pub bone: i32,
    pub group: i32,
    pub bbmin: vec3_t,
    pub bbmax: vec3_t,
}

#[repr(C)]
pub struct mstudiohitboxset_t {
    pub name: [c_char; 32],
    pub numhitboxes: i32,
    pub hitboxindex: i32,
}

#[repr(C)]
pub struct mstudioseqgroup_t {
    pub label: [c_char; 32],
    pub name: [c_char; 64],
    pub unused: i32,
    pub unused2: i32,
}

#[repr(C)]
pub struct mstudioattachment_t {
    pub unused: [c_char; 32],
    pub flags: i32,
    pub bone: i32,
    pub org: vec3_t,
    pub vectors: [vec3_t; 3],
}

#[repr(C)]
pub struct mstudioikerror_t {
    pub scale: [vec_t; 6],
    pub offset: [u16; 6],
}

#[repr(C)]
pub struct mstudioikrule_t {
    pub index: i32,
    pub type_: i32,
    pub chain: i32,
    pub bone: i32,
    pub attachment: i32,
    pub slot: i32,
    pub height: vec_t,
    pub radius: vec_t,
    pub floor: vec_t,
    pub pos: vec3_t,
    pub quat: vec4_t,
    pub ikerrorindex: i32,
    pub iStart: i32,
    pub start: vec_t,
    pub peak: vec_t,
    pub tail: vec_t,
    pub end: vec_t,
    pub contact: vec_t,
    pub drop: vec_t,
    pub top: vec_t,
    pub unused: [i32; 4],
}

#[repr(C)]
pub struct mstudioiklock_t {
    pub chain: i32,
    pub flPosWeight: vec_t,
    pub flLocalQWeight: vec_t,
    pub flags: i32,
    pub unused: [i32; 4],
}

#[repr(C)]
pub struct mstudiomovement_t {
    pub endframe: i32,
    pub motionflags: i32,
    pub v0: vec_t,
    pub v1: vec_t,
    pub angle: vec_t,
    pub vector: vec3_t,
    pub position: vec3_t,
}

#[repr(C)]
pub struct mstudioanimdesc_t {
    pub label: [c_char; 32],
    pub fps: vec_t,
    pub flags: i32,
    pub numframes: i32,
    pub nummovements: i32,
    pub movementindex: i32,
    pub numikrules: i32,
    pub ikruleindex: i32,
    pub unused: [i32; 8],
}

#[repr(C)]
pub struct mstudioautolayer_t {
    pub iSequence: i16,
    pub iPose: i16,
    pub flags: i32,
    pub start: vec_t,
    pub peak: vec_t,
    pub tail: vec_t,
    pub end: vec_t,
}

#[repr(C)]
pub struct mstudioseqdesc_s {
    pub label: [c_char; 32],
    pub fps: vec_t,
    pub flags: i32,
    pub activity: i32,
    pub actweight: i32,
    pub numevents: i32,
    pub eventindex: i32,
    pub numframes: i32,
    pub weightlistindex: i32,
    pub iklockindex: i32,
    pub motiontype: i32,
    pub motionbone: i32,
    pub linearmovement: vec3_t,
    pub autolayerindex: i32,
    pub keyvalueindex: i32,
    pub bbmin: vec3_t,
    pub bbmax: vec3_t,
    pub numblends: i32,
    pub animindex: i32,
    pub blendtype: [i32; 2],
    pub blendstart: [vec_t; 2],
    pub blendend: [vec_t; 2],
    pub groupsize: [u8; 2],
    pub numautolayers: u8,
    pub numiklocks: u8,
    pub seqgroup: i32,
    pub entrynode: i32,
    pub exitnode: i32,
    pub nodeflags: u8,
    pub cycleposeindex: u8,
    pub fadeintime: u8,
    pub fadeouttime: u8,
    pub animdescindex: i32,
}
pub type mstudioseqdesc_t = mstudioseqdesc_s;

#[repr(C)]
pub struct mstudioposeparamdesc_t {
    pub name: [c_char; 32],
    pub flags: i32,
    pub start: vec_t,
    pub end: vec_t,
    pub loop_: vec_t,
}

#[repr(C)]
pub struct mstudioanim_s {
    pub offset: [u16; 6],
}
pub type mstudioanim_t = mstudioanim_s;

#[repr(C)]
pub union mstudioanimvalue_t {
    pub num: mstudioanimvalue_t_num,
    pub value: i16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mstudioanimvalue_t_num {
    pub valid: u8,
    pub total: u8,
}

#[repr(C)]
pub struct mstudiobodyparts_t {
    pub name: [c_char; 64],
    pub nummodels: i32,
    pub base: i32,
    pub modelindex: i32,
}

#[repr(C)]
pub struct mstudiotex_s {
    pub name: [c_char; 64],
    pub flags: u32,
    pub width: i32,
    pub height: i32,
    pub index: i32,
}

#[repr(C)]
pub struct mstudioiklink_t {
    pub bone: i32,
    pub kneeDir: vec3_t,
    pub unused0: vec3_t,
}

#[repr(C)]
pub struct mstudioikchain_t {
    pub name: [c_char; 32],
    pub linktype: i32,
    pub numlinks: i32,
    pub linkindex: i32,
}

#[repr(C)]
pub struct mstudioboneweight_t {
    pub weight: [u8; 4],
    pub bone: [i8; 4],
}

#[repr(C)]
pub struct mstudiomodel_t {
    pub name: [c_char; 64],
    pub unused: i32,
    pub unused2: vec_t,
    pub nummesh: i32,
    pub meshindex: i32,
    pub numverts: i32,
    pub vertinfoindex: i32,
    pub vertindex: i32,
    pub numnorms: i32,
    pub norminfoindex: i32,
    pub normindex: i32,
    pub blendvertinfoindex: i32,
    pub blendnorminfoindex: i32,
}

#[repr(C)]
pub struct mstudiomesh_t {
    pub numtris: i32,
    pub triindex: i32,
    pub skinref: i32,
    pub numnorms: i32,
    pub unused: i32,
}

#[repr(C)]
pub struct mstudiotrivert_t {
    pub vertindex: i16,
    pub normindex: i16,
    pub s: i16,
    pub t: i16,
}

// extern "C" {
//     pub static svc_strings: [*const c_char; 60];
// }
// extern "C" {
//     pub static svc_legacy_strings: [*const c_char; 60];
// }
// extern "C" {
//     pub static svc_quake_strings: [*const c_char; 60];
// }
// extern "C" {
//     pub static svc_goldsrc_strings: [*const c_char; 60];
// }
// extern "C" {
//     pub static clc_strings: [*const c_char; 12];
// }

#[repr(C)]
pub struct sortedface_s {
    pub surf: *mut msurface_s,
    pub cull: c_int,
}

#[repr(C)]
pub struct ref_globals_s {
    pub developer: qboolean,
    pub width: c_int,
    pub height: c_int,
    pub fullScreen: qboolean,
    pub wideScreen: qboolean,
    pub vieworg: vec3_t,
    pub viewangles: vec3_t,
    pub draw_surfaces: *mut sortedface_s,
    pub max_surfaces: c_int,
    pub visbytes: usize,
    pub desktopBitsPixel: c_int,
}

#[repr(C)]
pub struct ref_client_s {
    pub time: f64,
    pub oldtime: f64,
    pub viewentity: c_int,
    pub playernum: c_int,
    pub maxclients: c_int,
    pub nummodels: c_int,
    pub models: [*mut model_s; MAX_MODELS + 1],
    pub paused: qboolean,
    pub simorg: vec3_t,
}

#[repr(C)]
pub struct ref_host_s {
    pub realtime: f64,
    pub frametime: f64,
    pub features: c_int,
}

pub const GL_KEEP_UNIT: _bindgen_ty_10 = -1;
pub const XASH_TEXTURE0: _bindgen_ty_10 = 0;
pub const XASH_TEXTURE1: _bindgen_ty_10 = 1;
pub const XASH_TEXTURE2: _bindgen_ty_10 = 2;
pub const XASH_TEXTURE3: _bindgen_ty_10 = 3;
pub const MAX_TEXTURE_UNITS: _bindgen_ty_10 = 32;
pub type _bindgen_ty_10 = c_int;

// r_speeds counters
pub const RS_ACTIVE_TENTS: _bindgen_ty_11 = 0;
pub type _bindgen_ty_11 = c_uint;

// refdll must expose this default textures using this names
pub const REF_DEFAULT_TEXTURE: &CStr = c"*default";
pub const REF_GRAY_TEXTURE: &CStr = c"*gray";
pub const REF_WHITE_TEXTURE: &CStr = c"*white";
pub const REF_BLACK_TEXTURE: &CStr = c"*black";
pub const REF_PARTICLE_TEXTURE: &CStr = c"*particle";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum connstate_e {
    /// Not talking to a server.
    Disconnected = 0,
    /// Sending request packets to the server.
    Connecting,
    /// netchan_t established, waiting for svc_serverdata.
    Connected,
    /// Download resources, validating, auth on server.
    Validate,
    /// Game views should be displayed.
    Active,
    /// Playing a cinematic, not connected to a server.
    Cinematic,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum ref_defaultsprite_e {
    DotSprite = 0,
    ChromeSprite = 1,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum ref_graphic_apis_e {
    // hypothetical: just make a surface to draw on, in software
    Software,
    // create GL context
    OpenGL,
    // Direct3D
    Direct3D,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum ref_safegl_context_t {
    SAFE_NO = 0,
    /// Skip MSAA.
    SAFE_NOMSAA,
    /// Don't set acceleration flag.
    SAFE_NOACC,
    /// Don't set stencil bits.
    SAFE_NOSTENCIL,
    /// Don't set alpha bits.
    SAFE_NOALPHA,
    /// Don't set depth bits.
    SAFE_NODEPTH,
    /// Don't set color bits.
    SAFE_NOCOLOR,
    /// Ignore everything, let SDL/EGL decide.
    SAFE_DONTCARE,
    /// Must be last.
    SAFE_LAST,
}

/// OpenGL configuration attributes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum GlConfigAttritube {
    RED_SIZE,
    GREEN_SIZE,
    BLUE_SIZE,
    ALPHA_SIZE,
    DOUBLEBUFFER,
    DEPTH_SIZE,
    STENCIL_SIZE,
    MULTISAMPLEBUFFERS,
    MULTISAMPLESAMPLES,
    ACCELERATED_VISUAL,
    CONTEXT_MAJOR_VERSION,
    CONTEXT_MINOR_VERSION,
    CONTEXT_EGL,
    CONTEXT_FLAGS,
    CONTEXT_PROFILE_MASK,
    SHARE_WITH_CURRENT_CONTEXT,
    FRAMEBUFFER_SRGB_CAPABLE,
    CONTEXT_RELEASE_BEHAVIOR,
    CONTEXT_RESET_NOTIFICATION,
    CONTEXT_NO_ERROR,
    ATTRIBUTES_COUNT,
}

pub const REF_GL_CONTEXT_PROFILE_CORE: GlContextProfile = 0x01;
pub const REF_GL_CONTEXT_PROFILE_COMPATIBILITY: GlContextProfile = 0x02;
pub const REF_GL_CONTEXT_PROFILE_ES: GlContextProfile = 0x04;
pub type GlContextProfile = c_uint;

pub const REF_GL_CONTEXT_DEBUG_FLAG: GlContext = 0x01;
pub const REF_GL_CONTEXT_FORWARD_COMPATIBLE_FLAG: GlContext = 0x02;
pub const REF_GL_CONTEXT_ROBUST_ACCESS_FLAG: GlContext = 0x04;
pub const REF_GL_CONTEXT_RESET_ISOLATION_FLAG: GlContext = 0x08;
/// Binary compatible with SDL and EGL_KHR_create_context(0x0007 mask).
pub type GlContext = c_uint;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(C)]
pub enum ref_screen_rotation_t {
    NONE = 0,
    CW = 1,
    UD = 2,
    CCW = 3,
}

#[repr(C)]
pub struct remap_info_s {
    /// Alias textures.
    pub textures: [c_ushort; MAX_SKINS],
    /// Array of textures with local copy of remapped textures.
    pub ptexture: *mut mstudiotex_s,
    /// Textures count.
    pub numtextures: c_short,
    /// Cached value.
    pub topcolor: c_short,
    /// Cached value.
    pub bottomcolor: c_short,
    /// For catch model changes.
    pub model: *mut model_s,
}

#[repr(C)]
pub struct convar_s {
    _unused: [u8; 0],
}

pub const PARM_DEV_OVERVIEW: RefParm = RefParm::new(-1);
pub const PARM_THIRDPERSON: RefParm = RefParm::new(-2);
pub const PARM_QUAKE_COMPATIBLE: RefParm = RefParm::new(-3);
pub const PARM_GET_CLIENT_PTR: RefParm = RefParm::new(-4);
pub const PARM_GET_HOST_PTR: RefParm = RefParm::new(-5);
pub const PARM_CONNSTATE: RefParm = RefParm::new(-6);
pub const PARM_PLAYING_DEMO: RefParm = RefParm::new(-7);
pub const PARM_WATER_LEVEL: RefParm = RefParm::new(-8);
pub const PARM_GET_WORLD_PTR: RefParm = RefParm::new(-9);
pub const PARM_LOCAL_HEALTH: RefParm = RefParm::new(-10);
pub const PARM_LOCAL_GAME: RefParm = RefParm::new(-11);
pub const PARM_NUMENTITIES: RefParm = RefParm::new(-12);
pub const PARM_GET_MOVEVARS_PTR: RefParm = RefParm::new(-13);
pub const PARM_GET_PALETTE_PTR: RefParm = RefParm::new(-14);
pub const PARM_GET_VIEWENT_PTR: RefParm = RefParm::new(-15);
pub const PARM_GET_TEXGAMMATABLE_PTR: RefParm = RefParm::new(-16);
pub const PARM_GET_LIGHTGAMMATABLE_PTR: RefParm = RefParm::new(-17);
pub const PARM_GET_SCREENGAMMATABLE_PTR: RefParm = RefParm::new(-18);
pub const PARM_GET_LINEARGAMMATABLE_PTR: RefParm = RefParm::new(-19);
pub const PARM_GET_LIGHTSTYLES_PTR: RefParm = RefParm::new(-20);
pub const PARM_GET_DLIGHTS_PTR: RefParm = RefParm::new(-21);
pub const PARM_GET_ELIGHTS_PTR: RefParm = RefParm::new(-22);

/// Returns non-null integer if filtering is enabled for texture.
///
/// Pass -1 to query global filtering settings.
pub const PARM_TEX_FILTERING: RefParm = RefParm::new(-0x10000);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ref_api_s {
    pub EngineGetParm: Option<unsafe extern "C" fn(parm: c_int, arg: c_int) -> isize>,
    pub Cvar_Get: Option<
        unsafe extern "C" fn(
            szName: *const c_char,
            szValue: *const c_char,
            flags: c_int,
            description: *const c_char,
        ) -> *mut cvar_s,
    >,
    pub pfnGetCvarPointer:
        Option<unsafe extern "C" fn(name: *const c_char, ignore_flags: c_int) -> *mut cvar_s>,
    pub pfnGetCvarFloat: Option<unsafe extern "C" fn(szName: *const c_char) -> f32>,
    pub pfnGetCvarString: Option<unsafe extern "C" fn(szName: *const c_char) -> *const c_char>,
    pub Cvar_SetValue: Option<unsafe extern "C" fn(name: *const c_char, value: f32)>,
    pub Cvar_Set: Option<unsafe extern "C" fn(name: *const c_char, value: *const c_char)>,
    pub Cvar_RegisterVariable: Option<unsafe extern "C" fn(var: *mut convar_s)>,
    pub Cvar_FullSet:
        Option<unsafe extern "C" fn(var_name: *const c_char, value: *const c_char, flags: c_int)>,
    pub Cmd_AddCommand: Option<
        unsafe extern "C" fn(
            cmd_name: *const c_char,
            function: Option<unsafe extern "C" fn()>,
            description: *const c_char,
        ) -> c_int,
    >,
    pub Cmd_RemoveCommand: Option<unsafe extern "C" fn(cmd_name: *const c_char)>,
    pub Cmd_Argc: Option<unsafe extern "C" fn() -> c_int>,
    pub Cmd_Argv: Option<unsafe extern "C" fn(arg: c_int) -> *const c_char>,
    pub Cmd_Args: Option<unsafe extern "C" fn() -> *const c_char>,
    pub Cbuf_AddText: Option<unsafe extern "C" fn(commands: *const c_char)>,
    pub Cbuf_InsertText: Option<unsafe extern "C" fn(commands: *const c_char)>,
    pub Cbuf_Execute: Option<unsafe extern "C" fn()>,
    pub Con_Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_DPrintf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_Reportf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_NPrintf: Option<unsafe extern "C" fn(pos: c_int, fmt: *const c_char, ...)>,
    pub Con_NXPrintf:
        Option<unsafe extern "C" fn(info: *mut con_nprint_s, fmt: *const c_char, ...)>,
    pub CL_CenterPrint: Option<unsafe extern "C" fn(s: *const c_char, y: f32)>,
    pub Con_DrawStringLen:
        Option<unsafe extern "C" fn(pText: *const c_char, length: *mut c_int, height: *mut c_int)>,
    pub Con_DrawString: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            string: *const c_char,
            setColor: *const rgba_t,
        ) -> c_int,
    >,
    pub CL_DrawCenterPrint: Option<unsafe extern "C" fn()>,
    pub R_BeamGetEntity: Option<unsafe extern "C" fn(index: c_int) -> *mut cl_entity_s>,
    pub CL_GetWaterEntity: Option<unsafe extern "C" fn(p: *const vec3_t) -> *mut cl_entity_s>,
    pub CL_AddVisibleEntity:
        Option<unsafe extern "C" fn(ent: *mut cl_entity_s, entityType: c_int) -> qboolean>,
    pub Mod_SampleSizeForFace: Option<unsafe extern "C" fn(surf: *const msurface_s) -> c_int>,
    pub Mod_BoxVisible: Option<
        unsafe extern "C" fn(
            mins: *const vec3_t,
            maxs: *const vec3_t,
            visbits: *const byte,
        ) -> qboolean,
    >,
    pub Mod_PointInLeaf:
        Option<unsafe extern "C" fn(p: *const vec3_t, node: *mut mnode_s) -> *mut mleaf_s>,
    pub R_DrawWorldHull: Option<unsafe extern "C" fn()>,
    pub R_DrawModelHull: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    pub R_StudioGetAnim: Option<
        unsafe extern "C" fn(
            m_pStudioHeader: *mut studiohdr_s,
            m_pSubModel: *mut model_s,
            pseqdesc: *mut mstudioseqdesc_t,
        ) -> *mut c_void,
    >,
    pub pfnStudioEvent:
        Option<unsafe extern "C" fn(event: *const mstudioevent_s, entity: *const cl_entity_s)>,
    pub CL_DrawEFX: Option<unsafe extern "C" fn(time: f32, fTrans: qboolean)>,
    pub CL_ThinkParticle: Option<unsafe extern "C" fn(frametime: f64, p: *mut particle_s)>,
    pub R_FreeDeadParticles: Option<unsafe extern "C" fn(ppparticles: *mut *mut particle_s)>,
    pub CL_AllocParticleFast: Option<unsafe extern "C" fn() -> *mut particle_s>,
    pub CL_AllocElight: Option<unsafe extern "C" fn(key: c_int) -> *mut dlight_s>,
    pub GetDefaultSprite: Option<unsafe extern "C" fn(spr: ref_defaultsprite_e) -> *mut model_s>,
    pub R_StoreEfrags: Option<unsafe extern "C" fn(ppefrag: *mut *mut efrag_s, framecount: c_int)>,
    pub Mod_ForName: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            crash: qboolean,
            trackCRC: qboolean,
        ) -> *mut model_s,
    >,
    pub Mod_Extradata:
        Option<unsafe extern "C" fn(type_: c_int, model: *mut model_s) -> *mut c_void>,
    pub CL_EntitySetRemapColors: Option<
        unsafe extern "C" fn(
            e: *mut cl_entity_s,
            mod_: *mut model_s,
            top: c_int,
            bottom: c_int,
        ) -> qboolean,
    >,
    pub CL_GetRemapInfoForEntity:
        Option<unsafe extern "C" fn(e: *mut cl_entity_s) -> *mut remap_info_s>,
    pub CL_ExtraUpdate: Option<unsafe extern "C" fn()>,
    pub Host_Error: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub COM_SetRandomSeed: Option<unsafe extern "C" fn(lSeed: c_int)>,
    pub COM_RandomFloat: Option<unsafe extern "C" fn(rmin: f32, rmax: f32) -> f32>,
    pub COM_RandomLong: Option<unsafe extern "C" fn(rmin: c_int, rmax: c_int) -> c_int>,
    pub GetScreenFade: Option<unsafe extern "C" fn() -> *mut screenfade_s>,
    pub CL_GetScreenInfo: Option<unsafe extern "C" fn(width: *mut c_int, height: *mut c_int)>,
    pub SetLocalLightLevel: Option<unsafe extern "C" fn(level: c_int)>,
    pub Sys_CheckParm: Option<unsafe extern "C" fn(flag: *const c_char) -> c_int>,
    pub pfnPlayerInfo: Option<unsafe extern "C" fn(index: c_int) -> *mut player_info_s>,
    pub pfnGetPlayerState: Option<unsafe extern "C" fn(index: c_int) -> *mut entity_state_s>,
    pub Mod_CacheCheck: Option<unsafe extern "C" fn(c: *mut cache_user_s) -> *mut c_void>,
    pub Mod_LoadCacheFile: Option<unsafe extern "C" fn(path: *const c_char, cu: *mut cache_user_s)>,
    pub Mod_Calloc: Option<unsafe extern "C" fn(number: c_int, size: usize) -> *mut c_void>,
    pub pfnGetStudioModelInterface: Option<
        unsafe extern "C" fn(
            version: c_int,
            ppinterface: *mut *mut r_studio_interface_s,
            pstudio: *mut engine_studio_api_s,
        ) -> c_int,
    >,
    pub _Mem_AllocPool: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            filename: *const c_char,
            fileline: c_int,
        ) -> poolhandle_t,
    >,
    pub _Mem_FreePool: Option<
        unsafe extern "C" fn(poolptr: *mut poolhandle_t, filename: *const c_char, fileline: c_int),
    >,
    pub _Mem_Alloc: Option<
        unsafe extern "C" fn(
            poolptr: poolhandle_t,
            size: usize,
            clear: qboolean,
            filename: *const c_char,
            fileline: c_int,
        ) -> *mut c_void,
    >,
    pub _Mem_Realloc: Option<
        unsafe extern "C" fn(
            poolptr: poolhandle_t,
            memptr: *mut c_void,
            size: usize,
            clear: qboolean,
            filename: *const c_char,
            fileline: c_int,
        ) -> *mut c_void,
    >,
    pub _Mem_Free:
        Option<unsafe extern "C" fn(data: *mut c_void, filename: *const c_char, fileline: c_int)>,
    pub COM_LoadLibrary: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            build_ordinals_table: c_int,
            directpath: qboolean,
        ) -> *mut c_void,
    >,
    pub COM_FreeLibrary: Option<unsafe extern "C" fn(handle: *mut c_void)>,
    pub COM_GetProcAddress:
        Option<unsafe extern "C" fn(handle: *mut c_void, name: *const c_char) -> *mut c_void>,
    pub R_Init_Video: Option<unsafe extern "C" fn(type_: c_int) -> qboolean>,
    pub R_Free_Video: Option<unsafe extern "C" fn()>,
    pub GL_SetAttribute: Option<unsafe extern "C" fn(attr: c_int, value: c_int) -> c_int>,
    pub GL_GetAttribute: Option<unsafe extern "C" fn(attr: c_int, value: *mut c_int) -> c_int>,
    pub GL_GetProcAddress: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,
    pub GL_SwapBuffers: Option<unsafe extern "C" fn()>,
    pub SW_CreateBuffer: Option<
        unsafe extern "C" fn(
            width: c_int,
            height: c_int,
            stride: *mut c_uint,
            bpp: *mut c_uint,
            r: *mut c_uint,
            g: *mut c_uint,
            b: *mut c_uint,
        ) -> qboolean,
    >,
    pub SW_LockBuffer: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub SW_UnlockBuffer: Option<unsafe extern "C" fn()>,
    pub R_FatPVS: Option<
        unsafe extern "C" fn(
            org: *const f32,
            radius: f32,
            visbuffer: *mut byte,
            merge: qboolean,
            fullvis: qboolean,
        ) -> c_int,
    >,
    pub GetOverviewParms: Option<unsafe extern "C" fn() -> *const ref_overview_s>,
    pub pfnTime: Option<unsafe extern "C" fn() -> f64>,
    pub EV_GetPhysent: Option<unsafe extern "C" fn(idx: c_int) -> *mut physent_s>,
    pub EV_TraceSurface: Option<
        unsafe extern "C" fn(ground: c_int, vstart: *mut f32, vend: *mut f32) -> *mut msurface_s,
    >,
    pub PM_TraceLine: Option<
        unsafe extern "C" fn(
            start: *mut f32,
            end: *mut f32,
            flags: c_int,
            usehull: c_int,
            ignore_pe: c_int,
        ) -> *mut pmtrace_s,
    >,
    pub EV_VisTraceLine: Option<
        unsafe extern "C" fn(start: *mut f32, end: *mut f32, flags: c_int) -> *mut pmtrace_s,
    >,
    pub CL_TraceLine: Option<
        unsafe extern "C" fn(start: *mut vec3_t, end: *mut vec3_t, flags: c_int) -> pmtrace_s,
    >,
    pub Image_AddCmdFlags: Option<unsafe extern "C" fn(flags: c_uint)>,
    pub Image_SetForceFlags: Option<unsafe extern "C" fn(flags: c_uint)>,
    pub Image_ClearForceFlags: Option<unsafe extern "C" fn()>,
    pub Image_CustomPalette: Option<unsafe extern "C" fn() -> qboolean>,
    pub Image_Process: Option<
        unsafe extern "C" fn(
            pix: *mut *mut rgbdata_t,
            width: c_int,
            height: c_int,
            flags: c_uint,
            reserved: f32,
        ) -> qboolean,
    >,
    pub FS_LoadImage: Option<
        unsafe extern "C" fn(
            filename: *const c_char,
            buffer: *const byte,
            size: usize,
        ) -> *mut rgbdata_t,
    >,
    pub FS_SaveImage:
        Option<unsafe extern "C" fn(filename: *const c_char, pix: *mut rgbdata_t) -> qboolean>,
    pub FS_CopyImage: Option<unsafe extern "C" fn(in_: *mut rgbdata_t) -> *mut rgbdata_t>,
    pub FS_FreeImage: Option<unsafe extern "C" fn(pack: *mut rgbdata_t)>,
    pub Image_SetMDLPointer: Option<unsafe extern "C" fn(p: *mut byte)>,
    pub Image_GetPFDesc: Option<unsafe extern "C" fn(idx: c_int) -> *const bpc_desc_s>,
    pub pfnDrawNormalTriangles: Option<unsafe extern "C" fn()>,
    pub pfnDrawTransparentTriangles: Option<unsafe extern "C" fn()>,
    pub drawFuncs: *mut render_interface_t,
    pub fsapi: *mut self::filesystem::fs_api_t,
}
pub type ref_api_t = ref_api_s;

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct ref_interface_s {
    pub R_Init: Option<unsafe extern "C" fn() -> qboolean>,
    pub R_Shutdown: Option<unsafe extern "C" fn()>,
    pub R_GetConfigName: Option<unsafe extern "C" fn() -> *const c_char>,
    pub R_SetDisplayTransform: Option<
        unsafe extern "C" fn(
            rotate: ref_screen_rotation_t,
            x: c_int,
            y: c_int,
            scale_x: f32,
            scale_y: f32,
        ) -> qboolean,
    >,
    pub GL_SetupAttributes: Option<unsafe extern "C" fn(safegl: c_int)>,
    pub GL_InitExtensions: Option<unsafe extern "C" fn()>,
    pub GL_ClearExtensions: Option<unsafe extern "C" fn()>,
    pub R_GammaChanged: Option<unsafe extern "C" fn(do_reset_gamma: qboolean)>,
    pub R_BeginFrame: Option<unsafe extern "C" fn(clearScene: qboolean)>,
    pub R_RenderScene: Option<unsafe extern "C" fn()>,
    pub R_EndFrame: Option<unsafe extern "C" fn()>,
    pub R_PushScene: Option<unsafe extern "C" fn()>,
    pub R_PopScene: Option<unsafe extern "C" fn()>,
    pub GL_BackendStartFrame: Option<unsafe extern "C" fn()>,
    pub GL_BackendEndFrame: Option<unsafe extern "C" fn()>,
    pub R_ClearScreen: Option<unsafe extern "C" fn()>,
    pub R_AllowFog: Option<unsafe extern "C" fn(allow: qboolean)>,
    pub GL_SetRenderMode: Option<unsafe extern "C" fn(renderMode: c_int)>,
    pub R_AddEntity:
        Option<unsafe extern "C" fn(clent: *mut cl_entity_s, type_: c_int) -> qboolean>,
    pub CL_AddCustomBeam: Option<unsafe extern "C" fn(pEnvBeam: *mut cl_entity_s)>,
    pub R_ProcessEntData: Option<
        unsafe extern "C" fn(allocate: qboolean, entities: *mut cl_entity_s, max_entities: c_uint),
    >,
    pub R_Flush: Option<unsafe extern "C" fn(flush_flags: c_uint)>,
    pub R_ShowTextures: Option<unsafe extern "C" fn()>,
    pub R_GetTextureOriginalBuffer: Option<unsafe extern "C" fn(idx: c_uint) -> *const byte>,
    pub GL_LoadTextureFromBuffer: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            pic: *mut rgbdata_t,
            flags: texFlags_t,
            update: qboolean,
        ) -> c_int,
    >,
    pub GL_ProcessTexture: Option<
        unsafe extern "C" fn(texnum: c_int, gamma: f32, topColor: c_int, bottomColor: c_int),
    >,
    pub R_SetupSky: Option<unsafe extern "C" fn(skyboxTextures: *mut c_int)>,
    pub R_Set2DMode: Option<unsafe extern "C" fn(enable: qboolean)>,
    pub R_DrawStretchRaw: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            cols: c_int,
            rows: c_int,
            data: *const byte,
            dirty: qboolean,
        ),
    >,
    pub R_DrawStretchPic: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            s1: f32,
            t1: f32,
            s2: f32,
            t2: f32,
            texnum: c_int,
        ),
    >,
    pub FillRGBA: Option<
        unsafe extern "C" fn(
            rendermode: c_int,
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            r: byte,
            g: byte,
            b: byte,
            a: byte,
        ),
    >,
    pub WorldToScreen:
        Option<unsafe extern "C" fn(world: *const vec3_t, screen: *mut vec3_t) -> c_int>,
    pub VID_ScreenShot:
        Option<unsafe extern "C" fn(filename: *const c_char, shot_type: c_int) -> qboolean>,
    pub VID_CubemapShot: Option<
        unsafe extern "C" fn(
            base: *const c_char,
            size: c_uint,
            vieworg: *const f32,
            skyshot: qboolean,
        ) -> qboolean,
    >,
    pub R_LightPoint: Option<unsafe extern "C" fn(p: *const f32) -> colorVec>,
    pub R_DecalShoot: Option<
        unsafe extern "C" fn(
            textureIndex: c_int,
            entityIndex: c_int,
            modelIndex: c_int,
            pos: *mut vec3_t,
            flags: c_int,
            scale: f32,
        ),
    >,
    pub R_DecalRemoveAll: Option<unsafe extern "C" fn(texture: c_int)>,
    pub R_CreateDecalList: Option<unsafe extern "C" fn(pList: *mut decallist_s) -> c_int>,
    pub R_ClearAllDecals: Option<unsafe extern "C" fn()>,
    pub R_StudioEstimateFrame: Option<
        unsafe extern "C" fn(
            e: *mut cl_entity_s,
            pseqdesc: *mut mstudioseqdesc_t,
            time: f64,
        ) -> f32,
    >,
    pub R_StudioLerpMovement: Option<
        unsafe extern "C" fn(
            e: *mut cl_entity_s,
            time: f64,
            origin: *mut vec3_t,
            angles: *mut vec3_t,
        ),
    >,
    pub CL_InitStudioAPI: Option<unsafe extern "C" fn()>,
    pub R_SetSkyCloudsTextures:
        Option<unsafe extern "C" fn(solidskyTexture: c_int, alphaskyTexture: c_int)>,
    pub GL_SubdivideSurface: Option<unsafe extern "C" fn(mod_: *mut model_s, fa: *mut msurface_s)>,
    pub CL_RunLightStyles: Option<unsafe extern "C" fn(ls: *mut lightstyle_t)>,
    pub R_GetSpriteParms: Option<
        unsafe extern "C" fn(
            frameWidth: *mut c_int,
            frameHeight: *mut c_int,
            numFrames: *mut c_int,
            currentFrame: c_int,
            pSprite: *const model_s,
        ),
    >,
    pub R_GetSpriteTexture:
        Option<unsafe extern "C" fn(m_pSpriteModel: *const model_s, frame: c_int) -> c_int>,
    pub Mod_ProcessRenderData: Option<
        unsafe extern "C" fn(mod_: *mut model_s, create: qboolean, buffer: *const byte) -> qboolean,
    >,
    pub Mod_StudioLoadTextures: Option<unsafe extern "C" fn(mod_: *mut model_s, data: *mut c_void)>,
    pub CL_DrawParticles:
        Option<unsafe extern "C" fn(frametime: f64, particles: *mut particle_s, partsize: f32)>,
    pub CL_DrawTracers: Option<unsafe extern "C" fn(frametime: f64, tracers: *mut particle_s)>,
    pub CL_DrawBeams: Option<unsafe extern "C" fn(fTrans: c_int, beams: *mut BEAM)>,
    pub R_BeamCull: Option<
        unsafe extern "C" fn(
            start: *const vec3_t,
            end: *const vec3_t,
            pvsOnly: qboolean,
        ) -> qboolean,
    >,
    pub RefGetParm: Option<unsafe extern "C" fn(parm: c_int, arg: c_int) -> c_int>,
    pub GetDetailScaleForTexture:
        Option<unsafe extern "C" fn(texture: c_int, xScale: *mut f32, yScale: *mut f32)>,
    pub GetExtraParmsForTexture: Option<
        unsafe extern "C" fn(
            texture: c_int,
            red: *mut byte,
            green: *mut byte,
            blue: *mut byte,
            alpha: *mut byte,
        ),
    >,
    pub GetFrameTime: Option<unsafe extern "C" fn() -> f32>,
    pub R_SetCurrentEntity: Option<unsafe extern "C" fn(ent: *mut cl_entity_s)>,
    pub R_SetCurrentModel: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    pub GL_FindTexture: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    pub GL_TextureName: Option<unsafe extern "C" fn(texnum: c_uint) -> *const c_char>,
    pub GL_TextureData: Option<unsafe extern "C" fn(texnum: c_uint) -> *const byte>,
    pub GL_LoadTexture: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            buf: *const byte,
            size: usize,
            flags: c_int,
        ) -> c_int,
    >,
    pub GL_CreateTexture: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            width: c_int,
            height: c_int,
            buffer: *const c_void,
            flags: texFlags_t,
        ) -> c_int,
    >,
    pub GL_LoadTextureArray:
        Option<unsafe extern "C" fn(names: *mut *const c_char, flags: c_int) -> c_int>,
    pub GL_CreateTextureArray: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            width: c_int,
            height: c_int,
            depth: c_int,
            buffer: *const c_void,
            flags: texFlags_t,
        ) -> c_int,
    >,
    pub GL_FreeTexture: Option<unsafe extern "C" fn(texnum: c_uint)>,
    pub R_OverrideTextureSourceSize:
        Option<unsafe extern "C" fn(texnum: c_uint, srcWidth: c_uint, srcHeight: c_uint)>,
    pub DrawSingleDecal: Option<unsafe extern "C" fn(pDecal: *mut decal_s, fa: *mut msurface_s)>,
    pub R_DecalSetupVerts: Option<
        unsafe extern "C" fn(
            pDecal: *mut decal_s,
            surf: *mut msurface_s,
            texture: c_int,
            outCount: *mut c_int,
        ) -> *mut f32,
    >,
    pub R_EntityRemoveDecals: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    pub AVI_UploadRawFrame: Option<
        unsafe extern "C" fn(
            texture: c_int,
            cols: c_int,
            rows: c_int,
            width: c_int,
            height: c_int,
            data: *const byte,
        ),
    >,
    pub GL_Bind: Option<unsafe extern "C" fn(tmu: c_int, texnum: c_uint)>,
    pub GL_SelectTexture: Option<unsafe extern "C" fn(tmu: c_int)>,
    pub GL_LoadTextureMatrix: Option<unsafe extern "C" fn(glmatrix: *const f32)>,
    pub GL_TexMatrixIdentity: Option<unsafe extern "C" fn()>,
    pub GL_CleanUpTextureUnits: Option<unsafe extern "C" fn(last: c_int)>,
    pub GL_TexGen: Option<unsafe extern "C" fn(coord: c_uint, mode: c_uint)>,
    pub GL_TextureTarget: Option<unsafe extern "C" fn(target: c_uint)>,
    pub GL_TexCoordArrayMode: Option<unsafe extern "C" fn(texmode: c_uint)>,
    pub GL_UpdateTexSize:
        Option<unsafe extern "C" fn(texnum: c_int, width: c_int, height: c_int, depth: c_int)>,
    pub GL_Reserved0: Option<unsafe extern "C" fn()>,
    pub GL_Reserved1: Option<unsafe extern "C" fn()>,
    pub GL_DrawParticles: Option<
        unsafe extern "C" fn(rvp: *const ref_viewpass_s, trans_pass: qboolean, frametime: f32),
    >,
    pub LightVec: Option<
        unsafe extern "C" fn(
            start: *const f32,
            end: *const f32,
            lightspot: *mut f32,
            lightvec: *mut f32,
        ) -> colorVec,
    >,
    pub StudioGetTexture: Option<unsafe extern "C" fn(e: *mut cl_entity_s) -> *mut mstudiotex_s>,
    pub GL_RenderFrame: Option<unsafe extern "C" fn(rvp: *const ref_viewpass_s)>,
    pub GL_OrthoBounds: Option<unsafe extern "C" fn(mins: *const f32, maxs: *const f32)>,
    pub R_SpeedsMessage: Option<unsafe extern "C" fn(out: *mut c_char, size: usize) -> qboolean>,
    pub Mod_GetCurrentVis: Option<unsafe extern "C" fn() -> *mut byte>,
    pub R_NewMap: Option<unsafe extern "C" fn()>,
    pub R_ClearScene: Option<unsafe extern "C" fn()>,
    pub R_GetProcAddress: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,
    pub TriRenderMode: Option<unsafe extern "C" fn(mode: c_int)>,
    pub Begin: Option<unsafe extern "C" fn(primitiveCode: c_int)>,
    pub End: Option<unsafe extern "C" fn()>,
    pub Color4f: Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32)>,
    pub Color4ub: Option<unsafe extern "C" fn(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar)>,
    pub TexCoord2f: Option<unsafe extern "C" fn(u: f32, v: f32)>,
    pub Vertex3fv: Option<unsafe extern "C" fn(worldPnt: *const f32)>,
    pub Vertex3f: Option<unsafe extern "C" fn(x: f32, y: f32, z: f32)>,
    pub Fog: Option<
        unsafe extern "C" fn(flFogColor: *mut [f32; 3], flStart: f32, flEnd: f32, bOn: c_int),
    >,
    pub ScreenToWorld: Option<unsafe extern "C" fn(screen: *const f32, world: *mut f32)>,
    pub GetMatrix: Option<unsafe extern "C" fn(pname: c_int, matrix: *mut f32)>,
    pub FogParams: Option<unsafe extern "C" fn(flDensity: f32, iFogSkybox: c_int)>,
    pub CullFace: Option<unsafe extern "C" fn(mode: TRICULLSTYLE)>,
    pub VGUI_SetupDrawing: Option<unsafe extern "C" fn(rect: qboolean)>,
    pub VGUI_UploadTextureBlock: Option<
        unsafe extern "C" fn(
            drawX: c_int,
            drawY: c_int,
            rgba: *const byte,
            blockWidth: c_int,
            blockHeight: c_int,
        ),
    >,
}
pub type ref_interface_t = ref_interface_s;

pub const GET_REF_API: &CStr = c"GetRefAPI";

pub type REFAPI = Option<
    unsafe extern "C" fn(
        version: c_int,
        exported_funcs: &mut ref_interface_t,
        engine_funcs: &ref_api_t,
        globals: *mut ref_globals_s,
    ) -> c_int,
>;
