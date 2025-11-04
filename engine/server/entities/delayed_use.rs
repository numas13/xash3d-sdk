use core::ffi::CStr;

use crate::{
    entity::{delegate_entity, BaseEntity, KeyValue, UseType},
    export::export_entity,
    prelude::*,
    str::MapString,
    utils,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
struct DelayedUseEntity {
    base: BaseEntity,
    use_type: UseType,
    kill_target: Option<MapString>,
}

impl DelayedUseEntity {
    const CLASS_NAME: &'static CStr = c"DelayedUse";

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
        activator: Option<&dyn Entity>,
    ) {
        if target.is_none() && kill_target.is_none() {
            return;
        }

        let temp = engine
            .new_entity_with::<DelayedUseEntity>(|base| {
                DelayedUseEntity::new(base, use_type, kill_target)
            })
            .class_name(Self::CLASS_NAME)
            .vars(|e| {
                e.set_next_think_time_from_now(delay);
                if let Some(target) = target {
                    e.set_target(target);
                }
            })
            .build();

        if let Some(activator) = activator {
            if activator.is_player() {
                temp.vars().set_owner(&activator);
            }
        }
    }
}

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

    fn think(&self) {
        let mut activator = None;
        if let Some(owner) = self.vars().owner() {
            activator = owner.get_entity();
        }
        if let Some(kill_target) = self.kill_target {
            utils::kill_targets(&self.engine(), &kill_target);
        }
        utils::use_targets(self.use_type, activator, self);
        self.remove_from_world();
    }
}

export_entity!(DelayedUse, DelayedUseEntity {});

#[cfg_attr(feature = "save", derive(Save, Restore))]
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

    pub fn kill_target(&self) -> Option<MapString> {
        self.kill_target
    }

    pub fn key_value(&mut self, data: &mut KeyValue) -> bool {
        if data.key_name() == c"delay" {
            self.delay = data.parse_or_default();
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

    pub fn use_targets(
        &self,
        use_type: UseType,
        activator: Option<&dyn Entity>,
        caller: &dyn Entity,
    ) {
        if self.delay != 0.0 {
            DelayedUseEntity::spawn_new(
                self.engine,
                self.delay,
                caller.vars().target(),
                use_type,
                self.kill_target,
                activator,
            );
        } else {
            if let Some(kill_target) = self.kill_target {
                utils::kill_targets(&self.engine, &kill_target);
            }
            utils::use_targets(use_type, activator, caller);
        }
    }
}
