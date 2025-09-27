mod cursor;
mod macros;

use core::{
    ffi::{c_char, c_float, c_int, c_short, c_uint, c_ushort, CStr},
    fmt, mem, ptr, slice,
};

use bitflags::bitflags;
use csz::{CStrArray, CStrThin};
use xash3d_shared::{
    ffi::{
        common::vec3_t,
        server::{edict_s, entvars_s, KeyValueData, ENTITYTABLE, SAVERESTOREDATA, TYPEDESCRIPTION},
    },
    utils::{cstr_or_none, slice_from_raw_parts_or_empty, slice_from_raw_parts_or_empty_mut},
};

use crate::{engine::ServerEngineRef, str::MapString};

pub use self::cursor::*;
pub use self::macros::*;

pub trait TypeDescription: Sized {
    const TYPE: FieldType;
}

macro_rules! impl_type_description {
    ($( $field_type:expr => $type:ty ),* $(,)?) => {
        $(
            impl TypeDescription for $type {
                const TYPE: FieldType = $field_type;
            }
            xash3d_shared::macros::const_assert_eq!(
                $field_type.size(),
                core::mem::size_of::<$type>(),
            );
        )*
    };
}

impl_type_description! {
    FieldType::CHARACTER => i8,
    FieldType::CHARACTER => u8,

    FieldType::SHORT => i16,
    FieldType::SHORT => u16,

    FieldType::INTEGER => i32,
    FieldType::INTEGER => u32,

    FieldType::FLOAT => f32,
    FieldType::TIME => Time,

    FieldType::VECTOR => vec3_t,
    FieldType::POSITION_VECTOR => PositionVector,

    FieldType::STRING => Option<MapString>,

    FieldType::EDICT => *const edict_s,
    FieldType::EDICT => *mut edict_s,
}

// define types for wrappers
impl_type_description! {
    FieldType::FLOAT => crate::sound::Attenuation,
}

impl<T: TypeDescription, const N: usize> TypeDescription for [T; N] {
    const TYPE: FieldType = T::TYPE;
}

impl<const N: usize> TypeDescription for CStrArray<N> {
    const TYPE: FieldType = FieldType::CHARACTER;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Time(pub f32);

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PositionVector(pub vec3_t);

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token(u16);

impl Token {
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub const fn to_u16(&self) -> u16 {
        self.0
    }

    pub const fn to_usize(&self) -> usize {
        self.0 as usize
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

    pub const fn size(&self) -> usize {
        match self {
            Self::FLOAT => mem::size_of::<c_float>(),
            Self::STRING => mem::size_of::<c_int>(),
            Self::ENTITY => mem::size_of::<c_int>(),
            Self::CLASSPTR => mem::size_of::<c_int>(),
            Self::EHANDLE => mem::size_of::<c_int>(),
            Self::EVARS => mem::size_of::<c_int>(),
            Self::EDICT => mem::size_of::<c_int>(),
            Self::VECTOR => mem::size_of::<c_float>() * 3,
            Self::POSITION_VECTOR => mem::size_of::<c_float>() * 3,
            Self::POINTER => mem::size_of::<*const c_int>(),
            Self::INTEGER => mem::size_of::<c_int>(),
            // #ifdef GNUC
            Self::FUNCTION => mem::size_of::<*const c_int>() * 2,
            // #else
            // Self::FUNCTION         => mem::size_of::<*const c_int>(),
            // #endif
            Self::BOOLEAN => mem::size_of::<c_int>(),
            Self::SHORT => mem::size_of::<c_short>(),
            Self::CHARACTER => mem::size_of::<c_char>(),
            Self::TIME => mem::size_of::<c_float>(),
            Self::MODELNAME => mem::size_of::<c_int>(),
            Self::SOUNDNAME => mem::size_of::<c_int>(),
            Self::TYPECOUNT => unreachable!(),
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
    SizeOverflow,
}

impl fmt::Display for SaveError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => fmt.write_str("empty"),
            Self::Overflow => fmt.write_str("overflow"),
            Self::SizeOverflow => fmt.write_str("overflow of field data size"),
        }
    }
}

pub type SaveResult<T, E = SaveError> = core::result::Result<T, E>;

/// Used to describe struct fields to save and restore from the save file.
///
/// # Safety
///
/// Behavior is undefined if any of the following conditions are violated:
///
/// * `fieldType` is not match the field type.
/// * `fieldOffset` is not match the offset to the field in struct.
/// * `fieldSize` is not match the length of the array field.
pub unsafe trait SaveFields {
    const SAVE_NAME: &'static CStr;

