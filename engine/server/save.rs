mod cursor;
mod macros;
mod save_restore_data;

use core::{
    ffi::{c_char, c_float, c_int, c_short, CStr},
    fmt, mem, ptr, slice, str,
};

use alloc::{ffi::CString, string::String, vec::Vec};
use bitflags::bitflags;
use csz::{CStrArray, CStrThin};
use xash3d_shared::{
    ffi::{
        self,
        common::{string_t, vec3_t},
        server::{edict_s, entvars_s, TYPEDESCRIPTION},
    },
    macros::define_enum_for_primitive,
    utils::cstr_or_none,
};

use crate::{
    engine::ServerEngineRef,
    entity::{Entity, EntityOffset},
    str::MapString,
    time::MapTime,
};

pub use xash3d_server_derive::{Restore, Save};

pub use self::cursor::*;
pub use self::macros::*;
pub use self::save_restore_data::*;

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
                $field_type.host_size(),
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
    FieldType::TIME => MapTime,

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

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum FieldType: ffi::server::FIELDTYPE {
        FLOAT(ffi::server::_fieldtypes_FIELD_FLOAT),
        STRING(ffi::server::_fieldtypes_FIELD_STRING),
        ENTITY(ffi::server::_fieldtypes_FIELD_ENTITY),
        CLASSPTR(ffi::server::_fieldtypes_FIELD_CLASSPTR),
        EHANDLE(ffi::server::_fieldtypes_FIELD_EHANDLE),
        EVARS(ffi::server::_fieldtypes_FIELD_EVARS),
        EDICT(ffi::server::_fieldtypes_FIELD_EDICT),
        VECTOR(ffi::server::_fieldtypes_FIELD_VECTOR),
        #[allow(non_camel_case_types)]
        POSITION_VECTOR(ffi::server::_fieldtypes_FIELD_POSITION_VECTOR),
        POINTER(ffi::server::_fieldtypes_FIELD_POINTER),
        INTEGER(ffi::server::_fieldtypes_FIELD_INTEGER),
        FUNCTION(ffi::server::_fieldtypes_FIELD_FUNCTION),
        BOOLEAN(ffi::server::_fieldtypes_FIELD_BOOLEAN),
        SHORT(ffi::server::_fieldtypes_FIELD_SHORT),
        CHARACTER(ffi::server::_fieldtypes_FIELD_CHARACTER),
        TIME(ffi::server::_fieldtypes_FIELD_TIME),
        /// An engine string that is a model name (needs precache).
        MODELNAME(ffi::server::_fieldtypes_FIELD_MODELNAME),
        /// An engine string that is a sound name (needs precache).
        SOUNDNAME(ffi::server::_fieldtypes_FIELD_SOUNDNAME),
    }
}

impl FieldType {
    pub const fn host_size(&self) -> usize {
        use core::mem::size_of;

        match self {
            Self::FLOAT => size_of::<c_float>(),
            Self::STRING => size_of::<string_t>(),
            Self::ENTITY => size_of::<EntityOffset>(),
            Self::CLASSPTR => size_of::<*const dyn Entity>(),
            // TODO: define EntityHandle
            Self::EHANDLE => todo!(), // size_of::<EntityHandle>(),
            Self::EVARS => size_of::<*const entvars_s>(),
            Self::EDICT => size_of::<*const edict_s>(),
            Self::VECTOR => size_of::<vec3_t>(),
            Self::POSITION_VECTOR => size_of::<vec3_t>(),
            Self::POINTER => size_of::<*const c_int>(),
            Self::INTEGER => size_of::<c_int>(),
            Self::FUNCTION => size_of::<fn()>(),
            Self::BOOLEAN => size_of::<c_int>(),
            Self::SHORT => size_of::<c_short>(),
            Self::CHARACTER => size_of::<c_char>(),
            Self::TIME => size_of::<c_float>(),
            Self::MODELNAME => size_of::<c_int>(),
            Self::SOUNDNAME => size_of::<c_int>(),
        }
    }

