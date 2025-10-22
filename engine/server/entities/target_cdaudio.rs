use xash3d_shared::entity::{EntityIndex, MoveType};

use crate::{
    entities::point_entity::PointEntity,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, Solid,
        UseType,
    },
    export::export_entity_default,
    sound::play_cd_track,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TargetCdAutio {
    base: PointEntity,
}

impl_entity_cast!(TargetCdAutio);

impl CreateEntity for TargetCdAutio {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl TargetCdAutio {
    fn play_track(&self) {
        let v = self.vars();
        play_cd_track(&self.engine(), v.health() as i32);
        v.set_health(0.0);
        self.remove_from_world();
    }
}

impl Entity for TargetCdAutio {
    delegate_entity!(base not { key_value, spawn, used, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"radius" {
            self.vars().set_scale(data.parse_or_default());
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        if v.scale() > 0.0 {
            v.set_next_think_time_from_now(1.0);
        }
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        self.play_track();
    }

    fn think(&self) {
        let Some(player) = self
            .engine()
            .get_entity_by_index(EntityIndex::SINGLE_PLAYER)
        else {
            return;
        };
        let v = self.vars();
        if (player.vars().origin() - v.origin()).length() <= v.scale() {
            self.play_track();
        }
        v.set_next_think_time_from_now(0.5);
    }
}

export_entity_default!("export-target_cdaudio", target_cdaudio, TargetCdAutio);
