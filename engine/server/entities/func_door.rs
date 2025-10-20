use core::{
    cell::{Cell, RefCell},
    ffi::CStr,
    mem,
};

use bitflags::bitflags;
use xash3d_shared::{
    entity::{DamageFlags, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    entities::delayed_use::DelayedUse,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityCast,
        EntityHandle, EntityVars, KeyValue, ObjectCaps, Solid, UseType,
    },
    export::export_entity_default,
    prelude::*,
    save::{PositionVector, Restore, Save},
    sound::{self, button_sound_or_default, LockSounds},
    str::MapString,
    utils,
};

#[cfg(feature = "save")]
use crate::save;

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

pub trait Move: Save + Restore + 'static {
    fn is_reversable(&self) -> bool {
        false
    }

    fn swap(&mut self, v: &EntityVars);

    fn realign_to(&self, v: &EntityVars, other: &EntityVars);

    /// Returns `true` if movement is finished.
    fn move_done(&self, v: &EntityVars) -> bool;

    /// Returns `true` if movement is finished.
    fn move_up(&self, v: &EntityVars, speed: f32, reverse: bool) -> bool;

    /// Returns `true` if movement is finished.
    fn move_down(&self, v: &EntityVars, speed: f32) -> bool;
}

#[derive(Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
struct LinearMove {
    start: PositionVector,
    end: PositionVector,
    dest: Cell<PositionVector>,
}

impl LinearMove {
    fn start_move(&self, v: &EntityVars, speed: f32, dest: vec3_t) -> bool {
        assert_ne!(speed, 0.0, "linear_move: speed is zero");

        self.dest.set(dest.into());
        if dest == v.origin() {
            return self.move_done(v);
        }

        let dest_delta = dest - v.origin();
        let travel_time = dest_delta.length() / speed;
        v.set_velocity(dest_delta / travel_time);
        v.set_next_think_time_from_last(travel_time);
        false
    }
}

impl Move for LinearMove {
    fn swap(&mut self, v: &EntityVars) {
        mem::swap(&mut self.start, &mut self.end);
        v.set_origin_and_link(self.start);
    }

    fn realign_to(&self, v: &EntityVars, other: &EntityVars) {
        if v.velocity() == other.velocity() {
            v.set_origin(other.origin());
            v.set_velocity(vec3_t::ZERO);
        }
    }

    fn move_done(&self, v: &EntityVars) -> bool {
        let dest = self.dest.get().to_vec();
        let delta = dest - v.origin();
        let error = delta.length();
        if error > 0.03125 {
            self.start_move(v, 100.0, dest);
            return false;
        }

        v.set_origin_and_link(dest);
        v.set_velocity(vec3_t::ZERO);
        v.stop_thinking();
        true
    }

    fn move_up(&self, v: &EntityVars, speed: f32, _: bool) -> bool {
        self.start_move(v, speed, self.end.into())
    }

    fn move_down(&self, v: &EntityVars, speed: f32) -> bool {
        self.start_move(v, speed, self.start.into())
    }
}

#[derive(Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
struct AngularMove {
    start: vec3_t,
    end: vec3_t,
    dest: Cell<vec3_t>,
}

impl AngularMove {
    fn start_move(&self, v: &EntityVars, speed: f32, dest: vec3_t) -> bool {
        assert_ne!(speed, 0.0, "angular_move: speed is zero");

        self.dest.set(dest);
        if dest == v.angles() {
            return self.move_done(v);
        }

        let delta = dest - v.angles();
        let travel_time = delta.length() / speed;
        v.set_angular_velocity(delta / travel_time);
        v.set_next_think_time_from_last(travel_time);
        false
    }
}

impl Move for AngularMove {
    fn is_reversable(&self) -> bool {
        true
    }

    fn swap(&mut self, v: &EntityVars) {
        mem::swap(&mut self.start, &mut self.end);
        v.set_angles(self.start);
        v.with_move_dir(|x| -x);
    }

