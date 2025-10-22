use core::cell::Cell;

use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, ObjectCaps,
        UseType,
    },
    export::export_entity_default,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

use super::triggers::init_trigger;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerEndSection {
    base: BaseEntity,
    enable_used: Cell<bool>,
    enable_touched: Cell<bool>,
}

impl TriggerEndSection {
    const SF_USEONLY: u32 = 1;

    fn end_section(&self, activator: &dyn Entity) {
        // TODO: add is_net_client method to Entity/EntityPlayer???
        if !activator.is_player() {
            return;
        }
        if let Some(message) = self.vars().message() {
            self.engine().end_section_by_name(message);
        }
        self.remove_from_world();
    }
}

impl_entity_cast!(TriggerEndSection);

impl CreateEntity for TriggerEndSection {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            enable_used: Cell::new(false),
            enable_touched: Cell::new(false),
        }
    }
}

impl Entity for TriggerEndSection {
    delegate_entity!(base not { object_caps, key_value, spawn, used, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"section" {
            let engine = self.engine();
            self.vars().set_message(engine.new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let global_state = self.global_state();
        let v = self.base.vars();

        if global_state.game_rules().is_deathmatch() {
            v.delayed_remove();
            return;
        }

        init_trigger(&engine, v);

        self.enable_used.set(true);
        self.enable_touched
            .set(v.spawn_flags() & Self::SF_USEONLY == 0);
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
        if self.enable_used.take() {
            self.end_section(activator.unwrap_or(caller));
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if self.enable_touched.take() {
            self.end_section(other);
        }
    }
}

export_entity_default!(
    "export-trigger_endsection",
    trigger_endsection,
    TriggerEndSection
);
