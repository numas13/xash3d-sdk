use crate::{
    entities::trigger::Trigger,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue},
    export::export_entity_default,
    str::MapString,
    utils,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerSave {
    base: Trigger,
    master: Option<MapString>,
}

impl_entity_cast!(TriggerSave);

impl CreateEntity for TriggerSave {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
            master: None,
        }
    }
}

impl Entity for TriggerSave {
    delegate_entity!(base not { key_value, spawn, touched });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"master" {
            self.master = Some(self.engine().new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        if self.global_state().game_rules().is_deathmatch() {
            self.remove_from_world();
            return;
        }

        self.base.spawn();
    }

    fn touched(&self, other: &dyn Entity) {
        let engine = self.engine();

        if !utils::is_master_triggered(&engine, self.master, Some(other)) {
            return;
        }

        if other.is_player() {
            self.remove_from_world();
            engine.server_command(c"autosave\n");
        }
    }
}

export_entity_default!("export-trigger_autosave", trigger_autosave, TriggerSave);