    /// Field descriptions.
    ///
    /// Use [define_fields] macro to generate the array.
    const SAVE_FIELDS: &'static [TYPEDESCRIPTION];
}

unsafe impl SaveFields for entvars_s {
    const SAVE_NAME: &'static CStr = c"ENTVARS";

    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![
        classname => unsafe FieldType::STRING,
        :global globalname => unsafe FieldType::STRING,
        origin => unsafe FieldType::POSITION_VECTOR,
        oldorigin => unsafe FieldType::POSITION_VECTOR,
        velocity,
        basevelocity,
        movedir,
        angles,
        avelocity,
        punchangle,
        v_angle,
        fixangle,
        idealpitch,
        pitch_speed,
        ideal_yaw,
        yaw_speed,
        modelindex,
        :global model => unsafe FieldType::MODELNAME,
        viewmodel => unsafe FieldType::MODELNAME,
        weaponmodel => unsafe FieldType::MODELNAME,
        absmin => unsafe FieldType::POSITION_VECTOR,
        absmax => unsafe FieldType::POSITION_VECTOR,
        :global mins,
        :global maxs,
        :global size,
        ltime => unsafe FieldType::TIME,
        nextthink => unsafe FieldType::TIME,
        solid,
        movetype,
        skin,
        body,
        effects,
        gravity,
        friction,
        light_level,
        frame,
        scale,
        sequence,
        animtime => unsafe FieldType::TIME,
        framerate,
        controller,
        blending,
        rendermode,
        renderamt,
        rendercolor,
        renderfx,
        health,
        frags,
        weapons,
        takedamage,
        deadflag,
        view_ofs,
        button,
        impulse,
        chain,
        dmg_inflictor,
        enemy,
        aiment,
        owner,
        groundentity,
        spawnflags,
        flags,
        colormap,
        team,
        max_health,
        teleport_time => unsafe FieldType::TIME,
        armortype,
        armorvalue,
        waterlevel,
        watertype,
        :global target => unsafe FieldType::STRING,
        :global targetname => unsafe FieldType::STRING,
        netname => unsafe FieldType::STRING,
        message => unsafe FieldType::STRING,
        dmg_take,
        dmg_save,
        dmg,
        dmgtime => unsafe FieldType::TIME,
        noise => unsafe FieldType::SOUNDNAME,
        noise1 => unsafe FieldType::SOUNDNAME,
        noise2 => unsafe FieldType::SOUNDNAME,
        noise3 => unsafe FieldType::SOUNDNAME,
        speed,
        air_finished => unsafe FieldType::TIME,
        pain_finished => unsafe FieldType::TIME,
        radsuit_finished => unsafe FieldType::TIME,
    ];
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

    pub fn as_slice(&self) -> &'a [u8] {
        if !self.data.pCurrentData.is_null() {
            let data = self.data.pCurrentData.cast();
            let len = self.available();
            unsafe { slice::from_raw_parts(data, len) }
        } else {
            &[]
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        if !self.data.pCurrentData.is_null() {
            let data = self.data.pCurrentData.cast();
            let len = self.available();
            unsafe { slice::from_raw_parts_mut(data, len) }
        } else {
            &mut []
        }
    }

    pub fn token_hash(&mut self, token: &CStr) -> Token {
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
                return Token::new(index as u16);
            }
        }

