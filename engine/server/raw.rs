use core::{
    ffi::{c_char, c_int, c_short, c_uint},
    mem,
};

use bitflags::bitflags;
use csz::{CStrSlice, CStrThin};
use shared::{
    ffi::server::{KeyValueData, ENTITYTABLE, LEVELLIST, SAVERESTOREDATA, TYPEDESCRIPTION},
    utils::{cstr_or_none, slice_from_raw_parts_or_empty, slice_from_raw_parts_or_empty_mut},
};

pub use shared::raw::*;

pub use crate::entity::EntityVarsExt;

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

pub trait KeyValueDataExt {
    /// Returns the class name of an entity related to the data.
    fn class_name(&self) -> Option<&CStrThin>;

    fn key_name(&self) -> &CStrThin;

    fn value(&self) -> &CStrThin;

    /// Returns `true` if the server DLL knows the key name.
    fn handled(&self) -> bool;

    fn set_handled(&mut self, handled: bool);
}

impl KeyValueDataExt for KeyValueData {
    /// Returns the class name of an entity related to the data.
    fn class_name(&self) -> Option<&CStrThin> {
        unsafe { cstr_or_none(self.szClassName) }
    }

    fn key_name(&self) -> &CStrThin {
        unsafe { cstr_or_none(self.szKeyName) }.unwrap()
    }

    fn value(&self) -> &CStrThin {
        unsafe { cstr_or_none(self.szValue) }.unwrap()
    }

    /// Returns `true` if the server DLL knows the key name.
    fn handled(&self) -> bool {
        self.fHandled != 0
    }

    fn set_handled(&mut self, handled: bool) {
        self.fHandled = handled as c_int;
    }
}

pub trait SaveRestoreExt {
    fn current_map_name(&self) -> &CStrThin;

    fn table(&self) -> &[ENTITYTABLE];

    fn table_mut(&mut self) -> &mut [ENTITYTABLE];

    fn tokens(&mut self) -> &[*mut c_char];

    fn tokens_mut(&mut self) -> &mut [*mut c_char];
}

impl SaveRestoreExt for SAVERESTOREDATA {
    fn current_map_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.szCurrentMapName.as_ptr()) }
    }

    fn table(&self) -> &[ENTITYTABLE] {
        let len = self.tableCount as usize;
        unsafe { slice_from_raw_parts_or_empty(self.pTable, len) }
    }

    fn table_mut(&mut self) -> &mut [ENTITYTABLE] {
        let len = self.tableCount as usize;
        unsafe { slice_from_raw_parts_or_empty_mut(self.pTable, len) }
    }

    fn tokens(&mut self) -> &[*mut c_char] {
        let len = self.tokenCount as usize;
        unsafe { slice_from_raw_parts_or_empty(self.pTokens, len) }
    }

    fn tokens_mut(&mut self) -> &mut [*mut c_char] {
        let len = self.tokenCount as usize;
        unsafe { slice_from_raw_parts_or_empty_mut(self.pTokens, len) }
    }
}

/// TYPEDESCRIPTION.fieldType
#[allow(non_camel_case_types)]
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

impl FieldType {
    pub fn from_raw(raw: c_uint) -> Option<Self> {
        match raw {
            0 => Some(Self::FLOAT),
            1 => Some(Self::STRING),
            2 => Some(Self::ENTITY),
            3 => Some(Self::CLASSPTR),
            4 => Some(Self::EHANDLE),
            5 => Some(Self::EVARS),
            6 => Some(Self::EDICT),
            7 => Some(Self::VECTOR),
            8 => Some(Self::POSITION_VECTOR),
            9 => Some(Self::POINTER),
            10 => Some(Self::INTEGER),
            11 => Some(Self::FUNCTION),
            12 => Some(Self::BOOLEAN),
            13 => Some(Self::SHORT),
            14 => Some(Self::CHARACTER),
            15 => Some(Self::TIME),
            16 => Some(Self::MODELNAME),
            17 => Some(Self::SOUNDNAME),
            18 => Some(Self::TYPECOUNT),
            _ => None,
        }
    }
}

bitflags! {
    /// TYPEDESCRIPTION.flags
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

pub trait TypeDescriptionExt {
    fn name(&self) -> Option<&CStrThin>;

    fn field_type(&self) -> FieldType;

    fn flags(&self) -> &FtypeDesc;
}

impl TypeDescriptionExt for TYPEDESCRIPTION {
    fn name(&self) -> Option<&CStrThin> {
        unsafe { cstr_or_none(self.fieldName) }
    }

    fn field_type(&self) -> FieldType {
        FieldType::from_raw(self.fieldType).unwrap()
    }

    fn flags(&self) -> &FtypeDesc {
        unsafe { mem::transmute(&self.flags) }
    }
}
