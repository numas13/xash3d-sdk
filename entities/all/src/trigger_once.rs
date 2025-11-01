use xash3d_server::{
    entity::{delegate_entity, BaseEntity},
    prelude::*,
    private::impl_private,
};

use crate::trigger_multiple::TriggerMultiple;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerOnce {
    base: TriggerMultiple,
}

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

impl_private!(TriggerOnce {});

define_export! {
    export_trigger_once as export if "trigger-once" {
        trigger_once = trigger_once::TriggerOnce,
    }
}
