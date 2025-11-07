use xash3d_shared::{
    csz::{CStrSlice, CStrThin},
    entity::MoveType,
    ffi::server::{ENTITYTABLE, FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, LEVELLIST},
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

pub fn in_transition_volume(
    engine: &ServerEngine,
    ent: EntityHandle,
    volume_name: &CStrThin,
) -> bool {
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

struct Table<'a> {
    raw: &'a mut [ENTITYTABLE],
}

impl<'a> Table<'a> {
    fn new(raw: &'a mut [ENTITYTABLE]) -> Self {
        Self { raw }
    }

    fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    fn set_flags(&mut self, ent: EntityHandle, flags: i32) {
        if let Some(i) = self.raw.iter_mut().find(|i| i.pent == ent.as_ptr()) {
            i.flags |= flags;
        }
    }
}

struct Level<'a> {
    engine: ServerEngineRef,
    raw: &'a LEVELLIST,
}

impl<'a> Level<'a> {
    fn new(engine: ServerEngineRef, raw: &'a LEVELLIST) -> Self {
        Self { engine, raw }
    }

    fn map_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.raw.mapName.as_ptr()) }
    }

    fn landmark_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.raw.landmarkName.as_ptr()) }
    }

    fn landmark(&self) -> EntityHandle {
        unsafe { EntityHandle::new_unchecked(self.engine, self.raw.pentLandmark) }
    }
}

struct LevelList<'a> {
    engine: ServerEngineRef,
    list: &'a mut [LEVELLIST],
    len: usize,
}

impl<'a> LevelList<'a> {
    fn new(engine: &ServerEngine, list: &'a mut [LEVELLIST]) -> Self {
        Self {
            engine: engine.engine_ref(),
            list,
            len: 0,
        }
    }

    fn capacity(&self) -> usize {
        self.list.len()
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_full(&self) -> bool {
        self.capacity() <= self.len()
    }

    fn iter(&self) -> impl Iterator<Item = Level<'_>> {
        self.list[..self.len]
            .iter()
            .map(|i| Level::new(self.engine, i))
    }

    fn contains(&self, map_name: &CStrThin, landmark: EntityHandle) -> bool {
        self.iter()
            .any(|i| i.landmark() == landmark && i.map_name() == map_name)
    }

    fn push(
        &mut self,
        map_name: &CStrThin,
        landmark_name: &CStrThin,
        landmark: EntityHandle,
    ) -> bool {
        if self.is_full() || self.contains(map_name, landmark) {
            return false;
        }
        let level = &mut self.list[self.len];
        CStrSlice::new_in_slice(&mut level.mapName)
            .cursor()
            .write_c_str(map_name)
            .unwrap();
        CStrSlice::new_in_slice(&mut level.landmarkName)
            .cursor()
            .write_c_str(landmark_name)
            .unwrap();
        level.pentLandmark = landmark.as_ptr();
        level.vecLandmarkOrigin = landmark.vars().origin();
        self.len += 1;
        true
    }
}

pub(crate) fn build_change_list(engine: &ServerEngine, save_data: &mut SaveRestoreData) -> usize {
    let (table, level_list) = save_data.split_level_list_mut();
    let mut table = Table::new(table);
    let mut level_list = LevelList::new(engine, level_list);

    for i in engine.entities().by_class_name(c"trigger_changelevel") {
        let Some(trigger) = i.downcast_ref::<dyn EntityChangeLevel>() else {
            continue;
        };

        let landmark_name = trigger.landmark_name();
        if let Some(landmark) = find_landmark(engine, landmark_name) {
            if !level_list.push(trigger.map_name(), landmark_name, landmark) {
                // no more space
                break;
            }
        }
    }

    if level_list.is_empty() || table.is_empty() {
        return level_list.len();
    }

    for (i, level) in level_list.iter().enumerate() {
        for ent in engine.entities().in_pvs(&level.landmark()) {
            let Some(entity) = ent.get_entity() else {
                continue;
            };

            let caps = entity.object_caps();
            if caps.intersects(ObjectCaps::DONT_SAVE) {
                continue;
            }

            let mut flags = 0;
            if caps.intersects(ObjectCaps::ACROSS_TRANSITION) {
                flags |= FENTTABLE_MOVEABLE;
            }
            if entity.globalname().is_some() && !entity.is_dormant() {
                flags |= FENTTABLE_GLOBAL;
            }

            if flags != 0 {
                let ent = ent.into();
                if in_transition_volume(engine, ent, level.landmark_name()) {
                    table.set_flags(ent, flags | (1 << i));
                }
            }
        }
    }

    level_list.len()
}
