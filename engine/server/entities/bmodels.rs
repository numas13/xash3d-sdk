use xash3d_shared::{
    entity::{EdictFlags, MoveType},
    ffi::common::vec3_t,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, ObjectCaps, Solid,
        UseType,
    },
    export::{export_entity_default, export_entity_stub},
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct FuncWall {
    base: BaseEntity,
}

impl_entity_cast!(FuncWall);

impl CreateEntity for FuncWall {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for FuncWall {
    delegate_entity!(base not { object_caps, spawn, used });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let v = self.vars_mut();
        v.set_angles(vec3_t::ZERO);
        v.set_solid(Solid::Bsp);
        v.set_move_type(MoveType::Push);
        v.flags_mut().insert(EdictFlags::WORLDBRUSH);
        v.reload_model();
    }

    fn used(&mut self, _: Option<&mut dyn Entity>, _: &mut dyn Entity, use_type: UseType, _: f32) {
        let v = self.base.vars_mut();
        if use_type.should_toggle(v.frame() != 0.0) {
            v.set_frame(1.0 - v.frame());
        }
    }
}

export_entity_default!("export-func_wall", func_wall, FuncWall);

export_entity_stub!(func_rotating);
export_entity_stub!(func_illusionary);
