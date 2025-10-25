use core::ptr;

use csz::{cstr, CStrArray, CStrThin};
use xash3d_shared::{
    entity::MoveType,
    ffi::server::{edict_s, FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, LEVELLIST},
};

use crate::{
    entities::trigger::Trigger,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityHandle,
        KeyValue, ObjectCaps,
    },
    export::export_entity_default,
    prelude::*,
    save::SaveRestoreData,
    str::MapString,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

const MAP_NAME_MAX: usize = 32;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct ChangeLevel {
    base: Trigger,
    map_name: CStrArray<MAP_NAME_MAX>,
    landmark_name: CStrArray<MAP_NAME_MAX>,
    target: Option<MapString>,
    change_target_delay: f32,
}

impl_entity_cast!(ChangeLevel);

impl CreateEntity for ChangeLevel {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
            map_name: Default::default(),
            landmark_name: Default::default(),
            target: None,
            change_target_delay: 0.0,
        }
    }
}

impl ChangeLevel {
    const SF_USE_ONLY: u32 = 1 << 1;

    fn change_level_now(&self, _other: &dyn Entity) {
        let engine = self.base.engine();
        if self.map_name.is_empty() {
            panic!("Detected problems with save/restore!!!");
        }

        if self.global_state().game_rules().is_deathmatch() {
            return;
        }

        let v = self.base.vars();
        let now = engine.globals.map_time();
        if now == v.damage_time() {
            return;
        }
        v.set_damage_time(now);

        let mut next_spot = cstr!("");
        if let Some(landmark) = find_landmark(engine, self.landmark_name.as_thin()) {
            next_spot = self.landmark_name.as_thin();
            let landmark_origin = landmark.vars().origin();
            engine.globals.set_landmark_offset(landmark_origin);
        }

        info!("CHANGE LEVEL: {:?} {next_spot:?}", self.map_name);
        engine.change_level(&self.map_name, next_spot);
    }
}

impl Entity for ChangeLevel {
    delegate_entity!(base not { key_value, spawn, touched });

    fn key_value(&mut self, data: &mut KeyValue) {
        let engine = self.base.engine();
        let value = data.value();

        match data.key_name().to_bytes() {
            b"map" => {
                if self.map_name.cursor().write_c_str(value).is_err() {
                    error!("Map name {value:?} too long ({MAP_NAME_MAX} chars)");
                }
            }
            b"landmark" => {
                if self.landmark_name.cursor().write_c_str(value).is_err() {
                    error!("Landmark name {value:?} too long ({MAP_NAME_MAX} chars)");
                }
            }
            b"changetarget" => {
                self.target = Some(engine.new_map_string(value));
            }
            b"changedelay" => {
                self.change_target_delay = data.parse_or_default();
            }
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
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

        self.base.spawn();

        if self.vars().spawn_flags() & Self::SF_USE_ONLY != 0 {
            // TODO: set touch
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if other.vars().classname().unwrap().as_thin() == c"player" {
            self.change_level_now(other);
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn add_transition_to_list(
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

fn find_landmark(engine: ServerEngineRef, landmark_name: &CStrThin) -> Option<EntityHandle> {
    engine
        .entities()
        .by_target_name(landmark_name)
        .find(|&ent| {
            ent.vars()
                .classname()
                .is_some_and(|s| s.as_thin() == c"info_landmark")
        })
        .map(|ent| ent.into())
}

fn in_transition_volume(
    engine: ServerEngineRef,
    ent: *mut edict_s,
    volume_name: &CStrThin,
) -> bool {
    let mut ent = unsafe { &mut *ent }.get_entity().unwrap();
    if ent.object_caps().intersects(ObjectCaps::FORCE_TRANSITION) {
        return true;
    }
    if let (MoveType::Follow, Some(aim)) = (ent.vars().move_type(), ent.vars().aim_entity()) {
        ent = aim.get_entity().unwrap();
    }

    for i in engine.entities().by_target_name(volume_name) {
        if let Some(volume) = i.get_entity() {
            if volume.is_classname(c"trigger_transition".into()) && volume.intersects(ent) {
                return true;
            }
        }
    }

    false
}

pub fn build_change_list(engine: ServerEngineRef, level_list: &mut [LEVELLIST]) -> usize {
    const MAX_ENTITY: usize = 512;

    let mut count = 0;
    for i in engine.entities().by_class_name(c"trigger_changelevel") {
        if let Some(trigger) = i.downcast_ref::<ChangeLevel>() {
            let map_name = trigger.map_name.as_thin();
            let landmark_name = trigger.landmark_name.as_thin();
            if let Some(landmark) = find_landmark(engine, landmark_name) {
                let is_added = add_transition_to_list(
                    level_list,
                    count,
                    map_name,
                    landmark_name,
                    landmark.as_ptr(),
                );
                if is_added {
                    count += 1;
                    if count >= level_list.len() {
                        break;
                    }
                }
            }
        }
    }

    if count == 0 {
        return 0;
    }

    if let Some(mut save_data) = engine.globals.save_data() {
        let save_data = SaveRestoreData::new(unsafe { save_data.as_mut() });
        if !save_data.table().is_empty() {
            for (i, level) in level_list.iter().enumerate().take(count) {
                let mut ent_count = 0;
                let mut ent_list = [ptr::null_mut(); MAX_ENTITY];
                let mut ent_flags = [0; MAX_ENTITY];

                for ent in engine.entities().in_pvs(unsafe { &*level.pentLandmark }) {
                    if let Some(entity) = ent.get_entity() {
                        let caps = entity.object_caps();
                        if !caps.intersects(ObjectCaps::DONT_SAVE) {
                            let mut flags = 0;

                            if caps.intersects(ObjectCaps::ACROSS_TRANSITION) {
                                flags |= FENTTABLE_MOVEABLE;
                            }
                            if entity.globalname().is_some() && !entity.is_dormant() {
                                flags |= FENTTABLE_GLOBAL;
                            }
                            if flags != 0 {
                                ent_list[ent_count] = entity.vars().containing_entity_raw();
                                ent_flags[ent_count] = flags;
                                ent_count += 1;
                            }
                        }
                    }
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

export_entity_default!(
    "export-trigger_changelevel",
    trigger_changelevel,
    ChangeLevel
);
