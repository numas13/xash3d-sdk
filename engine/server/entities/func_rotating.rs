use core::cell::Cell;

use bitflags::bitflags;
use xash3d_shared::{
    consts::CONTENTS_EMPTY,
    entity::{DamageFlags, MoveType},
    ffi::common::vec3_t,
    math::fabsf,
    sound::{Attenuation, Pitch, SoundFlags},
};

use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, EntityVars, KeyValue, ObjectCaps, Solid,
        TakeDamage, UseType,
    },
    export::export_entity_default,
    prelude::*,
    str::MapString,
};

trait EntityVarsExt {
    fn vars(&self) -> &EntityVars;

    fn noise_running(&self) -> Option<MapString> {
        self.vars().noise3()
    }

    fn set_noise_running(&self, sound: MapString) {
        self.vars().set_noise3(sound);
    }
}

impl EntityVarsExt for EntityVars {
    fn vars(&self) -> &Self {
        self
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const INSTANT       = 1 << 0;
        const BACKWARDS     = 1 << 1;
        const Z_AXIS        = 1 << 2;
        const X_AXIS        = 1 << 3;
        const ACC_DCC       = 1 << 4;
        const HURT          = 1 << 5;
        const NOT_SOLID     = 1 << 6;
        const SMALL_RADIUS  = 1 << 7;
        const MEDIUM_RADIUS = 1 << 8;
        const LARGE_RADIUS  = 1 << 9;
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    Instant,
    SpinUp,
    SpinDown,
    Rotate,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Rotating {
    base: BaseEntity,

    fan_friction: f32,
    attenuation: Attenuation,
    volume: f32,
    sounds: u8,

    think: Cell<Think>,
}

impl_entity_cast!(Rotating);

impl CreateEntity for Rotating {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            fan_friction: 0.0,
            attenuation: Attenuation::default(),
            volume: 0.0,
            sounds: 0,
            think: Cell::default(),
        }
    }
}

impl Rotating {
    const FAN_PITCH_MIN: i32 = 30;
    const FAN_PITCH_MAX: i32 = 100;

    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn start(&self) {
        let engine = self.engine();
        let v = self.vars();
        let sf = self.spawn_flags();
        if sf.intersects(SpawnFlags::ACC_DCC) {
            if v.angular_velocity() != vec3_t::ZERO {
                self.think.set(Think::SpinDown);
            } else {
                self.think.set(Think::SpinUp);
                if let Some(sound) = v.noise_running() {
                    engine
                        .build_sound()
                        .channel_static()
                        .volume(0.01)
                        .attenuation(self.attenuation)
                        .pitch(Self::FAN_PITCH_MIN)
                        .emit_dyn(sound, v);
                }
            }
            v.set_next_think_time_from_last(0.1);
        } else {
            //
            if v.angular_velocity() != vec3_t::ZERO {
                self.think.set(Think::SpinDown);
                v.set_next_think_time_from_last(0.1);
            } else {
                if let Some(sound) = v.noise_running() {
                    engine
                        .build_sound()
                        .channel_static()
                        .volume(self.volume)
                        .attenuation(self.attenuation)
                        .pitch(Self::FAN_PITCH_MAX)
                        .emit_dyn(sound, v);
                }
                v.set_angular_velocity(v.move_dir() * v.speed());
                self.think.set(Think::Rotate);
                self.rotate();
            }
        }
    }

    fn spin_up(&self) {
        let v = self.vars();
        v.with_angular_velocity(|x| x + v.move_dir() * (v.speed() * self.fan_friction));
        let cur = v.angular_velocity().abs();
        let req = (v.move_dir() * v.speed()).abs();
        if cur.x >= req.x && cur.y >= req.y && cur.z >= req.z {
            v.set_angular_velocity(req);
            if let Some(sound) = v.noise_running() {
                self.engine()
                    .build_sound()
                    .channel_static()
                    .volume(self.volume)
                    .attenuation(self.attenuation)
                    .pitch(Self::FAN_PITCH_MAX)
                    .flags(SoundFlags::CHANGE_VOL | SoundFlags::CHANGE_PITCH)
                    .emit_dyn(sound, v);
            }
            self.think.set(Think::Rotate);
            self.rotate();
        } else {
            v.set_next_think_time_from_last(0.1);
            self.ramp_pitch_vol();
        }
    }

    fn spin_down(&self) {
        let v = self.vars();
        v.with_angular_velocity(|x| x - v.move_dir() * (v.speed() * self.fan_friction));
        let cur = v.angular_velocity();
        let dir = first_non_zero_or_z(v.move_dir());
        if (dir > 0.0 && cur.x <= 0.0 && cur.y <= 0.0 && cur.z <= 0.0)
            || (dir < 0.0 && cur.x >= 0.0 && cur.y >= 0.0 && cur.z >= 0.0)
        {
            v.set_angular_velocity(vec3_t::ZERO);
            if let Some(sound) = v.noise_running() {
                self.engine()
                    .build_sound()
                    .channel_static()
                    .pitch(Pitch::NORM.to_i32() - 1)
                    .stop(sound, v);
            }
            self.think.set(Think::Rotate);
            self.rotate();
        } else {
            v.set_next_think_time_from_last(0.1);
            self.ramp_pitch_vol();
        }
    }

    fn rotate(&self) {
        self.vars().set_next_think_time_from_last(10.0);
    }

