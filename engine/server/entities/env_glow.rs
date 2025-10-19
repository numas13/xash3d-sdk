use core::cell::Cell;

use xash3d_shared::entity::MoveType;

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, Solid},
    export::export_entity_default,
    time::MapTime,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Glow {
    base: PointEntity,
    max_frame: f32,

    last_time: Cell<MapTime>,
}

impl Glow {
    fn animate(&self, frames: f32) {
        if self.max_frame > 0.0 {
            let v = self.base.vars();
            v.set_frame((v.frame() + frames) % self.max_frame);
        }
    }
}

impl_entity_cast!(Glow);

impl CreateEntity for Glow {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            max_frame: 0.0,

            last_time: Cell::new(MapTime::ZERO),
        }
    }
}

impl Entity for Glow {
    delegate_entity!(base not { spawn, think });

    fn spawn(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();

        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.remove_effects();
        v.set_frame(0.0);
        v.reload_model_with_precache();

        let v = self.base.vars();
        self.max_frame = (engine.model_frames(v.model_index_raw()) - 1) as f32;
        if self.max_frame > 1.0 && v.framerate() != 0.0 {
            v.set_next_think_time_from_now(0.1);
        }
        self.last_time.set(engine.globals.map_time());
    }

    fn think(&self) {
        let engine = self.base.engine();
        let v = self.base.vars();
        let now = engine.globals.map_time();
        self.animate(v.framerate() * now.duration_since(self.last_time.get()).as_secs_f32());

        self.vars().set_next_think_time_from_now(0.1);
        self.last_time.set(engine.globals.map_time());
    }
}

export_entity_default!("export-env_glow", env_glow, Glow);
