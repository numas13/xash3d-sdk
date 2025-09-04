#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_ushort, c_void},
    mem,
};

use bitflags::bitflags;
use csz::CStrThin;

use crate::{consts::MAX_LOCAL_WEAPONS, engine::cl_enginefuncs_s};

pub use shared::raw::*;

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
            pInvoker: *const fake_edict_s,
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
