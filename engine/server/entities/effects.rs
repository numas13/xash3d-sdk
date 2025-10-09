use xash3d_shared::entity::MoveType;

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entities::subs::PointEntity,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, Solid},
    time::MapTime,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Glow {
    base: PointEntity,
    last_time: MapTime,
    max_frame: f32,
}

impl Glow {
    fn animate(&mut self, frames: f32) {
        if self.max_frame > 0.0 {
            let v = self.base.vars_mut();
            v.set_frame((v.frame() + frames) % self.max_frame);
        }
    }
}

impl_entity_cast!(Glow);

impl CreateEntity for Glow {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            last_time: MapTime::ZERO,
            max_frame: 0.0,
        }
    }
}

impl Entity for Glow {
    delegate_entity!(base not { spawn, think });

    fn spawn(&mut self) {
        let engine = self.engine();
        let v = self.base.vars_mut();

        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.remove_effects();
        v.set_frame(0.0);
        v.reload_model_with_precache();

        let v = self.base.vars_mut();
        self.max_frame = (engine.model_frames(v.model_index_raw()) - 1) as f32;
        if self.max_frame > 1.0 && v.framerate() != 0.0 {
            v.set_next_think_time(0.1);
        }
        self.last_time = engine.globals.map_time();
    }

    fn think(&mut self) {
        let engine = self.base.engine();
        let v = self.base.vars();
        let now = engine.globals.map_time();
        self.animate(v.framerate() * now.duration_since(self.last_time).as_secs_f32());

        self.vars_mut().set_next_think_time(0.1);
        self.last_time = engine.globals.map_time();
    }
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(env_glow, Private<super::Glow>);

    export_entity!(env_bubbles, Private<StubEntity>);
    export_entity!(beam, Private<StubEntity>);
    export_entity!(env_lightning, Private<StubEntity>);
    export_entity!(env_beam, Private<StubEntity>);
    export_entity!(env_laser, Private<StubEntity>);
    export_entity!(env_sprite, Private<StubEntity>);
    export_entity!(gibshooter, Private<StubEntity>);
    export_entity!(env_shooter, Private<StubEntity>);
    export_entity!(test_effect, Private<StubEntity>);
    export_entity!(env_blood, Private<StubEntity>);
    export_entity!(env_shake, Private<StubEntity>);
    export_entity!(env_fade, Private<StubEntity>);
}