    fn realign_to(&self, v: &EntityVars, other: &EntityVars) {
        if v.angular_velocity() == other.angular_velocity() {
            v.set_angles(other.angles());
            v.set_angular_velocity(vec3_t::ZERO);
        }
    }

    fn move_done(&self, v: &EntityVars) -> bool {
        v.set_angles(self.dest.get());
        v.set_angular_velocity(vec3_t::ZERO);
        v.stop_thinking();
        true
    }

    fn move_up(&self, v: &EntityVars, speed: f32, reverse: bool) -> bool {
        if reverse {
            self.start_move(v, speed, -self.end)
        } else {
            self.start_move(v, speed, self.end)
        }
    }

    fn move_down(&self, v: &EntityVars, speed: f32) -> bool {
        self.start_move(v, speed, self.start)
    }
}

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
#[repr(u8)]
enum DoorState {
    #[default]
    Bottom = 0,
    Top,
    GoingDown,
    GoingUp,
}

impl DoorState {
    fn is_moving(&self) -> bool {
        matches!(self, Self::GoingUp | Self::GoingDown)
    }
}

#[cfg(feature = "save")]
impl Save for DoorState {
    fn save(&self, _: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        cur.write_u8(*self as u8)?;
        Ok(())
    }
}

#[cfg(feature = "save")]
impl Restore for DoorState {
    fn restore(&mut self, _: &save::RestoreState, cur: &mut save::Cursor) -> save::SaveResult<()> {
        *self = match cur.read_u8()? {
            0 => Self::Bottom,
            1 => Self::Top,
            2 => Self::GoingDown,
            3 => Self::GoingUp,
            _ => return Err(save::SaveError::InvalidEnum),
        };
        Ok(())
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct BaseDoor<T> {
    base: BaseEntity,
    delayed: DelayedUse,
    wait: f32,
    master: Option<MapString>,

    state: Cell<DoorState>,
    enable_touch: Cell<bool>,
    think: Cell<u8>,
    on_move_done: Cell<u8>,
    activator: Cell<Option<EntityHandle>>,

    door_move: T,

    move_sound: u8,
    stop_sound: u8,
    locked_sound: u8,
    locked_sentence: u8,
    unlocked_sound: u8,
    unlocked_sentence: u8,

    #[cfg_attr(feature = "save", save(skip))]
    lock_sounds: RefCell<LockSounds>,
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
            on_move_done: Default::default(),
            activator: Cell::default(),

            door_move: Default::default(),

            move_sound: 0,
            stop_sound: 0,
            locked_sound: 0,
            locked_sentence: 0,
            unlocked_sound: 0,
            unlocked_sentence: 0,

            lock_sounds: RefCell::default(),
        }
    }
}

impl<T: Move> BaseDoor<T> {
    const THINK_MOVE_DONE: u8 = 1;
    const THINK_DOOR_GO_DOWN: u8 = 2;

    const DOOR_HIT_TOP: u8 = 1;
    const DOOR_HIT_BOTTOM: u8 = 2;

