use xash3d_shared::{
    consts::SOLID_BSP,
    entity::{EdictFlags, MoveType},
    ffi::common::vec3_t,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, ObjectCaps, UseType,
    },
    str::MapString,
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
        let engine = self.engine();
        let ev = self.vars_mut().as_raw_mut();
        ev.angles = vec3_t::ZERO;
        ev.movetype = MoveType::Push.into();
        ev.solid = SOLID_BSP;
        ev.flags |= EdictFlags::WORLDBRUSH.bits();
        if let Some(model) = MapString::from_index(engine, ev.model) {
            engine.set_model(self, &model);
        }
    }

    fn used(&mut self, _: &mut dyn Entity, use_type: UseType, _: f32) {
        let ev = self.vars_mut().as_raw_mut();
        if use_type.should_toggle(ev.frame != 0.0) {
            ev.frame = 1.0 - ev.frame;
        }
    }
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(func_rotating, Private<StubEntity>);
    export_entity!(func_wall, Private<super::FuncWall>);
    export_entity!(func_illusionary, Private<StubEntity>);
}
