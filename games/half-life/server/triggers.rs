use core::{
    ffi::{c_int, CStr},
    ptr,
};

use csz::{cstr, CStrArray, CStrThin};
use xash3d_server::{
    consts::{FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, SOLID_TRIGGER},
    entity::{
        delegate_entity, BaseEntity, CreateEntity, Effects, Entity, GetPrivateData, MoveType,
        ObjectCaps,
    },
    export::export_entity,
    ffi::{
        common::vec3_t,
        server::{edict_s, entvars_s, KeyValueData, LEVELLIST, TYPEDESCRIPTION},
    },
    prelude::*,
    save::{define_fields, SaveFields, SaveReader, SaveRestoreData, SaveResult, SaveWriter},
    str::MapString,
};

use crate::{
    entity::{impl_cast, Private},
    todo::export_entity_stub,
};

const MAP_NAME_MAX: usize = 32;

export_entity_stub! {
    trigger_auto,
    trigger_autosave,
    trigger_cdaudio,
    // trigger_changelevel,
    trigger_hurt,
    trigger_multiple,
    trigger_once,
    trigger_push,
    trigger_relay,
    trigger_teleport,
    trigger_transition,
}

pub struct ChangeLevel {
    base: BaseEntity,
    map_name: CStrArray<MAP_NAME_MAX>,
    landmark_name: CStrArray<MAP_NAME_MAX>,
    target: Option<MapString>,
    change_target_delay: f32,
}

unsafe impl SaveFields for ChangeLevel {
    const SAVE_NAME: &'static CStr = c"CChangeLevel";

    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] =
        &define_fields![map_name, landmark_name, target, change_target_delay,];
}

impl ChangeLevel {
    fn init_trigger(&mut self) {
        let engine = self.base.engine;
        let ev = self.vars_mut().as_raw_mut();
        if ev.angles != vec3_t::ZERO {
            set_move_dir(engine, ev);
        }
        ev.solid = SOLID_TRIGGER;
        ev.movetype = MoveType::None.into();
        engine.set_model(unsafe { &mut *ev.pContainingEntity }, &ev.model().unwrap());
        if engine.get_cvar_float(c"showtriggers") == 0.0 {
            ev.effects_mut().insert(Effects::NODRAW);
        }
    }

    fn change_level_now(&mut self, _other: &mut dyn Entity) {
        let engine = self.base.engine;
        assert!(!self.map_name.is_empty());

        if self.game_rules().is_some_and(|rules| rules.is_deathmatch()) {
            return;
        }

        let globals = &engine.globals;
        let ev = self.vars_mut().as_raw_mut();
        let time = globals.map_time_f32();
        if time == ev.dmgtime {
            return;
        }

        ev.dmgtime = time;

        let landmark = find_landmark(engine, self.landmark_name.as_thin());
        let mut next_spot = cstr!("");
        if !engine.is_null_ent(landmark) {
            next_spot = self.landmark_name.as_thin();
            unsafe {
                globals.set_landmark_offset((*landmark).v.origin);
            }
        }

        info!("CHANGE LEVEL: {:?} {next_spot:?}", self.map_name);
        engine.change_level(&self.map_name, next_spot);
    }
}

impl_cast!(ChangeLevel);

impl CreateEntity for ChangeLevel {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            map_name: CStrArray::new(),
            landmark_name: CStrArray::new(),
            target: None,
            change_target_delay: 0.0,
        }
    }
}

