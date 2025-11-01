use bitflags::bitflags;
use xash3d_shared::entity::MoveType;

use crate::{
    entity::{delegate_entity, BaseEntity, KeyValue, ObjectCaps, Solid, UseType},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct SpeakerSpawnFlags: u32 {
        const START_SILENT = 1;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Speaker {
    base: BaseEntity,
    preset: i32,
}

impl CreateEntity for Speaker {
    fn create(base: BaseEntity) -> Self {
        Self { base, preset: 0 }
    }
}

impl Speaker {
    const ANNOUNCE_MINUTES_MIN: f32 = 0.25;
    const ANNOUNCE_MINUTES_MAX: f32 = 2.25;

    fn spawn_flags(&self) -> SpeakerSpawnFlags {
        SpeakerSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }
}

impl Entity for Speaker {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, used, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"preset" {
            self.preset = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn precache(&mut self) {
        let spawn_flags = self.spawn_flags();
        if !spawn_flags.intersects(SpeakerSpawnFlags::START_SILENT) {
            let time = self.engine().random_float(5.0, 15.0);
            self.vars().set_next_think_time_from_now(time);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();

        if self.preset == 0 && v.message().map_or(true, |s| s.is_empty()) {
            error!(
                "{}: with no Level/Sentence at {}",
                self.classname(),
                v.origin()
            );
            self.remove_from_world();
            return;
        }

        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);

        self.precache();
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let v = self.vars();
        let active = v.next_think_time() > 0.0;
        if !use_type.should_toggle(active) {
            return;
        }

        match use_type {
            UseType::On => v.set_next_think_time_from_now(0.1),
            UseType::Off => v.stop_thinking(),
            _ => {
                if !active {
                    v.set_next_think_time_from_now(0.1);
                } else {
                    v.stop_thinking();
                }
            }
        }
    }

    fn think(&self) {
        let engine = self.engine();
        let global_state = self.global_state();
        let v = self.vars();

        if engine.globals.map_time() <= global_state.talk_wait_time() {
            let time = engine.random_float(5.0, 1.0);
            v.set_next_think_time(global_state.talk_wait_time() + time);
            return;
        }

        let message = v.message();
        let sound_file = match self.preset {
            0 => message.as_ref().map_or(c"", |s| s.as_c_str()),
            1 => c"C1A0_",
            2 => c"C1A1_",
            3 => c"C1A2_",
            4 => c"C1A3_",
            5 => c"C1A4_",
            6 => c"C2A1_",
            7 => c"C2A2_",
            8 => c"C2A3_",
            9 => c"C2A4_",
            10 => c"C2A5_",
            11 => c"C3A1_",
            12 => c"C3A2_",
            _ => c"",
        };

        let sound = engine
            .build_sound()
            .volume(v.health() * 0.1)
            .attenuation(0.3);

        if let Some(b'!') = sound_file.to_bytes().first() {
            sound.ambient_emit_dyn(sound_file, v.origin(), v);
        } else {
            if sound.emit_random_sentence(sound_file.into(), v).is_none() {
                warn!(
                    "{}: invalid sentence group {sound_file:?}",
                    self.classname()
                );
            }

            let time = engine.random_float(
                Self::ANNOUNCE_MINUTES_MIN * 60.0,
                Self::ANNOUNCE_MINUTES_MAX * 60.0,
            );
            v.set_next_think_time_from_now(time);

            global_state.set_talk_wait_time_from_now(5.0);
        }
    }
}

impl_private!(Speaker {});

export_entity_default!("export-speaker", speaker, Speaker);
