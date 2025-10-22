use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityPlayer,
        KeyValue, ObjectCaps,
    },
    export::export_entity_default,
    str::MapString,
    utils,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

use super::triggers::init_trigger;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerSave {
    base: BaseEntity,
    master: Option<MapString>,
}

impl_entity_cast!(TriggerSave);

impl CreateEntity for TriggerSave {
    fn create(base: BaseEntity) -> Self {
        Self { base, master: None }
    }
}

impl Entity for TriggerSave {
    delegate_entity!(base not { object_caps, key_value, spawn, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

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

        init_trigger(&self.engine(), self.vars());
    }

    fn touched(&self, other: &dyn Entity) {
        let engine = self.engine();

        if !utils::is_master_triggered(&engine, self.master, Some(other)) {
            return;
        }

        if other.downcast_ref::<dyn EntityPlayer>().is_some() {
            self.remove_from_world();
            engine.server_command(c"autosave\n");
        }
    }
}

export_entity_default!("export-trigger_autosave", trigger_autosave, TriggerSave);
