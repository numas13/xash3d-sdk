use xash3d_server::{
    entities::trigger::Trigger,
    entity::{delegate_entity, BaseEntity},
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerGravity {
    base: Trigger,
}

impl CreateEntity for TriggerGravity {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
        }
    }
}

impl Entity for TriggerGravity {
    delegate_entity!(base not { touched });

    fn touched(&self, other: &dyn Entity) {
        if other.is_player() {
            other.vars().set_gravity(self.vars().gravity());
        }
    }
}

impl_private!(TriggerGravity {});

define_export! {
    export_trigger_gravity as export if "trigger-gravity" {
        trigger_gravity = trigger_gravity::TriggerGravity,
    }
}