    pub const fn save_size(&self) -> usize {
        use core::mem::size_of;

        match self {
            Self::FLOAT => size_of::<c_float>(),
            Self::STRING => size_of::<string_t>(),
            Self::ENTITY => size_of::<c_int>(),
            Self::CLASSPTR => size_of::<c_int>(),
            Self::EHANDLE => size_of::<c_int>(),
            Self::EVARS => size_of::<c_int>(),
            Self::EDICT => size_of::<c_int>(),
            Self::VECTOR => size_of::<vec3_t>(),
            Self::POSITION_VECTOR => size_of::<vec3_t>(),
            Self::POINTER => size_of::<*const c_int>(),
            Self::INTEGER => size_of::<c_int>(),
            Self::FUNCTION => size_of::<fn()>(),
            Self::BOOLEAN => size_of::<c_int>(),
            Self::SHORT => size_of::<c_short>(),
            Self::CHARACTER => size_of::<c_char>(),
            Self::TIME => size_of::<c_float>(),
            Self::MODELNAME => size_of::<c_int>(),
            Self::SOUNDNAME => size_of::<c_int>(),
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
    InvalidEnum,
    InvalidNumber,
    InvalidString,
}

impl fmt::Display for SaveError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => fmt.write_str("empty"),
            Self::Overflow => fmt.write_str("overflow"),
            Self::SizeOverflow => fmt.write_str("overflow of field data size"),
            Self::InvalidEnum => fmt.write_str("invalid enum"),
            Self::InvalidNumber => fmt.write_str("invalid numder"),
            Self::InvalidString => fmt.write_str("invalid string"),
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

pub struct SaveReader {
    engine: ServerEngineRef,
    global: bool,
    precache: bool,
}

impl SaveReader {
    pub fn new(engine: ServerEngineRef) -> Self {
        Self {
            engine,
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

    /// Read struct fields from a save file.
    ///
    /// # Safety
    ///
    /// * `base_data` must be non-null.
    /// * Field descriptions must be valid for the given `base_data` type.
    pub unsafe fn read_fields_raw(
        &mut self,
        save_data: &mut SaveRestoreData,
        name: &CStr,
        base_data: *mut u8,
        fields: &[TYPEDESCRIPTION],
    ) -> SaveResult<()> {
        let (buffer, state) = save_data.split_mut();
        let mut state = RestoreState::new(self.engine, state);
        state.set_precache(self.precache);
        state.set_global(self.global);
        let mut cur = Cursor::new(buffer.as_slice());
        let start_offset = cur.offset();
        let res = unsafe { read_fields_raw(&state, &mut cur, name, base_data, fields) };
        let size = cur.offset() - start_offset;
        buffer.advance(size)?;
        res
    }

    pub fn read_fields<T: SaveFields>(
        &mut self,
        save_data: &mut SaveRestoreData,
        value: &mut T,
    ) -> SaveResult<()> {
        let base_data = value as *mut T as *mut u8;
        unsafe { self.read_fields_raw(save_data, T::SAVE_NAME, base_data, T::SAVE_FIELDS) }
    }
}

pub struct SaveWriter {
    engine: ServerEngineRef,
}

impl SaveWriter {
    pub fn new(engine: ServerEngineRef) -> Self {
        Self { engine }
    }

    /// Write struct fields to a save file.
    ///
    /// # Safety
    ///
    /// * `base_data` must be non-null.
    /// * Field descriptions must be valid for the given `base_data` type.
    pub unsafe fn write_fields_raw(
        &mut self,
        save_data: &mut SaveRestoreData,
        name: &'static CStr,
        base_data: *const u8,
        fields: &[TYPEDESCRIPTION],
    ) -> SaveResult<()> {
        let (buffer, data) = save_data.split_mut();
        let mut state = SaveState::new(self.engine, data);
        let mut cur = CursorMut::new(buffer.as_slice_mut());
        let start_offset = cur.offset();
        unsafe {
            write_fields_raw(&mut state, &mut cur, name, base_data, fields)?;
        }
        let size = cur.offset() - start_offset;
        buffer.advance(size)
    }

    pub fn write_fields<T: SaveFields>(
        &mut self,
        save_data: &mut SaveRestoreData,
        value: &T,
    ) -> SaveResult<()> {
        let base_data = value as *const T as *const u8;
        unsafe { self.write_fields_raw(save_data, T::SAVE_NAME, base_data, T::SAVE_FIELDS) }
    }
}

/// Read struct fields from a save file.
///
/// # Safety
///
/// * `base_data` must be non-null.
/// * Field descriptions must be valid for the given `base_data` type.
pub unsafe fn read_fields_raw(
    state: &RestoreState,
    src: &mut Cursor,
    name: &CStr,
    base_data: *mut u8,
    fields: &[TYPEDESCRIPTION],
) -> SaveResult<()> {
    let engine = state.engine();
    let header = src.read_header()?;
    assert_eq!(header.size(), mem::size_of::<u32>() as u16);
    if state
        .get_token_hash(name)
        .is_some_and(|i| i != header.token())
    {
        return Ok(());
    }
    let field_count = src.read_u32_le()?;

    for i in fields.iter() {
        if state.global() && i.flags().intersects(FtypeDesc::GLOBAL) {
            continue;
        }
        let data = base_data.wrapping_add(i.fieldOffset as usize);
        let len = i.fieldSize as usize * i.field_type().save_size();
        unsafe {
            ptr::write_bytes(data, 0, len);
        }
    }

    let mut last_field = 0;
    for _ in 0..field_count {
        let field = src.read_field()?;
        let Some(name) = state.token_str(field.token()) else {
            warn!("restore: token({}) not found", field.token().to_u16());
            continue;
        };
        let src = &mut field.cursor();

        for i in 0..fields.len() {
            let field_index = (i + last_field) % fields.len();
            let field = &fields[field_index];
            let field_name = unsafe { CStrThin::from_ptr(field.fieldName) };
            if !field_name.eq_ignore_case(name) {
                continue;
            }

            if state.global() && field.flags().intersects(FtypeDesc::GLOBAL) {
                // skip global entity
                last_field = field_index;
                break;
            }

            let field_type = field.field_type();
            let count = field.fieldSize as usize;
            let dst_ptr = base_data.wrapping_add(field.fieldOffset as usize);
            let dst_len = field_type.host_size() * count;
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
                    let time = state.time();
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
                    let offset = state.use_landmark_offset().unwrap_or_default();
                    for _ in 0..count {
                        dst.write_f32_ne(src.read_f32_le()? + offset[0])?;
                        dst.write_f32_ne(src.read_f32_le()? + offset[1])?;
                        dst.write_f32_ne(src.read_f32_le()? + offset[2])?;
                    }
                }
                FieldType::EDICT => {
                    for _ in 0..count {
                        let index = src.read_i32_le()?;
                        let ent = state.entity_from_index(index);
                        dst.write_usize_ne(ent as usize)?;
                    }
                }
                FieldType::MODELNAME | FieldType::SOUNDNAME | FieldType::STRING => {
                    for name in src.as_slice().split_inclusive(|&i| i == b'\0') {
                        let name = unsafe { CStrThin::from_ptr(name.as_ptr().cast()) };
                        let mut index = 0;
                        if !name.is_empty() {
                            index = engine.new_map_string(name).index();
                            if state.precache() {
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

            if dst.avaiable() != 0 {
                warn!("field {name:?}({field_type:?}) was partially restored");
            }

            last_field = field_index;
            break;
        }

        last_field += 1;
    }

    Ok(())
}

pub fn read_fields<T: SaveFields>(
    state: &RestoreState,
    cur: &mut Cursor,
    value: &mut T,
) -> SaveResult<()> {
    let base_data = value as *mut T as *mut u8;
    unsafe { read_fields_raw(state, cur, T::SAVE_NAME, base_data, T::SAVE_FIELDS) }
}

/// Write struct fields to a save file.
///
/// # Safety
///
/// * `base_data` must be non-null.
/// * Field descriptions must be valid for the given `base_data` type.
pub unsafe fn write_fields_raw(
    state: &mut SaveState,
    dst: &mut CursorMut,
    name: &'static CStr,
    base_data: *const u8,
    fields: &[TYPEDESCRIPTION],
) -> SaveResult<()> {
    let engine = state.engine();
    let header_offset = dst.skip(2 * mem::size_of::<u16>() + mem::size_of::<u32>())?;
    let mut field_count = 0;
    for field in fields {
        let field_type = field.field_type();
        let count = field.fieldSize as usize;
        let src_ptr = base_data.wrapping_add(field.fieldOffset as usize);
        let src_len = field_type.host_size() * count;
        let src_slice = unsafe { slice::from_raw_parts(src_ptr, src_len) };
        let mut src = Cursor::new(src_slice);

        if src.as_slice().iter().all(|&i| i == 0) {
            continue;
        }

        let size_offset = dst.skip(mem::size_of::<u16>())?;
        let field_name = unsafe { CStr::from_ptr(field.fieldName) };
        dst.write_token(state.token_hash(field_name))?;

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
                let time = state.time();
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
                let offset = state.use_landmark_offset().unwrap_or_default();
                for _ in 0..count {
                    dst.write_f32_le(src.read_f32_ne()? - offset[0])?;
                    dst.write_f32_le(src.read_f32_ne()? - offset[1])?;
                    dst.write_f32_le(src.read_f32_ne()? - offset[2])?;
                }
            }
            FieldType::EDICT => {
                for _ in 0..count {
                    let ent = src.read_usize_ne()? as *mut edict_s;
                    let index = state.entity_index(ent).map_or(-1, |i| i as i32);
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
        dst.write_token(state.token_hash(name))?;
        dst.write_u32_le(field_count)
    })?;

    Ok(())
}

pub fn write_fields<T: SaveFields>(
    state: &mut SaveState,
    dst: &mut CursorMut,
    value: &T,
) -> SaveResult<()> {
    let base_data = value as *const T as *const u8;
    unsafe { write_fields_raw(state, dst, T::SAVE_NAME, base_data, T::SAVE_FIELDS) }
}

pub struct SaveState<'a> {
    engine: ServerEngineRef,
    state: &'a mut SaveRestoreState,
}

impl<'a> SaveState<'a> {
    pub fn new(engine: ServerEngineRef, state: &'a mut SaveRestoreState) -> Self {
        Self { engine, state }
    }

    pub fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    pub fn time(&self) -> f32 {
        self.state.time()
    }

    pub fn use_landmark_offset(&self) -> Option<vec3_t> {
        self.state.use_landmark_offset()
    }

    pub fn token_hash(&mut self, token: &'static CStr) -> Token {
        self.state.token_hash(token)
    }

    pub fn token_str(&self, token: Token) -> Option<&CStrThin> {
        self.state.token_str(token)
    }

    pub fn entity_index(&self, ent: *mut edict_s) -> Option<usize> {
        self.state.entity_index(ent)
    }
}

pub struct RestoreState<'a> {
    engine: ServerEngineRef,
    state: &'a mut SaveRestoreState,
    global: bool,
    precache: bool,
}

impl<'a> RestoreState<'a> {
    pub fn new(engine: ServerEngineRef, state: &'a mut SaveRestoreState) -> Self {
        Self {
            engine,
            state,
            global: false,
            precache: true,
        }
    }

    pub fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    pub fn global(&self) -> bool {
        self.global
    }

    pub fn set_global(&mut self, global: bool) {
        self.global = global;
    }

    pub fn precache(&self) -> bool {
        self.precache
    }

    pub fn set_precache(&mut self, precache: bool) {
        self.precache = precache;
    }

    pub fn time(&self) -> f32 {
        self.state.time()
    }

    pub fn use_landmark_offset(&self) -> Option<vec3_t> {
        self.state.use_landmark_offset()
    }

    pub fn token_str(&self, token: Token) -> Option<&CStrThin> {
        self.state.token_str(token)
    }

    pub fn get_token_hash(&self, str: &CStr) -> Option<Token> {
        self.state.get_token_hash(str)
    }

    pub fn entity_from_index(&self, index: i32) -> *mut edict_s {
        self.state.entity_from_index(index)
    }
}

pub trait Save {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()>;
}

pub trait Restore {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()>;
}

pub trait RestoreField: Restore {
    fn restore_field(
        &mut self,
        state: &RestoreState,
        cur: &mut Cursor,
        name: &CStr,
    ) -> SaveResult<bool>;
}

pub trait OnRestore {
    fn on_restore(&mut self);
}

macro_rules! impl_save_restore_for_num {
    ($( $ty:ty = $write:ident, $read:ident; )*) => {
        $(
            impl Save for $ty {
                fn save(
                    &self,
                    _: &mut SaveState,
                    cur: &mut CursorMut,
                ) -> SaveResult<()> {
                    cur.$write(*self)?;
                    Ok(())
                }
            }

            impl Restore for $ty {
                fn restore(
                    &mut self,
                    _: &RestoreState,
                    cur: &mut Cursor,
                ) -> SaveResult<()> {
                    *self = cur.$read()?;
                    Ok(())
                }
            }
        )*
    };
}

impl_save_restore_for_num! {
    bool = write_bool, read_bool;

    u8 = write_u8, read_u8;
    i8 = write_i8, read_i8;

    u16 = write_u16_le, read_u16_le;
    i16 = write_i16_le, read_i16_le;

    u32 = write_u32_le, read_u32_le;
    i32 = write_i32_le, read_i32_le;

    u64 = write_u64_le, read_u64_le;
    i64 = write_i64_le, read_i64_le;

    u128 = write_u128_le, read_u128_le;
    i128 = write_i128_le, read_i128_le;

    f32 = write_f32_le, read_f32_le;
    f64 = write_f64_le, read_f64_le;
}

impl<T: Save> Save for Option<T> {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        match self {
            None => {
                cur.write_u8(0)?;
            }
            Some(value) => {
                cur.write_u8(1)?;
                value.save(state, cur)?;
            }
        }
        Ok(())
    }
}

impl<T: Restore + Default> Restore for Option<T> {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        match cur.read_u8()? {
            0 => {
                *self = None;
            }
            _ => {
                let mut value = T::default();
                value.restore(state, cur)?;
                *self = Some(value);
            }
        }
        Ok(())
    }
}

impl<T: Save, E: Save> Save for Result<T, E> {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        match self {
            Err(value) => {
                cur.write_u8(0)?;
                value.save(state, cur)?;
            }
            Ok(value) => {
                cur.write_u8(1)?;
                value.save(state, cur)?;
            }
        }
        Ok(())
    }
}

impl<T: Restore + Default, E: Restore + Default> Restore for Result<T, E> {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        match cur.read_u8()? {
            0 => {
                let mut value = E::default();
                value.restore(state, cur)?;
                *self = Err(value);
            }
            _ => {
                let mut value = T::default();
                value.restore(state, cur)?;
                *self = Ok(value);
            }
        }
        Ok(())
    }
}

impl<const N: usize> Save for CStrArray<N> {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_bytes_with_size(self.to_bytes())?;
        Ok(())
    }
}

