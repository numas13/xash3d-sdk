use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue},
    export::export_entity_default,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct PathCorner {
    base: PointEntity,
    wait: f32,
}

impl CreateEntity for PathCorner {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            wait: 0.0,
        }
    }
}

impl_entity_cast!(PathCorner);

impl Entity for PathCorner {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"wait" {
            self.wait = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        if self.vars().target_name().is_none() {
            error!("{}: without a target name", self.pretty_name());
        }
    }
}

export_entity_default!("export-path_corner", path_corner, PathCorner);
