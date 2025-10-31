use xash3d_shared::{
    entity::{EdictFlags, Effects, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, ObjectCaps, Solid},
    export::export_entity_default,
    prelude::*,
    user_message,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct SparkShower {
    base: BaseEntity,
}

impl_entity_cast!(SparkShower);

impl CreateEntity for SparkShower {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for SparkShower {
    delegate_entity!(base not { object_caps, spawn, touched, think });

    fn object_caps(&self) -> ObjectCaps {
        ObjectCaps::DONT_SAVE
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();

        let mut velocity = v.angles() * engine.random_float(200.0, 300.0);
        velocity.x += engine.random_float(-100.0, 100.0);
        velocity.y += engine.random_float(-100.0, 100.0);
        if velocity.z >= 0.0 {
            velocity.z += 200.0;
        } else {
            velocity.z -= 200.0;
        }
        v.set_velocity(velocity);

        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.set_gravity(0.5);
        v.set_model(res::valve::models::GRENADE);
        v.set_size_and_link(vec3_t::ZERO, vec3_t::ZERO);
        v.with_effects(|f| f | Effects::NODRAW);
        v.set_speed(engine.random_float(0.5, 1.5));
        v.set_angles(vec3_t::ZERO);
        v.set_next_think_time_from_now(0.1);
    }

    fn touched(&self, _: &dyn Entity) {
        let v = self.vars();
        if v.flags().intersects(EdictFlags::ONGROUND) {
            v.with_velocity(|v| v * 0.1);
        } else {
            v.with_velocity(|v| v * 0.6);
        }

        let vel = v.velocity().truncate();
        if vel.dot(vel) < 10.0 {
            v.set_speed(0.0);
        }
    }

    fn think(&self) {
        let engine = self.engine();
        let v = self.vars();
        let msg = user_message::Sparks::new(v.origin());
        engine.msg_pvs(v.origin(), &msg);

        let speed = v.speed() - 0.1;
        v.set_speed(speed);
        if speed > 0.0 {
            v.set_next_think_time_from_now(0.1);
        } else {
            self.remove_from_world();
        }
        v.with_flags(|f| f.difference(EdictFlags::ONGROUND));
    }
}

export_entity_default!("export-spark_shower", spark_shower, SparkShower {});