impl<const N: usize> Restore for CStrArray<N> {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        self.cursor()
            .write_bytes(bytes)
            .map_err(|_| SaveError::InvalidString)
    }
}

impl Save for CString {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_bytes_with_size(self.to_bytes())?;
        Ok(())
    }
}

impl Restore for CString {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        *self = CString::new(bytes).map_err(|_| SaveError::InvalidString)?;
        Ok(())
    }
}

impl Save for String {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_bytes_with_size(self.as_bytes())?;
        Ok(())
    }
}

impl Restore for String {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        let s = str::from_utf8(bytes).map_err(|_| SaveError::InvalidString)?;
        self.push_str(s);
        Ok(())
    }
}

impl<T: Save> Save for Vec<T> {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        let len_offset = cur.skip(mem::size_of::<u16>())?;
        for i in self {
            i.save(state, cur)?;
        }
        let size = cur.offset() - len_offset - mem::size_of::<u16>();
        let size = size.try_into().map_err(|_| SaveError::SizeOverflow)?;
        cur.write_at(len_offset, |cur| {
            cur.write_u16_le(size)?;
            Ok(())
        })
    }
}

impl<T: Restore + Default> Restore for Vec<T> {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        self.clear();
        let len = cur.read_u16_le()?.into();
        self.reserve(len);
        for _ in 0..len {
            let mut value = T::default();
            value.restore(state, cur)?;
            self.push(value);
        }
        Ok(())
    }
}
