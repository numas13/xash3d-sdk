use xash3d_server::{
    consts::SOLID_NOT,
    entity::{delegate_entity, BaseEntity, CreateEntity, Entity, ObjectCaps},
    export::export_entity,
};

use crate::{entity::Private, impl_cast};

pub struct PointEntity {
    base: BaseEntity,
}

impl_cast!(PointEntity);

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

export_entity!(info_player_start, Private<PointEntity>);
export_entity!(info_landmark, Private<PointEntity>);
