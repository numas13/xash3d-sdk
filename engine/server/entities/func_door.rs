use core::{cell::Cell, ffi::CStr};

use bitflags::bitflags;
use xash3d_shared::entity::{DamageFlags, MoveType};

use crate::{
    entities::delayed_use::DelayedUse,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, EntityCast, EntityHandle, EntityVars,
        KeyValue, ObjectCaps, Solid, UseType,
    },
    export::export_entity_default,
    prelude::*,
    sound::LockSounds,
    str::MapString,
    utils::{self, AngularMove, LinearMove, Move, MoveState},
};

const MOVE_SOUNDS: &[&CStr] = &[
    res::valve::sound::common::NULL,
    res::valve::sound::doors::DOORMOVE1,
    res::valve::sound::doors::DOORMOVE2,
    res::valve::sound::doors::DOORMOVE3,
    res::valve::sound::doors::DOORMOVE4,
    res::valve::sound::doors::DOORMOVE5,
    res::valve::sound::doors::DOORMOVE6,
    res::valve::sound::doors::DOORMOVE7,
    res::valve::sound::doors::DOORMOVE8,
    res::valve::sound::doors::DOORMOVE9,
    res::valve::sound::doors::DOORMOVE10,
];

const STOP_SOUNDS: &[&CStr] = &[
    res::valve::sound::common::NULL,
    res::valve::sound::doors::DOORSTOP1,
    res::valve::sound::doors::DOORSTOP2,
    res::valve::sound::doors::DOORSTOP3,
    res::valve::sound::doors::DOORSTOP4,
    res::valve::sound::doors::DOORSTOP5,
    res::valve::sound::doors::DOORSTOP6,
    res::valve::sound::doors::DOORSTOP7,
    res::valve::sound::doors::DOORSTOP8,
];

trait EntityVarsExt {
    fn noise_moving(&self) -> Option<MapString>;

    fn set_noise_moving(&self, sound: MapString);

    fn noise_arrived(&self) -> Option<MapString>;

    fn set_noise_arrived(&self, sound: MapString);
}

impl EntityVarsExt for EntityVars {
    fn noise_moving(&self) -> Option<MapString> {
        self.noise1()
    }

    fn set_noise_moving(&self, sound: MapString) {
        self.set_noise1(sound);
    }

    fn noise_arrived(&self) -> Option<MapString> {
        self.noise2()
    }

