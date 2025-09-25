use xash3d_server::{
    consts::SOLID_BSP,
    entity::{
        delegate_entity, AsEdict, BaseEntity, CreateEntity, EdictFlags, Entity, MoveType,
        ObjectCaps, UseType,
    },
    export::export_entity,
    ffi::common::vec3_t,
    str::MapString,
};

use crate::{entity::Private, impl_cast};

pub struct FuncWall {
    base: BaseEntity,
}

impl_cast!(FuncWall);

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
            engine.set_model(self.as_edict_mut(), &model);
        }
    }

    fn used(&mut self, _: &mut dyn Entity, use_type: UseType, _: f32) {
        let ev = self.vars_mut().as_raw_mut();
        if use_type.should_toggle(ev.frame != 0.0) {
            ev.frame = 1.0 - ev.frame;
        }
    }
}

export_entity!(func_wall, Private<FuncWall>);
