#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_short, c_void},
    slice,
};

use bitflags::bitflags;
use csz::{CStrArray, CStrThin};

use crate::{consts::MAX_LEVEL_CONNECTIONS, str::MapString};

pub use shared::raw::*;

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

#[doc(hidden)]
#[deprecated]
pub type string_t = Option<crate::str::MapString>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyValueData {
    class_name: *mut c_char,
    key_name: *mut c_char,
    value: *mut c_char,
    handled: c_int,
}

impl KeyValueData {
    /// Returns the class name of an entity related to the data.
    pub fn class_name(&self) -> Option<&CStrThin> {
        if self.class_name.is_null() {
            return None;
        }
        Some(unsafe { CStrThin::from_ptr(self.class_name) })
    }

    pub fn key_name(&self) -> &CStrThin {
        assert!(!self.key_name.is_null());
        unsafe { CStrThin::from_ptr(self.key_name) }
    }

    pub fn value(&self) -> &CStrThin {
        assert!(!self.value.is_null());
        unsafe { CStrThin::from_ptr(self.value) }
    }

    /// Returns `true` if the server DLL knows the key name.
    pub fn handled(&self) -> bool {
        self.handled != 0
    }

    pub fn set_handled(&mut self, handled: bool) {
        self.handled = handled as c_int;
    }
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
