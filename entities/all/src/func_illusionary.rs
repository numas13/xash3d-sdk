use xash3d_server::{
    entity::{delegate_entity, BaseEntity, MoveType, ObjectCaps, Solid},
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Illusionary {
    base: BaseEntity,
}

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

impl_private!(Illusionary {});

define_export! {
    export_func_illusionary as export if "func-illusionary" {
        func_illusionary = func_illusionary::Illusionary,
    }
}
