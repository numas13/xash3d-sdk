use core::cell::Cell;

use xash3d_shared::entity::MoveType;

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entities::subs::PointEntity,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, Solid},
    export::{export_entity_default, export_entity_stub},
    time::MapTime,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Glow {
    base: PointEntity,
    last_time: Cell<MapTime>,
    max_frame: Cell<f32>,
}

impl Glow {
    fn animate(&self, frames: f32) {
        if self.max_frame.get() > 0.0 {
            let v = self.base.vars();
            v.set_frame((v.frame() + frames) % self.max_frame.get());
        }
    }
}

impl_entity_cast!(Glow);

impl CreateEntity for Glow {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            last_time: Cell::new(MapTime::ZERO),
            max_frame: Cell::new(0.0),
        }
    }
}

impl Entity for Glow {
    delegate_entity!(base not { spawn, think });

    fn spawn(&self) {
        let engine = self.engine();
        let v = self.base.vars();

        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.remove_effects();
        v.set_frame(0.0);
        v.reload_model_with_precache();

        let v = self.base.vars();
        self.max_frame
            .set((engine.model_frames(v.model_index_raw()) - 1) as f32);
        if self.max_frame.get() > 1.0 && v.framerate() != 0.0 {
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

export_entity_stub!(env_bubbles);
export_entity_stub!(beam);
export_entity_stub!(env_lightning);
export_entity_stub!(env_beam);
export_entity_stub!(env_laser);
export_entity_stub!(env_sprite);
export_entity_stub!(gibshooter);
export_entity_stub!(env_shooter);
export_entity_stub!(test_effect);
export_entity_stub!(env_blood);
export_entity_stub!(env_shake);
export_entity_stub!(env_fade);
