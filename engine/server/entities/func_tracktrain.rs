use core::{cell::Cell, ffi::CStr};

use bitflags::bitflags;
use xash3d_shared::{
    entity::{EdictFlags, MoveType},
    ffi::common::vec3_t,
    math::{angle_distance, approach_angle, fabsf},
};

use crate::{
    engine::EventIndex,
    entities::path_track::PathTrack,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, EntityHandle, KeyValue, ObjectCaps, Solid,
        UseType,
    },
    export::export_entity_default,
    prelude::*,
    utils,
};

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const NO_PITCH       = 1 << 0;
        const NO_CONTROL     = 1 << 1;
        const FORWARD_ONLY   = 1 << 2;
        const NOT_SOLID      = 1 << 3;
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    Find,
    Next,
    DeadEnd(bool),
    NearestPath,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TrackTrain {
    base: BaseEntity,

    height: f32,
    length: f32,
    start_speed: f32,
    speed: f32,
    direction: f32,
    bank: f32,
    control_min_size: vec3_t,
    control_max_size: vec3_t,

    sounds: u8,
    volume: f32,

    adjust_pitch: EventIndex,

    think: Cell<Think>,
    path: Cell<Option<EntityHandle>>,
    sound_playing: Cell<bool>,
}

impl_entity_cast!(TrackTrain);

impl CreateEntity for TrackTrain {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            height: 0.0,
            length: 0.0,
            start_speed: 0.0,
            speed: 0.0,
            direction: 0.0,
            bank: 0.0,
            control_min_size: vec3_t::ZERO,
            control_max_size: vec3_t::ZERO,

            sounds: 0,
            volume: 0.0,

            adjust_pitch: EventIndex::default(),

            think: Cell::default(),
            path: Cell::default(),
            sound_playing: Cell::default(),
        }
    }
}

impl TrackTrain {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_spawn_flags(&self, flags: SpawnFlags, state: bool) {
        self.vars().with_spawn_flags(|f| {
            if state {
                f | flags.bits()
            } else {
                f & !flags.bits()
            }
        });
    }

    fn stop_sound(&self) {
        if self.sound_playing.get() {
            let engine = self.engine();
            let v = self.vars();
            if v.noise().is_some() {
                engine
                    .build_playback_event()
                    .reliable()
                    .update()
                    .iparam1(((self.sounds & 0x7) as i32) << 12)
                    .bparam1(true)
                    .build(self.adjust_pitch, v);
            }

            engine
                .build_sound()
                .channel_item()
                .volume(self.volume)
                .emit_dyn(res::valve::sound::plats::TTRAIN_BRAKE1, v);
        }

        self.sound_playing.set(false);
    }

    fn update_sound(&self) {
        const START_PITCH: f32 = 60.0;
        const MAX_PITCH: f32 = 200.0;
        const MAX_SPEED: f32 = 1000.0;

        let engine = self.engine();
        let v = self.vars();
        let Some(noise) = v.noise() else {
            return;
        };
        let pitch = START_PITCH + fabsf(v.speed()) * (MAX_PITCH - START_PITCH) / MAX_SPEED;

        if !self.sound_playing.get() {
            engine
                .build_sound()
                .channel_item()
                .volume(self.volume)
                .emit_dyn(res::valve::sound::plats::TTRAIN_START1, v);

            engine
                .build_sound()
                .channel_static()
                .volume(self.volume)
                .pitch(pitch as i32)
                .emit_dyn(noise.as_thin(), v);

            self.sound_playing.set(true);
        } else {
            let sound = ((self.sounds & 0x7) as u16) << 12;
            let pitch = ((pitch / 10.0) as u16 & 0x3f) << 6;
            let volume = (self.volume * 40.0) as u16 & 0x3f;

            engine
                .build_playback_event()
                .reliable()
                .update()
                .iparam1((sound | pitch | volume) as i32)
                .build(self.adjust_pitch, v);
        }
    }

    fn set_next_think_time_from_last(&self, think: Think, relative: f32, always_think: bool) {
        let v = self.vars();
        v.with_flags(|mut f| {
            f.set(EdictFlags::ALWAYSTHINK, always_think);
            f
        });
        v.set_next_think_time_from_last(relative);
        self.think.set(think);
    }

    fn find(&self) {
        let Some(target) = self.target_entity() else {
            self.path.set(None);
            return;
        };

        let Some(target) = target.downcast_ref::<PathTrack>() else {
            let name = self.pretty_name();
            let target_name = target.pretty_name();
            error!("{name}: target {target_name} is not a PathTrack");
            self.path.set(None);
            return;
        };

        self.path.set(Some(target.entity_handle()));

        let engine = self.engine();
        let v = self.vars();
        let target_v = target.vars();

        let mut next_pos = target_v.origin();
        next_pos.z += self.height;
        let (look, _) = target.look_ahead(target_v.origin(), self.length, false);
        v.set_angles({
            let mut angles = engine.vec_to_angles(look - next_pos);
            if self.spawn_flags().intersects(SpawnFlags::NO_PITCH) {
                angles.x = 0.0;
            }
            angles.y += 180.0;
            angles
        });

        v.set_origin_and_link(next_pos);
        v.set_speed(self.start_speed);
        self.set_next_think_time_from_last(Think::Next, 0.1, false);
        self.update_sound();
    }

