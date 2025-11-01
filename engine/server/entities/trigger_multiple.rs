use core::cell::Cell;

use bitflags::bitflags;
use xash3d_shared::entity::EdictFlags;

use crate::{
    entities::{delayed_use::DelayedUse, trigger::Trigger},
    entity::{delegate_entity, BaseEntity, KeyValue, UseType},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
    str::MapString,
    time::MapTime,
    utils,
};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct TriggerSpawnFlags: u32 {
        /// Monsters allowed to fire this trigger.
        const ALLOW_MONSTERS = 1 << 0;
        /// Players not allowed to fire this trigger.
        const NO_CLIENTS = 1 << 1;
        /// Only pushables can fire this trigger.
        const PUSHABLES = 1 << 2;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerMultiple {
    base: Trigger,
    delayed: DelayedUse,

    master: Option<MapString>,
    /// Time in seconds before the trigger is ready to be re-triggered.
    pub(super) wait: f32,

    /// The time when this trigger can be re-triggered.
    reset_time: Cell<MapTime>,
}

impl CreateEntity for TriggerMultiple {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base: Trigger::create(base),
            delayed: DelayedUse::new(engine),

            master: None,
            wait: 0.2,

            reset_time: Cell::default(),
        }
    }
}

impl TriggerMultiple {
    fn activate_trigger(&self, other: &dyn Entity) {
        let engine = self.engine();
        let v = self.base.vars();

        if engine.globals.map_time() < self.reset_time.get() {
            // still waiting for reset time
            return;
        }

        if !utils::is_master_triggered(&engine, self.master, Some(other)) {
            return;
        }

        if let Some(noise) = v.noise() {
            engine.build_sound().channel_voice().emit(noise, self);
        }

        self.delayed.use_targets(UseType::Toggle, Some(other), self);

        if let Some(_message) = v.message() {
            // TODO: need HudText user message defined in xash3d-hl-shared =\
            warn!(
                "{}: show a hud message is not implemented",
                self.classname()
            );
        }

        if self.wait > 0.0 {
            self.reset_time.set(engine.globals.map_time() + self.wait);
        } else {
            self.remove_from_world();
        }
    }
}

impl Entity for TriggerMultiple {
    delegate_entity!(base not { key_value, touched, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"master" => self.master = Some(self.engine().new_map_string(data.value())),
            b"wait" => self.wait = data.parse_or_default(),
            _ => {
                if !self.delayed.key_value(data) {
                    self.base.key_value(data);
                }
                return;
            }
        }
        data.set_handled(true);
    }

    fn touched(&self, other: &dyn Entity) {
        let spawn_flags = TriggerSpawnFlags::from_bits_retain(self.vars().spawn_flags());
        let flags = other.vars().flags();
        if spawn_flags.intersects(TriggerSpawnFlags::NO_CLIENTS)
            && flags.intersects(EdictFlags::CLIENT)
        {
            return;
        }
        if !spawn_flags.intersects(TriggerSpawnFlags::ALLOW_MONSTERS)
            && flags.intersects(EdictFlags::MONSTER)
        {
            return;
        }
        if !spawn_flags.intersects(TriggerSpawnFlags::PUSHABLES)
            && other.is_classname(c"func_pushable".into())
        {
            return;
        }
        self.activate_trigger(other);
    }
}

impl_private!(TriggerMultiple {});

export_entity_default!("export-trigger_multiple", trigger_multiple, TriggerMultiple);
