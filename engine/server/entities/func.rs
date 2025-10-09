use xash3d_shared::{
    entity::{Effects, MoveType},
    ffi,
    render::RenderMode,
};

use crate::entity::{
    delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, ObjectCaps,
    Solid,
};
#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct FrictionModifier {
    base: BaseEntity,
    friction: f32,
}

impl_entity_cast!(FrictionModifier);

impl CreateEntity for FrictionModifier {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            friction: 1.0,
        }
    }
}

impl Entity for FrictionModifier {
    delegate_entity!(base not { object_caps, key_value, spawn, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"modifier" {
            self.friction = data.value_str().parse().unwrap_or(0.0) / 100.0;
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars_mut();
        v.set_solid(Solid::Trigger);
        v.set_move_type(MoveType::None);
        v.reload_model();
    }

    fn touched(&mut self, other: &mut dyn Entity) {
        match other.vars().move_type() {
            MoveType::Bounce | MoveType::BounceMissile => {}
            _ => other.vars_mut().set_friction(self.friction),
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Ladder {
    base: BaseEntity,
}

impl_entity_cast!(Ladder);

impl CreateEntity for Ladder {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for Ladder {
    delegate_entity!(base not { object_caps, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let v = self.base.vars_mut();
        v.set_skin(ffi::common::CONTENTS_LADDER);
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::Push);
        v.set_render_mode(RenderMode::TransTexture);
        v.set_render_amount(0.0);
        v.effects_mut().remove(Effects::NODRAW);
        v.reload_model();
    }
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{entity::Private, export::export_entity};

    export_entity!(func_friction, Private<super::FrictionModifier>);
    export_entity!(func_ladder, Private<super::Ladder>);
}