impl Entity for ChangeLevel {
    delegate_entity!(base not { object_caps, save, restore, key_value, spawn, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn save(&mut self, writer: &mut SaveWriter, save_data: &mut SaveRestoreData) -> SaveResult<()> {
        self.base.save(writer, save_data)?;
        writer.write_fields(save_data, self)
    }

    fn restore(
        &mut self,
        reader: &mut SaveReader,
        save_data: &mut SaveRestoreData,
    ) -> SaveResult<()> {
        self.base.restore(reader, save_data)?;
        reader.read_fields(save_data, self)
    }

    fn key_value(&mut self, data: &mut KeyValueData) {
        let engine = self.base.engine;
        let name = data.key_name();
        let value = data.value();

        if name == c"map" {
            if value.to_bytes().len() >= MAP_NAME_MAX {
                error!("Map name {value:?} too long ({MAP_NAME_MAX} chars)");
            }
            self.map_name.cursor().write_c_str(value).unwrap();
            data.set_handled(true);
        } else if name == c"landmark" {
            if value.to_bytes().len() >= MAP_NAME_MAX {
                error!("Landmark name {value:?} too long ({MAP_NAME_MAX} chars)");
            }
            self.landmark_name.cursor().write_c_str(value).unwrap();
            data.set_handled(true);
        } else if name == c"changetarget" {
            self.target = Some(engine.new_map_string(value));
            data.set_handled(true);
        } else if name == c"changedelay" {
            let s = value.to_str().ok();
            self.change_target_delay = s.and_then(|s| s.parse().ok()).unwrap_or(0.0);
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        if self.map_name.is_empty() {
            info!("A trigger_changelevel does not have a map");
        }

        if self.landmark_name.is_empty() {
            info!(
                "A trigger_changelevel to {:?} does not have a landmark",
                self.map_name
            );
        }

        if let Some(_target) = self.target {
            // TODO: use target name
        }

        self.init_trigger();

        const SF_CHANGELEVEL_USEONLY: c_int = 0x0002;
        if self.vars().as_raw().spawnflags & SF_CHANGELEVEL_USEONLY != 0 {
            // TODO: set touch
        }
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        if other.vars().classname().unwrap().as_thin() == c"player" {
            self.change_level_now(other);
        }
    }
}

export_entity!(trigger_changelevel, Private<ChangeLevel>);

fn add_transition_to_list(
    level_list: &mut [LEVELLIST],
    count: usize,
    map_name: &CStrThin,
    landmark_name: &CStrThin,
    landmark: *mut edict_s,
) -> bool {
    if landmark.is_null()
        || level_list[..count]
            .iter()
            .any(|i| i.pentLandmark == landmark || i.map_name() == map_name)
    {
        return false;
    }
    level_list[count]
        .map_name_new()
        .cursor()
        .write_c_str(map_name)
        .unwrap();
    level_list[count]
        .landmark_name_new()
        .cursor()
        .write_c_str(landmark_name)
        .unwrap();
    level_list[count].pentLandmark = landmark;
    level_list[count].vecLandmarkOrigin = unsafe { (*landmark).v.origin };
    true
}

fn set_move_dir(engine: ServerEngineRef, ev: &mut entvars_s) {
    if ev.angles == vec3_t::new(0.0, -1.0, 0.0) {
        ev.movedir = vec3_t::new(0.0, 0.0, 1.0);
    } else if ev.angles == vec3_t::new(0.0, -2.0, 0.0) {
        ev.movedir = vec3_t::new(0.0, 0.0, -1.0);
    } else {
        engine.make_vectors(ev.angles);
        ev.movedir = engine.globals.forward();
    }
    ev.angles = vec3_t::ZERO;
}

fn find_landmark(engine: ServerEngineRef, landmark_name: &CStrThin) -> *mut edict_s {
    engine
        .find_ent_by_targetname_iter(landmark_name)
        .find(|&ent| {
            let classname = unsafe { &*ent }.v.classname().unwrap();
            classname.as_thin() == c"info_landmark"
        })
        .unwrap_or(ptr::null_mut())
}

fn in_transition_volume(
    engine: ServerEngineRef,
    ent: *mut edict_s,
    volume_name: &CStrThin,
) -> bool {
    let mut ent = unsafe { &mut *ent }.get_private().unwrap().as_entity();
    if ent.object_caps().intersects(ObjectCaps::FORCE_TRANSITION) {
        return true;
    }
    if ent.vars().as_raw().movetype == MoveType::Follow.into()
        && !ent.vars().as_raw().aiment.is_null()
    {
        let aiment = unsafe { &mut *ent.vars().as_raw().aiment };
        ent = aiment.get_private().unwrap().as_entity();
    }

    let mut ent_volume = engine.find_ent_by_target_name(ptr::null_mut(), volume_name);
    while !engine.is_null_ent(ent_volume) {
        if let Some(volume) = unsafe { &mut *ent_volume }.get_entity_mut() {
            if volume.is_classname(c"trigger_transition".into()) && volume.intersects(ent) {
                return true;
            }
        }
        ent_volume = engine.find_ent_by_target_name(ent_volume, volume_name);
    }

    false
}

pub fn build_change_list(engine: ServerEngineRef, level_list: &mut [LEVELLIST]) -> usize {
    const MAX_ENTITY: usize = 512;

    let mut ent = engine.find_ent_by_classname(ptr::null_mut(), c"trigger_changelevel");
    if engine.is_null_ent(ent) {
        return 0;
    }
    let mut count = 0;
    while !engine.is_null_ent(ent) {
        let private = unsafe { &mut *ent }.get_private_mut().unwrap();
        if let Some(trigger) = private.downcast_mut::<ChangeLevel>() {
            let map_name = trigger.map_name.as_thin();
            let landmark_name = trigger.landmark_name.as_thin();
            let landmark = find_landmark(engine, landmark_name);
            if !landmark.is_null()
                && add_transition_to_list(level_list, count, map_name, landmark_name, landmark)
            {
                count += 1;
                if count >= level_list.len() {
                    break;
                }
            }
        }
        ent = engine.find_ent_by_classname(ent, c"trigger_changelevel");
    }

    if let Some(mut save_data) = engine.globals.save_data() {
        let save_data = &mut SaveRestoreData::new(unsafe { save_data.as_mut() });
        if !save_data.table().is_empty() {
            for (i, level) in level_list.iter().enumerate().take(count) {
                let mut ent_count = 0;
                let mut ent_list = [ptr::null_mut(); MAX_ENTITY];
                let mut ent_flags = [0; MAX_ENTITY];

                let mut ent = engine.entities_in_pvs(unsafe { &mut *level.pentLandmark });
                while !engine.is_null_ent(ent) {
                    if let Some(entity) = unsafe { &mut *ent }.get_entity_mut() {
                        let caps = entity.object_caps();
                        if !caps.intersects(ObjectCaps::DONT_SAVE) {
                            let mut flags = 0;

                            if caps.intersects(ObjectCaps::ACROSS_TRANSITION) {
                                flags |= FENTTABLE_MOVEABLE;
                            }
                            if entity.vars().globalname().is_some() && !entity.is_dormant() {
                                flags |= FENTTABLE_GLOBAL;
                            }
                            if flags != 0 {
                                ent_list[ent_count] = entity.as_edict_mut();
                                ent_flags[ent_count] = flags;
                                ent_count += 1;
                            }
                        }
                    }
                    ent = unsafe { (*ent).v.chain };
                }

                for j in 0..ent_count {
                    let landmark_name = level.landmark_name();
                    if ent_flags[j] != 0 && in_transition_volume(engine, ent_list[j], landmark_name)
                    {
                        if let Some(index) = save_data.entity_index(ent_list[j]) {
                            save_data.entity_flags_set(index, ent_flags[j] | (1 << i));
                        }
                    }
                }
            }
        }
    }

    count
}
