use core::{
    cmp,
    ffi::{c_char, c_float, c_int, c_short, c_uint, c_ushort, c_void, CStr},
    fmt,
    mem::{self, MaybeUninit},
    ptr, slice,
};

use csz::CStrThin;
use sv::{
    engine, globals,
    macros::define_entity_field,
    raw::{
        edict_s, entvars_s, string_t, vec3_t, FieldType, FtypeDesc, KeyValueData, MoveType,
        SAVERESTOREDATA, TYPEDESCRIPTION,
    },
    utils::array_from_slice,
};

use crate::{
    entity::ObjectCaps,
    global_state::{global_state, EntityState},
    private_data::Private,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Empty,
    Overflow,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => fmt.write_str("empty"),
            Self::Overflow => fmt.write_str("overflow"),
        }
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

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

struct Header<'a> {
    token: c_ushort,
    data: &'a [u8],
}

pub struct SaveRestore<'a> {
    pub data: &'a mut SAVERESTOREDATA,
    global: bool,
    precache: bool,
}

#[allow(dead_code)]
impl<'a> SaveRestore<'a> {
    pub fn new(data: &'a mut SAVERESTOREDATA) -> Self {
        Self {
            data,
            global: false,
            precache: true,
        }
    }

    fn precache_mode(&mut self, mode: bool) {
        self.precache = mode;
    }

    fn global_mode(&mut self, mode: bool) {
        self.global = mode;
    }

    fn size(&self) -> usize {
        self.data.size as usize
    }

    fn buffer_size(&self) -> usize {
        self.data.buffer_size as usize
    }

    fn available(&self) -> usize {
        self.buffer_size() - self.size()
    }

    fn is_empty(&self) -> bool {
        let diff = unsafe { self.data.current_data.offset_from(self.data.base_data) };
        diff >= self.buffer_size() as isize
    }

    fn buffer_rewind(&mut self, size: usize) {
        let size = cmp::min(size, self.size());
        self.data.current_data = unsafe { self.data.current_data.byte_sub(size) };
        self.data.size -= size as c_int;
    }

    fn check(&self, size: usize) -> Result<()> {
        if self.is_empty() {
            Err(Error::Empty)
        } else if size > self.available() {
            Err(Error::Overflow)
        } else {
            Ok(())
        }
    }

