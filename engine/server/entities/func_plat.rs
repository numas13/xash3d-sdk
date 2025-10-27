use core::cell::Cell;

use xash3d_shared::{
    entity::{DamageFlags, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityHandle,
        EntityVars, KeyValue, ObjectCaps, Private, Solid, UseType,
    },
    export::export_entity_default,
    prelude::*,
    str::MapString,
    utils::{LinearMove, Move, MoveState},
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

trait EntityVarsExt {
    fn vars(&self) -> &EntityVars;

    fn moving_noise(&self) -> Option<MapString> {
        self.vars().noise()
    }

    fn set_moving_noise(&self, sound: MapString) {
        self.vars().set_noise(Some(sound));
    }

    fn moving_stop_noise(&self) -> Option<MapString> {
        self.vars().noise1()
    }

    fn set_moving_stop_noise(&self, sound: MapString) {
        self.vars().set_noise1(Some(sound));
    }
}

impl EntityVarsExt for EntityVars {
    fn vars(&self) -> &EntityVars {
        self
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
struct PlatformTrigger {
    base: BaseEntity,
    platform: EntityHandle,
}

impl_entity_cast!(PlatformTrigger);

impl Entity for PlatformTrigger {
    delegate_entity!(base not { object_caps, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
            .union(ObjectCaps::DONT_SAVE)
    }

    fn touched(&self, other: &dyn Entity) {
        if !other.is_player() {
            return;
        }

        let Some(platform) = self.platform.get_entity() else {
            self.remove_from_world();
            return;
        };

        if !other.is_alive() {
            return;
        }

        // TODO: define platform/moving trait
        let platform = platform
            .downcast_ref::<Platform>()
            .or_else(|| platform.downcast_ref::<RotatingPlatform>().map(|i| &i.base))
            .expect("Platform or RotatingPlatform");

        match platform.state.get() {
            MoveState::AtStart => platform.go_end_with_delay(1.0),
            MoveState::AtEnd => {
                // delay going to start
                platform.vars().set_next_think_time_from_last(1.0);
            }
            _ => {}
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    MoveDone,
    GoStart,
    GoEnd,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Platform {
    base: BaseEntity,

    volume: f32,
    movesnd: u8,
    stopsnd: u8,
    wait: f32,

    height: f32,
    linear: LinearMove,

    rotation: f32,
    start_angles: vec3_t,
    end_angles: vec3_t,

    state: Cell<MoveState>,
    enable_use: Cell<bool>,
    think: Cell<Think>,
}

impl CreateEntity for Platform {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            volume: 0.0,
            movesnd: 0,
            stopsnd: 0,
            wait: 0.0,

            height: 0.0,
            linear: LinearMove::default(),

            rotation: 0.0,
            start_angles: vec3_t::ZERO,
            end_angles: vec3_t::ZERO,

            state: Cell::default(),
            enable_use: Cell::default(),
            think: Cell::default(),
        }
    }
}

impl Platform {
    const SF_TOGGLE: u32 = 1 << 0;

    pub fn is_toggle_platform(&self) -> bool {
        self.vars().spawn_flags() & Self::SF_TOGGLE != 0
    }

    fn emit_moving_noise(&self) {
        let v = self.vars();
        if let Some(sound) = v.moving_noise() {
            self.engine()
                .build_sound()
                .channel_static()
                .volume(self.volume)
                .emit_dyn(sound, v);
        }
    }

    fn stop_moving_noise(&self) {
        let v = self.vars();
        if let Some(sound) = v.moving_noise() {
            self.engine().build_sound().channel_static().stop(sound, v);
        }
    }

    fn emit_moving_stop_noise(&self) {
        let v = self.vars();
        if let Some(sound) = v.moving_stop_noise() {
            self.engine()
                .build_sound()
                .channel_static()
                .volume(self.volume)
                .emit_dyn(sound, v);
        }
    }

    fn rot_move(&self, dest: vec3_t) {
        let v = self.vars();
        let delta = dest - v.angles();
        let time = v.next_think_time() - v.last_think_time();
        if time >= 0.1 {
            v.set_angular_velocity(delta / time);
        } else {
            v.set_angular_velocity(delta);
            v.set_next_think_time_from_last(1.0);
        }
    }

    fn go_start(&self) {
        self.emit_moving_noise();

        assert!(matches!(
            self.state.get(),
            MoveState::AtEnd | MoveState::GoingToEnd
        ));
        self.state.set(MoveState::GoingToStart);

        let v = self.vars();
        if self.linear.move_to_start(v, v.speed()) {
            self.hit_start();
        } else {
            self.think.set(Think::MoveDone);
        }

        if self.rotation != 0.0 {
            self.rot_move(self.start_angles);
        }
    }

    fn hit_start(&self) {
        self.stop_moving_noise();
        self.emit_moving_stop_noise();

        assert_eq!(self.state.get(), MoveState::GoingToStart);
        self.state.set(MoveState::AtStart);

        let v = self.vars();
        v.set_velocity(vec3_t::ZERO);

        if self.rotation != 0.0 {
            v.set_angular_velocity(vec3_t::ZERO);
            v.set_angles(self.start_angles);
        }
    }

    fn go_end_with_delay(&self, delay: f32) {
        if self.state.get() == MoveState::AtStart && self.think.get() != Think::GoEnd {
            self.think.set(Think::GoEnd);
            self.vars().set_next_think_time_from_last(delay);
        }
    }

    fn go_end(&self) {
        self.emit_moving_noise();

        assert!(matches!(
            self.state.get(),
            MoveState::AtStart | MoveState::GoingToStart
        ));
        self.state.set(MoveState::GoingToEnd);

        let v = self.vars();
        if self.linear.move_to_end(v, v.speed(), false) {
            self.hit_end();
        } else {
            self.think.set(Think::MoveDone);
        }

        if self.rotation != 0.0 {
            self.rot_move(self.end_angles);
        }
    }

    fn hit_end(&self) {
        self.stop_moving_noise();
        self.emit_moving_stop_noise();

        assert_eq!(self.state.get(), MoveState::GoingToEnd);
        self.state.set(MoveState::AtEnd);

        if !self.is_toggle_platform() {
            self.think.set(Think::GoStart);
            self.vars().set_next_think_time_from_last(self.wait);
        }

        let v = self.vars();
        v.set_velocity(vec3_t::ZERO);

        if self.rotation != 0.0 {
            v.set_angular_velocity(vec3_t::ZERO);
            v.set_angles(self.end_angles);
        }
    }

    fn create_trigger(&self) {
        self.engine()
            .new_entity_with::<Private<_>>(|base| PlatformTrigger {
                base,
                platform: self.entity_handle(),
            })
            .vars(|v| {
                let platform = self.vars();
                v.set_solid(Solid::Trigger);
                v.set_move_type(MoveType::None);
                v.set_origin(platform.origin());

                let mut min = platform.min_size() + vec3_t::new(25.0, 25.0, 0.0);
                let mut max = platform.max_size() + vec3_t::new(25.0, 25.0, 8.0);
                min.z = max.z - (self.linear.start().z - self.linear.end().z + 8.0);

                if platform.size().x <= 50.0 {
                    min.x = (platform.min_size().x + platform.max_size().x) / 2.0;
                    max.x = min.x + 1.0;
                }
                if platform.size().y <= 50.0 {
                    min.y = (platform.min_size().y + platform.max_size().y) / 2.0;
                    max.y = min.y + 1.0;
                }
                v.set_size_and_link(min, max);
            });
    }
}

impl_entity_cast!(Platform);

impl Entity for Platform {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, used, blocked, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"wait" => self.wait = data.parse_or_default(),
            b"height" => self.height = data.parse_or_default(),
            b"volume" => self.volume = data.parse_or_default(),
            b"movesnd" => self.movesnd = data.parse_or_default(),
            b"stopsnd" => self.stopsnd = data.parse_or_default(),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();

        let move_sounds = &[
            res::valve::sound::common::NULL,
            res::valve::sound::plats::BIGMOVE1,
            res::valve::sound::plats::BIGMOVE2,
            res::valve::sound::plats::ELEVMOVE1,
            res::valve::sound::plats::ELEVMOVE2,
            res::valve::sound::plats::ELEVMOVE3,
            res::valve::sound::plats::FREIGHTMOVE1,
            res::valve::sound::plats::FREIGHTMOVE2,
            res::valve::sound::plats::HEAVYMOVE1,
            res::valve::sound::plats::RACKMOVE1,
            res::valve::sound::plats::RAILMOVE1,
            res::valve::sound::plats::SQUEEKMOVE1,
            res::valve::sound::plats::TALKMOVE1,
            res::valve::sound::plats::TALKMOVE2,
        ];

        let move_sound = move_sounds
            .get(self.movesnd as usize)
            .inspect(|&&sound| {
                if self.movesnd != 0 {
                    engine.precache_sound(sound);
                }
            })
            .unwrap_or(&move_sounds[0]);

        v.set_moving_noise(engine.new_map_string(*move_sound));

        let stop_sounds = &[
            res::valve::sound::common::NULL,
            res::valve::sound::plats::BIGSTOP1,
            res::valve::sound::plats::BIGSTOP2,
            res::valve::sound::plats::FREIGHTSTOP1,
            res::valve::sound::plats::HEAVYSTOP2,
            res::valve::sound::plats::RACKSTOP1,
            res::valve::sound::plats::RAILSTOP1,
            res::valve::sound::plats::SQUEEKSTOP1,
            res::valve::sound::plats::TALKSTOP1,
        ];

        let stop_sound = stop_sounds
            .get(self.stopsnd as usize)
            .inspect(|&&sound| {
                if self.stopsnd != 0 {
                    engine.precache_sound(sound);
                }
            })
            .unwrap_or(&move_sounds[0]);

        v.set_moving_stop_noise(engine.new_map_string(*stop_sound));

        if !self.is_toggle_platform() {
            self.create_trigger();
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_angles(vec3_t::ZERO);
        v.set_solid(Solid::Bsp);
        v.set_move_type(MoveType::Push);
        v.link();
        v.set_size_and_link(v.min_size(), v.max_size());
        v.reload_model();

        if v.speed() == 0.0 {
            v.set_speed(150.0);
        }

        if self.volume == 0.0 {
            self.volume = 0.85;
        }

        if self.wait == 0.0 {
            self.wait = 3.0;
        }

        self.linear.set_start(v.origin());
        let mut end = v.origin();
        if self.height != 0.0 {
            end.z -= self.height;
        } else {
            end.z = end.z - v.size().z + 8.0;
        }
        self.linear.set_end(end);

        self.precache();

        let v = self.base.vars();
        if v.target_name().is_some() {
            v.set_origin_and_link(self.linear.start());
            self.state.set(MoveState::AtStart);
            self.enable_use.set(true);
        } else {
            v.set_origin_and_link(self.linear.end());
            self.state.set(MoveState::AtEnd);
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        if self.is_toggle_platform() {
            let state = self.state.get();
            if !use_type.should_toggle(state == MoveState::AtEnd) {
                return;
            }
            match state {
                MoveState::AtStart => self.go_end(),
                MoveState::AtEnd => self.go_start(),
                _ => {}
            }
        } else {
            self.enable_use.set(false);
            if self.state.get() == MoveState::AtStart {
                self.go_end();
            }
        }
    }

    fn blocked(&self, other: &dyn Entity) {
        trace!("{}: blocked by {}", self.pretty_name(), other.pretty_name());

        let v = self.vars();
        other.take_damage(1.0, DamageFlags::CRUSH, v, Some(v));

        self.stop_moving_noise();

        match self.state.get() {
            MoveState::GoingToStart => self.go_end(),
            MoveState::GoingToEnd => self.go_start(),
            state => unreachable!("blocked: unexpected state {state:?}"),
        }
    }

    fn think(&self) {
        match self.think.take() {
            Think::None => {}
            Think::MoveDone => match self.state.get() {
                MoveState::GoingToStart => self.hit_start(),
                MoveState::GoingToEnd => self.hit_end(),
                state => unreachable!("think: unexpected state {state:?}"),
            },
            Think::GoEnd => self.go_end(),
            Think::GoStart => self.go_start(),
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct RotatingPlatform {
    base: Platform,
}

impl_entity_cast!(RotatingPlatform);

impl CreateEntity for RotatingPlatform {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Platform::create(base),
        }
    }
}

impl RotatingPlatform {
    const SF_ROTATE_Z: u32 = 1 << 6;
    const SF_ROTATE_X: u32 = 1 << 7;
}

impl Entity for RotatingPlatform {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"rotation" {
            self.base.rotation = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        self.base.spawn();

        let v = self.base.base.vars();
        if self.base.rotation != 0.0 {
            v.set_move_dir_from_spawn_flags(Self::SF_ROTATE_X, Self::SF_ROTATE_Z);
            self.base.start_angles = v.angles() + v.move_dir() * self.base.rotation;
            self.base.end_angles = v.angles();
        } else {
            self.base.start_angles = vec3_t::ZERO;
            self.base.end_angles = vec3_t::ZERO;
        }

        if v.target_name().is_some() {
            v.set_angles(self.base.start_angles);
        }
    }
}

export_entity_default!("export-func_plat", func_plat, Platform);
export_entity_default!("export-func_platrot", func_platrot, RotatingPlatform);
