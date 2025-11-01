use xash3d_server::{
    entity::{EdictFlags, MoveType, delegate_entity, BaseEntity, ObjectCaps, Solid, UseType},
    prelude::*,
    private::impl_private,
    ffi::common::vec3_t,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct FuncWall {
    base: BaseEntity,
}

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
        let v = self.vars();
        v.set_angles(vec3_t::ZERO);
        v.set_solid(Solid::Bsp);
        v.set_move_type(MoveType::Push);
        v.with_flags(|f| f | EdictFlags::WORLDBRUSH);
        v.reload_model();
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let v = self.base.vars();
        if use_type.should_toggle(v.frame() != 0.0) {
            v.set_frame(1.0 - v.frame());
        }
    }
}

impl_private!(FuncWall {});

define_export! {
    export_func_wall as export if "func-wall" {
        func_wall = func_wall::FuncWall,
    }
}
