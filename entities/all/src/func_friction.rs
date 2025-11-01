use xash3d_server::{
    entity::{delegate_entity, BaseEntity, KeyValue, MoveType, ObjectCaps, Solid},
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct FrictionModifier {
    base: BaseEntity,
    friction: f32,
}

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
        let v = self.vars();
        v.set_solid(Solid::Trigger);
        v.set_move_type(MoveType::None);
        v.reload_model();
    }

    fn touched(&self, other: &dyn Entity) {
        match other.vars().move_type() {
            MoveType::Bounce | MoveType::BounceMissile => {}
            _ => other.vars().set_friction(self.friction),
        }
    }
}

impl_private!(FrictionModifier {});

define_export! {
    export_func_friction as export if "func-friction" {
        func_friction = func_friction::FrictionModifier,
    }
}