    fn nearest_path(&self) {
        let name = self.pretty_name();
        let engine = self.engine();
        let v = self.vars();

        let Some((nearest, dist)) = engine
            .entities()
            .in_sphere(v.origin(), 1024.0)
            .filter_map(|track| {
                let track_v = track.vars();
                let flags = track_v.flags();
                if flags.intersects(EdictFlags::CLIENT | EdictFlags::MONSTER) {
                    return None;
                }
                if !track_v.is_class_name(PathTrack::CLASS_NAME) {
                    return None;
                }
                let dist = (v.origin() - track_v.origin()).length();
                Some((track, dist))
            })
            .min_by(|(_, dist1), (_, dist2)| dist1.partial_cmp(dist2).unwrap())
        else {
            debug!("{name}: failed to find a nearby track");
            return;
        };

        trace!("{name}: nearest track is {}", nearest.vars().pretty_name());

        let mut nearest = nearest.downcast_ref::<PathTrack>().expect("PathTrack");
        if let Some(next) = nearest.next() {
            let next_dist = (v.origin() - next.vars().origin()).length();
            if dist < next_dist {
                nearest = next;
            }
        }

        self.path.set(Some(nearest.entity_handle()));

        if v.speed() != 0.0 {
            self.set_next_think_time_from_last(Think::Next, 0.1, false);
        }
    }

    fn next(&self) {
        let engine = self.engine();
        let v = self.vars();
        let name = self.pretty_name();

        if v.speed() == 0.0 {
            trace!("{name}: speed is 0.0");
            self.stop_sound();
            return;
        }

        let Some(path) = self.path.get() else {
            trace!("{name}: lost path");
            self.stop_sound();
            return;
        };

        let path = path.downcast_ref::<PathTrack>().expect("PathTrack");

        self.update_sound();

        let mut next_pos = v.origin();
        next_pos.z -= self.height;
        let (mut next_pos, next) = path.look_ahead(next_pos, v.speed() * 0.1, true);
        next_pos.z += self.height;
        let dist = if self.length > 0.0 {
            self.length
        } else {
            100.0
        };
        v.set_velocity((next_pos - v.origin()) * 10.0);

        let mut next_front = v.origin();
        next_front.z -= self.height;
        let (mut next_front, _) = path.look_ahead(next_front, dist, false);
        next_front.z += self.height;
        let delta = next_front - v.origin();
        let mut angles = engine.vec_to_angles(delta);
        angles.y += 180.0;

        angles = fix_angles(angles);
        v.with_angles(fix_angles);

        if next.is_none() || (delta.x == 0.0 && delta.y == 0.0) {
            angles = v.angles();
        }

        let vx = if !self.spawn_flags().intersects(SpawnFlags::NO_PITCH) {
            angle_distance(angles.x, v.angles().x)
        } else {
            0.0
        };
        let vy = angle_distance(angles.y, v.angles().y);

        v.with_angular_velocity(|x| x.with_x(vx * 10.0).with_y(vy * 10.0));

        if self.bank != 0.0 {
            v.with_angular_velocity(|mut avel| {
                let angles = v.angles();
                if avel.y < -5.0 {
                    let next = approach_angle(-self.bank, angles.z, self.bank * 2.0);
                    avel.z = angle_distance(next, angles.z);
                } else if avel.y > 5.0 {
                    let next = approach_angle(self.bank, angles.z, self.bank * 2.0);
                    avel.z = angle_distance(next, angles.z);
                } else {
                    let next = approach_angle(0.0, angles.z, self.bank * 4.0);
                    avel.z = angle_distance(next, angles.z) * 4.0;
                }
                avel
            });
        }

        if let Some(next) = next {
            if next.entity_handle() != path.entity_handle() {
                let fire = if v.speed() >= 0.0 { next } else { path };

                self.path.set(Some(next.entity_handle()));

                fire.fire_targets(self);

                if fire.disable_train() {
                    self.set_spawn_flags(SpawnFlags::NO_CONTROL, true);
                }

                if self.spawn_flags().intersects(SpawnFlags::NO_CONTROL) {
                    let fire_v = fire.vars();
                    if fire_v.speed() != 0.0 {
                        v.set_speed(fire_v.speed());
                        trace!("{name}: set speed to {:4.2}", v.speed());
                    }
                }
            }
            self.set_next_think_time_from_last(Think::Next, 0.5, true);
        } else {
            self.stop_sound();
            v.set_velocity(next_pos - v.origin());
            v.set_angular_velocity(vec3_t::ZERO);
            let distance = v.velocity().length();
            let old_speed = v.speed();
            v.set_speed(0.0);

            let backward = old_speed < 0.0;
            if distance > 0.0 {
                let time = distance / old_speed;
                v.with_velocity(|x| x * (old_speed / distance));
                self.set_next_think_time_from_last(Think::DeadEnd(backward), time, false);
            } else {
                self.dead_end(backward);
            }
        }
    }

