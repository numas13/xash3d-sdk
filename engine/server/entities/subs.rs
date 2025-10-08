use xash3d_shared::consts::SOLID_NOT;

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, ObjectCaps,
    },
    str::MapString,
    utils,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct PointEntity {
    base: BaseEntity,
}

impl_entity_cast!(PointEntity);

impl CreateEntity for PointEntity {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for PointEntity {
    delegate_entity!(base not { object_caps, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let ev = self.vars_mut().as_raw_mut();
        ev.solid = SOLID_NOT;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct DeathMatchStart {
    base: BaseEntity,
}

impl_entity_cast!(DeathMatchStart);

impl CreateEntity for DeathMatchStart {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for DeathMatchStart {
    delegate_entity!(base not { key_value, is_triggered });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"master" {
            let engine = self.engine();
            let ev = self.vars_mut().as_raw_mut();
            ev.netname = engine.new_map_string(data.value()).index();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn is_triggered(&self, activator: &dyn Entity) -> bool {
        let engine = self.engine();
        if let Some(master) = MapString::from_index(engine, self.vars().as_raw().netname) {
            utils::is_master_triggered(&engine, master, activator)
        } else {
            true
        }
    }
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use super::PointEntity;
    use crate::{entity::Private, export::export_entity};

    export_entity!(info_player_deathmatch, Private<super::DeathMatchStart>);
    export_entity!(info_player_start, Private<PointEntity>);
    export_entity!(info_landmark, Private<PointEntity>);
    // Lightning target, just alias landmark.
    export_entity!(info_target, Private<PointEntity>);
}
