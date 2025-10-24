use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, ObjectCaps},
    export::export_entity_default,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

use super::triggers::init_trigger;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerGravity {
    base: BaseEntity,
}

impl_entity_cast!(TriggerGravity);

impl CreateEntity for TriggerGravity {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for TriggerGravity {
    delegate_entity!(base not { object_caps, spawn, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        init_trigger(&self.engine(), self.vars());
    }

    fn touched(&self, other: &dyn Entity) {
        if other.is_player() {
            other.vars().set_gravity(self.vars().gravity());
        }
    }
}

export_entity_default!("export-trigger_gravity", trigger_gravity, TriggerGravity);
