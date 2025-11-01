use core::cell::RefCell;

use alloc::vec::Vec;
use xash3d_server::{
    entities::{point_entity::PointEntity},
    entity::{
        delegate_entity, BaseEntity, EntityHandle, KeyValue, MoveType, ObjectCaps, Solid, UseType,
    },
    global_state::EntityState,
    prelude::*,
    private::impl_private,
    str::MapString,
    utils,
};

#[cfg(feature = "save")]
use xash3d_server::save;

use crate::multi_manager::MultiManager;

#[derive(Copy, Clone)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
struct Triggered {
    entity: EntityHandle,
    triggered: bool,
}

#[cfg(feature = "save")]
impl save::RestoreWithDefault for Triggered {
    fn default_for_restore(state: &save::RestoreState) -> Self {
        Self::new(state.engine().get_world_spawn_entity())
    }
}

impl Triggered {
    fn new(entity: EntityHandle) -> Self {
        Self {
            entity,
            triggered: false,
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct MultiSource {
    base: PointEntity,
    global_state_name: Option<MapString>,
    entities: RefCell<Vec<Triggered>>,
}

impl CreateEntity for MultiSource {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            global_state_name: None,
            entities: Default::default(),
        }
    }
}

impl MultiSource {
    const UNINITIIALIZED: u32 = 1;
}

impl Entity for MultiSource {
    delegate_entity!(base not { object_caps, key_value, spawn, think, used, is_triggered });

    fn object_caps(&self) -> ObjectCaps {
        self.base.object_caps().union(ObjectCaps::MASTER)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        trace!(
            "{}: key_value({:?}, {:?})",
            self.classname(),
            data.key_name(),
            data.value()
        );
        if data.key_name() == c"globalstate" {
            self.global_state_name = Some(self.engine().new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.with_spawn_flags(|f| f | Self::UNINITIIALIZED);
        v.set_next_think_time_from_now(0.1);
    }

    fn think(&self) {
        let engine = self.engine();
        let v = self.base.vars();
        if v.spawn_flags() & Self::UNINITIIALIZED == 0 {
            return;
        }

        if let Some(target_name) = v.target_name() {
            let target_name = target_name.as_thin();
            let mut entities = self.entities.borrow_mut();

            entities.extend(
                engine
                    .entities()
                    .by_target(target_name)
                    .filter_map(|i| i.get_entity())
                    .map(|i| Triggered::new(i.entity_handle())),
            );

            entities.extend(
                engine
                    .entities()
                    .by_class_name(c"multi_manager")
                    .filter_map(|i| i.downcast_ref::<MultiManager>())
                    .filter(|i| i.has_target(target_name))
                    .map(|i| Triggered::new(i.entity_handle())),
            );

            trace!("{}:", self.pretty_name());
            for (i, j) in entities.iter().enumerate() {
                trace!("  {i}: {}", j.entity.vars().pretty_name());
            }
        }

        v.with_spawn_flags(|f| f & !Self::UNINITIIALIZED);
    }

    fn used(&self, use_type: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
        let mut entities = self.entities.borrow_mut();
        let Some(i) = entities
            .iter_mut()
            .find(|i| i.entity == caller.entity_handle())
        else {
            let name = self.pretty_name();
            warn!("{name}: used by non-member entity {}", caller.pretty_name());
            return;
        };

        match use_type {
            UseType::Off => i.triggered = false,
            UseType::On => i.triggered = true,
            UseType::Set(value) => i.triggered = value != 0.0,
            UseType::Toggle => i.triggered ^= true,
        }

        let inputs = entities.len();
        // used in is_triggered
        drop(entities);

        if self.is_triggered(activator) {
            let name = self.pretty_name();
            trace!("{name}: enabled ({inputs} inputs)");
            let use_type = if self.global_state_name.is_some() {
                UseType::On
            } else {
                UseType::Toggle
            };
            utils::use_targets(use_type, None, self);
        }
    }

    fn is_triggered(&self, _: Option<&dyn Entity>) -> bool {
        let v = self.base.vars();
        if v.spawn_flags() & Self::UNINITIIALIZED != 0 {
            return false;
        }

        if !self.entities.borrow().iter().all(|i| i.triggered) {
            return false;
        }

        let triggered = self
            .global_state_name
            .map(|name| self.global_state().entity_state(name) == EntityState::On)
            .unwrap_or(true);

        trace!("{}: triggered={triggered}", self.pretty_name());
        triggered
    }
}

impl_private!(MultiSource {});

define_export! {
    export_multisource as export if "multisource" {
        multisource = multisource::MultiSource,
    }
}