    fn ramp_pitch_vol(&self) {
        let v = self.vars();
        let Some(sound) = v.noise_running() else {
            return;
        };
        let avel = v.angular_velocity();
        let v_cur = fabsf(first_non_zero_or_z(avel));
        let v_final = fabsf(first_non_zero_or_z(v.move_dir()) * v.speed());
        let p = v_cur / v_final;
        let vol = self.volume * p;
        let mut pitch = (Self::FAN_PITCH_MIN as f32
            + (Self::FAN_PITCH_MAX - Self::FAN_PITCH_MIN) as f32 * p)
            as i32;
        if pitch == Pitch::NORM.to_i32() {
            pitch = Pitch::NORM.to_i32() - 1;
        };

        self.engine()
            .build_sound()
            .channel_static()
            .volume(vol)
            .attenuation(self.attenuation)
            .pitch(pitch)
            .flags(SoundFlags::CHANGE_VOL | SoundFlags::CHANGE_PITCH)
            .emit_dyn(sound, v);
    }
}

fn first_non_zero_or_z(v: vec3_t) -> f32 {
    if v.x != 0.0 {
        v.x
    } else if v.y != 0.0 {
        v.y
    } else {
        v.z
    }
}

impl Entity for Rotating {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, used, touched, blocked, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"fanfriction" => self.fan_friction = data.parse_or_default(),
            b"Volume" => self.volume = data.parse_or_default::<f32>().clamp(0.0, 1.0),
            b"spawnorigin" => {
                let origin = data.parse_vec3_or_default();
                if origin != vec3_t::ZERO {
                    self.vars().set_origin(origin);
                }
            }
            b"sounds" => self.sounds = data.parse_or_default(),
            _ => {
                self.base.key_value(data);
                return;
            }
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();
        let message = v.message();
        if let Some((false, sound)) = message.as_deref().map(|s| (s.is_empty(), s)) {
            engine.precache_sound(sound);
            v.set_noise_running(engine.new_map_string(sound));
        } else {
            let mut is_null = false;
            let sound = match self.sounds {
                1 => res::valve::sound::fans::FAN1,
                2 => res::valve::sound::fans::FAN2,
                3 => res::valve::sound::fans::FAN3,
                4 => res::valve::sound::fans::FAN4,
                5 => res::valve::sound::fans::FAN5,
                _ => {
                    is_null = true;
                    res::valve::sound::common::NULL
                }
            };
            if !is_null {
                engine.precache_sound(sound);
            }
            v.set_noise_running(engine.new_map_string(sound));
        }

        if v.angular_velocity() != vec3_t::ZERO {
            self.think.set(Think::SpinUp);
            v.set_next_think_time_from_last(1.5);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        let sf = self.spawn_flags();

        if self.volume == 0.0 {
            self.volume = 1.0;
        }

        self.attenuation = Attenuation::NORM;
        if sf.intersects(SpawnFlags::SMALL_RADIUS) {
            self.attenuation = Attenuation::IDLE;
        } else if sf.intersects(SpawnFlags::MEDIUM_RADIUS) {
            self.attenuation = Attenuation::STATIC;
        } else if sf.intersects(SpawnFlags::LARGE_RADIUS) {
            self.attenuation = Attenuation::NORM;
        }

        if self.fan_friction == 0.0 {
            self.fan_friction = 1.0;
        }

        if sf.intersects(SpawnFlags::Z_AXIS) {
            v.set_move_dir(vec3_t::Z);
        } else if sf.intersects(SpawnFlags::X_AXIS) {
            v.set_move_dir(vec3_t::X);
        } else {
            v.set_move_dir(vec3_t::Y);
        }

        if sf.intersects(SpawnFlags::BACKWARDS) {
            v.with_move_dir(|x| -x);
        }

        if sf.intersects(SpawnFlags::NOT_SOLID) {
            v.set_solid(Solid::Not);
            v.set_skin(CONTENTS_EMPTY);
        } else {
            v.set_solid(Solid::Bsp);
        }
        v.set_move_type(MoveType::Push);

        v.link();
        v.reload_model();

        if v.speed() <= 0.0 {
            v.set_speed(0.0);
        }

        if sf.intersects(SpawnFlags::INSTANT) {
            self.think.set(Think::Instant);
            v.set_next_think_time_from_last(1.5);
        }

        self.precache();
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        self.start();
    }

    fn touched(&self, other: &dyn Entity) {
        if !self.spawn_flags().intersects(SpawnFlags::HURT) {
            return;
        }

        let other_v = other.vars();
        if other_v.take_damage() == TakeDamage::No {
            return;
        }

        let v = self.vars();
        v.set_damage(v.angular_velocity().length() / 10.0);
        other.take_damage(v.damage(), DamageFlags::CRUSH, v, Some(v));
        other_v.set_velocity(other_v.origin() - v.bmodel_origin().normalize() * v.damage());
    }

    fn blocked(&self, other: &dyn Entity) {
        let v = self.vars();
        other.take_damage(v.damage(), DamageFlags::CRUSH, v, Some(v));
    }

    fn think(&self) {
        match self.think.get() {
            Think::None => {}
            Think::Instant => self.start(),
            Think::SpinUp => self.spin_up(),
            Think::SpinDown => self.spin_down(),
            Think::Rotate => self.rotate(),
        }
    }
}

export_entity_default!("export-func_rotating", func_rotating, Rotating {});