        error!("Save::token_hash is COMPLETELY FULL!");
        Token::new(0)
    }

    pub fn advance(&mut self, len: usize) -> SaveResult<()> {
        let new_size = self.data.size + len as c_int;
        if new_size < self.data.bufferSize {
            self.data.pCurrentData = self.data.pCurrentData.wrapping_add(len);
            self.data.size += len as c_int;
            Ok(())
        } else {
            Err(SaveError::Overflow)
        }
    }

    pub fn entity_index(&self, ent: *mut edict_s) -> Option<usize> {
        if !ent.is_null() {
            self.table().iter().position(|i| i.pent == ent)
        } else {
            None
        }
    }

    pub fn entity_from_index(&self, index: i32) -> *mut edict_s {
        if index < 0 {
            return ptr::null_mut();
        }
        self.table()
            .iter()
            .find(|i| i.id == index)
            .map_or(ptr::null_mut(), |i| i.pent)
    }

    pub fn entity_flags_set(&mut self, index: usize, flags: c_int) -> c_int {
        if let Some(i) = self.table_mut().get_mut(index) {
            i.flags |= flags;
            i.flags
        } else {
            0
        }
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

    unsafe fn read_field(
        &mut self,
        base_data: *mut u8,
        fields: &[TYPEDESCRIPTION],
        start_field: usize,
        name: &CStrThin,
        src: &mut Cursor,
    ) -> SaveResult<usize> {
        let engine = self.engine;
        for i in 0..fields.len() {
            let field_index = (i + start_field) % fields.len();
            let field = &fields[field_index];
            let field_name = unsafe { CStrThin::from_ptr(field.fieldName) };
            if !field_name.eq_ignore_case(name) {
                continue;
            }

            if self.global && field.flags().intersects(FtypeDesc::GLOBAL) {
                // skip global entity
                return Ok(field_index);
            }

            let field_type = field.field_type();
            let count = field.fieldSize as usize;
            let dst_ptr = base_data.wrapping_add(field.fieldOffset as usize);
            let dst_len = field_type.size() * count;
            let dst_slice = unsafe { slice::from_raw_parts_mut(dst_ptr.cast::<u8>(), dst_len) };
            let mut dst = CursorMut::new(dst_slice);

            match field_type {
                FieldType::CHARACTER => {
                    dst.write(src.as_slice())?;
                }
                FieldType::SHORT => {
                    for _ in 0..count {
                        dst.write_u16_ne(src.read_u16_le()?)?;
                    }
                }
                FieldType::INTEGER => {
                    for _ in 0..count {
                        dst.write_u32_ne(src.read_u32_le()?)?;
                    }
                }
                FieldType::FLOAT => {
                    for _ in 0..count {
                        dst.write_f32_ne(src.read_f32_le()?)?;
                    }
                }
                FieldType::TIME => {
                    let time = self.data.time();
                    for _ in 0..count {
                        dst.write_f32_ne(src.read_f32_le()? + time)?;
                    }
                }
                FieldType::VECTOR => {
                    for _ in 0..count {
                        dst.write_f32_ne(src.read_f32_le()?)?;
                        dst.write_f32_ne(src.read_f32_le()?)?;
                        dst.write_f32_ne(src.read_f32_le()?)?;
                    }
                }
                FieldType::POSITION_VECTOR => {
                    let offset = self.data.use_landmark_offset().unwrap_or_default();
                    for _ in 0..count {
                        dst.write_f32_ne(src.read_f32_le()? + offset[0])?;
                        dst.write_f32_ne(src.read_f32_le()? + offset[1])?;
                        dst.write_f32_ne(src.read_f32_le()? + offset[2])?;
                    }
                }
                FieldType::EDICT => {
                    for _ in 0..count {
                        let index = src.read_i32_le()?;
                        let ent = self.data.entity_from_index(index);
                        dst.write_usize_ne(ent as usize)?;
                    }
                }
                FieldType::MODELNAME | FieldType::SOUNDNAME | FieldType::STRING => {
                    for name in src.as_slice().split_inclusive(|&i| i == b'\0') {
                        let name = unsafe { CStrThin::from_ptr(name.as_ptr().cast()) };
                        let mut index = 0;
                        if !name.is_empty() {
                            index = engine.new_map_string(name).index();
                            if self.precache {
                                match field_type {
                                    FieldType::MODELNAME => {
                                        engine.precache_model(name);
                                    }
                                    FieldType::SOUNDNAME => {
                                        engine.precache_sound(name);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        dst.write_i32_ne(index)?;
                    }
                }
                _ => warn!("unimplemented field({field_type:?}) read for {name:?}"),
            }

            if dst.capacity() != 0 {
                warn!("field {name:?}({field_type:?}) was partially restored");
            }

            return Ok(field_index);
        }

        Ok(start_field)
    }

    /// Read struct fields from a save file.
    ///
    /// # Safety
    ///
    /// * `base_data` must be non-null.
    /// * Field descriptions must be valid for the given `base_data` type.
    pub unsafe fn read_fields_raw(
        &mut self,
        name: &CStr,
        base_data: *mut u8,
        fields: &[TYPEDESCRIPTION],
    ) -> SaveResult<()> {
        let mut src = Cursor::new(self.data.as_slice());

        let header = src.read_header()?;
        assert_eq!(header.size(), mem::size_of::<u32>() as u16);
        if header.token() != self.data.token_hash(name) {
            return Ok(());
        }
        let field_count = src.read_u32_le()?;

        for i in fields.iter() {
            if self.global && i.flags().intersects(FtypeDesc::GLOBAL) {
                continue;
            }
            let data = base_data.wrapping_add(i.fieldOffset as usize);
            let len = i.fieldSize as usize * i.field_type().size();
            unsafe {
                ptr::write_bytes(data, 0, len);
            }
        }

        let mut last_field = 0;
        for _ in 0..field_count {
            let field = src.read_field()?;
            let token = self.data.data.tokens_mut()[field.token().to_usize()];
            let token = unsafe { CStrThin::from_ptr(token) };
            let data = &mut Cursor::new(field.data());
            last_field = unsafe { self.read_field(base_data, fields, last_field, token, data)? };
            last_field += 1;
        }

        self.data.advance(src.offset())
    }

    pub fn read_fields<T: SaveFields>(&mut self, value: &mut T) -> SaveResult<()> {
        let base_data = value as *mut T as *mut u8;
        unsafe { self.read_fields_raw(T::SAVE_NAME, base_data, T::SAVE_FIELDS) }
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

    /// Write struct fields to a save file.
    ///
    /// # Safety
    ///
    /// * `base_data` must be non-null.
    /// * Field descriptions must be valid for the given `base_data` type.
    pub unsafe fn write_fields_raw(
        &mut self,
        name: &CStr,
        base_data: *const u8,
        fields: &[TYPEDESCRIPTION],
    ) -> SaveResult<()> {
        if self.data.data.pCurrentData.is_null() {
            return Ok(());
        }

        let engine = self.engine;
        let dst_slice = {
            let data = self.data.data.pCurrentData.cast();
            let len = self.data.available();
            unsafe { slice::from_raw_parts_mut(data, len) }
        };
        let mut dst = CursorMut::new(dst_slice);
        let header_offset = dst.skip(2 * mem::size_of::<u16>() + mem::size_of::<u32>())?;
        let mut field_count = 0;
        for field in fields {
            let field_type = field.field_type();
            let count = field.fieldSize as usize;
            let src_ptr = base_data.wrapping_add(field.fieldOffset as usize);
            let src_len = field_type.size() * count;
            let src_slice = unsafe { slice::from_raw_parts(src_ptr, src_len) };
            let mut src = Cursor::new(src_slice);

            if src.as_slice().iter().all(|&i| i == 0) {
                continue;
            }

            let size_offset = dst.skip(mem::size_of::<u16>())?;
            let field_name = unsafe { CStr::from_ptr(field.fieldName) };
            dst.write_token(self.data.token_hash(field_name))?;

            let data_offset = dst.offset();
            match field_type {
                FieldType::CHARACTER => {
                    dst.write(src.as_slice())?;
                }
                FieldType::SHORT => {
                    for _ in 0..count {
                        dst.write_u16_le(src.read_u16_ne()?)?;
                    }
                }
                FieldType::INTEGER => {
                    for _ in 0..count {
                        dst.write_u32_le(src.read_u32_ne()?)?;
                    }
                }
                FieldType::FLOAT => {
                    for _ in 0..count {
                        dst.write_f32_le(src.read_f32_ne()?)?;
                    }
                }
                FieldType::TIME => {
                    let time = self.data.time();
                    for _ in 0..count {
                        dst.write_f32_le(src.read_f32_ne()? - time)?;
                    }
                }
                FieldType::VECTOR => {
                    for _ in 0..count {
                        dst.write_f32_le(src.read_f32_ne()?)?;
                        dst.write_f32_le(src.read_f32_ne()?)?;
                        dst.write_f32_le(src.read_f32_ne()?)?;
                    }
                }
                FieldType::POSITION_VECTOR => {
                    let offset = self.data.use_landmark_offset().unwrap_or_default();
                    for _ in 0..count {
                        dst.write_f32_le(src.read_f32_ne()? - offset[0])?;
                        dst.write_f32_le(src.read_f32_ne()? - offset[1])?;
                        dst.write_f32_le(src.read_f32_ne()? - offset[2])?;
                    }
                }
                FieldType::EDICT => {
                    for _ in 0..count {
                        let ent = src.read_usize_ne()? as *mut edict_s;
                        let index = self.data.entity_index(ent).map_or(-1, |i| i as i32);
                        dst.write_i32_le(index)?;
                    }
                }
                FieldType::MODELNAME | FieldType::SOUNDNAME | FieldType::STRING => {
                    for _ in 0..count {
                        let index = src.read_i32_ne()?;
                        if let Some(name) = MapString::from_index(engine, index) {
                            dst.write(name.to_bytes_with_nul())?;
                        } else {
                            dst.write_u8(0)?;
                        }
                    }
                }
                _ => warn!("unimplemented field({field_type:?}) write for {name:?}.{field_name:?}"),
            }
            let data_size =
                u16::try_from(dst.offset() - data_offset).map_err(|_| SaveError::SizeOverflow)?;

            // write the actual field data size
            dst.write_at(size_offset, |dst| dst.write_u16_le(data_size))?;

            field_count += 1;
        }

        // write the header
        dst.write_at(header_offset, |dst| {
            dst.write_u16_le(mem::size_of::<u32>() as u16)?;
            dst.write_token(self.data.token_hash(name))?;
            dst.write_u32_le(field_count)
        })?;

        self.data.advance(dst.offset() - header_offset)
    }

    pub fn write_fields<T: SaveFields>(&mut self, value: &T) -> SaveResult<()> {
        let base_data = value as *const T as *const u8;
        unsafe { self.write_fields_raw(T::SAVE_NAME, base_data, T::SAVE_FIELDS) }
    }
}