    fn as_slice(&self) -> &'a [u8] {
        if !self.data.current_data.is_null() {
            unsafe {
                let data = self.data.current_data.cast();
                let len = self.available();
                slice::from_raw_parts(data, len)
            }
        } else {
            &[]
        }
    }

    fn token_hash(&mut self, token: &CStr) -> c_ushort {
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

    fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
        self.check(len).map(|_| {
            let data = self.data.current_data.cast();
            let output = unsafe { slice::from_raw_parts(data, len) };
            self.data.current_data = unsafe { self.data.current_data.byte_add(output.len()) };
            self.data.size += output.len() as c_int;
            output
        })
    }

    fn read_bytes(&mut self, output: &mut [u8]) -> Result<()> {
        match self.read_slice(output.len()) {
            Ok(slice) => {
                output.copy_from_slice(slice);
                Ok(())
            }
            Err(e) => {
                output.fill(0);
                if e == Error::Empty {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut output = [0; N];
        self.read_bytes(&mut output).map(|_| output)
    }

    fn read_short(&mut self) -> Result<c_short> {
        self.read_array().map(c_short::from_le_bytes)
    }

    fn read_ushort(&mut self) -> Result<c_ushort> {
        self.read_array().map(c_ushort::from_le_bytes)
    }

    fn read_int(&mut self) -> Result<c_int> {
        self.read_array().map(c_int::from_le_bytes)
    }

    fn read_uint(&mut self) -> Result<c_uint> {
        self.read_array().map(c_uint::from_le_bytes)
    }

    fn read_header(&mut self) -> Result<Header<'a>> {
        let size = self.read_ushort()?;
        let token = self.read_ushort()?;
        let data = self.read_slice(size as usize)?;
        Ok(Header { token, data })
    }

    fn read_field(
        &mut self,
        base_data: *mut c_void,
        fields: &[TYPEDESCRIPTION],
        start_field: usize,
        name: &CStr,
        data: &[u8],
    ) -> Result<usize> {
        use FieldType as F;

        let time = self.data.time;
        let position = if self.data.use_landmark != 0 {
            self.data.landmark_offset
        } else {
            vec3_t::ZERO
        };

        for i in 0..fields.len() {
            let field_index = (i + start_field) % fields.len();
            let field = &fields[field_index];

            if field.name().is_some_and(|s| s.eq_ignore_case(name.into())) {
                if !self.global || !field.flags.intersects(FtypeDesc::GLOBAL) {
                    let ptr = unsafe { base_data.byte_add(field.fieldOffset as usize) };
                    let size = field_size(field.fieldType);
                    let count = field.fieldSize as usize;
                    let output =
                        unsafe { slice::from_raw_parts_mut(ptr.cast::<u8>(), size * count) };

                    match field.fieldType {
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
                            let engine = engine();
                            let mut iter = data.split_inclusive(|&i| i == b'\0');
                            for dst in output.chunks_mut(size) {
                                let chunk = iter.next().unwrap();
                                let str = CStr::from_bytes_with_nul(chunk).unwrap();
                                if !str.is_empty() {
                                    let id = engine.alloc_string(str);
                                    dst.copy_from_slice(&id.0.to_ne_bytes());
                                    if self.precache {
                                        match field.fieldType {
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
                                let ent = engine().entity_of_ent_index(index);
                                dst.copy_from_slice(&(ent as usize).to_ne_bytes());
                            }
                        }
                        _ => {
                            debug!(
                                "read_field({name:?}, {:?}, {})",
                                field.fieldType, field.fieldSize
                            );
                        }
                    }
                }
                return Ok(field_index);
            }
        }

        Ok(start_field)
    }

    pub fn read_fields(
        &mut self,
        name: &CStr,
        base_data: *mut c_void,
        fields: &[TYPEDESCRIPTION],
    ) -> Result<()> {
        let size = self.read_ushort()? as usize;
        assert_eq!(size, mem::size_of::<c_int>());

        let token = self.read_ushort()?;
        if token != self.token_hash(name) {
            self.buffer_rewind(2 * mem::size_of::<c_short>());
            return Ok(());
        }

        let file_count = self.read_uint()?;
        for i in fields.iter() {
            if !self.global || !i.flags.intersects(FtypeDesc::GLOBAL) {
                unsafe {
                    let data = base_data.byte_add(i.fieldOffset as usize);
                    let len = i.fieldSize as usize * field_size(i.fieldType);
                    ptr::write_bytes(data, 0, len);
                }
            }
        }

        let mut last_field = 0;
        for _ in 0..file_count {
            let header = self.read_header()?;
            let token = self.data.tokens_mut()[header.token as usize];
            let token = unsafe { CStr::from_ptr(token) };
            last_field = self.read_field(base_data, fields, last_field, token, header.data)?;
            last_field += 1;
        }

        Ok(())
    }

    pub fn read_ent_vars(&mut self, name: &CStr, ev: *mut entvars_s) -> Result<()> {
        self.read_fields(name, ev as *mut _, ENTVARS_DESCRIPTION)
    }

    pub fn write_fields(
        &mut self,
        _name: &CStr,
        _base_data: *mut c_void,
        _fields: &[TYPEDESCRIPTION],
    ) -> Result<()> {
        error!("TODO: SaveWriteFields");
        Ok(())
    }

    pub fn entity_index(&self, ent: *mut edict_s) -> Option<usize> {
        if !ent.is_null() {
            self.data.table().iter().position(|i| i.pent == ent)
        } else {
            None
        }
    }

    pub fn entity_flags_set(&mut self, index: usize, flags: u32) -> u32 {
        if let Some(i) = self.data.table_mut().get_mut(index) {
            i.flags |= flags as c_int;
            i.flags as u32
        } else {
            0
        }
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

pub fn write_fields(
    save_data: &mut SAVERESTOREDATA,
    name: &CStr,
    base_data: *mut c_void,
    fields: &[TYPEDESCRIPTION],
) {
    if let Err(err) = SaveRestore::new(save_data).write_fields(name, base_data, fields) {
        error!("save::write_fields({name:?}): {err}");
    }
}

pub fn read_fields(
    save_data: &mut SAVERESTOREDATA,
    name: &CStr,
    base_data: *mut c_void,
    fields: &[TYPEDESCRIPTION],
) {
    if let Err(err) = SaveRestore::new(save_data).read_fields(name, base_data, fields) {
        error!("save::read_fields({name:?}): {err}");
    }
}

fn entvars_key_value(ev: &mut entvars_s, data: &mut KeyValueData) {
    let key_name = unsafe { CStrThin::from_ptr(data.szKeyName) };
    let field = ENTVARS_DESCRIPTION
        .iter()
        .find(|i| i.name().unwrap().eq_ignore_case(key_name));

    if let Some(field) = field {
        let pev = ev as *mut _ as *mut u8;
        let p = unsafe { pev.offset(field.fieldOffset as isize) };
        let value = unsafe { CStr::from_ptr(data.szValue) };

        match field.fieldType {
            FieldType::MODELNAME | FieldType::SOUNDNAME | FieldType::STRING => {
                let s = engine().alloc_string(value);
                unsafe {
                    ptr::write(p.cast::<c_int>(), s.0);
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
        data.fHandled = 1;
    }
}

pub fn dispatch_key_value(ent: &mut edict_s, data: &mut KeyValueData) {
    entvars_key_value(&mut ent.v, data);

    if data.fHandled != 0 || data.szClassName.is_null() {
        return;
    }

    if let Some(ent) = ent.private_mut() {
        ent.key_value(data);
    }
}

pub fn dispatch_save(ent: &mut edict_s, save_data: &mut SAVERESTOREDATA) {
    let size = save_data.size;
    let current_index = save_data.current_index as usize;
    let table = &mut save_data.table_mut()[current_index];
    if table.pent != ent {
        error!("Entity table or index is wrong");
    }

    let Some(entity) = ent.private_mut() else {
        return;
    };
    if entity.object_caps().intersects(ObjectCaps::DONT_SAVE) {
        return;
    }

    if entity.vars().movetype == MoveType::Push {
        let delta = entity.vars().nextthink - entity.vars().ltime;
        entity.vars_mut().ltime = globals().time;
        entity.vars_mut().nextthink = entity.vars().ltime + delta;
    }

    table.location = size;
    table.classname = entity.vars().classname;

    entity.save(&mut SaveRestore::new(save_data)).unwrap();

    let table = &mut save_data.table_mut()[current_index];
    table.size = size - table.location;
}

pub fn dispatch_restore(
    mut ent: *mut edict_s,
    save_data: &mut SAVERESTOREDATA,
    global_entity: bool,
) -> c_int {
    let mut global_vars = MaybeUninit::<entvars_s>::uninit();

    if global_entity {
        let mut restore = SaveRestore::new(save_data);
        restore.precache_mode(false);
        restore
            .read_ent_vars(c"ENTVARS", global_vars.as_mut_ptr())
            .unwrap();
    }

    let mut restore = SaveRestore::new(save_data);
    let mut old_offset = vec3_t::ZERO;

    if global_entity {
        let tmp_vars = unsafe { global_vars.assume_init_mut() };
        // HACK: restore save pointers
        restore.data.size = restore.data.table()[restore.data.current_index as usize].location;
        unsafe {
            restore.data.current_data = restore.data.base_data.add(restore.data.size as usize);
        }

        let mut entities = global_state().entities.borrow_mut();
        let global = entities.find_string(tmp_vars.globalname).unwrap();
        if restore.data.current_map_name != *global.map_name() {
            return 0;
        }

        old_offset = restore.data.landmark_offset;
        if let Some(new_ent) = find_global_entity(tmp_vars.classname, tmp_vars.globalname) {
            let new_ent = unsafe { &mut *new_ent };
            restore.global_mode(true);
            restore.data.landmark_offset -= new_ent.v.mins;
            restore.data.landmark_offset += tmp_vars.mins;
            ent = new_ent;
            entities.update(unsafe { (*ent).v.globalname }, globals().mapname);
        } else {
            return 0;
        }
    }

    let Some(entity) = (unsafe { (*ent).private_mut() }) else {
        return 0;
    };
    entity.restore(&mut restore).unwrap();
    if entity.object_caps().intersects(ObjectCaps::MUST_SPAWN) {
        entity.spawn();
    } else {
        entity.precache();
    }

    let entity = unsafe { (*ent).private_mut() };
    if global_entity {
        restore.data.landmark_offset = old_offset;
        if let Some(entity) = entity {
            let origin = entity.vars().origin;
            engine().set_origin(entity.ent_mut(), origin);
            entity.override_reset();
            return 0;
        }
    } else if let Some(entity) = entity {
        if !entity.vars().globalname.is_null() {
            let mut entities = global_state().entities.borrow_mut();
            if let Some(global) = entities.find_string(entity.vars().globalname) {
                if global.is_dead() {
                    return -1;
                }
                let globals = globals();
                let map_name: &CStrThin = globals.string(globals.mapname).into();
                if map_name != global.map_name() {
                    entity.make_dormant();
                }
            } else {
                let globalname = entity.globalname();
                let classname = entity.classname();
                error!("Global entity {globalname:?} ({classname:?}) not in table!!!");
                entities.add_string(entity.vars().globalname, globals().mapname, EntityState::On);
            }
        }
    }

    0
}

fn find_global_entity(classname: string_t, globalname: string_t) -> Option<*mut edict_s> {
    let globals = globals();
    let globalname = globals.string(globalname);
    let classname = globals.string(classname);
    engine()
        .find_ent_by_globalname_iter(globalname)
        .find(|&ent| {
            if let Some(private) = unsafe { *ent }.private_mut() {
                if private.is_classname(classname) {
                    return true;
                } else {
                    debug!("Global entity found {globalname:?}, wrong class {classname:?}");
                }
            }
            false
        })
}
