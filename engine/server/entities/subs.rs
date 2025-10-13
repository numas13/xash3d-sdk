#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityPlayer,
        KeyValue, ObjectCaps, Private, Solid, UseType,
    },
    export::{export_entity, export_entity_default},
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
        let v = self.vars_mut();
        v.set_solid(Solid::Not);
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
            let v = self.vars_mut();
            v.set_net_name(engine.new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn is_triggered(&self, activator: &dyn Entity) -> bool {
        let engine = self.engine();
        if let Some(master) = self.vars().net_name() {
            utils::is_master_triggered(&engine, master, activator)
        } else {
            true
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
struct DelayedUseEntity {
    base: BaseEntity,
    use_type: UseType,
    kill_target: Option<MapString>,
}

impl DelayedUseEntity {
    fn new(base: BaseEntity, use_type: UseType, kill_target: Option<MapString>) -> Self {
        Self {
            base,
            use_type,
            kill_target,
        }
    }

    fn spawn_new(
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
            .new_entity_with::<Private<DelayedUseEntity>>(|base| {
                DelayedUseEntity::new(base, use_type, kill_target)
            })
            .class_name(c"DelayedUse")
            .vars(|e| {
                e.set_next_think_time_from_now(delay);
                if let Some(target) = target {
                    e.set_target(target);
                }
            })
            .build();

        if let Some(activator) = activator {
            if activator.downcast_ref::<dyn EntityPlayer>().is_some() {
                temp.vars_mut().set_owner(&activator);
            }
        }
    }
}

impl_entity_cast!(DelayedUseEntity);

impl CreateEntity for DelayedUseEntity {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            use_type: UseType::Off,
            kill_target: None,
        }
    }
}

impl Entity for DelayedUseEntity {
    delegate_entity!(base not { think });

    fn think(&mut self) {
        let mut activator = None;
        if let Some(owner) = self.vars().owner().map(|mut e| unsafe { e.as_mut() }) {
            activator = owner.get_entity_mut();
        }
        utils::use_targets(self.kill_target, self.use_type, 0.0, activator, self);
        self.remove_from_world();
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
#[derive(Copy, Clone)]
pub struct DelayedUse {
    #[cfg_attr(feature = "save", save(skip))]
    engine: ServerEngineRef,
    delay: f32,
    kill_target: Option<MapString>,
}

impl DelayedUse {
    pub fn new(engine: ServerEngineRef) -> Self {
        Self {
            engine,
            delay: 0.0,
            kill_target: None,
        }
    }

    pub fn delay(&self) -> f32 {
        self.delay
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.delay = delay;
    }

    pub fn kill_target(&self) -> Option<MapString> {
        self.kill_target
    }

    pub fn set_kill_target(&mut self, kill_target: impl Into<Option<MapString>>) {
        self.kill_target = kill_target.into();
    }

    pub fn key_value(&mut self, data: &mut KeyValue) -> bool {
        if data.key_name() == c"delay" {
            self.delay = data.value_str().parse().unwrap_or(0.0);
            data.set_handled(true);
            true
        } else if data.key_name() == c"killtarget" {
            self.kill_target = Some(self.engine.new_map_string(data.value()));
            data.set_handled(true);
            true
        } else {
            false
        }
    }

    pub fn use_targets(self, use_type: UseType, caller: &mut dyn Entity) {
        if self.delay != 0.0 {
            DelayedUseEntity::spawn_new(
                self.engine,
                self.delay,
                caller.vars().target(),
                use_type,
                self.kill_target,
                Some(caller),
            );
        } else {
            utils::use_targets(self.kill_target, use_type, 0.0, None, caller);
        }
    }
}

export_entity!(DelayedUse, Private<DelayedUseEntity>);

export_entity_default!(
    "export-info_player_deathmatch",
    info_player_deathmatch,
    DeathMatchStart
);
export_entity_default!("export-info_player_start", info_player_start, PointEntity);
export_entity_default!("export-info_landmark", info_landmark, PointEntity);
// Lightning target, just alias landmark.
export_entity_default!("export-info_target", info_target, PointEntity);
