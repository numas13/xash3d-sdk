use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, ObjectCaps,
        StubEntity, TakeDamage,
    },
    export::export_entity_stub,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Button {
    base: StubEntity,
}

impl_entity_cast!(Button);

impl CreateEntity for Button {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: StubEntity::new(base, false),
        }
    }
}

impl Entity for Button {
    delegate_entity!(base not { object_caps });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
            .union(if self.vars().take_damage() == TakeDamage::No {
                ObjectCaps::IMPULSE_USE
            } else {
                ObjectCaps::NONE
            })
    }
}

export_entity_stub!(func_button, Button);
