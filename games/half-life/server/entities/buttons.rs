use core::ffi::CStr;

use xash3d_server::{
    entity::{delegate_entity, BaseEntity, CreateEntity, Entity, EntityVars, UseType},
    export::export_entity,
    ffi::{
        common::vec3_t,
        server::{KeyValueData, TYPEDESCRIPTION},
    },
    prelude::*,
    save::{
        define_fields, FieldType, KeyValueDataExt, SaveFields, SaveReader, SaveRestoreData,
        SaveResult, SaveWriter,
    },
    svc,
};

use crate::{entity::Private, impl_cast};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
#[repr(i32)]
enum EnvSparkState {
    Off,
    On,
    AlwaysOn,
}

pub struct EnvSpark {
    base: BaseEntity,
    delay: f32,
    state: EnvSparkState,
}

unsafe impl SaveFields for EnvSpark {
    const SAVE_NAME: &'static CStr = c"EnvSpark";

    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![
        delay,
        state => unsafe FieldType::INTEGER,
    ];
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

impl_cast!(EnvSpark);

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
    delegate_entity!(base not { key_value, save, restore, precache, spawn, think, used });

    fn key_value(&mut self, data: &mut KeyValueData) {
        let name = data.key_name();
        if name == c"MaxDelay" {
            let value = data.value().to_str().unwrap_or("");
            self.delay = value.parse().unwrap_or(0.0);
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

    fn save(&mut self, writer: &mut SaveWriter, save_data: &mut SaveRestoreData) -> SaveResult<()> {
        self.base.save(writer, save_data)?;
        writer.write_fields(save_data, self)
    }

    fn restore(
        &mut self,
        reader: &mut SaveReader,
        save_data: &mut SaveRestoreData,
    ) -> SaveResult<()> {
        self.base.restore(reader, save_data)?;
        reader.read_fields(save_data, self)
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

    fn used(&mut self, _: &mut dyn Entity, _: UseType, _: f32) {
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

export_entity!(env_spark, Private<EnvSpark>);
export_entity!(env_debris, Private<EnvSpark>);

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
    engine.msg_pvs(pos, &svc::Sparks::new(pos));
    let volume = engine.random_float(0.25, 0.75) * 0.4;
    let index = (engine.random_float(0.0, 1.0) * SPARK_SOUNDS.len() as f32) as usize;
    engine
        .build_sound()
        .channel_voice()
        .volume(volume)
        .emit(SPARK_SOUNDS[index], vars.as_edict_mut());
}
