use crate::{
    entities::trigger::Trigger,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, UseType},
    export::export_entity_default,
    sound::play_cd_track,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerCdAudio {
    base: Trigger,
}

impl_entity_cast!(TriggerCdAudio);

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

export_entity_default!("export-trigger_cdaudio", trigger_cdaudio, TriggerCdAudio);
