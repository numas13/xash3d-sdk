use bitflags::bitflags;

use crate::{
    entities::delayed_use::DelayedUse,
    entity::{delegate_entity, BaseEntity, KeyValue, ObjectCaps, UseType},
    export::export_entity_default,
    prelude::*,
};

bitflags! {
    #[derive(Copy, Clone)]
    pub struct SpawnFlags: u32 {
        const FIRE_ONCE = 1 << 0;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerRelay {
    base: BaseEntity,
    delayed: DelayedUse,
    use_type: UseType,
}

impl CreateEntity for TriggerRelay {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base,
            delayed: DelayedUse::new(engine),
            use_type: UseType::Off,
        }
    }
}

impl TriggerRelay {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }
}

impl Entity for TriggerRelay {
    delegate_entity!(base not { object_caps, key_value, spawn, used });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"triggerstate" {
            match data.parse_or_default() {
                0 => self.use_type = UseType::Off,
                2 => self.use_type = UseType::Toggle,
                _ => self.use_type = UseType::On,
            }
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {}

    fn used(&self, _: UseType, _: Option<&dyn Entity>, caller: &dyn Entity) {
        trace!("{}: used by {}", self.pretty_name(), caller.pretty_name());
        self.delayed.use_targets(self.use_type, Some(self), self);
        if self.spawn_flags().intersects(SpawnFlags::FIRE_ONCE) {
            self.remove_from_world();
        }
    }
}

export_entity_default!("export-trigger_relay", trigger_relay, TriggerRelay {});
