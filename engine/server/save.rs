use core::{
    cmp,
    ffi::{c_char, c_float, c_int, c_short, c_uint, c_ushort, c_void, CStr},
    fmt, mem, ptr, slice,
};

use bitflags::bitflags;
use csz::CStrThin;
use xash3d_shared::{
    ffi::{
        common::vec3_t,
        server::{edict_s, entvars_s, KeyValueData, ENTITYTABLE, SAVERESTOREDATA, TYPEDESCRIPTION},
    },
    utils::{
        array_from_slice, cstr_or_none, slice_from_raw_parts_or_empty,
        slice_from_raw_parts_or_empty_mut,
    },
};

use crate::engine::ServerEngineRef;

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
        self.fHandled = handled.into();
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    Empty,
    Overflow,
}

impl fmt::Display for SaveError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => fmt.write_str("empty"),
            Self::Overflow => fmt.write_str("overflow"),
        }
    }
}

pub type SaveResult<T, E = SaveError> = core::result::Result<T, E>;

const ENTVARS_DESCRIPTION: &[TYPEDESCRIPTION] = &[
    define_entity_field!(classname, FieldType::STRING),
    define_entity_field!(globalname, FieldType::STRING, global),
    define_entity_field!(origin, FieldType::POSITION_VECTOR),
    define_entity_field!(oldorigin, FieldType::POSITION_VECTOR),
    define_entity_field!(velocity, FieldType::VECTOR),
    define_entity_field!(basevelocity, FieldType::VECTOR),
    define_entity_field!(movedir, FieldType::VECTOR),
    define_entity_field!(angles, FieldType::VECTOR),
    define_entity_field!(avelocity, FieldType::VECTOR),
    define_entity_field!(punchangle, FieldType::VECTOR),
    define_entity_field!(v_angle, FieldType::VECTOR),
    define_entity_field!(fixangle, FieldType::FLOAT),
    define_entity_field!(idealpitch, FieldType::FLOAT),
    define_entity_field!(pitch_speed, FieldType::FLOAT),
    define_entity_field!(ideal_yaw, FieldType::FLOAT),
    define_entity_field!(yaw_speed, FieldType::FLOAT),
    define_entity_field!(modelindex, FieldType::INTEGER),
    define_entity_field!(model, FieldType::MODELNAME, global),
    define_entity_field!(viewmodel, FieldType::MODELNAME),
    define_entity_field!(weaponmodel, FieldType::MODELNAME),
    define_entity_field!(absmin, FieldType::POSITION_VECTOR),
    define_entity_field!(absmax, FieldType::POSITION_VECTOR),
    define_entity_field!(mins, FieldType::VECTOR, global),
    define_entity_field!(maxs, FieldType::VECTOR, global),
    define_entity_field!(size, FieldType::VECTOR, global),
    define_entity_field!(ltime, FieldType::TIME),
    define_entity_field!(nextthink, FieldType::TIME),
    define_entity_field!(solid, FieldType::INTEGER),
    define_entity_field!(movetype, FieldType::INTEGER),
    define_entity_field!(skin, FieldType::INTEGER),
    define_entity_field!(body, FieldType::INTEGER),
    define_entity_field!(effects, FieldType::INTEGER),
    define_entity_field!(gravity, FieldType::FLOAT),
    define_entity_field!(friction, FieldType::FLOAT),
    define_entity_field!(light_level, FieldType::FLOAT),
    define_entity_field!(frame, FieldType::FLOAT),
    define_entity_field!(scale, FieldType::FLOAT),
    define_entity_field!(sequence, FieldType::INTEGER),
    define_entity_field!(animtime, FieldType::TIME),
    define_entity_field!(framerate, FieldType::FLOAT),
    define_entity_field!(controller, FieldType::INTEGER),
    define_entity_field!(blending, FieldType::INTEGER),
    define_entity_field!(rendermode, FieldType::INTEGER),
    define_entity_field!(renderamt, FieldType::FLOAT),
    define_entity_field!(rendercolor, FieldType::VECTOR),
    define_entity_field!(renderfx, FieldType::INTEGER),
    define_entity_field!(health, FieldType::FLOAT),
    define_entity_field!(frags, FieldType::FLOAT),
    define_entity_field!(weapons, FieldType::INTEGER),
    define_entity_field!(takedamage, FieldType::FLOAT),
    define_entity_field!(deadflag, FieldType::FLOAT),
    define_entity_field!(view_ofs, FieldType::VECTOR),
    define_entity_field!(button, FieldType::INTEGER),
    define_entity_field!(impulse, FieldType::INTEGER),
    define_entity_field!(chain, FieldType::EDICT),
    define_entity_field!(dmg_inflictor, FieldType::EDICT),
    define_entity_field!(enemy, FieldType::EDICT),
    define_entity_field!(aiment, FieldType::EDICT),
    define_entity_field!(owner, FieldType::EDICT),
    define_entity_field!(groundentity, FieldType::EDICT),
    define_entity_field!(spawnflags, FieldType::INTEGER),
    define_entity_field!(flags, FieldType::FLOAT),
    define_entity_field!(colormap, FieldType::INTEGER),
    define_entity_field!(team, FieldType::INTEGER),
    define_entity_field!(max_health, FieldType::FLOAT),
    define_entity_field!(teleport_time, FieldType::TIME),
    define_entity_field!(armortype, FieldType::FLOAT),
    define_entity_field!(armorvalue, FieldType::FLOAT),
    define_entity_field!(waterlevel, FieldType::INTEGER),
    define_entity_field!(watertype, FieldType::INTEGER),
    define_entity_field!(target, FieldType::STRING, global),
    define_entity_field!(targetname, FieldType::STRING, global),
    define_entity_field!(netname, FieldType::STRING),
    define_entity_field!(message, FieldType::STRING),
    define_entity_field!(dmg_take, FieldType::FLOAT),
    define_entity_field!(dmg_save, FieldType::FLOAT),
    define_entity_field!(dmg, FieldType::FLOAT),
    define_entity_field!(dmgtime, FieldType::TIME),
    define_entity_field!(noise, FieldType::SOUNDNAME),
    define_entity_field!(noise1, FieldType::SOUNDNAME),
    define_entity_field!(noise2, FieldType::SOUNDNAME),
    define_entity_field!(noise3, FieldType::SOUNDNAME),
    define_entity_field!(speed, FieldType::FLOAT),
    define_entity_field!(air_finished, FieldType::TIME),
    define_entity_field!(pain_finished, FieldType::TIME),
    define_entity_field!(radsuit_finished, FieldType::TIME),
];

