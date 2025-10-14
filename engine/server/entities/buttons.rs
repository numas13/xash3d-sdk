use core::ffi::CStr;

use bitflags::bitflags;
use xash3d_shared::ffi::common::vec3_t;

#[cfg(feature = "save")]
use crate::save::{self, Restore, Save};
use crate::{
    engine::ServerEngineRef,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityVars, KeyValue,
        ObjectCaps, StubEntity, TakeDamage, UseType,
    },
    export::{export_entity_default, export_entity_stub},
    prelude::*,
    user_message,
};

// TODO: derive Save and Restore
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
enum EnvSparkState {
    Off = 0,
    On,
    AlwaysOn,
}

#[cfg(feature = "save")]
impl Save for EnvSparkState {
    fn save(&self, _: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        cur.write_u8(*self as u8)?;
        Ok(())
    }
}

#[cfg(feature = "save")]
impl Restore for EnvSparkState {
    fn restore(&mut self, _: &save::RestoreState, cur: &mut save::Cursor) -> save::SaveResult<()> {
        match cur.read_u8()? {
            0 => *self = Self::Off,
            1 => *self = Self::On,
            2 => *self = Self::AlwaysOn,
            _ => return Err(save::SaveError::InvalidEnum),
        }
        Ok(())
    }
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
    state: EnvSparkState,
}

impl EnvSpark {
    fn spawn_flags(&self) -> EnvSparkSpawnFlags {
        EnvSparkSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_next_think_time(&mut self) {
        let engine = self.engine();
        let delay = engine.random_float(0.0, self.delay);
        self.vars_mut().set_next_think_time_from_now(0.1 + delay);
    }
}

impl_entity_cast!(EnvSpark);

impl CreateEntity for EnvSpark {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            delay: 0.0,
            state: EnvSparkState::Off,
        }
    }
}

impl Entity for EnvSpark {
    delegate_entity!(base not { key_value, precache, spawn, think, used });

    fn key_value(&mut self, data: &mut KeyValue) {
        let name = data.key_name();
        if name == c"MaxDelay" {
            self.delay = data.value_str().parse().unwrap_or(0.0);
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
        for &i in SPARK_SOUNDS {
            engine.precache_sound(i);
        }
    }

    fn spawn(&mut self) {
        if self.delay <= 0.0 {
            self.delay = 1.5;
        }

        let spawn_flags = self.spawn_flags();
        if spawn_flags.intersects(EnvSparkSpawnFlags::USE) {
            if spawn_flags.intersects(EnvSparkSpawnFlags::USE_START_ON) {
                self.set_next_think_time();
                self.state = EnvSparkState::On;
            } else {
                self.state = EnvSparkState::Off;
            }
        } else {
            self.set_next_think_time();
            self.state = EnvSparkState::AlwaysOn;
        }

        self.precache();
    }

    fn think(&mut self) {
        if matches!(self.state, EnvSparkState::On | EnvSparkState::AlwaysOn) {
            self.set_next_think_time();
            let engine = self.engine();
            let v = self.vars_mut();
            let location = v.origin();
            do_spark(engine, v, location);
        }
    }

    fn used(&mut self, _: UseType, _: Option<&mut dyn Entity>, _: &mut dyn Entity) {
        match self.state {
            EnvSparkState::Off => {
                self.set_next_think_time();
                self.state = EnvSparkState::On;
            }
            EnvSparkState::On => {
                self.state = EnvSparkState::Off;
            }
            _ => {}
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Button {
    base: StubEntity,
}

impl_entity_cast!(Button);

impl CreateEntity for Button {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: StubEntity::new(base, false),
        }
    }
}

impl Entity for Button {
    delegate_entity!(base not { object_caps });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
            .union(if self.vars().take_damage() == TakeDamage::No {
                ObjectCaps::IMPULSE_USE
            } else {
                ObjectCaps::NONE
            })
    }
}

const SPARK_SOUNDS: &[&CStr] = &[
    res::valve::sound::buttons::SPARK1,
    res::valve::sound::buttons::SPARK2,
    res::valve::sound::buttons::SPARK3,
    res::valve::sound::buttons::SPARK4,
    res::valve::sound::buttons::SPARK5,
    res::valve::sound::buttons::SPARK6,
];

fn do_spark(engine: ServerEngineRef, vars: &mut EntityVars, location: vec3_t) {
    let pos = location + vars.size() * 0.5;
    engine.msg_pvs(pos, &user_message::Sparks::new(pos));
    let volume = engine.random_float(0.25, 0.75) * 0.4;
    let index = (engine.random_float(0.0, 1.0) * SPARK_SOUNDS.len() as f32) as usize;
    engine
        .build_sound()
        .channel_voice()
        .volume(volume)
        .emit(SPARK_SOUNDS[index], vars);
}

export_entity_default!("export-env_spark", env_spark, EnvSpark);
export_entity_default!("export-env_debris", env_debris, EnvSpark);

export_entity_stub!(button_target);
export_entity_stub!(env_global);
export_entity_stub!(func_button, Button);
export_entity_stub!(func_rot_button, Button);
export_entity_stub!(momentary_rot_button);
export_entity_stub!(multisource);
