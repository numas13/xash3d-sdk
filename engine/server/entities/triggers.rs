use core::{
    mem,
    ptr::{self, NonNull},
};

use alloc::vec::Vec;
use bitflags::bitflags;
use csz::{cstr, CStrArray, CStrThin};
use xash3d_shared::{
    entity::{DamageFlags, EdictFlags, Effects, EntityIndex, MoveType},
    ffi::{
        common::vec3_t,
        server::{edict_s, FENTTABLE_GLOBAL, FENTTABLE_MOVEABLE, LEVELLIST},
    },
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entities::subs::{DelayedUse, PointEntity},
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Dead, Entity, EntityPlayer,
        EntityVars, KeyValue, ObjectCaps, Private, Solid, TakeDamage, UseType,
    },
    export::{export_entity_default, export_entity_stub},
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
    delayed: DelayedUse,
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
            delayed: DelayedUse::new(base.engine()),
            base,
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
            _ => {
                if !self.delayed.key_value(data) {
                    self.base.key_value(data);
                }
                return;
            }
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        self.vars().set_next_think_time_from_now(0.1);
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

        self.delayed.use_targets(self.trigger_type, self);

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

fn init_trigger(engine: &ServerEngine, v: &EntityVars) {
    if v.angles() != vec3_t::ZERO {
        v.set_move_dir_from_angles();
    }
    v.set_solid(Solid::Trigger);
    v.set_move_type(MoveType::None);
    v.reload_model();
    if !engine.get_cvar::<bool>(c"showtriggers") {
        v.with_effects(|f| f | Effects::NODRAW);
    }
}

