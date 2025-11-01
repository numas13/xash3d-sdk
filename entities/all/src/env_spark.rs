use core::cell::Cell;

use bitflags::bitflags;
use xash3d_server::{
    entity::{delegate_entity, BaseEntity, KeyValue, UseType},
    prelude::*,
    private::impl_private,
    utils::Sparks,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum State {
    Off = 0,
    On,
    AlwaysOn,
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    struct SpawnFlags: u32 {
        const USE = 1 << 5;
        const USE_START_ON = 1 << 6;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Spark {
    base: BaseEntity,
    delay: f32,
    state: Cell<State>,
}

impl Spark {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_next_think_time(&self) {
        let engine = self.engine();
        let delay = engine.random_float(0.0, self.delay);
        self.vars().set_next_think_time_from_now(0.1 + delay);
    }
}

impl CreateEntity for Spark {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            delay: 0.0,
            state: Cell::new(State::Off),
        }
    }
}

impl Entity for Spark {
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
        if spawn_flags.intersects(SpawnFlags::USE) {
            if spawn_flags.intersects(SpawnFlags::USE_START_ON) {
                self.set_next_think_time();
                self.state.set(State::On);
            } else {
                self.state.set(State::Off);
            }
        } else {
            self.set_next_think_time();
            self.state.set(State::AlwaysOn);
        }

        self.precache();
    }

    fn think(&self) {
        if matches!(self.state.get(), State::On | State::AlwaysOn) {
            self.set_next_think_time();
            let engine = self.engine();
            let v = self.vars();
            Sparks::new(engine).emit(v.origin(), v);
        }
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        match self.state.get() {
            State::Off => {
                self.set_next_think_time();
                self.state.set(State::On);
            }
            State::On => {
                self.state.set(State::Off);
            }
            _ => {}
        }
    }
}

impl_private!(Spark {});

define_export! {
    export_env_spark as export if "env-spark" {
        env_spark = env_spark::Spark,
    }
}
