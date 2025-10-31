use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity},
    export::export_entity_default,
    prelude::*,
};

use super::trigger_multiple::TriggerMultiple;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerOnce {
    base: TriggerMultiple,
}

impl_entity_cast!(TriggerOnce);

impl CreateEntity for TriggerOnce {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: TriggerMultiple::create(base),
        }
    }
}

impl Entity for TriggerOnce {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.base.wait = -1.0;
        self.base.spawn();
    }
}

export_entity_default!("export-trigger_once", trigger_once, TriggerOnce {});
