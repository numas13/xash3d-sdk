use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue},
    export::export_entity_default,
    utils,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

use super::point_entity::PointEntity;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct DeathMatchStart {
    base: PointEntity,
}

impl_entity_cast!(DeathMatchStart);

impl CreateEntity for DeathMatchStart {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl Entity for DeathMatchStart {
    delegate_entity!(base not { key_value, is_triggered });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"master" {
            let engine = self.engine();
            self.vars()
                .set_net_name(engine.new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn is_triggered(&self, activator: Option<&dyn Entity>) -> bool {
        let engine = self.engine();
        utils::is_master_triggered(&engine, self.vars().net_name(), activator)
    }
}

export_entity_default!(
    "export-info_player_deathmatch",
    info_player_deathmatch,
    DeathMatchStart
);