#[derive(Copy, Clone)]
pub struct Field<'a> {
    token: u16,
    data: &'a [u8],
}

impl<'a> Field<'a> {
    pub fn token(&self) -> u16 {
        self.token
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

pub struct SaveRestoreData<'a> {
    data: &'a mut SAVERESTOREDATA,
}

impl<'a> SaveRestoreData<'a> {
    pub fn new(data: &'a mut SAVERESTOREDATA) -> Self {
        Self { data }
    }

    pub fn restore_save_pointers(&mut self) {
        self.data.size = self.data.table()[self.current_index()].location;
        self.data.pCurrentData = self.data.pBaseData.wrapping_add(self.data.size as usize);
    }

    pub fn time(&self) -> f32 {
        self.data.time
    }

    pub fn use_landmark_offset(&self) -> Option<vec3_t> {
        if self.data.fUseLandmark != 0 {
            Some(self.data.vecLandmarkOffset)
        } else {
            None
        }
    }

    pub fn landmark_offset(&self) -> vec3_t {
        self.data.vecLandmarkOffset
    }

    pub fn set_landmark_offset(&mut self, landmark_offset: vec3_t) {
        self.data.vecLandmarkOffset = landmark_offset;
    }

    pub fn current_map_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.data.szCurrentMapName.as_ptr()) }
    }

    pub fn current_index(&self) -> usize {
        self.data.currentIndex as usize
    }

    pub fn table(&self) -> &[ENTITYTABLE] {
        let len = self.data.tableCount as usize;
        unsafe { slice_from_raw_parts_or_empty(self.data.pTable, len) }
    }

    pub fn table_mut(&mut self) -> &mut [ENTITYTABLE] {
        let len = self.data.tableCount as usize;
        unsafe { slice_from_raw_parts_or_empty_mut(self.data.pTable, len) }
    }

    pub fn tokens(&mut self) -> &[*mut c_char] {
        let len = self.data.tokenCount as usize;
        unsafe { slice_from_raw_parts_or_empty(self.data.pTokens, len) }
    }

    pub fn tokens_mut(&mut self) -> &mut [*mut c_char] {
        let len = self.data.tokenCount as usize;
        unsafe { slice_from_raw_parts_or_empty_mut(self.data.pTokens, len) }
    }

    pub fn size(&self) -> usize {
        self.data.size as usize
    }

    pub fn buffer_size(&self) -> usize {
        self.data.bufferSize as usize
    }

    pub fn available(&self) -> usize {
        self.buffer_size() - self.size()
    }

    pub fn is_empty(&self) -> bool {
        let diff = unsafe { self.data.pCurrentData.offset_from(self.data.pBaseData) };
        diff >= self.buffer_size() as isize
    }

    fn buffer_rewind(&mut self, size: usize) {
        let size = cmp::min(size, self.size());
        self.data.pCurrentData = unsafe { self.data.pCurrentData.byte_sub(size) };
        self.data.size -= size as c_int;
    }

    pub fn field_rewind(&mut self, field: &Field) {
        self.buffer_rewind(2 * mem::size_of::<u16>() + field.data.len());
    }

    pub fn check(&self, size: usize) -> SaveResult<()> {
        if self.is_empty() {
            Err(SaveError::Empty)
        } else if size > self.available() {
            Err(SaveError::Overflow)
        } else {
            Ok(())
        }
    }

    pub fn as_slice(&self) -> &'a [u8] {
        if !self.data.pCurrentData.is_null() {
            unsafe {
                let data = self.data.pCurrentData.cast();
                let len = self.available();
                slice::from_raw_parts(data, len)
            }
        } else {
            &[]
        }
    }

    pub fn token_hash(&mut self, token: &CStr) -> c_ushort {
        fn hash_string(token: &CStr) -> c_uint {
            token
                .to_bytes()
                .iter()
                .fold(0, |hash, &byte| hash.rotate_right(4) ^ (byte as c_uint))
        }

        let tokens = self.data.tokens_mut();
        let hash = (hash_string(token) % (tokens.len() as c_uint)) as c_ushort;
        for i in 0..tokens.len() {
            let mut index = i + hash as usize;
            if index >= tokens.len() {
                index -= tokens.len();
            }
            if tokens[index].is_null() || token == unsafe { CStr::from_ptr(tokens[index]) } {
                tokens[index] = token.as_ptr() as *mut c_char;
                return index as c_ushort;
            }
        }

        error!("Save::token_hash is COMPLETELY FULL!");
        0
    }

    unsafe fn advance(&mut self, len: usize) {
        self.data.pCurrentData = unsafe { self.data.pCurrentData.byte_add(len) };
        self.data.size += len as c_int;
    }

    pub fn read_slice(&mut self, len: usize) -> SaveResult<&'a [u8]> {
        self.check(len).map(|_| {
            let data = self.data.pCurrentData.cast();
            let output = unsafe { slice::from_raw_parts(data, len) };
            unsafe {
                self.advance(output.len());
            }
            output
        })
    }

    pub fn read_bytes(&mut self, output: &mut [u8]) -> SaveResult<()> {
        match self.read_slice(output.len()) {
            Ok(slice) => {
                output.copy_from_slice(slice);
                Ok(())
            }
            Err(e) => {
                output.fill(0);
                if e == SaveError::Empty {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> SaveResult<()> {
        self.check(bytes.len()).map(|_| {
            let data = self.data.pCurrentData.cast();
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), data, bytes.len());
                self.advance(bytes.len());
            }
        })
    }

    pub fn read_array<const N: usize>(&mut self) -> SaveResult<[u8; N]> {
        let mut output = [0; N];
        self.read_bytes(&mut output).map(|_| output)
    }

    pub fn read_i16(&mut self) -> SaveResult<i16> {
        self.read_array().map(i16::from_le_bytes)
    }

    pub fn read_u16(&mut self) -> SaveResult<u16> {
        self.read_array().map(u16::from_le_bytes)
    }

    pub fn read_i32(&mut self) -> SaveResult<i32> {
        self.read_array().map(i32::from_le_bytes)
    }

    pub fn read_u32(&mut self) -> SaveResult<u32> {
        self.read_array().map(u32::from_le_bytes)
    }

    pub fn write_i16(&mut self, value: i16) -> SaveResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_u16(&mut self, value: u16) -> SaveResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_i32(&mut self, value: i32) -> SaveResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_u32(&mut self, value: u32) -> SaveResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn read_field(&mut self) -> SaveResult<Field<'a>> {
        let size = self.read_u16()?;
        let token = self.read_u16()?;
        let data = self.read_slice(size as usize)?;
        Ok(Field { token, data })
    }

    pub fn write_field(&mut self, name: &CStr, bytes: &[u8]) -> SaveResult<()> {
        let size = bytes.len().try_into().unwrap();
        let token = self.token_hash(name);
        self.write_u16(size)?;
        self.write_u16(token)?;
        self.write_bytes(bytes)
    }
}

