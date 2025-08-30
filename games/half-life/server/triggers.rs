use core::{ffi::c_int, ptr};

use csz::{cstr, CStrArray, CStrThin};
use sv::{
    consts::{FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, SOLID_TRIGGER},
    engine, globals, globals_mut,
    macros::define_field,
    raw::{
        edict_s, entvars_s, string_t, vec3_t, Effects, FieldType, KeyValueData, MoveType,
        LEVELLIST, SAVERESTOREDATA, TYPEDESCRIPTION,
    },
};

use crate::{
    entity::{impl_cast, Entity, EntityVars, ObjectCaps},
    gamerules::game_rules,
    macros::link_entity,
    private_data::Private,
    save::{self, SaveRestore},
    todo::link_entity_stub,
};

const MAP_NAME_MAX: usize = 32;

link_entity_stub! {
    trigger_auto,
    trigger_autosave,
    // trigger_changelevel,
    trigger_hurt,
    trigger_multiple,
    trigger_once,
    trigger_relay,
}

pub struct ChangeLevel {
    vars: *mut entvars_s,
    map_name: CStrArray<MAP_NAME_MAX>,
    landmark_name: CStrArray<MAP_NAME_MAX>,
    target: string_t,
    change_target_delay: f32,
}

impl ChangeLevel {
    fn new(vars: *mut entvars_s) -> Self {
        Self {
            vars,
            map_name: CStrArray::new(),
            landmark_name: CStrArray::new(),
            target: string_t::null(),
            change_target_delay: 0.0,
        }
    }

    fn init_trigger(&mut self) {
        let ev = self.vars_mut();
        if ev.angles != vec3_t::ZERO {
            set_move_dir(ev);
        }
        ev.solid = SOLID_TRIGGER;
        ev.movetype = MoveType::None;
        let engine = engine();
        engine.set_model(
            unsafe { &mut *ev.pContainingEntity },
            globals().string(ev.model),
        );
        if engine.get_cvar_float(c"showtriggers") == 0.0 {
            ev.effects.insert(Effects::NODRAW);
        }
    }

    fn change_level_now(&mut self, _other: &mut dyn Entity) {
        assert!(!self.map_name.is_empty());

        if game_rules().is_some_and(|rules| rules.is_deathmatch()) {
            return;
        }

        let ev = unsafe { &mut *self.vars };
        if globals().time == ev.dmgtime {
            return;
        }

        ev.dmgtime = globals().time;

        let landmark = find_landmark(self.landmark_name.as_thin());
        let engine = engine();
        let mut next_spot = cstr!("");
        if !engine.is_null_ent(landmark) {
            next_spot = self.landmark_name.as_thin();
            globals_mut().vecLandmarkOffset = unsafe { (*landmark).v.origin };
        }

        info!("CHANGE LEVEL: {:?} {next_spot:?}", self.map_name);
        engine.change_level(&self.map_name, next_spot);
    }
}

impl_cast!(ChangeLevel);

impl EntityVars for ChangeLevel {
    fn vars_ptr(&self) -> *mut entvars_s {
        self.vars
    }
}

impl Entity for ChangeLevel {
    fn object_caps(&self) -> ObjectCaps {
        ObjectCaps::NONE
    }

    fn restore(&mut self, _restore: &mut SaveRestore) -> save::Result<()> {
        Ok(())
    }

    fn key_value(&mut self, data: &mut KeyValueData) {
        let name = unsafe { CStrThin::from_ptr(data.szKeyName) };
        let value = unsafe { CStrThin::from_ptr(data.szValue) };

        if name == c"map" {
            if value.to_bytes().len() >= MAP_NAME_MAX {
                error!("Map name {value:?} too long ({MAP_NAME_MAX} chars)");
            }
            self.map_name.cursor().write_c_str(value).unwrap();
            data.fHandled = 1;
        } else if name == c"landmark" {
            if value.to_bytes().len() >= MAP_NAME_MAX {
                error!("Landmark name {value:?} too long ({MAP_NAME_MAX} chars)");
            }
            self.landmark_name.cursor().write_c_str(value).unwrap();
            data.fHandled = 1;
        } else if name == c"changetarget" {
            self.target = engine().alloc_string(value);
            data.fHandled = 1;
        } else if name == c"changedelay" {
            let s = value.to_str().ok();
            self.change_target_delay = s.and_then(|s| s.parse().ok()).unwrap_or(0.0);
            data.fHandled = 1;
        } else {
            debug!("TODO: ChangeLevel::key_value({name:?}, {value:?})");
        }
    }

    fn spawn(&mut self) -> bool {
        if self.map_name.is_empty() {
            info!("A trigger_changelevel does not have a map");
        }

        if self.landmark_name.is_empty() {
            info!(
                "A trigger_changelevel to {:?} does not have a landmark",
                self.map_name
            );
        }

        if !self.target.is_null() {
            // TODO: use target name
        }

        self.init_trigger();

        const SF_CHANGELEVEL_USEONLY: c_int = 0x0002;
        if self.vars().spawnflags & SF_CHANGELEVEL_USEONLY != 0 {
            // TODO: set touch
        }

        true
    }

    fn touch(&mut self, other: &mut dyn Entity) {
        let classname = globals().string(other.vars().classname);
        if classname == c"player" {
            self.change_level_now(other);
        }
    }

