use core::cell::Cell;

use xash3d_shared::ffi::common::vec3_t;

use crate::{
    engine::TraceIgnore,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, UseType,
    },
    export::export_entity_default,
    user_message,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Decal {
    base: BaseEntity,
    state: Cell<u8>,
}

impl Decal {
    const SF_NOTINDEATHMATCH: u32 = 1 << 11;

    const STATE_STATIC: u8 = 1;
    const STATE_TRIGGER: u8 = 2;
    const STATE_REMOVE: u8 = 3;

    fn static_decal(&self) {
        let engine = self.engine();
        let v = self.base.vars();
        let trace = engine.trace_line(
            v.origin() - vec3_t::splat(5.0),
            v.origin() + vec3_t::splat(5.0),
            TraceIgnore::MONSTERS,
            Some(self),
        );
        let entity = trace.hit_entity().entity_index();
        let model_index = if !entity.is_world_spawn() {
            trace.hit_entity().vars().model_index_raw()
        } else {
            0
        };
        engine.static_decal(v.origin(), v.skin() as u16, entity, model_index as u16);
    }
}

impl_entity_cast!(Decal);

impl CreateEntity for Decal {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            state: Cell::new(0),
        }
    }
}

impl Entity for Decal {
    delegate_entity!(base not { key_value, spawn, think, used });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"texture" {
            let engine = self.engine();
            if let Some(skin) = engine.decal_index(data.value()) {
                self.vars().set_skin(skin.into());
                data.set_handled(true);
            } else {
                warn!("failed to find decal {}", data.value());
            }
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();
        if v.skin() < 0
            || (engine.globals.is_deathmatch() && v.spawn_flags() & Self::SF_NOTINDEATHMATCH != 0)
        {
            v.delayed_remove();
            return;
        }

        if v.target_name().map_or(true, |s| s.is_empty()) {
            self.state.set(Self::STATE_STATIC);
            // spawn the decal as soon as the world is done spawning
            v.set_next_think_time_from_now(0.0);
        } else {
            self.state.set(Self::STATE_TRIGGER);
        }
    }

    fn think(&self) {
        match self.state.get() {
            Self::STATE_STATIC => {
                self.state.set(0);
                self.static_decal();
                self.remove_from_world();
            }
            Self::STATE_REMOVE => {
                self.state.set(0);
                self.remove_from_world();
            }
            _ => {}
        }
    }

    #[allow(unused_variables)]
    fn used(&self, use_type: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
        if self.state.get() != Self::STATE_TRIGGER {
            return;
        }

        let engine = self.engine();

        let origin = self.vars().origin();
        let start = origin - vec3_t::splat(5.0);
        let end = origin + vec3_t::splat(5.0);
        let trace = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(self));

        let msg = user_message::BspDecal {
            position: origin.into(),
            texture_index: self.vars().skin() as u16,
            entity: engine.get_entity_index(&trace.hit_entity()),
            model_index: trace.hit_entity().vars().model_index_raw() as u16,
        };
        engine.msg_broadcast(&msg);

        // if log::log_enabled!(log::Level::Trace) {
        //     let msg = user_message::Line {
        //         start: start.into(),
        //         end: end.into(),
        //         duration: f32::MAX.into(),
        //         color: crate::color::RGB::RED,
        //     };
        //     engine.msg_broadcast(&msg);
        // }

        self.state.set(Self::STATE_REMOVE);
        self.vars().set_next_think_time_from_now(0.1);
    }
}

export_entity_default!("export-infodecal", infodecal, Decal);
