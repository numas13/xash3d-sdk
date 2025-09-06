#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_short},
    mem,
};

use bitflags::bitflags;
use csz::CStrThin;

use crate::consts::MAX_LOCAL_WEAPONS;

pub use shared::raw::*;

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
