use crate::{
    entities::trigger::Trigger,
    entity::{delegate_entity, BaseEntity},
    export::export_entity_default,
    prelude::*,
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

export_entity_default!("export-trigger_gravity", trigger_gravity, TriggerGravity {});