    fn fields(&self) -> &'static [TYPEDESCRIPTION] {
        const FIELDS: &[TYPEDESCRIPTION] = &[
            define_field!(ChangeLevel, map_name, FieldType::CHARACTER, MAP_NAME_MAX),
            define_field!(
                ChangeLevel,
                landmark_name,
                FieldType::CHARACTER,
                MAP_NAME_MAX
            ),
            define_field!(ChangeLevel, target, FieldType::STRING),
            define_field!(ChangeLevel, change_target_delay, FieldType::FLOAT),
        ];
        FIELDS
    }
}

link_entity!(trigger_changelevel, ChangeLevel::new);

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
            .any(|i| i.pentLandmark == landmark || i.mapName == *map_name)
    {
        return false;
    }
    level_list[count]
        .mapName
        .cursor()
        .write_c_str(map_name)
        .unwrap();
    level_list[count]
        .landmarkName
        .cursor()
        .write_c_str(landmark_name)
        .unwrap();
    level_list[count].pentLandmark = landmark;
    level_list[count].vecLandmarkOrigin = unsafe { (*landmark).v.origin };
    true
}

fn set_move_dir(ev: &mut entvars_s) {
    if ev.angles == vec3_t::new(0.0, -1.0, 0.0) {
        ev.movedir = vec3_t::new(0.0, 0.0, 1.0);
    } else if ev.angles == vec3_t::new(0.0, -2.0, 0.0) {
        ev.movedir = vec3_t::new(0.0, 0.0, -1.0);
    } else {
        engine().make_vectors(ev.angles);
        ev.movedir = globals().v_forward;
    }
    ev.angles = vec3_t::ZERO;
}

fn find_landmark(landmark_name: &CStrThin) -> *mut edict_s {
    engine()
        .find_ent_by_targetname_iter(landmark_name)
        .find(|&ent| {
            let classname = globals().string(unsafe { (*ent).v.classname });
            classname == c"info_landmark"
        })
        .unwrap_or(ptr::null_mut())
}

fn in_transition_volume(ent: *mut edict_s, volume_name: &CStrThin) -> bool {
    let mut ent = unsafe { &*ent }.private().unwrap();

    if ent.object_caps().intersects(ObjectCaps::FORCE_TRANSITION) {
        return true;
    }

    if ent.vars().movetype == MoveType::Follow && !ent.vars().aiment.is_null() {
        ent = unsafe { &*ent.vars().aiment }.private().unwrap();
    }

    let engine = engine();
    let mut ent_volume = engine.find_ent_by_target_name(ptr::null(), volume_name);
    while !engine.is_null_ent(ent_volume) {
        if let Some(volume) = unsafe { &*ent_volume }.private() {
            if volume.is_classname(c"trigger_transition".into()) && volume.intersects(&**ent) {
                return true;
            }
        }
        ent_volume = engine.find_ent_by_target_name(ent_volume, volume_name);
    }

    false
}

pub fn build_change_list(level_list: &mut [LEVELLIST]) -> usize {
    const MAX_ENTITY: usize = 512;

    let engine = engine();
    let mut ent = engine.find_ent_by_classname(ptr::null(), c"trigger_changelevel");
    if engine.is_null_ent(ent) {
        return 0;
    }
    let mut count = 0;
    while !engine.is_null_ent(ent) {
        if let Some(trigger) = unsafe { &mut *ent }.downcast_mut::<ChangeLevel>() {
            let map_name = trigger.map_name.as_thin();
            let landmark_name = trigger.landmark_name.as_thin();
            let landmark = find_landmark(landmark_name);
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

    if !globals().pSaveData.is_null() {
        let save_data = unsafe { &mut *globals().pSaveData.cast::<SAVERESTOREDATA>() };
        if !save_data.table().is_empty() {
            let mut save_helper = SaveRestore::new(save_data);
            for (i, level) in level_list.iter().enumerate().take(count) {
                let mut ent_count = 0;
                let mut ent_list = [ptr::null_mut(); MAX_ENTITY];
                let mut ent_flags = [0; MAX_ENTITY];

                let mut ent = engine.entities_in_pvs(level.pentLandmark);
                while !engine.is_null_ent(ent) {
                    if let Some(entity) = unsafe { &mut *ent }.private_mut() {
                        let caps = entity.object_caps();
                        if !caps.intersects(ObjectCaps::DONT_SAVE) {
                            let mut flags = 0;

                            if caps.intersects(ObjectCaps::ACROSS_TRANSITION) {
                                flags |= FENTTABLE_MOVEABLE;
                            }
                            if !entity.vars().globalname.is_null() && !entity.is_dormant() {
                                flags |= FENTTABLE_GLOBAL;
                            }
                            if flags != 0 {
                                ent_list[ent_count] = entity.ent_mut();
                                ent_flags[ent_count] = flags;
                                ent_count += 1;
                            }
                        }
                    }
                    ent = unsafe { (*ent).v.chain };
                }

                for j in 0..ent_count {
                    let landmark_name = level.landmarkName.as_thin();
                    if ent_flags[j] != 0 && in_transition_volume(ent_list[j], landmark_name) {
                        if let Some(index) = save_helper.entity_index(ent_list[j]) {
                            save_helper.entity_flags_set(index, ent_flags[j] | (1 << i));
                        }
                    }
                }
            }
        }
    }

    count
}
