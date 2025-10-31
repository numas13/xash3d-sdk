use bitflags::bitflags;
use xash3d_shared::{
    entity::{Effects, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    engine::TraceIgnore,
    entity::{
        create_entity, delegate_entity, impl_entity_cast, BaseEntity, KeyValue, Solid, UseType,
    },
    export::export_entity_default,
    prelude::*,
    user_message, utils,
};

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const NO_DAMAGE     = 1 << 0;
        const REPEATABLE    = 1 << 1;
        const NO_FIREBALL   = 1 << 2;
        const NO_SMOKE      = 1 << 3;
        const NO_DECAL      = 1 << 4;
        const NO_SPARKS     = 1 << 5;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Explosion {
    base: BaseEntity,
    magnitude: i16,
    sprite_scale: u8,
}

impl_entity_cast!(Explosion);

impl CreateEntity for Explosion {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            magnitude: 0,
            sprite_scale: 0,
        }
    }
}

impl Explosion {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }
}

impl Entity for Explosion {
    delegate_entity!(base not { key_value, spawn, used, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"iMagnitude" {
            self.magnitude = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.set_effects(Effects::NODRAW);
        let scale = (self.magnitude / 10 - 5) as f32 * 0.6;
        self.sprite_scale = (scale as u8).max(1);
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let engine = self.engine();
        let global_state = self.global_state();
        let sf = self.spawn_flags();
        let v = self.vars();

        let start = v.origin() + vec3_t::new(0.0, 0.0, 8.0);
        let end = start + vec3_t::new(0.0, 0.0, -40.0);
        let trace = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(v));
        if trace.fraction() != 1.0 {
            v.set_origin(
                trace.end_position() + trace.plane_normal() * (self.magnitude - 24) as f32 * 0.6,
            );
        }

        if !sf.intersects(SpawnFlags::NO_DECAL) {
            let sprite_index = global_state.decals().get_random_scorch();
            utils::decal_trace(&engine, &trace, sprite_index);
        }

        let msg = user_message::Explosion {
            position: v.origin().into(),
            sprite_index: global_state.sprites().fireball(),
            scale: if !sf.intersects(SpawnFlags::NO_FIREBALL) {
                self.sprite_scale.into()
            } else {
                0_u8.into()
            },
            frame_rate: 15,
            flags: user_message::ExplosionFlags::NONE,
        };
        engine.msg_pas(v.origin(), &msg);

        if !sf.intersects(SpawnFlags::NO_DAMAGE) {
            warn!("{}: damage is not implemented yet", self.pretty_name());
        }

        if !sf.intersects(SpawnFlags::NO_SPARKS) {
            for _ in 0..engine.random_int(0, 3) {
                let angles = trace.plane_normal();
                create_entity(&engine, c"spark_shower", v.origin(), angles, None).ok();
            }
        }

        v.set_next_think_time_from_now(0.3);
    }

    fn think(&self) {
        let sf = self.spawn_flags();

        if !sf.intersects(SpawnFlags::NO_SMOKE) {
            let engine = self.engine();
            let global_state = self.global_state();
            let v = self.vars();
            let msg = user_message::Smoke {
                position: v.origin().into(),
                sprite_index: global_state.sprites().smoke(),
                scale: self.sprite_scale.into(),
                frame_rate: 12,
            };
            engine.msg_pas(v.origin(), &msg);
        }

        if !sf.intersects(SpawnFlags::REPEATABLE) {
            self.remove_from_world();
        }
    }
}

export_entity_default!("export-env_explosion", env_explosion, Explosion {});
