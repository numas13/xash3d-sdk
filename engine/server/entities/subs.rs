use xash3d_shared::consts::SOLID_NOT;

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityPlayer,
        KeyValue, ObjectCaps, Private, UseType,
    },
    prelude::*,
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
    base: PointEntity,
}

impl_entity_cast!(DeathMatchStart);

impl CreateEntity for DeathMatchStart {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
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

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct DelayedUse {
    base: BaseEntity,
    use_type: UseType,
    kill_target: Option<MapString>,
}

impl DelayedUse {
    pub fn new(base: BaseEntity, use_type: UseType, kill_target: Option<MapString>) -> Self {
        Self {
            base,
            use_type,
            kill_target,
        }
    }

    pub fn create(
        engine: ServerEngineRef,
        delay: f32,
        target: Option<MapString>,
        use_type: UseType,
        kill_target: Option<MapString>,
        activator: Option<&mut dyn Entity>,
    ) {
        if target.is_none() && kill_target.is_none() {
            return;
        }

        let temp = engine
            .new_entity_with::<Private<DelayedUse>>(|base| {
                DelayedUse::new(base, use_type, kill_target)
            })
            .class_name(c"DelayedUse")
            .vars(|e| {
                e.set_next_think_time(delay);
                if let Some(target) = target {
                    e.as_raw_mut().target = target.index();
                }
            })
            .build();

        if let Some(activator) = activator {
            if activator.downcast_ref::<dyn EntityPlayer>().is_some() {
                temp.vars_mut().as_raw_mut().owner = activator.as_edict_mut();
            }
        }
    }
}

impl_entity_cast!(DelayedUse);

impl Entity for DelayedUse {
    delegate_entity!(base not { think });

    fn think(&mut self) {
        if let Some(activator) = unsafe { self.vars().as_raw().owner.as_mut() } {
            if let Some(activator) = activator.get_entity_mut() {
                utils::use_targets(self.kill_target, self.use_type, 0.0, activator, Some(self));
                self.remove_from_world();
                return;
            }
        }

        utils::use_targets(self.kill_target, self.use_type, 0.0, self, None);
        self.remove_from_world();
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