pub struct SaveReader<'a> {
    engine: ServerEngineRef,
    pub data: SaveRestoreData<'a>,
    global: bool,
    precache: bool,
}

#[allow(dead_code)]
impl<'a> SaveReader<'a> {
    pub fn new(engine: ServerEngineRef, data: &'a mut SAVERESTOREDATA) -> Self {
        Self {
            engine,
            data: SaveRestoreData::new(data),
            global: false,
            precache: true,
        }
    }

    pub fn precache_mode(&mut self, mode: bool) {
        self.precache = mode;
    }

    pub fn global_mode(&mut self, mode: bool) {
        self.global = mode;
    }

    fn read_field(
        &mut self,
        base_data: *mut c_void,
        fields: &[TYPEDESCRIPTION],
        start_field: usize,
        name: &CStrThin,
        data: &[u8],
    ) -> SaveResult<usize> {
        use FieldType as F;

        let engine = self.engine;
        let time = self.data.time();
        let position = self.data.use_landmark_offset().unwrap_or_default();

        for i in 0..fields.len() {
            let field_index = (i + start_field) % fields.len();
            let field = &fields[field_index];
            if !field.name().is_some_and(|s| s.eq_ignore_case(name)) {
                continue;
            }

            if self.global && field.flags().intersects(FtypeDesc::GLOBAL) {
                // skip global entity
                return Ok(field_index);
            }

            let ptr = unsafe { base_data.byte_add(field.fieldOffset as usize) };
            let size = field_size(field.field_type());
            let count = field.fieldSize as usize;
            let output = unsafe { slice::from_raw_parts_mut(ptr.cast::<u8>(), size * count) };

            match field.field_type() {
                F::TIME => {
                    for (dst, src) in output.chunks_mut(size).zip(data.chunks(size)) {
                        let value = c_float::from_le_bytes(array_from_slice(src)) + time;
                        dst.copy_from_slice(&value.to_ne_bytes());
                    }
                }
                F::CHARACTER => {
                    output.copy_from_slice(data);
                }
                F::SHORT | F::INTEGER | F::FLOAT => {
                    // TODO: byte-ordering
                    output.copy_from_slice(data);
                }
                F::VECTOR => {
                    // TODO: byte-ordering
                    output.copy_from_slice(data);
                }
                F::MODELNAME | F::SOUNDNAME | F::STRING => {
                    let mut iter = data.split_inclusive(|&i| i == b'\0');
                    for dst in output.chunks_mut(size) {
                        let chunk = iter.next().unwrap();
                        let str = CStr::from_bytes_with_nul(chunk).unwrap();
                        if !str.is_empty() {
                            let id = engine.new_map_string(str);
                            dst.copy_from_slice(&id.index().to_ne_bytes());
                            if self.precache {
                                match field.field_type() {
                                    F::MODELNAME => {
                                        engine.precache_model(str);
                                    }
                                    F::SOUNDNAME => {
                                        engine.precache_sound(str);
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            dst.copy_from_slice(&[0; 4]);
                        }
                    }
                }
                F::POSITION_VECTOR => {
                    for (dst, src) in output.chunks_mut(size).zip(data.chunks(size)) {
                        for i in 0..3 {
                            let start = i * 4;
                            let end = start + 4;
                            let src = array_from_slice(&src[start..end]);
                            let value = c_float::from_le_bytes(src) + position[i];
                            dst[start..end].copy_from_slice(&value.to_ne_bytes());
                        }
                    }
                }
                F::EDICT => {
                    let dst = output.chunks_mut(mem::size_of::<*mut edict_s>());
                    let src = data.chunks(size);
                    for (dst, src) in dst.zip(src) {
                        let index = c_int::from_le_bytes(array_from_slice(src));
                        let ent = engine.entity_of_ent_index(index);
                        dst.copy_from_slice(&(ent as usize).to_ne_bytes());
                    }
                }
                _ => {
                    let field_type = field.fieldType;
                    warn!("unimplemented field({field_type}) read for \"{name}\"");
                }
            }

            return Ok(field_index);
        }

        Ok(start_field)
    }

    pub fn read_fields(
        &mut self,
        name: &CStr,
        base_data: *mut c_void,
        fields: &[TYPEDESCRIPTION],
    ) -> SaveResult<()> {
        let header = self.data.read_field()?;
        assert_eq!(header.data.len(), mem::size_of::<u32>());
        if header.token != self.data.token_hash(name) {
            self.data.field_rewind(&header);
            return Ok(());
        }
        let field_count = u32::from_le_bytes(array_from_slice(header.data));

        for i in fields.iter() {
            if self.global && i.flags().intersects(FtypeDesc::GLOBAL) {
                continue;
            }
            let data = base_data.wrapping_add(i.fieldOffset as usize);
            let len = i.fieldSize as usize * field_size(i.field_type());
            unsafe {
                ptr::write_bytes(data, 0, len);
            }
        }

        let mut last_field = 0;
        for _ in 0..field_count {
            let field = self.data.read_field()?;
            let token = self.data.data.tokens_mut()[field.token as usize];
            let token = unsafe { CStrThin::from_ptr(token) };
            last_field = self.read_field(base_data, fields, last_field, token, field.data)?;
            last_field += 1;
        }

        Ok(())
    }

    pub fn read_ent_vars(&mut self, name: &CStr, ev: *mut entvars_s) -> SaveResult<()> {
        self.read_fields(name, ev as *mut _, ENTVARS_DESCRIPTION)
    }

    pub fn entity_index(&self, ent: *mut edict_s) -> Option<usize> {
        if !ent.is_null() {
            self.data.table().iter().position(|i| i.pent == ent)
        } else {
            None
        }
    }

    pub fn entity_flags_set(&mut self, index: usize, flags: c_int) -> c_int {
        if let Some(i) = self.data.table_mut().get_mut(index) {
            i.flags |= flags;
            i.flags
        } else {
            0
        }
    }
}

#[allow(dead_code)]
pub struct SaveWriter<'a> {
    engine: ServerEngineRef,
    pub data: SaveRestoreData<'a>,
    global: bool,
    precache: bool,
}

#[allow(dead_code)]
impl<'a> SaveWriter<'a> {
    pub fn new(engine: ServerEngineRef, data: &'a mut SAVERESTOREDATA) -> Self {
        Self {
            engine,
            data: SaveRestoreData::new(data),
            global: false,
            precache: true,
        }
    }

    pub fn write_fields(
        &mut self,
        name: &CStrThin,
        _base_data: *mut c_void,
        _fields: &[TYPEDESCRIPTION],
    ) -> SaveResult<()> {
        error!("TODO: SaveWriteFields({name})");
        self.data
            .write_field(name.as_c_str(), &0_u32.to_le_bytes())?;
        Ok(())
    }
}

fn field_size(ty: FieldType) -> usize {
    use FieldType as F;
    match ty {
        F::FLOAT => mem::size_of::<c_float>(),
        F::STRING => mem::size_of::<c_int>(),
        F::ENTITY => mem::size_of::<c_int>(),
        F::CLASSPTR => mem::size_of::<c_int>(),
        F::EHANDLE => mem::size_of::<c_int>(),
        F::EVARS => mem::size_of::<c_int>(),
        F::EDICT => mem::size_of::<c_int>(),
        F::VECTOR => mem::size_of::<c_float>() * 3,
        F::POSITION_VECTOR => mem::size_of::<c_float>() * 3,
        F::POINTER => mem::size_of::<*const c_int>(),
        F::INTEGER => mem::size_of::<c_int>(),
        // #ifdef GNUC
        F::FUNCTION => mem::size_of::<*const c_int>() * 2,
        // #else
        //      F::FUNCTION         => mem::size_of::<*const c_int>(),
        // #endif
        F::BOOLEAN => mem::size_of::<c_int>(),
        F::SHORT => mem::size_of::<c_short>(),
        F::CHARACTER => mem::size_of::<c_char>(),
        F::TIME => mem::size_of::<c_float>(),
        F::MODELNAME => mem::size_of::<c_int>(),
        F::SOUNDNAME => mem::size_of::<c_int>(),
        F::TYPECOUNT => unreachable!(),
    }
}

pub fn entvars_key_value(engine: ServerEngineRef, ev: &mut entvars_s, data: &mut KeyValueData) {
    let key_name = data.key_name();
    let field = ENTVARS_DESCRIPTION
        .iter()
        .find(|i| i.name().unwrap().eq_ignore_case(key_name));

    if let Some(field) = field {
        let pev = ev as *mut _ as *mut u8;
        let p = unsafe { pev.offset(field.fieldOffset as isize) };
        let value = data.value();

        match field.field_type() {
            FieldType::MODELNAME | FieldType::SOUNDNAME | FieldType::STRING => {
                let s = engine.new_map_string(value);
                unsafe {
                    ptr::write(p.cast::<c_int>(), s.index());
                }
            }
            FieldType::TIME | FieldType::FLOAT => {
                let s = value.to_str().ok();
                let v = s.and_then(|s| s.parse().ok()).unwrap_or(0.0);
                unsafe {
                    ptr::write(p.cast::<f32>(), v);
                }
            }
            FieldType::INTEGER => {
                let s = value.to_str().ok();
                let v = s.and_then(|s| s.parse().ok()).unwrap_or(0);
                unsafe {
                    ptr::write(p.cast::<c_int>(), v);
                }
            }
            FieldType::POSITION_VECTOR | FieldType::VECTOR => {
                let s = value.to_str().unwrap();
                let mut v = vec3_t::ZERO;
                for (i, s) in s.split(' ').enumerate() {
                    v[i] = s.parse().unwrap_or(0.0);
                }
                unsafe {
                    ptr::write(p.cast::<vec3_t>(), v);
                }
            }
            _ => {
                let name = unsafe { CStr::from_ptr(field.fieldName) };
                error!("Bad field({name:?}, {:?}) in entity", field.fieldType);
            }
        }
        data.set_handled(true);
    }
}
