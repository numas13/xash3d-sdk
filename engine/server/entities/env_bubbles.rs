use core::cell::Cell;

use xash3d_shared::{ffi::common::vec3_t, math::fabsf, render::RenderMode};

use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, ObjectCaps,
        Solid, UseType,
    },
    export::export_entity_default,
    user_message,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Bubbles {
    base: BaseEntity,
    density: u8,
    frequency: u16,
    #[cfg_attr(feature = "save", save(skip))]
    bubble_model: u16,
    state: Cell<bool>,
}

impl_entity_cast!(Bubbles);

impl CreateEntity for Bubbles {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            density: 0,
            frequency: 0,
            bubble_model: 0,
            state: Cell::default(),
        }
    }
}

impl Bubbles {
    const SF_STARTOFF: u32 = 1 << 0;

    fn fizz(&self) {
        let v = self.vars();
        let msg = user_message::Fizz {
            entity: self.entity_index(),
            sprite_index: self.bubble_model,
            density: self.density,
        };
        self.engine().msg_pas(v.bmodel_origin(), &msg);

        if self.frequency > 19 {
            v.set_next_think_time_from_now(0.5);
        } else {
            v.set_next_think_time_from_now(2.5 - (0.1 * self.frequency as f32));
        }
    }
}

impl Entity for Bubbles {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, used, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"density" => self.density = data.parse_or_default(),
            b"frequency" => self.frequency = data.parse_or_default(),
            b"current" => self.vars().set_speed(data.parse_or_default()),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        self.bubble_model = self.engine().precache_model(res::valve::sprites::BUBBLE) as u16;
    }

    fn spawn(&mut self) {
        self.precache();

        let v = self.base.vars();
        v.reload_model();
        v.set_solid(Solid::Not);
        v.set_render_amount(0.0);
        v.set_render_mode(RenderMode::TransTexture);

        let speed = fabsf(v.speed()) as u32;
        v.set_render_color(vec3_t::new(
            (speed >> 8) as f32,
            (speed & 0xff) as f32,
            if v.speed() < 0.0 { 1.0 } else { 0.0 },
        ));

        if v.spawn_flags() & Self::SF_STARTOFF == 0 {
            v.set_next_think_time_from_now(2.0);
            self.state.set(true);
        } else {
            self.state.set(false);
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        if use_type.should_toggle(self.state.get()) {
            self.state.set(!self.state.get());
        }

        let v = self.vars();
        if self.state.get() {
            v.set_next_think_time_from_now(0.1);
        } else {
            v.stop_thinking();
        }
    }

    fn think(&self) {
        if self.state.get() {
            self.fizz();
        }
    }
}

export_entity_default!("export-env_bubbles", env_bubbles, Bubbles);