    fn spawn_flags(&self) -> DoorSpawnFlags {
        DoorSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn door_hit_top(&self) {
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

        assert_eq!(self.state.get(), DoorState::GoingUp);
        self.state.set(DoorState::Top);

        if spawn_flags.intersects(DoorSpawnFlags::NO_AUTO_RETURN) {
            if !spawn_flags.intersects(DoorSpawnFlags::USE_ONLY) {
                self.enable_touch.set(true);
            }
        } else {
            v.set_next_think_time_from_last(self.wait);
            self.think.set(Self::THINK_DOOR_GO_DOWN);

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

    fn door_hit_bottom(&self) {
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

        assert_eq!(self.state.get(), DoorState::GoingDown);
        self.state.set(DoorState::Bottom);
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

    fn on_move_done(&self) {
        match self.on_move_done.get() {
            Self::DOOR_HIT_TOP => self.door_hit_top(),
            Self::DOOR_HIT_BOTTOM => self.door_hit_bottom(),
            kind => {
                warn!("{}: unimplemented move done {kind}", self.classname())
            }
        }
    }

    fn door_go_up(&self) {
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

        self.state.set(DoorState::GoingUp);
        self.on_move_done.set(Self::DOOR_HIT_TOP);

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

        if self.door_move.move_up(v, v.speed(), reverse) {
            self.on_move_done();
        } else {
            self.think.set(Self::THINK_MOVE_DONE);
        }
    }

    fn door_go_down(&self) {
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

        self.state.set(DoorState::GoingDown);
        self.on_move_done.set(Self::DOOR_HIT_BOTTOM);

        let v = self.base.vars();
        if self.door_move.move_down(v, v.speed()) {
            self.on_move_done();
        } else {
            self.think.set(Self::THINK_MOVE_DONE);
        }
    }

    fn door_activate(&self, activator: Option<&dyn Entity>) -> bool {
        let engine = self.engine();
        if let Some((master, activator)) = self.master.zip(activator) {
            if !utils::is_master_triggered(&engine, master, activator) {
                return false;
            }
        }

        self.activator.set(activator.map(|i| i.entity_handle()));

        let spawn_flags = self.spawn_flags();
        if spawn_flags.intersects(DoorSpawnFlags::NO_AUTO_RETURN)
            && self.state.get() == DoorState::Top
        {
            self.door_go_down();
        } else {
            // TODO: give health to player???
            self.lock_sounds
                .borrow_mut()
                .play_door(false, self.base.vars());
            self.door_go_up();
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

        if self.state.get() == DoorState::GoingDown {
            self.door_go_up();
        } else {
            self.door_go_down();
        }
    }
}

impl<T: Move> EntityCast for BaseDoor<T> {
    impl_entity_cast!(cast BaseDoor<T>);
}

impl<T: Move> Entity for BaseDoor<T> {
    delegate_entity!(base not {
        object_caps, key_value, precache, used, touched, blocked, think
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
            b"movesnd" => self.move_sound = data.parse_or_default(),
            b"stopsnd" => self.stop_sound = data.parse_or_default(),
            b"locked_sound" => self.locked_sound = data.parse_or_default(),
            b"locked_sentence" => self.locked_sentence = data.parse_or_default(),
            b"unlocked_sound" => self.unlocked_sound = data.parse_or_default(),
            b"unlocked_sentence" => self.unlocked_sentence = data.parse_or_default(),
            b"wait" => self.wait = data.parse_or_default(),
            b"master" => {
                self.master = Some(self.engine().new_map_string(data.value()));
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

        let mut lock_sounds = self.lock_sounds.borrow_mut();

        lock_sounds.locked_sound = if self.locked_sound != 0 {
            let sound = button_sound_or_default(self.locked_sound as usize);
            engine.precache_sound(sound);
            Some(engine.new_map_string(sound))
        } else {
            None
        };

        lock_sounds.unlocked_sound = if self.unlocked_sound != 0 {
            let sound = button_sound_or_default(self.unlocked_sound as usize);
            engine.precache_sound(sound);
            Some(engine.new_map_string(sound))
        } else {
            None
        };

        lock_sounds.locked_sentence = self
            .locked_sentence
            .checked_sub(1)
            .and_then(|index| sound::LOCK_SENTENCES.get(index as usize))
            .map(|&s| engine.new_map_string(s));

        lock_sounds.unlocked_sentence = self
            .unlocked_sentence
            .checked_sub(1)
            .and_then(|index| sound::UNLOCK_SENTENCES.get(index as usize))
            .map(|&s| engine.new_map_string(s));
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, _: &dyn Entity) {
        let spawn_flags = self.spawn_flags();
        match self.state.get() {
            DoorState::Bottom => {
                self.door_activate(activator);
            }
            DoorState::Top => {
                if spawn_flags.intersects(DoorSpawnFlags::NO_AUTO_RETURN) {
                    self.door_activate(activator);
                }
            }
            _ => {}
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if !self.enable_touch.get() {
            return;
        }

        let engine = self.engine();
        let v = self.base.vars();

        if let Some(master) = self.master {
            if !utils::is_master_triggered(&engine, master, other) {
                self.lock_sounds.borrow_mut().play_door(true, v);
            }
        }

        if v.target_name().is_some() {
            // touching does nothing if the door is somebody's target
            self.lock_sounds.borrow_mut().play_door(true, v);
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

            if self.state.get() == DoorState::GoingDown {
                self.door_go_up();
            } else {
                self.door_go_down();
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
            Self::THINK_MOVE_DONE => {
                if self.door_move.move_done(self.base.vars()) {
                    self.on_move_done();
                }
            }
            Self::THINK_DOOR_GO_DOWN => {
                self.door_go_down();
            }
            think => {
                warn!("{}: unimplemented think kind {think}", self.classname());
            }
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Door {
    base: BaseDoor<LinearMove>,
    lip: f32,
}

impl_entity_cast!(Door);

impl CreateEntity for Door {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseDoor::create(base),
            lip: 0.0,
        }
    }
}

impl Entity for Door {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"lip" {
            self.lip = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

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

        v.set_move_type(MoveType::Push);
        v.link();
        v.reload_model();

        if v.speed() == 0.0 {
            v.set_speed(100.0);
        }

        let start = v.origin();
        let tmp = (v.move_dir() * (v.size() - 2.0)).abs();
        let end = start + (v.move_dir() * (tmp.element_sum() - self.lip));
        self.base.door_move.start = start.into();
        self.base.door_move.end = end.into();

        if spawn_flags.intersects(DoorSpawnFlags::START_OPEN) {
            self.base.door_move.swap(self.base.base.vars());
        }

        self.base.state.set(DoorState::Bottom);
        self.base
            .enable_touch
            .set(!spawn_flags.intersects(DoorSpawnFlags::USE_ONLY));
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct RotatingDoor {
    base: BaseDoor<AngularMove>,

    move_distance: f32,
}

impl RotatingDoor {
    fn set_move_dir_from_spawn_flags(&self) {
        let v = self.vars();
        let flags = self.base.spawn_flags();
        if flags.intersects(DoorSpawnFlags::ROTATE_Z) {
            v.set_move_dir(vec3_t::Z);
        } else if flags.intersects(DoorSpawnFlags::ROTATE_X) {
            v.set_move_dir(vec3_t::X);
        } else {
            v.set_move_dir(vec3_t::Y);
        }
    }
}

impl_entity_cast!(RotatingDoor);

impl CreateEntity for RotatingDoor {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseDoor::create(base),

            move_distance: Default::default(),
        }
    }
}

impl Entity for RotatingDoor {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"distance" {
            self.move_distance = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        self.precache();

        self.set_move_dir_from_spawn_flags();

        let v = self.base.vars();
        let spawn_flags = self.base.spawn_flags();
        if spawn_flags.intersects(DoorSpawnFlags::ROTATE_BACKWARDS) {
            v.with_move_dir(|x| -x);
        }

        let start_angle = v.angles();
        let move_dir = v.move_dir();
        self.base.door_move.start = start_angle;
        self.base.door_move.end = start_angle + move_dir * self.move_distance;
        assert_ne!(
            self.base.door_move.start, self.base.door_move.end,
            "rotating door start and end angles are equal"
        );

        let v = self.base.vars();
        if spawn_flags.intersects(DoorSpawnFlags::PASSABLE) {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }

        v.set_move_type(MoveType::Push);
        v.link();
        v.reload_model();

        if v.speed() == 0.0 {
            v.set_speed(100.0);
        }

        if spawn_flags.intersects(DoorSpawnFlags::START_OPEN) {
            self.base.door_move.swap(self.base.base.vars());
        }
        self.base.state.set(DoorState::Bottom);
        self.base
            .enable_touch
            .set(!spawn_flags.intersects(DoorSpawnFlags::USE_ONLY));
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