fn toggle_use(ent: &mut impl Entity) {
    let engine = ent.engine();
    let v = ent.vars();
    match v.solid() {
        Solid::Not => {
            v.set_solid(Solid::Trigger);
            engine.globals.force_retouch();
        }
        _ => {
            v.set_solid(Solid::Not);
        }
    }
    v.link();
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerMultiple {
    base: BaseEntity,
    delayed: DelayedUse,
    /// Time in seconds before the trigger is ready to be re-triggered.
    wait: f32,
    /// The time when this trigger can be re-triggered.
    reset_time: MapTime,
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

        let v = self.base.vars();
        if let Some(noise) = v.noise() {
            engine.build_sound().channel_voice().emit(noise, self);
        }

        self.delayed.use_targets(UseType::Toggle, self);

        let v = self.base.vars();
        if let Some(_message) = v.message() {
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
            delayed: DelayedUse::new(base.engine()),
            base,
            wait: 0.2,
            reset_time: MapTime::ZERO,
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
            _ => {
                if !self.delayed.key_value(data) {
                    self.base.key_value(data);
                }
                return;
            }
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        init_trigger(&self.engine(), self.vars());
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

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct TriggerPushSpawnFlags: u32 {
        const PUSH_ONCE = 1 << 0;
        // spawnflag that makes trigger_push spawn turned OFF
        const START_OFF = 1 << 1;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerPush {
    base: BaseEntity,
}

impl_entity_cast!(TriggerPush);

impl CreateEntity for TriggerPush {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for TriggerPush {
    delegate_entity!(base not { object_caps, spawn, used, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let engine = self.base.engine();
        let v = self.base.vars();

        if v.angles() == vec3_t::ZERO {
            v.with_angles(|v| v.with_y(360.0));
        }
        init_trigger(&engine, v);

        if v.speed() == 0.0 {
            v.set_speed(100.0);
        }

        let spawn_flags = TriggerPushSpawnFlags::from_bits_retain(v.spawn_flags());
        if spawn_flags.intersects(TriggerPushSpawnFlags::START_OFF) {
            v.set_solid(Solid::Not);
        }

        v.link();
    }

    fn used(&mut self, _: UseType, _: Option<&mut dyn Entity>, _: &mut dyn Entity) {
        toggle_use(self);
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        let other_v = other.vars();
        if let MoveType::None | MoveType::Push | MoveType::NoClip | MoveType::Follow =
            other_v.move_type()
        {
            return;
        }
        if let Solid::Not | Solid::Bsp = other_v.solid() {
            return;
        }

        let v = self.base.vars();
        let push_vec = v.move_dir() * v.speed();
        let spawn_flags = TriggerPushSpawnFlags::from_bits_retain(self.vars().spawn_flags());
        if spawn_flags.intersects(TriggerPushSpawnFlags::PUSH_ONCE) {
            other_v.with_velocity(|v| v + push_vec);
            if other_v.velocity().z > 0.0 {
                other_v.with_flags(|f| f.difference(EdictFlags::ONGROUND));
            }
            self.remove_from_world();
        } else if other_v.flags().intersects(EdictFlags::BASEVELOCITY) {
            other_v.with_base_velocity(|v| v + push_vec);
        } else {
            other_v.with_flags(|f| f | EdictFlags::BASEVELOCITY);
            other_v.set_base_velocity(push_vec);
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct TriggerHurtSpawnFlags: u32 {
        /// Only fire hurt target once.
        const TARGET_ONCE = 1 << 0;
        /// Spawnflag that makes trigger_push spawn turned OFF.
        const START_OFF = 1 << 1;
        /// Players not allowed to fire this trigger.
        const NO_CLIENTS = 1 << 3;
        /// Trigger hurt will only fire its target if it is hurting a client.
        const CLIENT_ONLY_FIRE = 1 << 4;
        /// Only clients may touch this trigger.
        const CLIENT_ONLY_TOUCH = 1 << 4;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerHurt {
    base: BaseEntity,
    delayed: DelayedUse,
    damage_type: DamageFlags,
}

impl TriggerHurt {
    fn spawn_flags(&self) -> TriggerHurtSpawnFlags {
        TriggerHurtSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }
}

impl_entity_cast!(TriggerHurt);

impl CreateEntity for TriggerHurt {
    fn create(base: BaseEntity) -> Self {
        Self {
            delayed: DelayedUse::new(base.engine()),
            base,
            damage_type: DamageFlags::default(),
        }
    }
}

impl Entity for TriggerHurt {
    delegate_entity!(base not { object_caps, key_value, spawn, used, touched, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"damagetype" => {
                let bits = data.value_str().parse().unwrap_or(0);
                self.damage_type = DamageFlags::from_bits_retain(bits);
            }
            _ => {
                if !self.delayed.key_value(data) {
                    self.base.key_value(data);
                }
                return;
            }
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let spawn_flags = self.spawn_flags();
        let engine = self.base.engine();
        let v = self.base.vars();
        init_trigger(&engine, v);

        if self.damage_type.intersects(DamageFlags::RADIATION) {
            v.set_next_think_time_from_now(engine.random_float(0.0, 0.5));
        }

        if spawn_flags.intersects(TriggerHurtSpawnFlags::START_OFF) {
            v.set_solid(Solid::Not);
        }

        v.link();
    }

    fn used(&mut self, _: UseType, _: Option<&mut dyn Entity>, _: &mut dyn Entity) {
        if self.vars().target_name().is_some() {
            toggle_use(self);
        }
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        if other.vars().take_damage() == TakeDamage::No {
            return;
        }

        let engine = self.base.engine();
        let global_state = self.base.global_state();
        let spawn_flags = self.spawn_flags();

        let is_player = other.downcast_ref::<dyn EntityPlayer>().is_some();
        if spawn_flags.intersects(TriggerHurtSpawnFlags::NO_CLIENTS) && is_player {
            return;
        }

        let v = self.base.vars();
        let now = engine.globals.map_time();
        let is_multiplayer = global_state.game_rules().is_multiplayer();
        if is_multiplayer {
            warn!(
                "{}: touched is not implemented in multiplayer",
                self.classname()
            );
            return;
        } else if now <= v.damage_time() && now != v.pain_finished_time() {
            return;
        }

        let dmg = v.damage() * 0.5;
        if dmg < 0.0 {
            if !(is_multiplayer && is_player && other.vars().dead() != Dead::No) {
                other.take_health(-dmg, self.damage_type);
            }
        } else {
            other.take_damage(dmg, self.damage_type, v, None);
        }

        v.set_pain_finished_time(now);
        v.set_damage_time(now + 0.5);

        if v.target().is_some() {
            if spawn_flags.intersects(TriggerHurtSpawnFlags::CLIENT_ONLY_FIRE) && !is_player {
                return;
            }
            self.delayed.use_targets(UseType::Toggle, self);
            if spawn_flags.intersects(TriggerHurtSpawnFlags::TARGET_ONCE) {
                self.vars().set_target(None);
            }
        }
    }

    fn think(&mut self) {
        if !self.damage_type.intersects(DamageFlags::RADIATION) {
            return;
        }

        let engine = self.base.engine();
        let v = self.base.vars();

        // set origin to center of trigger so that this check works
        let orig_origin = v.origin();
        let orig_view_ofs = v.view_ofs();
        v.set_origin(v.abs_center());
        v.set_view_ofs(vec3_t::ZERO);

        let player = engine
            .find_client_in_pvs(v)
            .and_then(|mut i| unsafe { i.as_mut() }.get_entity_mut());

        v.set_origin(orig_origin);
        v.set_view_ofs(orig_view_ofs);

        if let Some(player) = player.and_then(|i| i.downcast_mut::<dyn EntityPlayer>()) {
            let spot1 = v.abs_center();
            let spot2 = player.vars().abs_center();
            let range = (spot1 - spot2).length();
            player.set_geiger_range(range);
        }

        v.set_next_think_time_from_now(0.25);
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerSave {
    base: BaseEntity,
    master: Option<MapString>,
}

impl_entity_cast!(TriggerSave);

impl CreateEntity for TriggerSave {
    fn create(base: BaseEntity) -> Self {
        Self { base, master: None }
    }
}

impl Entity for TriggerSave {
    delegate_entity!(base not { object_caps, key_value, spawn, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"master" {
            self.master = Some(self.engine().new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        if self.global_state().game_rules().is_deathmatch() {
            self.remove_from_world();
            return;
        }

        init_trigger(&self.engine(), self.vars());
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        let engine = self.engine();

        if let Some(master) = self.master {
            if !utils::is_master_triggered(&engine, master, other) {
                return;
            }
        }

        if other.downcast_ref::<dyn EntityPlayer>().is_some() {
            self.remove_from_world();
            engine.server_command(c"autosave\n");
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerVolume {
    base: PointEntity,
}

impl_entity_cast!(TriggerVolume);

impl CreateEntity for TriggerVolume {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl Entity for TriggerVolume {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        let v = self.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.reload_model();
        v.remove_model();
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerEndSection {
    base: BaseEntity,
    enable_used: bool,
    enable_touched: bool,
}

impl TriggerEndSection {
    const SF_USEONLY: u32 = 1;

    fn end_section(&mut self, activator: &dyn Entity) {
        // TODO: add is_net_client method to Entity/EntityPlayer???
        if activator.downcast_ref::<dyn EntityPlayer>().is_none() {
            return;
        }
        if let Some(message) = self.vars().message() {
            self.engine().end_section_by_name(message);
        }
        self.remove_from_world();
    }
}

impl_entity_cast!(TriggerEndSection);

impl CreateEntity for TriggerEndSection {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            enable_used: false,
            enable_touched: false,
        }
    }
}

impl Entity for TriggerEndSection {
    delegate_entity!(base not { object_caps, key_value, spawn, used, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"section" {
            let engine = self.engine();
            self.vars().set_message(engine.new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let global_state = self.global_state();
        let v = self.base.vars();

        if global_state.game_rules().is_deathmatch() {
            v.delayed_remove();
            return;
        }

        init_trigger(&engine, v);

        self.enable_used = true;
        self.enable_touched = v.spawn_flags() & Self::SF_USEONLY == 0;
    }

    fn used(&mut self, _: UseType, activator: Option<&mut dyn Entity>, caller: &mut dyn Entity) {
        if self.enable_used {
            self.enable_used = false;
            self.end_section(activator.unwrap_or(caller));
        }
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        if self.enable_touched {
            self.enable_touched = false;
            self.end_section(other);
        }
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
            let landmark_origin = unsafe { landmark.as_ref() }.v.origin;
            engine.globals.set_landmark_offset(landmark_origin);
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

        init_trigger(&self.engine(), self.vars());

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
    let mut ent = unsafe { &mut *ent }.get_entity().unwrap();
    if ent.object_caps().intersects(ObjectCaps::FORCE_TRANSITION) {
        return true;
    }
    if let (MoveType::Follow, Some(mut aim)) = (ent.vars().move_type(), ent.vars().aim_entity()) {
        ent = unsafe { aim.as_mut() }.get_entity().unwrap();
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
                                ent_list[ent_count] = entity.as_entity_handle();
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

#[derive(Copy, Clone, Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
struct MultiManagerTarget {
    name: Option<MapString>,
    delay: f32,
}

impl MultiManagerTarget {
    fn new(name: MapString, delay: f32) -> Self {
        Self {
            name: Some(name),
            delay,
        }
    }
}

bitflags! {
    struct MultiManagerSpawnFlags: u32 {
        /// Create clones when triggered.
        const THREAD = 1 << 0;
        /// This is a clone for a threaded execution.
        const CLONE = 1 << 31;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct MultiManager {
    base: BaseEntity,
    targets: Vec<MultiManagerTarget>,
    wait: f32,
    start_time: MapTime,
    activator: EntityIndex,
    index: u32,
    enable_use: bool,
    enable_think: bool,
}

impl MultiManager {
    fn spawn_flags(&self) -> MultiManagerSpawnFlags {
        MultiManagerSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn is_clone(&self) -> bool {
        self.spawn_flags().intersects(MultiManagerSpawnFlags::CLONE)
    }

    fn should_clone(&self) -> bool {
        !self.is_clone()
            && self
                .spawn_flags()
                .intersects(MultiManagerSpawnFlags::THREAD)
    }

    fn clone(&mut self) -> *mut Self {
        let engine = self.engine();
        let multi = engine.new_entity::<Private<Self>>().build();
        let edict = multi.vars().containing_entity();
        unsafe {
            ptr::copy_nonoverlapping(self.vars().as_ptr(), multi.vars().as_mut_ptr(), 1);
        }
        let v = multi.vars();
        v.set_containing_entity(edict.map(|e| unsafe { e.as_ref() }));
        v.with_spawn_flags(|f| f | MultiManagerSpawnFlags::CLONE.bits());
        multi.targets = self.targets.clone();
        multi
    }
}

impl_entity_cast!(MultiManager);

impl CreateEntity for MultiManager {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            targets: Vec::default(),
            wait: 0.0,
            start_time: MapTime::ZERO,
            activator: EntityIndex::ZERO,
            index: 0,
            enable_use: false,
            enable_think: false,
        }
    }
}

impl Entity for MultiManager {
    delegate_entity!(base not { object_caps, key_value, spawn, used, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        let key = data.key_name();
        if key == c"wait" {
            self.wait = data.parse_or(0.0);
            data.set_handled(true);
        } else {
            let mut tmp = CStrArray::<128>::new();
            match utils::strip_token(key.into(), &mut tmp) {
                Ok(()) => {
                    let name = self.engine().new_map_string(&tmp);
                    let delay = data.parse_or_default();
                    self.targets.push(MultiManagerTarget::new(name, delay))
                }
                Err(_) => {
                    error!("{}: failed to strip token {key:?}", self.classname());
                }
            }
        }
    }

    fn spawn(&mut self) {
        self.vars().set_solid(Solid::Not);
        self.targets
            .sort_by(|a, b| a.delay.partial_cmp(&b.delay).unwrap());
        self.enable_use = true;
    }

    fn used(
        &mut self,
        _use_type: UseType,
        activator: Option<&mut dyn Entity>,
        caller: &mut dyn Entity,
    ) {
        if !self.enable_use {
            return;
        }

        if self.should_clone() {
            let clone = unsafe { &mut *self.clone() };
            clone.used(_use_type, activator, caller);
            return;
        }

        let engine = self.engine();
        self.activator = engine.ent_index(activator.unwrap_or(caller));
        self.index = 0;
        self.start_time = engine.globals.map_time();
        self.enable_use = false;
        self.enable_think = true;
        self.vars().set_next_think_time_from_now(0.0);
    }

    fn think(&mut self) {
        if !self.enable_think {
            return;
        }

        let engine = self.engine();
        let time = engine.globals.map_time() - self.start_time;
        let mut activator = unsafe {
            engine
                .entity_of_ent_index(self.activator)
                .as_mut()
                .and_then(|i| i.get_entity_mut())
        };

        let targets = mem::take(&mut self.targets);
        for target in targets.iter().skip(self.index as usize) {
            if target.delay > time {
                break;
            }
            if let Some(target_name) = target.name {
                utils::fire_targets(
                    &target_name,
                    UseType::Toggle,
                    activator.as_deref_mut(),
                    self,
                );
            }
            self.index += 1;
        }
        self.targets = targets;

        if self.index as usize >= self.targets.len() {
            self.enable_think = false;
            if self.is_clone() {
                self.remove_from_world();
            }
            self.enable_use = true;
        } else if let Some(target) = self.targets.get(self.index as usize) {
            let next_time = self.start_time + target.delay;
            self.base.vars().set_next_think_time(next_time);
        }
    }
}

export_entity_default!("export-multi_manager", multi_manager, MultiManager);

export_entity_default!("export-trigger_auto", trigger_auto, AutoTrigger);
export_entity_default!("export-trigger_autosave", trigger_autosave, TriggerSave);
export_entity_default!(
    "export-trigger_changelevel",
    trigger_changelevel,
    ChangeLevel
);
export_entity_default!("export-trigger_hurt", trigger_hurt, TriggerHurt);
export_entity_default!("export-trigger_multiple", trigger_multiple, TriggerMultiple);
export_entity_default!("export-trigger_once", trigger_once, TriggerOnce);
export_entity_default!("export-trigger_push", trigger_push, TriggerPush);
export_entity_default!(
    "export-trigger_transition",
    trigger_transition,
    TriggerVolume
);
export_entity_default!(
    "export-trigger_endsection",
    trigger_endsection,
    TriggerEndSection
);

export_entity_stub!(env_render);
export_entity_stub!(fireanddie);
export_entity_stub!(info_teleport_destination);
export_entity_stub!(target_cdaudio);
export_entity_stub!(trigger);
export_entity_stub!(trigger_camera);
export_entity_stub!(trigger_cdaudio);
export_entity_stub!(trigger_changetarget);
export_entity_stub!(trigger_counter);
export_entity_stub!(trigger_gravity);
export_entity_stub!(trigger_monsterjump);
export_entity_stub!(trigger_relay);
export_entity_stub!(trigger_teleport);
