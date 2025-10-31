use core::ptr;

use csz::{cstr, CStrArray, CStrThin};
use xash3d_shared::{
    entity::MoveType,
    ffi::server::{edict_s, FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, LEVELLIST},
};

use crate::{
    entities::trigger::Trigger,
    entity::{delegate_entity, BaseEntity, EntityHandle, KeyValue, ObjectCaps, UseType},
    export::export_entity_default,
    prelude::*,
    save::SaveRestoreData,
    str::MapString,
    utils,
};

const MAP_NAME_MAX: usize = 32;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct ChangeLevel {
    base: Trigger,

    map_name: CStrArray<MAP_NAME_MAX>,
    landmark_name: CStrArray<MAP_NAME_MAX>,
    change_target: Option<MapString>,
    change_target_delay: f32,
}

impl CreateEntity for ChangeLevel {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
            map_name: Default::default(),
            landmark_name: Default::default(),
            change_target: None,
            change_target_delay: 0.0,
        }
    }
}

impl ChangeLevel {
    const SF_USE_ONLY: u32 = 1 << 1;

    fn change_level_now(&self, activator: Option<&dyn Entity>) {
        let name = self.pretty_name();
        let engine = self.engine();
        let v = self.vars();

        if self.map_name.is_empty() {
            panic!("{name}: Detected problems with save/restore!!!");
        }

        if self.global_state().game_rules().is_deathmatch() {
            return;
        }

        // do not fire multiple times per frame
        let now = engine.globals.map_time();
        if now == v.damage_time() {
            return;
        }
        v.set_damage_time(now);

        let player = engine.get_single_player().expect("player entity");
        if !in_transition_volume(&engine, unsafe { &*player.as_ptr() }, &self.landmark_name) {
            let landmark = &self.landmark_name;
            debug!("{name}: player is not in the transition volume {landmark}, aborting");
            return;
        }

        if let Some(change_target) = self.change_target {
            warn!("{name}: change target ({change_target}) is not implemented yet");
        }

        utils::use_targets(UseType::Toggle, activator, self);

        let mut next_spot = cstr!("");
        if let Some(landmark) = find_landmark(engine, &self.landmark_name) {
            next_spot = &self.landmark_name;
            engine.globals.set_landmark_offset(landmark.vars().origin());
        }

        info!("CHANGE LEVEL: {:?} {next_spot:?}", self.map_name);
        engine.change_level(&self.map_name, next_spot);
    }
}

impl Entity for ChangeLevel {
    delegate_entity!(base not { key_value, spawn, used, touched });

    fn key_value(&mut self, data: &mut KeyValue) {
        let engine = self.engine();
        let value = data.value();

        match data.key_name().to_bytes() {
            b"map" => {
                if self.map_name.cursor().write_c_str(value).is_err() {
                    let name = self.pretty_name();
                    error!("{name}: map name is too long ({value:?})");
                }
            }
            b"landmark" => {
                if self.landmark_name.cursor().write_c_str(value).is_err() {
                    let name = self.pretty_name();
                    error!("{name}: landmark name is too long ({value:?})");
                }
            }
            b"changetarget" => self.change_target = Some(engine.new_map_string(value)),
            b"changedelay" => self.change_target_delay = data.parse_or_default(),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let name = self.pretty_name();
        if self.map_name.is_empty() {
            warn!("{name}: map is empty");
        }
        if self.landmark_name.is_empty() {
            info!("{name}: does not have a landmark to map {}", self.map_name);
        }
        self.base.spawn();
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, _: &dyn Entity) {
        if self.vars().target_name().is_some() {
            self.change_level_now(activator);
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if self.vars().spawn_flags() & Self::SF_USE_ONLY != 0 {
            return;
        }
        if other.vars().is_class_name(c"player") {
            self.change_level_now(Some(other));
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

fn in_transition_volume(engine: &ServerEngine, ent: &edict_s, volume_name: &CStrThin) -> bool {
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
                    if ent_flags[j] == 0 {
                        continue;
                    }
                    let landmark_name = level.landmark_name();
                    let ent = unsafe { &*ent_list[j] };
                    if in_transition_volume(&engine, ent, landmark_name) {
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
    ChangeLevel {}
);