    fn set_noise_arrived(&self, sound: MapString) {
        self.set_noise2(sound);
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    struct DoorSpawnFlags: u32 {
        const ROTATE_Y          = 0;
        const START_OPEN        = 1 << 0;
        const ROTATE_BACKWARDS  = 1 << 1;
        const PASSABLE          = 1 << 3;
        const ONEWAY            = 1 << 4;
        const NO_AUTO_RETURN    = 1 << 5;
        const ROTATE_Z          = 1 << 6;
        const ROTATE_X          = 1 << 7;
        /// The door must be opened by player's use button.
        const USE_ONLY          = 1 << 8;
        const NO_MONSTERS       = 1 << 9;
        const SILENT            = 1 << 31;
    }
}

impl DoorSpawnFlags {
    fn is_silent(&self) -> bool {
        self.intersects(Self::SILENT)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum DoorThink {
    #[default]
    None = 0,
    MoveDone,
    DoorGoDown,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct BaseDoor<T> {
    base: BaseEntity,
    delayed: DelayedUse,
    wait: f32,
    master: Option<MapString>,

    state: Cell<MoveState>,
    enable_touch: Cell<bool>,
    think: Cell<DoorThink>,
    activator: Cell<Option<EntityHandle>>,

    door_move: T,

    move_sound: u8,
    stop_sound: u8,

    lock_sounds: LockSounds,
}

impl<T: Move + Default> CreateEntity for BaseDoor<T> {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base,
            delayed: DelayedUse::new(engine),
            wait: 0.0,
            master: None,

            state: Default::default(),
            enable_touch: Default::default(),
            think: Default::default(),
            activator: Cell::default(),

            door_move: Default::default(),

            move_sound: 0,
            stop_sound: 0,

            lock_sounds: LockSounds::new(engine),
        }
    }
}

impl<T: Move> BaseDoor<T> {
    fn spawn_flags(&self) -> DoorSpawnFlags {
        DoorSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn go_end(&self) {
        let engine = self.engine();
        let v = self.base.vars();
        let sf = self.spawn_flags();
        if !sf.is_silent() && !self.state.get().is_moving() {
            if let Some(noise_moving) = v.noise_moving() {
                engine
                    .build_sound()
                    .channel_static()
                    .emit_dyn(noise_moving, v);
            }
        }

        self.state.set(MoveState::GoingToEnd);

        let mut reverse = false;
        if self.door_move.is_reversable() {
            if let Some(activator) = self.activator.get() {
                let av = activator.vars();
                if !sf.intersects(DoorSpawnFlags::ONEWAY) && v.move_dir().y != 0.0 {
                    let vec = av.origin() - v.origin();
                    let forward = av.angles().angle_vectors().forward();
                    let v_next = (av.origin() + forward * 10.0) - v.origin();
                    reverse = vec.x * v_next.y - vec.y * v_next.x < 0.0;
                }
            }
        }

        if self.door_move.move_to_end(v, v.speed(), reverse) {
            self.hit_end();
        } else {
            self.think.set(DoorThink::MoveDone);
        }
    }

    fn hit_end(&self) {
        let spawn_flags = self.spawn_flags();
        let engine = self.engine();
        let v = self.base.vars();

        if !spawn_flags.is_silent() {
            if let Some(noise_moving) = v.noise_moving() {
                engine.build_sound().channel_static().stop(noise_moving, v);
            }
            if let Some(noise_arrived) = v.noise_arrived() {
                engine
                    .build_sound()
                    .channel_static()
                    .emit_dyn(noise_arrived, v);
            }
        }

        assert_eq!(self.state.get(), MoveState::GoingToEnd);
        self.state.set(MoveState::AtEnd);

        if spawn_flags.intersects(DoorSpawnFlags::NO_AUTO_RETURN) {
            if !spawn_flags.intersects(DoorSpawnFlags::USE_ONLY) {
                self.enable_touch.set(true);
            }
        } else {
            v.set_next_think_time_from_last(self.wait);
            self.think.set(DoorThink::DoorGoDown);

            if self.wait == -1.0 {
                v.stop_thinking();
            }
        }

        let activator = self.activator.get().get_entity();
        if let Some(net_name) = v.net_name() {
            if spawn_flags.intersects(DoorSpawnFlags::START_OPEN) {
                utils::fire_targets(&net_name, UseType::Toggle, activator, self);
            }
        }
        self.delayed.use_targets(UseType::Toggle, activator, self);
    }

    fn go_start(&self) {
        let engine = self.engine();
        let spawn_flags = self.spawn_flags();
        if !spawn_flags.is_silent() && !self.state.get().is_moving() {
            let v = self.base.vars();
            if let Some(noise_moving) = v.noise_moving() {
                engine
                    .build_sound()
                    .channel_static()
                    .emit_dyn(noise_moving, v);
            }
        }

        self.state.set(MoveState::GoingToStart);

        let v = self.base.vars();
        if self.door_move.move_to_start(v, v.speed()) {
            self.hit_start();
        } else {
            self.think.set(DoorThink::MoveDone);
        }
    }

    fn hit_start(&self) {
        let spawn_flags = self.spawn_flags();
        let engine = self.engine();
        let v = self.base.vars();

        if !spawn_flags.is_silent() {
            if let Some(noise_moving) = v.noise_moving() {
                engine.build_sound().channel_static().stop(noise_moving, v);
            }
            if let Some(noise_arrived) = v.noise_arrived() {
                engine
                    .build_sound()
                    .channel_static()
                    .emit_dyn(noise_arrived, v);
            }
        }

        assert_eq!(self.state.get(), MoveState::GoingToStart);
        self.state.set(MoveState::AtStart);
        self.enable_touch
            .set(!spawn_flags.intersects(DoorSpawnFlags::USE_ONLY));

        let activator = self.activator.get().get_entity();
        if let Some(net_name) = v.net_name() {
            if !spawn_flags.intersects(DoorSpawnFlags::START_OPEN) {
                utils::fire_targets(&net_name, UseType::Toggle, activator, self);
            }
        }
        self.delayed.use_targets(UseType::Toggle, activator, self);
    }

    fn door_activate(&self, activator: Option<&dyn Entity>) -> bool {
        let engine = self.engine();
        if !utils::is_master_triggered(&engine, self.master, activator) {
            return false;
        }

        self.activator.set(activator.map(|i| i.entity_handle()));

        let spawn_flags = self.spawn_flags();
        if spawn_flags.intersects(DoorSpawnFlags::NO_AUTO_RETURN)
            && self.state.get() == MoveState::AtEnd
        {
            self.go_start();
        } else {
            // TODO: give health to player???
            self.lock_sounds.play_door(false, self.base.vars());
            self.go_end();
        }

        true
    }

    fn block(&self, other_v: &EntityVars) {
        if self.wait < 0.0 {
            return;
        }

        let engine = self.engine();
        let v = self.base.vars();
        self.door_move.realign_to(v, other_v);

        if !self.spawn_flags().intersects(DoorSpawnFlags::SILENT) {
            if let Some(noise_moving) = v.noise_moving() {
                engine.build_sound().channel_static().stop(noise_moving, v);
            }
        }

        if self.state.get() == MoveState::GoingToStart {
            self.go_end();
        } else {
            self.go_start();
        }
    }
}

impl<T: Move> EntityCast for BaseDoor<T> {
    impl_entity_cast!(cast BaseDoor<T>);
}

impl<T: Move> Entity for BaseDoor<T> {
    delegate_entity!(base not {
        object_caps, key_value, precache, spawn, used, touched, blocked, think
    });

