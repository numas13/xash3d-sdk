use xash3d_server::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, ObjectCaps, Private,
        StubEntity,
    },
    export::export_entity,
    save::{Restore, Save},
};

#[derive(Save, Restore)]
pub struct WallHealthCharger {
    base: StubEntity,
}

impl_entity_cast!(WallHealthCharger);

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

export_entity!(item_healthkit, Private<StubEntity>);
export_entity!(func_healthcharger, Private<WallHealthCharger>);
