use core::ffi::CStr;

use xash3d_shared::ffi::common::vec3_t;

#[cfg(feature = "save")]
use crate::save::{self, Restore, Save};
use crate::{
    engine::ServerEngineRef,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityVars, KeyValue,
        UseType,
    },
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

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct EnvSpark {
    base: BaseEntity,
    delay: f32,
    state: EnvSparkState,
}

impl EnvSpark {
    pub const SF_USE: i32 = 1 << 5;
    pub const SF_USE_START_ON: i32 = 1 << 6;

    fn set_next_think_time(&mut self) {
        let engine = self.engine();
        let delay = engine.random_float(0.0, self.delay);
        self.vars_mut().set_next_think_time(0.1 + delay);
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

        let ev = self.vars_mut().as_raw_mut();
        if ev.spawnflags & Self::SF_USE != 0 {
            if ev.spawnflags & Self::SF_USE_START_ON != 0 {
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
            let location = self.vars().as_raw().origin;
            do_spark(engine, self.vars_mut(), location);
        }
    }

    fn used(&mut self, _: &mut dyn Entity, _: Option<&mut dyn Entity>, _: UseType, _: f32) {
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

const SPARK_SOUNDS: &[&CStr] = &[
    res::valve::sound::buttons::SPARK1,
    res::valve::sound::buttons::SPARK2,
    res::valve::sound::buttons::SPARK3,
    res::valve::sound::buttons::SPARK4,
    res::valve::sound::buttons::SPARK5,
    res::valve::sound::buttons::SPARK6,
];

fn do_spark(engine: ServerEngineRef, vars: &mut EntityVars, location: vec3_t) {
    let ev = vars.as_raw();
    let pos = location + ev.size * 0.5;
    engine.msg_pvs(pos, &user_message::Sparks::new(pos));
    let volume = engine.random_float(0.25, 0.75) * 0.4;
    let index = (engine.random_float(0.0, 1.0) * SPARK_SOUNDS.len() as f32) as usize;
    engine
        .build_sound()
        .channel_voice()
        .volume(volume)
        .emit(SPARK_SOUNDS[index], vars.as_edict_mut());
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use super::EnvSpark;
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(env_spark, Private<EnvSpark>);
    export_entity!(env_debris, Private<EnvSpark>);

    export_entity!(button_target, Private<StubEntity>);
    export_entity!(env_global, Private<StubEntity>);
    export_entity!(func_button, Private<StubEntity>);
    export_entity!(func_rot_button, Private<StubEntity>);
    export_entity!(momentary_rot_button, Private<StubEntity>);
    export_entity!(multisource, Private<StubEntity>);
}
