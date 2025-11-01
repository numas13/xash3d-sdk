use core::cell::Cell;

use bitflags::bitflags;

use crate::{
    entity::{delegate_entity, BaseEntity, KeyValue, UseType},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
    utils::Sparks,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum EnvSparkState {
    Off = 0,
    On,
    AlwaysOn,
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct EnvSparkSpawnFlags: u32 {
        const USE = 1 << 5;
        const USE_START_ON = 1 << 6;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct EnvSpark {
    base: BaseEntity,
    delay: f32,
    state: Cell<EnvSparkState>,
}

impl EnvSpark {
    fn spawn_flags(&self) -> EnvSparkSpawnFlags {
        EnvSparkSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_next_think_time(&self) {
        let engine = self.engine();
        let delay = engine.random_float(0.0, self.delay);
        self.vars().set_next_think_time_from_now(0.1 + delay);
    }
}

impl CreateEntity for EnvSpark {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            delay: 0.0,
            state: Cell::new(EnvSparkState::Off),
        }
    }
}

impl Entity for EnvSpark {
    delegate_entity!(base not { key_value, precache, spawn, think, used });

    fn key_value(&mut self, data: &mut KeyValue) {
        let name = data.key_name();
        if name == c"MaxDelay" {
            self.delay = data.parse_or_default();
            data.set_handled(true);
        } else if name == c"style"
            || name == c"height"
            || name == c"killtarget"
            || name == c"value1"
            || name == c"value2"
            || name == c"value3"
        {
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn precache(&mut self) {
        let engine = self.engine();
        Sparks::new(engine).precache();
    }

    fn spawn(&mut self) {
        if self.delay <= 0.0 {
            self.delay = 1.5;
        }

        let spawn_flags = self.spawn_flags();
        if spawn_flags.intersects(EnvSparkSpawnFlags::USE) {
            if spawn_flags.intersects(EnvSparkSpawnFlags::USE_START_ON) {
                self.set_next_think_time();
                self.state.set(EnvSparkState::On);
            } else {
                self.state.set(EnvSparkState::Off);
            }
        } else {
            self.set_next_think_time();
            self.state.set(EnvSparkState::AlwaysOn);
        }

        self.precache();
    }

    fn think(&self) {
        if matches!(
            self.state.get(),
            EnvSparkState::On | EnvSparkState::AlwaysOn
        ) {
            self.set_next_think_time();
            let engine = self.engine();
            let v = self.vars();
            Sparks::new(engine).emit(v.origin(), v);
        }
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        match self.state.get() {
            EnvSparkState::Off => {
                self.set_next_think_time();
                self.state.set(EnvSparkState::On);
            }
            EnvSparkState::On => {
                self.state.set(EnvSparkState::Off);
            }
            _ => {}
        }
    }
}

impl_private!(EnvSpark {});

export_entity_default!("export-env_spark", env_spark, EnvSpark);