    fn dead_end(&self, backward: bool) {
        let name = self.pretty_name();
        let v = self.vars();
        v.set_velocity(vec3_t::ZERO);
        v.set_angular_velocity(vec3_t::ZERO);

        let Some(mut track) = self.path.get().downcast_ref::<PathTrack>() else {
            trace!("{name}: dead end");
            return;
        };

        if backward {
            track = track.first(true);
        } else {
            track = track.last(true);
        }

        trace!("{name}: dead end at {}", track.pretty_name());

        if let Some(net_name) = track.vars().net_name().as_deref() {
            utils::fire_targets(net_name, UseType::Toggle, Some(self), self);
        }
    }
}

fn fix_angles(mut v: vec3_t) -> vec3_t {
    for i in v.as_mut() {
        while *i < 0.0 {
            *i += 360.0;
        }
        while *i > 360.0 {
            *i -= 360.0;
        }
    }
    v
}

impl Entity for TrackTrain {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, used, think, override_reset });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
            .union(ObjectCaps::DIRECTIONAL_USE)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"wheels" => self.length = data.parse_or_default(),
            b"height" => self.height = data.parse_or_default(),
            b"startspeed" => self.start_speed = data.parse_or_default(),
            b"bank" => self.bank = data.parse_or_default(),
            b"sounds" => self.sounds = data.parse_or_default(),
            b"volume" => self.volume = data.parse_or_default::<u8>() as f32 * 0.1,
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();

        if self.volume == 0.0 {
            self.volume = 1.0;
        }

        const SOUNDS: &[&CStr] = &[
            res::valve::sound::plats::TTRAIN1,
            res::valve::sound::plats::TTRAIN2,
            res::valve::sound::plats::TTRAIN3,
            res::valve::sound::plats::TTRAIN4,
            res::valve::sound::plats::TTRAIN6,
            res::valve::sound::plats::TTRAIN7,
        ];

        let index = self.sounds.checked_sub(1);
        match index.and_then(|i| SOUNDS.get(i as usize)) {
            Some(sound) => {
                engine.precache_sound(*sound);
                v.set_noise(engine.new_map_string(*sound));
            }
            None => v.set_noise(None),
        }

        engine.precache_sound(res::valve::sound::plats::TTRAIN_BRAKE1);
        engine.precache_sound(res::valve::sound::plats::TTRAIN_START1);

        self.adjust_pitch = engine.precache_event(res::valve::events::TRAIN);
    }

    fn spawn(&mut self) {
        let sf = self.spawn_flags();
        let v = self.base.vars();

        if v.speed() == 0.0 {
            self.speed = 100.0;
        } else {
            self.speed = v.speed();
        }

        v.set_speed(0.0);
        v.set_velocity(vec3_t::ZERO);
        v.set_angular_velocity(vec3_t::ZERO);
        v.set_impulse(self.speed as u32);

        if v.target().is_none() {
            error!("{}: no target", self.pretty_name());
        }

        self.direction = 1.0;

        if sf.intersects(SpawnFlags::NOT_SOLID) {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }
        v.set_move_type(MoveType::Push);

        v.reload_model();
        v.set_size_and_link(v.min_size(), v.max_size());
        v.link();

        v.set_old_origin(v.origin());

        self.control_min_size = v.min_size();
        self.control_max_size = v.max_size();
        self.control_max_size.z += 72.0;

        self.set_next_think_time_from_last(Think::Find, 0.1, false);

        self.precache();
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let v = self.vars();
        if let UseType::Set(value) = use_type {
            let delta = ((v.speed() * 4.0) as i32 / self.speed as i32) as f32 * 0.25 + 0.25 * value;
            let delta = if self.spawn_flags().intersects(SpawnFlags::FORWARD_ONLY) {
                delta.clamp(0.0, 1.0)
            } else {
                delta.clamp(-1.0, 1.0)
            };
            v.set_speed(self.speed * delta);
            self.next();
        } else if use_type.should_toggle(v.speed() != 0.0) {
            if v.speed() == 0.0 {
                v.set_speed(self.speed * self.direction);
                self.next();
            } else {
                v.set_speed(0.0);
                v.set_velocity(vec3_t::ZERO);
                v.set_angular_velocity(vec3_t::ZERO);
                self.think.set(Think::None);
                self.stop_sound();
            }
        }
        trace!("{}: change speed to {:.2}", self.pretty_name(), v.speed());
    }

    fn think(&self) {
        match self.think.get() {
            Think::None => {}
            Think::Find => {
                self.think.set(Think::None);
                self.find();
            }
            Think::Next => {
                self.think.set(Think::None);
                self.next();
            }
            Think::DeadEnd(backward) => {
                self.think.set(Think::None);
                self.dead_end(backward);
            }
            Think::NearestPath => {
                self.think.set(Think::None);
                self.nearest_path();
            }
        }
    }

    fn override_reset(&self) {
        self.set_next_think_time_from_last(Think::NearestPath, 0.1, false);
    }
}

export_entity_default!("export-func_tracktrain", func_tracktrain, TrackTrain {});
