use xash3d_entities::stub::StubEntity;
use xash3d_server::{
    entity::{delegate_entity, BaseEntity, ObjectCaps},
    export::export_entity,
    prelude::*,
    save::{Restore, Save},
};

#[derive(Save, Restore)]
pub struct WallHealthCharger {
    base: StubEntity,
}

impl CreateEntity for WallHealthCharger {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: StubEntity::new(base, false),
        }
    }
}

impl Entity for WallHealthCharger {
    delegate_entity!(base not { object_caps });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
            .union(ObjectCaps::CONTINUOUS_USE)
    }
}

export_entity!(func_healthcharger, WallHealthCharger {});

export_entity!(item_healthkit, StubEntity);
