use xash3d_shared::consts::SOLID_NOT;

use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, ObjectCaps},
    save::{Restore, Save},
};

#[derive(Save, Restore)]
pub struct PointEntity {
    base: BaseEntity,
}

impl_entity_cast!(PointEntity);

impl CreateEntity for PointEntity {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for PointEntity {
    delegate_entity!(base not { object_caps, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let ev = self.vars_mut().as_raw_mut();
        ev.solid = SOLID_NOT;
    }
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use super::PointEntity;
    use crate::{entity::Private, export::export_entity};

    export_entity!(info_player_start, Private<PointEntity>);
    export_entity!(info_landmark, Private<PointEntity>);
    // Lightning target, just alias landmark.
    export_entity!(info_target, Private<PointEntity>);
}