    fn object_caps(&self) -> ObjectCaps {
        let caps = self
            .base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION);
        if self.spawn_flags().intersects(DoorSpawnFlags::USE_ONLY) {
            caps.union(ObjectCaps::IMPULSE_USE)
        } else {
            caps
        }
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"master" => {
                self.master = Some(self.engine().new_map_string(data.value()));
            }
            b"wait" => self.wait = data.parse_or_default(),
            b"movesnd" => self.move_sound = data.parse_or_default(),
            b"stopsnd" => self.stop_sound = data.parse_or_default(),
            _ => {
                if self.lock_sounds.key_value(data) {
                    return;
                }
                if self.door_move.key_value(data) {
                    return;
                }
                if self.delayed.key_value(data) {
                    return;
                }
                self.base.key_value(data);
                return;
            }
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();

        let get_sound = |sounds: &[&'static CStr], default: &'static CStr, index: u8| {
            let sound = sounds.get(index as usize).copied().unwrap_or(default);
            if sound != res::valve::sound::common::NULL {
                engine.precache_sound(sound);
            }
            engine.new_map_string(sound)
        };

        v.set_noise_moving(get_sound(MOVE_SOUNDS, MOVE_SOUNDS[0], self.move_sound));
        v.set_noise_arrived(get_sound(STOP_SOUNDS, STOP_SOUNDS[0], self.stop_sound));

        self.lock_sounds.precache();
    }

