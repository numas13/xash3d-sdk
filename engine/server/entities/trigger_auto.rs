use crate::{
    entities::{delayed_use::DelayedUse, trigger::Trigger},
    entity::{delegate_entity, BaseEntity, KeyValue, UseType},
    export::export_entity_default,
    global_state::EntityState,
    prelude::*,
    private::impl_private,
    str::MapString,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct AutoTrigger {
    base: Trigger,
    delayed: DelayedUse,
    global_state: Option<MapString>,
    trigger_type: UseType,
}

impl CreateEntity for AutoTrigger {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base: Trigger::create(base),
            delayed: DelayedUse::new(engine),
            global_state: None,
            trigger_type: UseType::Off,
        }
    }
}

impl AutoTrigger {
    /// Remove this trigger after firing.
    const SF_FIREONCE: u32 = 1 << 0;
}

impl Entity for AutoTrigger {
    delegate_entity!(base not { key_value, precache, spawn, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"globalstate" => {
                self.global_state = Some(self.engine().new_map_string(data.value()));
            }
            b"triggerstate" => match data.value().to_bytes() {
                b"0" => self.trigger_type = UseType::Off,
                b"2" => self.trigger_type = UseType::Toggle,
                _ => self.trigger_type = UseType::On,
            },
            _ => {
                if !self.delayed.key_value(data) {
                    self.base.key_value(data);
                }
                return;
            }
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        self.vars().set_next_think_time_from_now(0.1);
    }

    fn spawn(&mut self) {
        self.precache();
    }

    fn think(&self) {
        if !self.global_state.map_or(true, |name| {
            self.global_state().entity_state(name) == EntityState::On
        }) {
            return;
        }

        self.delayed
            .use_targets(self.trigger_type, Some(self), self);

        if self.vars().spawn_flags() & Self::SF_FIREONCE != 0 {
            self.remove_from_world();
        }
    }
}

impl_private!(AutoTrigger {});

export_entity_default!("export-trigger_auto", trigger_auto, AutoTrigger);
