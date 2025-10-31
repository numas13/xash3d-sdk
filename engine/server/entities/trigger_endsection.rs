use core::cell::Cell;

use crate::{
    entities::trigger::Trigger,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, KeyValue, UseType},
    export::export_entity_default,
    prelude::*,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerEndSection {
    base: Trigger,
    enable_used: Cell<bool>,
    enable_touched: Cell<bool>,
}

impl_entity_cast!(TriggerEndSection);

impl CreateEntity for TriggerEndSection {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
            enable_used: Cell::new(false),
            enable_touched: Cell::new(false),
        }
    }
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

impl Entity for TriggerEndSection {
    delegate_entity!(base not { key_value, spawn, used, touched });

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
        let global_state = self.global_state();
        let v = self.base.vars();

        if global_state.game_rules().is_deathmatch() {
            v.delayed_remove();
            return;
        }

        self.enable_used.set(true);
        self.enable_touched
            .set(v.spawn_flags() & Self::SF_USEONLY == 0);

        self.base.spawn();
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
    TriggerEndSection {}
);
