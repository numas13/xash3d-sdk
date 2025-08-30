#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_ushort, c_void},
    mem,
};

use bitflags::bitflags;
use csz::{CStrArray, CStrThin};

use crate::{consts::MAX_LOCAL_WEAPONS, cvar::cvar_s};

pub use shared::raw::*;

pub type HSPRITE = c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct triangleapi_s {
    pub version: c_int,
    pub RenderMode: Option<unsafe extern "C" fn(mode: c_int)>,
    pub Begin: Option<unsafe extern "C" fn(primitiveCode: c_int)>,
    pub End: Option<unsafe extern "C" fn()>,
    pub Color4f: Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32)>,
    pub Color4ub: Option<unsafe extern "C" fn(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar)>,
    pub TexCoord2f: Option<unsafe extern "C" fn(u: f32, v: f32)>,
    pub Vertex3fv: Option<unsafe extern "C" fn(worldPnt: *const f32)>,
    pub Vertex3f: Option<unsafe extern "C" fn(x: f32, y: f32, z: f32)>,
    pub Brightness: Option<unsafe extern "C" fn(brightness: f32)>,
    pub CullFace: Option<unsafe extern "C" fn(style: TRICULLSTYLE)>,
    pub SpriteTexture:
        Option<unsafe extern "C" fn(pSpriteModel: *mut model_s, frame: c_int) -> c_int>,
    pub WorldToScreen: Option<unsafe extern "C" fn(world: *const f32, screen: *mut f32) -> c_int>,
    pub Fog: Option<
        unsafe extern "C" fn(flFogColor: *mut [f32; 3usize], flStart: f32, flEnd: f32, bOn: c_int),
    >,
    pub ScreenToWorld: Option<unsafe extern "C" fn(screen: *const f32, world: *mut f32)>,
    pub GetMatrix: Option<unsafe extern "C" fn(pname: c_int, matrix: *mut f32)>,
    pub BoxInPVS: Option<unsafe extern "C" fn(mins: *mut f32, maxs: *mut f32) -> c_int>,
    pub LightAtPoint: Option<unsafe extern "C" fn(pos: *mut f32, value: *mut f32)>,
    pub Color4fRendermode:
        Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32, rendermode: c_int)>,
    pub FogParams: Option<unsafe extern "C" fn(flDensity: f32, iFogSkybox: c_int)>,
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    #[repr(transparent)]
    pub struct TempEntFlags: c_int {
        const NONE                  = 0;
        const SINEWAVE              = 1 << 0;
        const GRAVITY               = 1 << 1;
        const ROTATE                = 1 << 2;
        const SLOWGRAVITY           = 1 << 3;
        const SMOKETRAIL            = 1 << 4;
        const COLLIDEWORLD          = 1 << 5;
        const FLICKER               = 1 << 6;
        const FADEOUT               = 1 << 7;
        const SPRANIMATE            = 1 << 8;
        const HITSOUND              = 1 << 9;
        const SPIRAL                = 1 << 10;
        const SPRCYCLE              = 1 << 11;
        const COLLIDEALL            = 1 << 12;
        const PERSIST               = 1 << 13;
        const COLLIDEKILL           = 1 << 14;
        const PLYRATTACHMENT        = 1 << 15;
        const SPRANIMATELOOP        = 1 << 16;
        const SPARKSHOWER           = 1 << 17;
        const NOMODEL               = 1 << 18;
        const CLIENTCUSTOM          = 1 << 19;
        const SCALE                 = 1 << 20;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tempent_s {
    pub flags: TempEntFlags,
    pub die: f32,
    pub frameMax: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub fadeSpeed: f32,
    pub bounceFactor: f32,
    pub hitSound: c_int,
    pub hitcallback: Option<unsafe extern "C" fn(ent: *mut TEMPENTITY, ptr: *mut pmtrace_s)>,
    pub callback:
        Option<unsafe extern "C" fn(ent: *mut TEMPENTITY, frametime: f32, currenttime: f32)>,
    pub next: *mut TEMPENTITY,
    pub priority: c_int,
    pub clientIndex: c_short,
    pub tentOffset: vec3_t,
    pub entity: cl_entity_s,
}
pub type TEMPENTITY = tempent_s;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct efx_api_s {
    pub R_AllocParticle: Option<
        unsafe extern "C" fn(
            callback: Option<unsafe extern "C" fn(particle: *mut particle_s, frametime: f32)>,
        ) -> *mut particle_s,
    >,
    pub R_BlobExplosion: Option<unsafe extern "C" fn(org: *const f32)>,
    pub R_Blood:
        Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, pcolor: c_int, speed: c_int)>,
    pub R_BloodSprite: Option<
        unsafe extern "C" fn(
            org: *const f32,
            colorindex: c_int,
            modelIndex: c_int,
            modelIndex2: c_int,
            size: f32,
        ),
    >,
    pub R_BloodStream:
        Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, pcolor: c_int, speed: c_int)>,
    pub R_BreakModel: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            size: *const f32,
            dir: *const f32,
            random: f32,
            life: f32,
            count: c_int,
            modelIndex: c_int,
            flags: c_char,
        ),
    >,
    pub R_Bubbles: Option<
        unsafe extern "C" fn(
            mins: *const f32,
            maxs: *const f32,
            height: f32,
            modelIndex: c_int,
            count: c_int,
            speed: f32,
        ),
    >,
    pub R_BubbleTrail: Option<
        unsafe extern "C" fn(
            start: *const f32,
            end: *const f32,
            height: f32,
            modelIndex: c_int,
            count: c_int,
            speed: f32,
        ),
    >,
    pub R_BulletImpactParticles: Option<unsafe extern "C" fn(pos: *const f32)>,
    pub R_EntityParticles: Option<unsafe extern "C" fn(ent: *mut cl_entity_s)>,
    pub R_Explosion: Option<
        unsafe extern "C" fn(pos: *mut f32, model: c_int, scale: f32, framerate: f32, flags: c_int),
    >,
    pub R_FizzEffect:
        Option<unsafe extern "C" fn(pent: *mut cl_entity_s, modelIndex: c_int, density: c_int)>,
    pub R_FireField: Option<
        unsafe extern "C" fn(
            org: *mut f32,
            radius: c_int,
            modelIndex: c_int,
            count: c_int,
            flags: c_int,
            life: f32,
        ),
    >,
    pub R_FlickerParticles: Option<unsafe extern "C" fn(org: *const f32)>,
    pub R_FunnelSprite:
        Option<unsafe extern "C" fn(org: *const f32, modelIndex: c_int, reverse: c_int)>,
    pub R_Implosion:
        Option<unsafe extern "C" fn(end: *const f32, radius: f32, count: c_int, life: f32)>,
    pub R_LargeFunnel: Option<unsafe extern "C" fn(org: *const f32, reverse: c_int)>,
    pub R_LavaSplash: Option<unsafe extern "C" fn(org: *const f32)>,
    pub R_MultiGunshot: Option<
        unsafe extern "C" fn(
            org: *const f32,
            dir: *const f32,
            noise: *const f32,
            count: c_int,
            decalCount: c_int,
            decalIndices: *mut c_int,
        ),
    >,
    pub R_MuzzleFlash: Option<unsafe extern "C" fn(pos1: *const f32, type_: c_int)>,
    pub R_ParticleBox: Option<
        unsafe extern "C" fn(
            mins: *const f32,
            maxs: *const f32,
            r: c_uchar,
            g: c_uchar,
            b: c_uchar,
            life: f32,
        ),
    >,
    pub R_ParticleBurst:
        Option<unsafe extern "C" fn(pos: *const f32, size: c_int, color: c_int, life: f32)>,
    pub R_ParticleExplosion: Option<unsafe extern "C" fn(org: *const f32)>,
    pub R_ParticleExplosion2:
        Option<unsafe extern "C" fn(org: *const f32, colorStart: c_int, colorLength: c_int)>,
    pub R_ParticleLine: Option<
        unsafe extern "C" fn(
            start: *const f32,
            end: *const f32,
            r: c_uchar,
            g: c_uchar,
            b: c_uchar,
            life: f32,
        ),
    >,
    pub R_PlayerSprites:
        Option<unsafe extern "C" fn(client: c_int, modelIndex: c_int, count: c_int, size: c_int)>,
    pub R_Projectile: Option<
        unsafe extern "C" fn(
            origin: *const f32,
            velocity: *const f32,
            modelIndex: c_int,
            life: c_int,
            owner: c_int,
            hitcallback: Option<unsafe extern "C" fn(ent: *mut TEMPENTITY, ptr: *mut pmtrace_s)>,
        ),
    >,
    pub R_RicochetSound: Option<unsafe extern "C" fn(pos: *const f32)>,
    pub R_RicochetSprite: Option<
        unsafe extern "C" fn(pos: *const f32, pmodel: *mut model_s, duration: f32, scale: f32),
    >,
    pub R_RocketFlare: Option<unsafe extern "C" fn(pos: *const f32)>,
    pub R_RocketTrail:
        Option<unsafe extern "C" fn(start: *const f32, end: *const f32, type_: c_int)>,
    pub R_RunParticleEffect:
        Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, color: c_int, count: c_int)>,
    pub R_ShowLine: Option<unsafe extern "C" fn(start: *const f32, end: *const f32)>,
    pub R_SparkEffect: Option<
        unsafe extern "C" fn(pos: *const f32, count: c_int, velocityMin: c_int, velocityMax: c_int),
    >,
    pub R_SparkShower: Option<unsafe extern "C" fn(pos: *const f32)>,
    pub R_SparkStreaks: Option<
        unsafe extern "C" fn(pos: *const f32, count: c_int, velocityMin: c_int, velocityMax: c_int),
    >,
    pub R_Spray: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            dir: *const f32,
            modelIndex: c_int,
            count: c_int,
            speed: c_int,
            spread: c_int,
            rendermode: c_int,
        ),
    >,
    pub R_Sprite_Explode:
        Option<unsafe extern "C" fn(pTemp: *mut TEMPENTITY, scale: f32, flags: c_int)>,
    pub R_Sprite_Smoke: Option<unsafe extern "C" fn(pTemp: *mut TEMPENTITY, scale: f32)>,
    pub R_Sprite_Spray: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            dir: *const f32,
            modelIndex: c_int,
            count: c_int,
            speed: c_int,
            iRand: c_int,
        ),
    >,
    pub R_Sprite_Trail: Option<
        unsafe extern "C" fn(
            type_: c_int,
            start: *const f32,
            end: *const f32,
            modelIndex: c_int,
            count: c_int,
            life: f32,
            size: f32,
            amplitude: f32,
            renderamt: c_int,
            speed: f32,
        ),
    >,
    pub R_Sprite_WallPuff: Option<unsafe extern "C" fn(pTemp: *mut TEMPENTITY, scale: f32)>,
    pub R_StreakSplash: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            dir: *const f32,
            color: c_int,
            count: c_int,
            speed: f32,
            velocityMin: c_int,
            velocityMax: c_int,
        ),
    >,
    pub R_TracerEffect: Option<unsafe extern "C" fn(start: *const f32, end: *const f32)>,
    pub R_UserTracerParticle: Option<
        unsafe extern "C" fn(
            org: *mut f32,
            vel: *mut f32,
            life: f32,
            colorIndex: c_int,
            length: f32,
            deathcontext: c_uchar,
            deathfunc: Option<unsafe extern "C" fn(particle: *mut particle_s)>,
        ),
    >,
    pub R_TracerParticles:
        Option<unsafe extern "C" fn(org: *mut f32, vel: *mut f32, life: f32) -> *mut particle_s>,
    pub R_TeleportSplash: Option<unsafe extern "C" fn(org: *const f32)>,
    pub R_TempSphereModel: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            speed: f32,
            life: f32,
            count: c_int,
            modelIndex: c_int,
        ),
    >,
    pub R_TempModel: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            dir: *const f32,
            angles: *const f32,
            life: f32,
            modelIndex: c_int,
            soundtype: c_int,
        ) -> *mut TEMPENTITY,
    >,
    pub R_DefaultSprite: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            spriteIndex: c_int,
            framerate: f32,
        ) -> *mut TEMPENTITY,
    >,
    pub R_TempSprite: Option<
        unsafe extern "C" fn(
            pos: *const f32,
            dir: *const f32,
            scale: f32,
            modelIndex: c_int,
            rendermode: RenderMode,
            renderfx: RenderFx,
            a: f32,
            life: f32,
            flags: c_int,
        ) -> *mut TEMPENTITY,
    >,
    pub Draw_DecalIndex: Option<unsafe extern "C" fn(id: c_int) -> c_int>,
    pub Draw_DecalIndexFromName: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    pub R_DecalShoot: Option<
        unsafe extern "C" fn(
            textureIndex: c_int,
            entity: c_int,
            modelIndex: c_int,
            position: *const f32,
            flags: c_int,
        ),
    >,
    pub R_AttachTentToPlayer:
        Option<unsafe extern "C" fn(client: c_int, modelIndex: c_int, zoffset: f32, life: f32)>,
    pub R_KillAttachedTents: Option<unsafe extern "C" fn(client: c_int)>,
    pub R_BeamCirclePoints: Option<
        unsafe extern "C" fn(
            type_: c_int,
            start: *mut f32,
            end: *mut f32,
            modelIndex: c_int,
            life: f32,
            width: f32,
            amplitude: f32,
            brightness: f32,
            speed: f32,
            startFrame: c_int,
            framerate: f32,
            r: f32,
            g: f32,
            b: f32,
        ) -> *mut BEAM,
    >,
    pub R_BeamEntPoint: Option<
        unsafe extern "C" fn(
            startEnt: c_int,
            end: *const f32,
            modelIndex: c_int,
            life: f32,
            width: f32,
            amplitude: f32,
            brightness: f32,
            speed: f32,
            startFrame: c_int,
            framerate: f32,
            r: f32,
            g: f32,
            b: f32,
        ) -> *mut BEAM,
    >,
    pub R_BeamEnts: Option<
        unsafe extern "C" fn(
            startEnt: c_int,
            endEnt: c_int,
            modelIndex: c_int,
            life: f32,
            width: f32,
            amplitude: f32,
            brightness: f32,
            speed: f32,
            startFrame: c_int,
            framerate: f32,
            r: f32,
            g: f32,
            b: f32,
        ) -> *mut BEAM,
    >,
    pub R_BeamFollow: Option<
        unsafe extern "C" fn(
            startEnt: c_int,
            modelIndex: c_int,
            life: f32,
            width: f32,
            r: f32,
            g: f32,
            b: f32,
            brightness: f32,
        ) -> *mut BEAM,
    >,
    pub R_BeamKill: Option<unsafe extern "C" fn(deadEntity: c_int)>,
    pub R_BeamLightning: Option<
        unsafe extern "C" fn(
            start: *mut f32,
            end: *mut f32,
            modelIndex: c_int,
            life: f32,
            width: f32,
            amplitude: f32,
            brightness: f32,
            speed: f32,
        ) -> *mut BEAM,
    >,
    pub R_BeamPoints: Option<
        unsafe extern "C" fn(
            start: *const f32,
            end: *const f32,
            modelIndex: c_int,
            life: f32,
            width: f32,
            amplitude: f32,
            brightness: f32,
            speed: f32,
            startFrame: c_int,
            framerate: f32,
            r: f32,
            g: f32,
            b: f32,
        ) -> *mut BEAM,
    >,
    pub R_BeamRing: Option<
        unsafe extern "C" fn(
            startEnt: c_int,
            endEnt: c_int,
            modelIndex: c_int,
            life: f32,
            width: f32,
            amplitude: f32,
            brightness: f32,
            speed: f32,
            startFrame: c_int,
            framerate: f32,
            r: f32,
            g: f32,
            b: f32,
        ) -> *mut BEAM,
    >,
    pub CL_AllocDlight: Option<unsafe extern "C" fn(key: c_int) -> *mut dlight_s>,
    pub CL_AllocElight: Option<unsafe extern "C" fn(key: c_int) -> *mut dlight_s>,
    pub CL_TempEntAlloc:
        Option<unsafe extern "C" fn(org: *const f32, model: *mut model_s) -> *mut TEMPENTITY>,
    pub CL_TempEntAllocNoModel: Option<unsafe extern "C" fn(org: *const f32) -> *mut TEMPENTITY>,
    pub CL_TempEntAllocHigh:
        Option<unsafe extern "C" fn(org: *const f32, model: *mut model_s) -> *mut TEMPENTITY>,
    pub CL_TentEntAllocCustom: Option<
        unsafe extern "C" fn(
            origin: *const f32,
            model: *mut model_s,
            high: c_int,
            callback: Option<
                unsafe extern "C" fn(ent: *mut TEMPENTITY, frametime: f32, currenttime: f32),
            >,
        ) -> *mut TEMPENTITY,
    >,
    pub R_GetPackedColor: Option<unsafe extern "C" fn(packed: *mut c_short, color: c_short)>,
    pub R_LookupColor: Option<unsafe extern "C" fn(r: c_uchar, g: c_uchar, b: c_uchar) -> c_short>,
    pub R_DecalRemoveAll: Option<unsafe extern "C" fn(textureIndex: c_int)>,
    pub R_FireCustomDecal: Option<
        unsafe extern "C" fn(
            textureIndex: c_int,
            entity: c_int,
            modelIndex: c_int,
            position: *mut f32,
            flags: c_int,
            scale: f32,
        ),
    >,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_api_s {
    pub version: c_int,
    pub EV_PlaySound: Option<
        unsafe extern "C" fn(
            ent: c_int,
            origin: *const f32,
            channel: c_int,
            sample: *const c_char,
            volume: f32,
            attenuation: f32,
            flags: SoundFlags,
            pitch: c_int,
        ),
    >,
    pub EV_StopSound:
        Option<unsafe extern "C" fn(ent: c_int, channel: c_int, sample: *const c_char)>,
    pub EV_FindModelIndex: Option<unsafe extern "C" fn(pmodel: *const c_char) -> c_int>,
    pub EV_IsLocal: Option<unsafe extern "C" fn(playernum: c_int) -> c_int>,
    pub EV_LocalPlayerDucking: Option<unsafe extern "C" fn() -> c_int>,
    pub EV_LocalPlayerViewheight: Option<unsafe extern "C" fn(arg1: *mut f32)>,
    pub EV_LocalPlayerBounds:
        Option<unsafe extern "C" fn(hull: c_int, mins: *mut f32, maxs: *mut f32)>,
    pub EV_IndexFromTrace: Option<unsafe extern "C" fn(pTrace: *const pmtrace_s) -> c_int>,
    pub EV_GetPhysent: Option<unsafe extern "C" fn(idx: c_int) -> *mut physent_s>,
    pub EV_SetUpPlayerPrediction:
        Option<unsafe extern "C" fn(dopred: c_int, bIncludeLocalClient: c_int)>,
    pub EV_PushPMStates: Option<unsafe extern "C" fn()>,
    pub EV_PopPMStates: Option<unsafe extern "C" fn()>,
    pub EV_SetSolidPlayers: Option<unsafe extern "C" fn(playernum: c_int)>,
    pub EV_SetTraceHull: Option<unsafe extern "C" fn(hull: c_int)>,
    pub EV_PlayerTrace: Option<
        unsafe extern "C" fn(
            start: *const vec3_t,
            end: *const vec3_t,
            traceFlags: c_int,
            ignore_pe: c_int,
            tr: *mut pmtrace_s,
        ),
    >,
    pub EV_WeaponAnimation: Option<unsafe extern "C" fn(sequence: c_int, body: c_int)>,
    pub EV_PrecacheEvent:
        Option<unsafe extern "C" fn(type_: c_int, psz: *const c_char) -> c_ushort>,
    pub EV_PlaybackEvent: Option<
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
    pub EV_TraceTexture: Option<
        unsafe extern "C" fn(ground: c_int, vstart: *const f32, vend: *const f32) -> *const c_char,
    >,
    pub EV_StopAllSounds: Option<unsafe extern "C" fn(entnum: c_int, entchannel: c_int)>,
    pub EV_KillEvents: Option<unsafe extern "C" fn(entnum: c_int, eventname: *const c_char)>,
    pub EV_PlayerTraceExt: Option<
        unsafe extern "C" fn(
            start: *mut f32,
            end: *mut f32,
            traceFlags: c_int,
            pfnIgnore: Option<unsafe extern "C" fn(pe: *mut physent_s) -> c_int>,
            tr: *mut pmtrace_s,
        ),
    >,
    pub EV_SoundForIndex: Option<unsafe extern "C" fn(index: c_int) -> *const c_char>,
    pub EV_TraceSurface: Option<
        unsafe extern "C" fn(ground: c_int, vstart: *mut f32, vend: *mut f32) -> *mut msurface_s,
    >,
    pub EV_GetMovevars: Option<unsafe extern "C" fn() -> *mut movevars_s>,
    pub EV_VisTraceLine: Option<
        unsafe extern "C" fn(start: *mut f32, end: *mut f32, flags: c_int) -> *mut pmtrace_s,
    >,
    pub EV_GetVisent: Option<unsafe extern "C" fn(idx: c_int) -> *mut physent_s>,
    pub EV_TestLine:
        Option<unsafe extern "C" fn(start: *mut vec3_t, end: *mut vec3_t, flags: c_int) -> c_int>,
    pub EV_PushTraceBounds:
        Option<unsafe extern "C" fn(hullnum: c_int, mins: *const f32, maxs: *const f32)>,
    pub EV_PopTraceBounds: Option<unsafe extern "C" fn()>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct demo_api_s {
    pub IsRecording: Option<unsafe extern "C" fn() -> c_int>,
    pub IsPlayingback: Option<unsafe extern "C" fn() -> c_int>,
    pub IsTimeDemo: Option<unsafe extern "C" fn() -> c_int>,
    pub WriteBuffer: Option<unsafe extern "C" fn(size: c_int, buffer: *mut c_uchar)>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum VoiceTweakControl {
    MicrophoneVolume = 0,
    OtherSpeakerScale = 1,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IVoiceTweak {
    pub StartVoiceTweakMode: Option<unsafe extern "C" fn() -> c_int>,
    pub EndVoiceTweakMode: Option<unsafe extern "C" fn()>,
    pub SetControlFloat: Option<unsafe extern "C" fn(iControl: VoiceTweakControl, value: f32)>,
    pub GetControlFloat: Option<unsafe extern "C" fn(iControl: VoiceTweakControl) -> f32>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct client_sprite_s {
    pub name: CStrArray<64>,
    pub sprite: CStrArray<64>,
    pub hspr: c_int,
    pub res: c_int,
    pub rc: wrect_s,
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct ScreenInfoFlags: c_int {
        const NONE = 0;
        const SCREENFLASH = 1;
        const STRETCHED = 2;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SCREENINFO {
    size: c_int,
    pub width: c_int,
    pub height: c_int,
    pub flags: ScreenInfoFlags,
    pub char_height: c_int,
    pub char_widths: [c_short; 256],
}

impl SCREENINFO {
    pub fn sprite_resolution(&self) -> u32 {
        if self.width > 2560 && self.height > 1600 {
            2560
        } else if self.width >= 1280 && self.height > 720 {
            1280
        } else if self.width >= 640 {
            640
        } else {
            320
        }
    }

    pub fn scale(&self) -> u32 {
        if self.width > 2560 && self.height > 1600 {
            4
        } else if self.width >= 1280 && self.height > 720 {
            3
        } else if self.width >= 640 {
            2
        } else {
            1
        }
    }
}

impl Default for SCREENINFO {
    fn default() -> Self {
        Self {
            size: mem::size_of::<Self>() as c_int,
            width: 0,
            height: 0,
            flags: ScreenInfoFlags::NONE,
            char_height: 0,
            char_widths: [0; 256],
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct hud_player_info_s {
    pub name: *mut c_char,
    pub ping: c_short,
    pub thisplayer: byte,
    pub spectator: byte,
    pub packetloss: byte,
    pub model: *mut c_char,
    pub topcolor: c_short,
    pub bottomcolor: c_short,
    pub m_nSteamID: u64,
}

impl hud_player_info_s {
    pub fn name(&self) -> Option<&CStrThin> {
        if self.name.is_null() {
            None
        } else {
            Some(unsafe { CStrThin::from_ptr(self.name) })
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct client_textmessage_s {
    pub effect: c_int,
    pub r1: byte,
    pub g1: byte,
    pub b1: byte,
    pub a1: byte,
    pub r2: byte,
    pub g2: byte,
    pub b2: byte,
    pub a2: byte,
    pub x: f32,
    pub y: f32,
    pub fadein: f32,
    pub fadeout: f32,
    pub holdtime: f32,
    pub fxtime: f32,
    pub pName: *const c_char,
    pub pMessage: *const c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_args_s {
    pub flags: c_int,
    pub entindex: c_int,
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub velocity: vec3_t,
    pub ducking: c_int,
    pub fparam1: f32,
    pub fparam2: f32,
    pub iparam1: c_int,
    pub iparam2: c_int,
    pub bparam1: c_int,
    pub bparam2: c_int,
}

pub type pfnUserMsgHook =
    Option<unsafe extern "C" fn(pszName: *const c_char, iSize: c_int, pbuf: *mut c_void) -> c_int>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagPOINT {
    _unused: [u8; 0],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdalias_s {
    pub next: *mut cmdalias_s,
    pub name: [c_char; 32usize],
    pub value: *mut c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cl_enginefuncs_s {
    pub pfnSPR_Load: Option<unsafe extern "C" fn(szPicName: *const c_char) -> HSPRITE>,
    pub pfnSPR_Frames: Option<unsafe extern "C" fn(hPic: HSPRITE) -> c_int>,
    pub pfnSPR_Height: Option<unsafe extern "C" fn(hPic: HSPRITE, frame: c_int) -> c_int>,
    pub pfnSPR_Width: Option<unsafe extern "C" fn(hPic: HSPRITE, frame: c_int) -> c_int>,
    pub pfnSPR_Set: Option<unsafe extern "C" fn(hPic: HSPRITE, r: c_int, g: c_int, b: c_int)>,
    pub pfnSPR_Draw:
        Option<unsafe extern "C" fn(frame: c_int, x: c_int, y: c_int, prc: Option<&wrect_s>)>,
    pub pfnSPR_DrawHoles:
        Option<unsafe extern "C" fn(frame: c_int, x: c_int, y: c_int, prc: Option<&wrect_s>)>,
    pub pfnSPR_DrawAdditive:
        Option<unsafe extern "C" fn(frame: c_int, x: c_int, y: c_int, prc: Option<&wrect_s>)>,
    pub pfnSPR_EnableScissor:
        Option<unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int)>,
    pub pfnSPR_DisableScissor: Option<unsafe extern "C" fn()>,
    pub pfnSPR_GetList: Option<
        unsafe extern "C" fn(name: *const c_char, count: *mut c_int) -> *mut client_sprite_s,
    >,
    pub pfnFillRGBA: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            width: c_int,
            height: c_int,
            r: c_int,
            g: c_int,
            b: c_int,
            a: c_int,
        ),
    >,
    pub pfnGetScreenInfo: Option<unsafe extern "C" fn(pscrinfo: *mut SCREENINFO) -> c_int>,
    pub pfnSetCrosshair:
        Option<unsafe extern "C" fn(hspr: HSPRITE, rc: wrect_s, r: c_int, g: c_int, b: c_int)>,
    pub pfnRegisterVariable: Option<
        unsafe extern "C" fn(
            szName: *const c_char,
            szValue: *const c_char,
            flags: c_int,
        ) -> *mut cvar_s,
    >,
    pub pfnGetCvarFloat: Option<unsafe extern "C" fn(szName: *const c_char) -> f32>,
    pub pfnGetCvarString: Option<unsafe extern "C" fn(szName: *const c_char) -> *const c_char>,
    pub pfnAddCommand: Option<
        unsafe extern "C" fn(cmd_name: *const c_char, function: unsafe extern "C" fn()) -> c_int,
    >,
    pub pfnHookUserMsg:
        Option<unsafe extern "C" fn(szMsgName: *const c_char, pfn: pfnUserMsgHook) -> c_int>,
    pub pfnServerCmd: Option<unsafe extern "C" fn(szCmdString: *const c_char) -> c_int>,
    pub pfnClientCmd: Option<unsafe extern "C" fn(szCmdString: *const c_char) -> c_int>,
    pub pfnGetPlayerInfo:
        Option<unsafe extern "C" fn(ent_num: c_int, pinfo: *mut hud_player_info_s)>,
    pub pfnPlaySoundByName: Option<unsafe extern "C" fn(szSound: *const c_char, volume: f32)>,
    pub pfnPlaySoundByIndex: Option<unsafe extern "C" fn(iSound: c_int, volume: f32)>,
    pub pfnAngleVectors: Option<
        unsafe extern "C" fn(
            vecAngles: *const f32,
            forward: *mut f32,
            right: *mut f32,
            up: *mut f32,
        ),
    >,
    pub pfnTextMessageGet:
        Option<unsafe extern "C" fn(pName: *const c_char) -> *mut client_textmessage_s>,
    pub pfnDrawCharacter: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            number: c_int,
            r: c_int,
            g: c_int,
            b: c_int,
        ) -> c_int,
    >,
    pub pfnDrawConsoleString:
        Option<unsafe extern "C" fn(x: c_int, y: c_int, string: *const c_char) -> c_int>,
    pub pfnDrawSetTextColor: Option<unsafe extern "C" fn(r: f32, g: f32, b: f32)>,
    pub pfnDrawConsoleStringLen:
        Option<unsafe extern "C" fn(string: *const c_char, length: *mut c_int, height: *mut c_int)>,
    pub pfnConsolePrint: Option<unsafe extern "C" fn(string: *const c_char)>,
    pub pfnCenterPrint: Option<unsafe extern "C" fn(string: *const c_char)>,
    pub GetWindowCenterX: Option<unsafe extern "C" fn() -> c_int>,
    pub GetWindowCenterY: Option<unsafe extern "C" fn() -> c_int>,
    pub GetViewAngles: Option<unsafe extern "C" fn(arg1: *mut vec3_t)>,
    pub SetViewAngles: Option<unsafe extern "C" fn(arg1: *const vec3_t)>,
    pub GetMaxClients: Option<unsafe extern "C" fn() -> c_int>,
    pub Cvar_SetValue: Option<unsafe extern "C" fn(cvar: *const c_char, value: f32)>,
    pub Cmd_Argc: Option<unsafe extern "C" fn() -> c_int>,
    pub Cmd_Argv: Option<unsafe extern "C" fn(arg: c_int) -> *const c_char>,
    pub Con_Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_DPrintf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_NPrintf: Option<unsafe extern "C" fn(pos: c_int, fmt: *const c_char, ...)>,
    pub Con_NXPrintf:
        Option<unsafe extern "C" fn(info: *mut con_nprint_s, fmt: *const c_char, ...)>,
    pub PhysInfo_ValueForKey: Option<unsafe extern "C" fn(key: *const c_char) -> *const c_char>,
    pub ServerInfo_ValueForKey: Option<unsafe extern "C" fn(key: *const c_char) -> *const c_char>,
    pub GetClientMaxspeed: Option<unsafe extern "C" fn() -> f32>,
    pub CheckParm:
        Option<unsafe extern "C" fn(parm: *const c_char, ppnext: *mut *mut c_char) -> c_int>,
    pub Key_Event: Option<unsafe extern "C" fn(key: c_int, down: c_int)>,
    pub GetMousePosition: Option<unsafe extern "C" fn(mx: *mut c_int, my: *mut c_int)>,
    pub IsNoClipping: Option<unsafe extern "C" fn() -> c_int>,
    pub GetLocalPlayer: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    pub GetViewModel: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    pub GetEntityByIndex: Option<unsafe extern "C" fn(idx: c_int) -> *mut cl_entity_s>,
    pub GetClientTime: Option<unsafe extern "C" fn() -> f32>,
    pub V_CalcShake: Option<unsafe extern "C" fn()>,
    pub V_ApplyShake:
        Option<unsafe extern "C" fn(origin: *mut vec3_t, angles: *mut vec3_t, factor: f32)>,
    pub PM_PointContents:
        Option<unsafe extern "C" fn(point: *const f32, truecontents: *mut c_int) -> c_int>,
    pub PM_WaterEntity: Option<unsafe extern "C" fn(p: *const f32) -> c_int>,
    pub PM_TraceLine: Option<
        unsafe extern "C" fn(
            start: *mut f32,
            end: *mut f32,
            flags: c_int,
            usehull: c_int,
            ignore_pe: c_int,
        ) -> *mut pmtrace_s,
    >,
    pub CL_LoadModel:
        Option<unsafe extern "C" fn(modelname: *const c_char, index: *mut c_int) -> *mut model_s>,
    pub CL_CreateVisibleEntity:
        Option<unsafe extern "C" fn(type_: c_int, ent: *mut cl_entity_s) -> c_int>,
    pub GetSpritePointer: Option<unsafe extern "C" fn(hSprite: HSPRITE) -> *const model_s>,
    pub pfnPlaySoundByNameAtLocation:
        Option<unsafe extern "C" fn(szSound: *mut c_char, volume: f32, origin: *mut f32)>,
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
    pub pfnWeaponAnim: Option<unsafe extern "C" fn(iAnim: c_int, body: c_int)>,
    pub pfnRandomFloat: Option<unsafe extern "C" fn(flLow: f32, flHigh: f32) -> f32>,
    pub pfnRandomLong: Option<unsafe extern "C" fn(lLow: c_int, lHigh: c_int) -> c_int>,
    pub pfnHookEvent: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            pfnEvent: Option<unsafe extern "C" fn(args: *mut event_args_s)>,
        ),
    >,
    pub Con_IsVisible: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetGameDirectory: Option<unsafe extern "C" fn() -> *const c_char>,
    pub pfnGetCvarPointer: Option<unsafe extern "C" fn(szName: *const c_char) -> *mut cvar_s>,
    pub Key_LookupBinding: Option<unsafe extern "C" fn(pBinding: *const c_char) -> *const c_char>,
    pub pfnGetLevelName: Option<unsafe extern "C" fn() -> *const c_char>,
    pub pfnGetScreenFade: Option<unsafe extern "C" fn(fade: *mut screenfade_s)>,
    pub pfnSetScreenFade: Option<unsafe extern "C" fn(fade: *mut screenfade_s)>,
    pub VGui_GetPanel: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub VGui_ViewportPaintBackground: Option<unsafe extern "C" fn(extents: *mut [c_int; 4usize])>,
    pub COM_LoadFile: Option<
        unsafe extern "C" fn(path: *const c_char, usehunk: c_int, pLength: *mut c_int) -> *mut byte,
    >,
    pub COM_ParseFile:
        Option<unsafe extern "C" fn(data: *mut c_char, token: *mut c_char) -> *mut c_char>,
    pub COM_FreeFile: Option<unsafe extern "C" fn(buffer: *mut c_void)>,
    pub pTriAPI: *mut triangleapi_s,
    pub pEfxAPI: *mut efx_api_s,
    pub pEventAPI: *mut event_api_s,
    pub pDemoAPI: *mut demo_api_s,
    pub pNetAPI: *mut net_api_s,
    pub pVoiceTweak: *mut IVoiceTweak,
    pub IsSpectateOnly: Option<unsafe extern "C" fn() -> c_int>,
    pub LoadMapSprite: Option<unsafe extern "C" fn(filename: *const c_char) -> *mut model_s>,
    pub COM_AddAppDirectoryToSearchPath:
        Option<unsafe extern "C" fn(pszBaseDir: *const c_char, appName: *const c_char)>,
    pub COM_ExpandFilename: Option<
        unsafe extern "C" fn(
            fileName: *const c_char,
            nameOutBuffer: *mut c_char,
            nameOutBufferSize: c_int,
        ) -> c_int,
    >,
    pub PlayerInfo_ValueForKey:
        Option<unsafe extern "C" fn(playerNum: c_int, key: *const c_char) -> *const c_char>,
    pub PlayerInfo_SetValueForKey:
        Option<unsafe extern "C" fn(key: *const c_char, value: *const c_char)>,
    pub GetPlayerUniqueID:
        Option<unsafe extern "C" fn(iPlayer: c_int, playerID: *mut [c_char; 16usize]) -> qboolean>,
    pub GetTrackerIDForPlayer: Option<unsafe extern "C" fn(playerSlot: c_int) -> c_int>,
    pub GetPlayerForTrackerID: Option<unsafe extern "C" fn(trackerID: c_int) -> c_int>,
    pub pfnServerCmdUnreliable: Option<unsafe extern "C" fn(szCmdString: *mut c_char) -> c_int>,
    pub pfnGetMousePos: Option<unsafe extern "C" fn(ppt: *mut tagPOINT)>,
    pub pfnSetMousePos: Option<unsafe extern "C" fn(x: c_int, y: c_int)>,
    pub pfnSetMouseEnable: Option<unsafe extern "C" fn(fEnable: qboolean)>,
    pub pfnGetFirstCvarPtr: Option<unsafe extern "C" fn() -> *mut cvar_s>,
    pub pfnGetFirstCmdFunctionHandle: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub pfnGetNextCmdFunctionHandle:
        Option<unsafe extern "C" fn(cmdhandle: *mut c_void) -> *mut c_void>,
    pub pfnGetCmdFunctionName:
        Option<unsafe extern "C" fn(cmdhandle: *mut c_void) -> *const c_char>,
    pub pfnGetClientOldTime: Option<unsafe extern "C" fn() -> f32>,
    pub pfnGetGravity: Option<unsafe extern "C" fn() -> f32>,
    pub pfnGetModelByIndex: Option<unsafe extern "C" fn(index: c_int) -> *mut model_s>,
    pub pfnSetFilterMode: Option<unsafe extern "C" fn(mode: c_int)>,
    pub pfnSetFilterColor: Option<unsafe extern "C" fn(red: f32, green: f32, blue: f32)>,
    pub pfnSetFilterBrightness: Option<unsafe extern "C" fn(brightness: f32)>,
    pub pfnSequenceGet: Option<
        unsafe extern "C" fn(fileName: *const c_char, entryName: *const c_char) -> *mut c_void,
    >,
    pub pfnSPR_DrawGeneric: Option<
        unsafe extern "C" fn(
            frame: c_int,
            x: c_int,
            y: c_int,
            prc: *const wrect_s,
            blendsrc: c_int,
            blenddst: c_int,
            width: c_int,
            height: c_int,
        ),
    >,
    pub pfnSequencePickSentence: Option<
        unsafe extern "C" fn(
            groupName: *const c_char,
            pickMethod: c_int,
            entryPicked: *mut c_int,
        ) -> *mut c_void,
    >,
    pub pfnDrawString: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            str_: *const c_char,
            r: c_int,
            g: c_int,
            b: c_int,
        ) -> c_int,
    >,
    pub pfnDrawStringReverse: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            str_: *const c_char,
            r: c_int,
            g: c_int,
            b: c_int,
        ) -> c_int,
    >,
    pub LocalPlayerInfo_ValueForKey:
        Option<unsafe extern "C" fn(key: *const c_char) -> *const c_char>,
    pub pfnVGUI2DrawCharacter:
        Option<unsafe extern "C" fn(x: c_int, y: c_int, ch: c_int, font: c_uint) -> c_int>,
    pub pfnVGUI2DrawCharacterAdditive: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            ch: c_int,
            r: c_int,
            g: c_int,
            b: c_int,
            font: c_uint,
        ) -> c_int,
    >,
    pub pfnGetApproxWavePlayLen: Option<unsafe extern "C" fn(filename: *const c_char) -> c_uint>,
    pub GetCareerGameUI: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub Cvar_Set: Option<unsafe extern "C" fn(name: *const c_char, value: *const c_char)>,
    pub pfnIsPlayingCareerMatch: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnPlaySoundVoiceByName:
        Option<unsafe extern "C" fn(szSound: *mut c_char, volume: f32, pitch: c_int)>,
    pub pfnPrimeMusicStream: Option<unsafe extern "C" fn(filename: *mut c_char, looping: c_int)>,
    pub pfnSys_FloatTime: Option<unsafe extern "C" fn() -> f64>,
    pub pfnProcessTutorMessageDecayBuffer:
        Option<unsafe extern "C" fn(buffer: *mut c_int, buflen: c_int)>,
    pub pfnConstructTutorMessageDecayBuffer:
        Option<unsafe extern "C" fn(buffer: *mut c_int, buflen: c_int)>,
    pub pfnResetTutorMessageDecayData: Option<unsafe extern "C" fn()>,
    pub pfnPlaySoundByNameAtPitch:
        Option<unsafe extern "C" fn(szSound: *mut c_char, volume: f32, pitch: c_int)>,
    pub pfnFillRGBABlend: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            width: c_int,
            height: c_int,
            r: c_int,
            g: c_int,
            b: c_int,
            a: c_int,
        ),
    >,
    pub pfnGetAppID: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetAliases: Option<unsafe extern "C" fn() -> *mut cmdalias_s>,
    pub pfnVguiWrap2_GetMouseDelta: Option<unsafe extern "C" fn(x: *mut c_int, y: *mut c_int)>,
    pub pfnFilteredClientCmd: Option<unsafe extern "C" fn(cmd: *const c_char) -> c_int>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct client_data_s {
    pub origin: vec3_t,
    pub viewangles: vec3_t,
    pub iWeaponBits: c_int,
    pub fov: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ref_params_s {
    pub vieworg: vec3_t,
    pub viewangles: vec3_t,
    pub forward: vec3_t,
    pub right: vec3_t,
    pub up: vec3_t,
    pub frametime: f32,
    pub time: f32,
    pub intermission: c_int,
    pub paused: c_int,
    pub spectator: c_int,
    pub onground: c_int,
    pub waterlevel: c_int,
    pub simvel: vec3_t,
    pub simorg: vec3_t,
    pub viewheight: vec3_t,
    pub idealpitch: f32,
    pub cl_viewangles: vec3_t,
    pub health: c_int,
    pub crosshairangle: vec3_t,
    pub viewsize: f32,
    pub punchangle: vec3_t,
    pub maxclients: c_int,
    pub viewentity: c_int,
    pub playernum: c_int,
    pub max_entities: c_int,
    pub demoplayback: c_int,
    pub hardware: c_int,
    pub smoothing: c_int,
    pub cmd: *mut usercmd_s,
    pub movevars: *mut movevars_s,
    pub viewport: [c_int; 4usize],
    pub nextView: c_int,
    pub onlyClientDraw: c_int,
}

impl ref_params_s {
    pub fn movevars(&self) -> &movevars_s {
        unsafe { &*self.movevars }
    }

    pub fn cmd(&self) -> &usercmd_s {
        unsafe { &*self.cmd }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum EntityType {
    Normal = 0,
    Player = 1,
    TempEntity = 2,
    Beam = 3,
    Fragmented = 4,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct local_state_s {
    pub playerstate: entity_state_s,
    pub client: clientdata_s,
    pub weapondata: [weapon_data_s; MAX_LOCAL_WEAPONS],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cldll_func_s {
    pub pfnInitialize: Option<
        unsafe extern "C" fn(pEnginefuncs: Option<&cl_enginefuncs_s>, iVersion: c_int) -> c_int,
    >,
    pub pfnInit: Option<unsafe extern "C" fn()>,
    pub pfnVidInit: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnRedraw: Option<unsafe extern "C" fn(flTime: f32, intermission: c_int) -> c_int>,
    pub pfnUpdateClientData:
        Option<unsafe extern "C" fn(cdata: Option<&mut client_data_s>, flTime: f32) -> c_int>,
    pub pfnReset: Option<unsafe extern "C" fn()>,
    pub pfnPlayerMove: Option<unsafe extern "C" fn(ppmove: *mut playermove_s, server: c_int)>,
    pub pfnPlayerMoveInit: Option<unsafe extern "C" fn(ppmove: *mut playermove_s)>,
    pub pfnPlayerMoveTexture: Option<unsafe extern "C" fn(name: *const c_char) -> c_char>,
    pub IN_ActivateMouse: Option<unsafe extern "C" fn()>,
    pub IN_DeactivateMouse: Option<unsafe extern "C" fn()>,
    pub IN_MouseEvent: Option<unsafe extern "C" fn(mstate: c_int)>,
    pub IN_ClearStates: Option<unsafe extern "C" fn()>,
    pub IN_Accumulate: Option<unsafe extern "C" fn()>,
    pub CL_CreateMove:
        Option<unsafe extern "C" fn(frametime: f32, cmd: *mut usercmd_s, active: c_int)>,
    pub CL_IsThirdPerson: Option<unsafe extern "C" fn() -> c_int>,
    pub CL_CameraOffset: Option<unsafe extern "C" fn(ofs: *mut vec3_t)>,
    pub KB_Find: Option<unsafe extern "C" fn(name: *const c_char) -> *mut kbutton_t>,
    pub CAM_Think: Option<unsafe extern "C" fn()>,
    pub pfnCalcRefdef: Option<unsafe extern "C" fn(pparams: Option<&mut ref_params_s>)>,
    pub pfnAddEntity: Option<
        unsafe extern "C" fn(
            entity_type: EntityType,
            entity: Option<&mut cl_entity_s>,
            model_name: *const c_char,
        ) -> c_int,
    >,
    pub pfnCreateEntities: Option<unsafe extern "C" fn()>,
    pub pfnDrawNormalTriangles: Option<unsafe extern "C" fn()>,
    pub pfnDrawTransparentTriangles: Option<unsafe extern "C" fn()>,
    pub pfnStudioEvent:
        Option<unsafe extern "C" fn(event: *const mstudioevent_s, entity: *const cl_entity_s)>,
    pub pfnPostRunCmd: Option<
        unsafe extern "C" fn(
            from: Option<&mut local_state_s>,
            to: Option<&mut local_state_s>,
            cmd: Option<&mut usercmd_s>,
            runfuncs: c_int,
            time: f64,
            random_seed: c_uint,
        ),
    >,
    pub pfnShutdown: Option<unsafe extern "C" fn()>,
    pub pfnTxferLocalOverrides: Option<
        unsafe extern "C" fn(state: Option<&mut entity_state_s>, client: Option<&clientdata_s>),
    >,
    pub pfnProcessPlayerState: Option<
        unsafe extern "C" fn(dst: Option<&mut entity_state_s>, src: Option<&entity_state_s>),
    >,
    pub pfnTxferPredictionData: Option<
        unsafe extern "C" fn(
            ps: Option<&mut entity_state_s>,
            pps: Option<&entity_state_s>,
            pcd: Option<&mut clientdata_s>,
            ppcd: Option<&clientdata_s>,
            wd: *mut weapon_data_s,
            pwd: *const weapon_data_s,
        ),
    >,
    pub pfnDemo_ReadBuffer: Option<unsafe extern "C" fn(size: c_int, buffer: *mut byte)>,
    pub pfnConnectionlessPacket: Option<
        unsafe extern "C" fn(
            net_from: *const netadr_s,
            args: *const c_char,
            buffer: *mut c_char,
            size: *mut c_int,
        ) -> c_int,
    >,
    pub pfnGetHullBounds: Option<
        unsafe extern "C" fn(
            hullnumber: c_int,
            mins: Option<&mut vec3_t>,
            maxs: Option<&mut vec3_t>,
        ) -> c_int,
    >,
    pub pfnFrame: Option<unsafe extern "C" fn(time: f64)>,
    pub pfnKey_Event: Option<
        unsafe extern "C" fn(
            eventcode: c_int,
            keynum: c_int,
            pszCurrentBinding: *const c_char,
        ) -> c_int,
    >,
    pub pfnTempEntUpdate: Option<
        unsafe extern "C" fn(
            frametime: f64,
            client_time: f64,
            cl_gravity: f64,
            ppTempEntFree: *mut *mut TEMPENTITY,
            ppTempEntActive: *mut *mut TEMPENTITY,
            AddVisibleEntity: unsafe extern "C" fn(pEntity: *mut cl_entity_s) -> c_int,
            TempEntPlaySound: unsafe extern "C" fn(pTemp: *mut TEMPENTITY, damp: f32),
        ),
    >,
    pub pfnGetUserEntity: Option<unsafe extern "C" fn(index: c_int) -> *mut cl_entity_s>,
    pub pfnVoiceStatus: Option<unsafe extern "C" fn(entindex: c_int, bTalking: qboolean)>,
    pub pfnDirectorMessage: Option<unsafe extern "C" fn(iSize: c_int, pbuf: *const c_void)>,
    pub pfnGetStudioModelInterface: Option<
        unsafe extern "C" fn(
            version: c_int,
            ppinterface: *mut *mut r_studio_interface_s,
            pstudio: *mut engine_studio_api_s,
        ) -> c_int,
    >,
    pub pfnChatInputPosition: Option<unsafe extern "C" fn(x: *mut c_int, y: *mut c_int)>,
    // TODO:
    // pub pfnGetRenderInterface: Option<
    //     unsafe extern "C" fn(
    //         version: c_int,
    //         renderfuncs: *mut render_api_t,
    //         callback: *mut render_interface_t,
    //     ) -> c_int,
    // >,
    // pub pfnClipMoveToEntity: Option<
    //     unsafe extern "C" fn(
    //         pe: *mut physent_s,
    //         start: *mut vec3_t,
    //         mins: *mut vec3_t,
    //         maxs: *mut vec3_t,
    //         end: *mut vec3_t,
    //         tr: *mut pmtrace_s,
    //     ),
    // >,
    // pub pfnTouchEvent: Option<
    //     unsafe extern "C" fn(
    //         type_: c_int,
    //         fingerID: c_int,
    //         x: f32,
    //         y: f32,
    //         dx: f32,
    //         dy: f32,
    //     ) -> c_int,
    // >,
    // pub pfnMoveEvent: Option<unsafe extern "C" fn(forwardmove: f32, sidemove: f32)>,
    // pub pfnLookEvent: Option<unsafe extern "C" fn(relyaw: f32, relpitch: f32)>,
}
