use core::ptr::{self, NonNull};

use bitflags::bitflags;
use csz::{cstr, CStrArray, CStrThin};
use xash3d_shared::{
    consts::SOLID_TRIGGER,
    entity::{EdictFlags, Effects, MoveType},
    ffi::{
        common::vec3_t,
        server::{edict_s, FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, LEVELLIST},
    },
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entities::subs::DelayedUse,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityVars, KeyValue,
        ObjectCaps, UseType,
    },
    global_state::EntityState,
    prelude::*,
    save::SaveRestoreData,
    str::MapString,
    time::MapTime,
    utils,
};

const MAP_NAME_MAX: usize = 32;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct AutoTrigger {
    base: BaseEntity,
    delay: f32,
    kill_target: Option<MapString>,
    global_state: Option<MapString>,
    trigger_type: UseType,
}

impl AutoTrigger {
    /// Remove this trigger after firing.
    const SF_FIREONCE: u32 = 1 << 0;
}

impl_entity_cast!(AutoTrigger);

impl CreateEntity for AutoTrigger {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            delay: 0.0,
            kill_target: None,
            global_state: None,
            trigger_type: UseType::Off,
        }
    }
}

impl Entity for AutoTrigger {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"globalstate" => {
                self.global_state = Some(self.engine().new_map_string(data.value()));
            }
            b"triggerstate" => match data.value().to_bytes() {
                b"0" => self.trigger_type = UseType::Off,
                b"2" => self.trigger_type = UseType::Toggle,
                _ => self.trigger_type = UseType::On,
            },
            b"delay" => {
                self.delay = data.value_str().parse().unwrap_or(0.0);
            }
            b"killtarget" => {
                self.kill_target = Some(self.engine().new_map_string(data.value()));
            }
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        self.vars_mut().set_next_think_time(0.1);
    }

    fn spawn(&mut self) {
        self.precache();
    }

    fn think(&mut self) {
        if !self.global_state.map_or(true, |name| {
            self.global_state().entity_state(name) == EntityState::On
        }) {
            return;
        }

        if self.delay != 0.0 {
            DelayedUse::create(
                self.engine(),
                self.delay,
                self.vars().target(),
                self.trigger_type,
                self.kill_target,
                Some(self),
            );
        } else {
            utils::use_targets(self.kill_target, self.trigger_type, 0.0, None, self);
        }

        if self.vars().spawn_flags() & Self::SF_FIREONCE != 0 {
            self.remove_from_world();
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct TriggerSpawnFlags: u32 {
        /// Monsters allowed to fire this trigger.
        const ALLOW_MONSTERS = 1 << 0;
        /// Players not allowed to fire this trigger.
        const NO_CLIENTS = 1 << 1;
        /// Only pushables can fire this trigger.
        const PUSHABLES = 1 << 2;
    }
}

fn init_trigger(engine: &ServerEngine, v: &mut EntityVars) {
    if v.angles() != vec3_t::ZERO {
        v.set_move_dir();
    }
    let ev = v.as_raw_mut();
    ev.solid = SOLID_TRIGGER;
    ev.movetype = MoveType::None.into();
    if let Some(model) = ev.model() {
        engine.set_model(ev, &model);
    }
    if !engine.get_cvar::<bool>(c"showtriggers") {
        ev.effects_mut().insert(Effects::NODRAW);
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerMultiple {
    base: BaseEntity,
    delay: f32,
    /// Time in seconds before the trigger is ready to be re-triggered.
    wait: f32,
    /// The time when this trigger can be re-triggered.
    reset_time: MapTime,
    kill_target: Option<MapString>,
    master: Option<MapString>,
}

impl TriggerMultiple {
    fn activate_trigger(&mut self, other: &mut dyn Entity) {
        let engine = self.engine();
        if engine.globals.map_time() < self.reset_time {
            // still waiting for reset time
            return;
        }

        if let Some(master) = self.master {
            if !utils::is_master_triggered(&engine, master, other) {
                return;
            }
        }

        let v = self.base.vars_mut();
        if let Some(noise) = MapString::from_index(engine, v.as_raw().noise) {
            engine.build_sound().channel_voice().emit(&noise, self);
        }

        if self.delay != 0.0 {
            DelayedUse::create(
                self.engine(),
                self.delay,
                self.vars().target(),
                UseType::Toggle,
                self.kill_target,
                Some(self),
            );
        } else {
            utils::use_targets(self.kill_target, UseType::Toggle, 0.0, Some(other), self);
        }

        let v = self.base.vars_mut();
        if let Some(_message) = MapString::from_index(engine, v.as_raw().message) {
            // TODO: need HudText user message defined in xash3d-hl-shared =\
            warn!(
                "{}: show a hud message is not implemented",
                self.classname()
            );
        }

        if self.wait > 0.0 {
            self.reset_time = engine.globals.map_time() + self.wait;
        } else {
            self.remove_from_world();
        }
    }
}

impl_entity_cast!(TriggerMultiple);

impl CreateEntity for TriggerMultiple {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            delay: 0.0,
            wait: 0.2,
            reset_time: MapTime::ZERO,
            kill_target: None,
            master: None,
        }
    }
}

impl Entity for TriggerMultiple {
    delegate_entity!(base not { object_caps, key_value, spawn, touched, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"master" => {
                self.master = Some(self.engine().new_map_string(data.value()));
            }
            b"wait" => {
                self.wait = data.value_str().parse().unwrap_or(0.0);
            }
            b"delay" => {
                self.delay = data.value_str().parse().unwrap_or(0.0);
            }
            b"killtarget" => {
                self.kill_target = Some(self.engine().new_map_string(data.value()));
            }
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        init_trigger(&self.engine(), self.vars_mut());
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        let spawn_flags = TriggerSpawnFlags::from_bits_retain(self.vars().spawn_flags());
        let flags = other.vars().flags();
        if spawn_flags.intersects(TriggerSpawnFlags::NO_CLIENTS)
            && flags.intersects(EdictFlags::CLIENT)
        {
            return;
        }
        if !spawn_flags.intersects(TriggerSpawnFlags::ALLOW_MONSTERS)
            && flags.intersects(EdictFlags::MONSTER)
        {
            return;
        }
        if !spawn_flags.intersects(TriggerSpawnFlags::PUSHABLES)
            && other.is_classname(c"func_pushable".into())
        {
            return;
        }
        self.activate_trigger(other);
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerOnce {
    base: TriggerMultiple,
}

impl_entity_cast!(TriggerOnce);

impl CreateEntity for TriggerOnce {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: TriggerMultiple::create(base),
        }
    }
}

impl Entity for TriggerOnce {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.base.wait = -1.0;
        self.base.spawn();
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct ChangeLevel {
    base: BaseEntity,
    map_name: CStrArray<MAP_NAME_MAX>,
    landmark_name: CStrArray<MAP_NAME_MAX>,
    target: Option<MapString>,
    change_target_delay: f32,
}

impl ChangeLevel {
    const SF_USE_ONLY: u32 = 1 << 1;

    fn change_level_now(&mut self, _other: &mut dyn Entity) {
        let engine = self.base.engine;
        assert!(!self.map_name.is_empty());

        if self.global_state().game_rules().is_deathmatch() {
            return;
        }

        let globals = &engine.globals;
        let ev = self.vars_mut().as_raw_mut();
        let time = globals.map_time_f32();
        if time == ev.dmgtime {
            return;
        }

        ev.dmgtime = time;

        let mut next_spot = cstr!("");
        if let Some(landmark) = find_landmark(engine, self.landmark_name.as_thin()) {
            next_spot = self.landmark_name.as_thin();
            unsafe {
                globals.set_landmark_offset(landmark.as_ref().v.origin);
            }
        }

        info!("CHANGE LEVEL: {:?} {next_spot:?}", self.map_name);
        engine.change_level(&self.map_name, next_spot);
    }
}

impl_entity_cast!(ChangeLevel);

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
    delegate_entity!(base not { object_caps, key_value, spawn, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        let engine = self.base.engine;
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
                self.change_target_delay = data.value_str().parse().unwrap_or(0.0);
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

        init_trigger(&self.engine(), self.vars_mut());

        if self.vars().spawn_flags() & Self::SF_USE_ONLY != 0 {
            // TODO: set touch
        }
    }

    fn touched(&mut self, other: &mut dyn Entity) {
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

fn find_landmark(engine: ServerEngineRef, landmark_name: &CStrThin) -> Option<NonNull<edict_s>> {
    engine
        .find_ent_by_targetname_iter(landmark_name)
        .find(|&ent| {
            let classname = unsafe { ent.as_ref() }.v.classname().unwrap();
            classname.as_thin() == c"info_landmark"
        })
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
            if let Some(landmark) = find_landmark(engine, landmark_name) {
                if add_transition_to_list(
                    level_list,
                    count,
                    map_name,
                    landmark_name,
                    landmark.as_ptr(),
                ) {
                    count += 1;
                    if count >= level_list.len() {
                        break;
                    }
                }
            }
        }
        ent = engine.find_ent_by_classname(ent, c"trigger_changelevel");
    }

    if let Some(mut save_data) = engine.globals.save_data() {
        let save_data = SaveRestoreData::new(unsafe { save_data.as_mut() });
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
                            if entity.globalname().is_some() && !entity.is_dormant() {
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

#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(trigger_auto, Private<super::AutoTrigger>);
    export_entity!(trigger_multiple, Private<super::TriggerMultiple>);
    export_entity!(trigger_once, Private<super::TriggerOnce>);
    export_entity!(trigger_changelevel, Private<super::ChangeLevel>);

    export_entity!(env_render, Private<StubEntity>);
    export_entity!(fireanddie, Private<StubEntity>);
    export_entity!(func_friction, Private<StubEntity>);
    export_entity!(func_ladder, Private<StubEntity>);
    export_entity!(info_teleport_destination, Private<StubEntity>);
    export_entity!(multi_manager, Private<StubEntity>);
    export_entity!(target_cdaudio, Private<StubEntity>);
    export_entity!(trigger, Private<StubEntity>);
    export_entity!(trigger_autosave, Private<StubEntity>);
    export_entity!(trigger_camera, Private<StubEntity>);
    export_entity!(trigger_cdaudio, Private<StubEntity>);
    export_entity!(trigger_changetarget, Private<StubEntity>);
    export_entity!(trigger_counter, Private<StubEntity>);
    export_entity!(trigger_endsection, Private<StubEntity>);
    export_entity!(trigger_gravity, Private<StubEntity>);
    export_entity!(trigger_hurt, Private<StubEntity>);
    export_entity!(trigger_monsterjump, Private<StubEntity>);
    export_entity!(trigger_push, Private<StubEntity>);
    export_entity!(trigger_relay, Private<StubEntity>);
    export_entity!(trigger_teleport, Private<StubEntity>);
    export_entity!(trigger_transition, Private<StubEntity>);
}
