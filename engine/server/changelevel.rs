use core::ptr;

use csz::{CStrSlice, CStrThin};
use xash3d_shared::{
    entity::MoveType,
    ffi::server::{edict_s, FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, LEVELLIST},
};

use crate::{
    entity::{EntityChangeLevel, EntityHandle, ObjectCaps},
    prelude::*,
    save::SaveRestoreData,
};

pub fn find_landmark(engine: &ServerEngine, landmark_name: &CStrThin) -> Option<EntityHandle> {
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

pub fn in_transition_volume(engine: &ServerEngine, ent: &edict_s, volume_name: &CStrThin) -> bool {
    let Some(mut ent) = ent.get_entity() else {
        warn!("in_transition_volume: failed to get entity");
        return false;
    };

    if ent.object_caps().intersects(ObjectCaps::FORCE_TRANSITION) {
        return true;
    }

    if ent.vars().move_type() == MoveType::Follow {
        if let Some(aim) = ent.vars().aim_entity().get_entity() {
            ent = aim;
        }
    }

    let mut found_volume = false;
    engine
        .entities()
        .by_target_name(volume_name)
        .filter_map(|i| i.get_entity())
        .filter(|i| i.is_classname(c"trigger_transition".into()))
        .any(|i| {
            found_volume = true;
            i.intersects(ent)
        })
        || !found_volume
}

trait LevelListExt {
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

pub(crate) fn build_change_list(engine: &ServerEngine, level_list: &mut [LEVELLIST]) -> usize {
    const MAX_ENTITY: usize = 512;

    let mut count = 0;
    for i in engine.entities().by_class_name(c"trigger_changelevel") {
        if let Some(trigger) = i.downcast_ref::<dyn EntityChangeLevel>() {
            let map_name = trigger.map_name();
            let landmark_name = trigger.landmark_name();
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
                    if ent_flags[j] == 0 {
                        continue;
                    }
                    let landmark_name = level.landmark_name();
                    let ent = unsafe { &*ent_list[j] };
                    if in_transition_volume(engine, ent, landmark_name) {
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
