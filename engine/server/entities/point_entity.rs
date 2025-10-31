use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, ObjectCaps, Solid},
    prelude::*,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
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
        let v = self.vars();
        v.set_solid(Solid::Not);
    }
}
