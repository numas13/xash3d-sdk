use core::cell::Cell;

use bitflags::bitflags;
use xash3d_server::{
    entity::{
        delegate_entity, BaseEntity, DamageFlags, KeyValue, MoveType, ObjectCaps, Solid,
        TakeDamage, UseType,
    },
    ffi::common::vec3_t,
    math::fabsf,
    prelude::*,
    private::impl_private,
    time::MapTime,
};

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const INSTANT       = 1 << 0;
        const SWING         = 1 << 1;
        const PASSABLE      = 1 << 3;
        const AUTO_RETURN   = 1 << 4;
        const ROTATE_Z      = 1 << 6;
        const ROTATE_X      = 1 << 7;
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    Start,
    Swing,
    Stop,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Pendulum {
    base: BaseEntity,

    accel: f32,
    distance: f32,
    time: Cell<MapTime>,
    damp: f32,
    max_speed: f32,
    damp_speed: Cell<f32>,
    center: vec3_t,
    start: vec3_t,

    think: Cell<Think>,
}

impl CreateEntity for Pendulum {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            accel: 0.0,
            distance: 0.0,
            time: Cell::default(),
            damp: 0.0,
            max_speed: 0.0,
            damp_speed: Cell::default(),
            center: vec3_t::ZERO,
            start: vec3_t::ZERO,

            think: Cell::default(),
        }
    }
}

impl Pendulum {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn angles_delta(&self, target: vec3_t) -> f32 {
        let sf = self.spawn_flags();
        let angles = self.vars().angles();
        if sf.intersects(SpawnFlags::ROTATE_Z) {
            angles.z - target.z
        } else if sf.intersects(SpawnFlags::ROTATE_X) {
            angles.x - target.x
        } else {
            angles.y - target.y
        }
    }

    fn start(&self) {
        let v = self.vars();
        let sf = self.spawn_flags();
        if v.speed() != 0.0 {
            if sf.intersects(SpawnFlags::AUTO_RETURN) {
                let delta = self.angles_delta(self.start);
                v.set_angular_velocity(v.move_dir() * self.max_speed);
                v.set_next_think_time_from_last(delta / self.max_speed);
                self.think.set(Think::Stop);
            } else {
                v.set_speed(0.0);
                v.set_angular_velocity(vec3_t::ZERO);
                v.stop_thinking();
                self.think.set(Think::None);
            }
        } else {
            self.time.set(self.engine().globals.map_time());
            self.damp_speed.set(self.max_speed);
            v.set_next_think_time_from_last(0.1);
            self.think.set(Think::Swing);
        }
    }

    fn swing(&self) {
        let v = self.vars();
        let delta = self.angles_delta(self.center);
        let now = self.engine().globals.map_time();
        let dt = now - self.time.replace(now);

        if delta > 0.0 && self.accel > 0.0 {
            v.set_speed(v.speed() - self.accel * dt);
        } else {
            v.set_speed(v.speed() + self.accel * dt);
        }

        if v.speed() > self.max_speed {
            v.set_speed(self.max_speed);
        } else if v.speed() < -self.max_speed {
            v.set_speed(-self.max_speed);
        }
        v.set_angular_velocity(v.move_dir() * v.speed());
        v.set_next_think_time_from_last(0.1);

        if self.damp != 0.0 {
            let damp_speed = self.damp_speed.get();
            let damp_speed = damp_speed - self.damp * damp_speed * dt;
            self.damp_speed.set(damp_speed);

            if damp_speed < 30.0 {
                v.set_angles(self.center);
                v.set_speed(0.0);
                v.set_angular_velocity(vec3_t::ZERO);
                v.stop_thinking();
                self.think.set(Think::None);
            } else if v.speed() > damp_speed {
                v.set_speed(damp_speed);
            } else if v.speed() < -damp_speed {
                v.set_speed(-damp_speed);
            }
        }
    }

    fn stop(&self) {
        let v = self.vars();
        v.set_angles(self.start);
        v.set_speed(0.0);
        v.set_angular_velocity(vec3_t::ZERO);
        self.think.set(Think::None);
    }
}

impl Entity for Pendulum {
    delegate_entity!(base not { object_caps, key_value, spawn, used, touched, blocked, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"distance" => self.distance = data.parse_or_default(),
            b"damp" => self.damp = data.parse_or_default::<f32>() * 0.001,
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let v = self.base.vars();

        v.set_move_dir_from_spawn_flags(SpawnFlags::ROTATE_X.bits(), SpawnFlags::ROTATE_Z.bits());

        let sf = self.spawn_flags();
        if sf.intersects(SpawnFlags::PASSABLE) {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }
        v.set_move_type(MoveType::Push);
        v.link();
        v.reload_model();

        if self.distance == 0.0 {
            return;
        }

        if v.speed() == 0.0 {
            v.set_speed(100.0);
        }

        self.accel = (v.speed() * v.speed()) / (2.0 * fabsf(self.distance));
        self.max_speed = v.speed();
        self.start = v.angles();
        self.center = v.angles() + v.move_dir() * (self.distance * 0.5);

        if sf.intersects(SpawnFlags::INSTANT) {
            self.think.set(Think::Start);
            v.set_next_think_time_from_now(0.1);
        }

        v.set_speed(0.0);

        if sf.intersects(SpawnFlags::SWING) {
            let name = self.pretty_name();
            warn!("{name}: swing spawn flag is not implemented yet");
        }
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        self.start();
    }

    fn touched(&self, other: &dyn Entity) {
        let v = self.vars();
        let other_v = other.vars();
        if v.damage() <= 0.0 || other_v.take_damage() == TakeDamage::No {
            return;
        }
        let damage = fabsf(v.damage() * v.speed() * 0.01);
        other.take_damage(damage, DamageFlags::CRUSH, v, Some(v));
        other_v.set_velocity((other_v.origin() - v.bmodel_origin()).normalize() * damage);
    }

    fn blocked(&self, _: &dyn Entity) {
        self.time.set(self.engine().globals.map_time());
    }

    fn think(&self) {
        match self.think.get() {
            Think::None => {}
            Think::Start => self.start(),
            Think::Swing => self.swing(),
            Think::Stop => self.stop(),
        }
    }
}

impl_private!(Pendulum {});

define_export! {
    export_func_pendulum as export if "func-pendulum" {
        func_pendulum = func_pendulum::Pendulum,
    }
}