    fn spawn(&mut self) {
        let spawn_flags = self.spawn_flags();
        let v = self.base.vars();

        v.set_move_type(MoveType::Push);
        v.link();
        v.reload_model();

        if v.speed() == 0.0 {
            v.set_speed(100.0);
        }

        self.door_move.init(v);
        if spawn_flags.intersects(DoorSpawnFlags::START_OPEN) {
            self.door_move.swap(v);
        }

        self.state.set(MoveState::AtStart);
        self.enable_touch
            .set(!spawn_flags.intersects(DoorSpawnFlags::USE_ONLY));
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, _: &dyn Entity) {
        let spawn_flags = self.spawn_flags();
        match self.state.get() {
            MoveState::AtStart => {
                self.door_activate(activator);
            }
            MoveState::AtEnd => {
                if spawn_flags.intersects(DoorSpawnFlags::NO_AUTO_RETURN) {
                    self.door_activate(activator);
                }
            }
            _ => {}
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if !self.enable_touch.get() || !other.is_player() {
            return;
        }

        let engine = self.engine();
        let v = self.base.vars();

        if !utils::is_master_triggered(&engine, self.master, Some(other)) {
            self.lock_sounds.play_door(true, v);
            return;
        }

        if v.target_name().is_some() {
            // touching does nothing if the door is somebody's target
            self.lock_sounds.play_door(true, v);
            return;
        }

        if self.door_activate(Some(other)) {
            // temporary disable touched until movement is finished
            self.enable_touch.set(false);
        }
    }

    fn blocked(&self, other: &dyn Entity) {
        let engine = self.engine();
        let v = self.base.vars();

        if v.damage() != 0.0 {
            other.take_damage(v.damage(), DamageFlags::CRUSH, v, Some(v));
        }

        // TODO: satchelfix

        if self.wait >= 0.0 {
            if !self.spawn_flags().intersects(DoorSpawnFlags::SILENT) {
                if let Some(noise_moving) = v.noise_moving() {
                    engine.build_sound().channel_static().stop(noise_moving, v);
                }
            }

            if self.state.get() == MoveState::GoingToStart {
                self.go_end();
            } else {
                self.go_start();
            }
        }

        // block all door pieces with the same target name
        if let Some(target_name) = v.target_name() {
            for i in engine.entities().by_target_name(target_name) {
                if i.vars() != *v {
                    // TODO: define door trait?
                    if let Some(door) = i.downcast_ref::<Door>() {
                        door.base.block(v);
                    } else if let Some(door) = i.downcast_ref::<RotatingDoor>() {
                        door.base.block(v);
                    }
                }
            }
        }
    }

    fn think(&self) {
        match self.think.take() {
            DoorThink::None => {}
            DoorThink::MoveDone => {
                if self.door_move.move_done(self.base.vars()) {
                    match self.state.get() {
                        MoveState::GoingToStart => self.hit_start(),
                        MoveState::GoingToEnd => self.hit_end(),
                        state => unreachable!("think: unexpected start {state:?}"),
                    }
                }
            }
            DoorThink::DoorGoDown => self.go_start(),
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Door {
    base: BaseDoor<LinearMove>,
}

impl_entity_cast!(Door);

impl CreateEntity for Door {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseDoor::create(base),
        }
    }
}

impl Entity for Door {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.precache();

        let spawn_flags = self.base.spawn_flags();
        let v = self.base.vars();
        v.set_move_dir_from_angles();

        if v.skin() == 0 {
            // normal door
            if spawn_flags.intersects(DoorSpawnFlags::PASSABLE) {
                v.set_solid(Solid::Not);
            } else {
                v.set_solid(Solid::Bsp);
            }
        } else {
            // special contents
            v.set_solid(Solid::Not);
            v.with_spawn_flags(|f| f | DoorSpawnFlags::SILENT.bits());
        }

        self.base.spawn();
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct RotatingDoor {
    base: BaseDoor<AngularMove>,
}

impl_entity_cast!(RotatingDoor);

impl CreateEntity for RotatingDoor {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseDoor::create(base),
        }
    }
}

impl Entity for RotatingDoor {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.precache();

        let sf = self.base.spawn_flags();
        let v = self.base.vars();

        v.set_move_dir_from_spawn_flags(
            DoorSpawnFlags::ROTATE_X.bits(),
            DoorSpawnFlags::ROTATE_Z.bits(),
        );

        if sf.intersects(DoorSpawnFlags::ROTATE_BACKWARDS) {
            v.with_move_dir(|x| -x);
        }

        if sf.intersects(DoorSpawnFlags::PASSABLE) {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }

        self.base.spawn();
    }
}

export_entity_default!("export-func_door", func_door, Door);
// func_water is the same as a door.
export_entity_default!("export-func_water", func_water, Door);

export_entity_default!(
    "export-func_door_rotating",
    func_door_rotating,
    RotatingDoor
);
