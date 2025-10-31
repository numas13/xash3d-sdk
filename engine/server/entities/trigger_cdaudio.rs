use crate::{
    entities::trigger::Trigger,
    entity::{delegate_entity, BaseEntity, UseType},
    export::export_entity_default,
    prelude::*,
    sound::play_cd_track,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerCdAudio {
    base: Trigger,
}

impl CreateEntity for TriggerCdAudio {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
        }
    }
}

impl TriggerCdAudio {
    fn play_track(&self) {
        let v = self.vars();
        play_cd_track(&self.engine(), v.health() as i32);
        v.set_health(0.0);
        self.remove_from_world();
    }
}

impl Entity for TriggerCdAudio {
    delegate_entity!(base not { used, touched });

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        self.play_track();
    }

    fn touched(&self, _: &dyn Entity) {
        self.play_track();
    }
}

export_entity_default!("export-trigger_cdaudio", trigger_cdaudio, TriggerCdAudio {});
