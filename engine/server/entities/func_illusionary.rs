use xash3d_shared::{entity::MoveType, ffi::common::vec3_t};

use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, ObjectCaps, Solid},
    export::export_entity_default,
    prelude::*,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Illusionary {
    base: BaseEntity,
}

impl_entity_cast!(Illusionary);

impl CreateEntity for Illusionary {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for Illusionary {
    delegate_entity!(base not { object_caps, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.set_angles(vec3_t::ZERO);
        v.reload_model();
    }
}

export_entity_default!("export-func_illusionary", func_illusionary, Illusionary);
