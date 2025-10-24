use core::ffi::CStr;

use crate::{
    entity::{
        create_entity, delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity,
        EntityCast, KeyValue,
    },
    prelude::*,
    save::{Restore, Save},
};

pub trait WorldItemsNames: 'static + Save + Restore {
    fn create(engine: ServerEngineRef) -> Self;
    fn get_class_name(&self, item_type: u16) -> Option<&CStr>;
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct WorldItems<T> {
    base: BaseEntity,
    item_type: u16,
    items: T,
}

impl<T: WorldItemsNames> EntityCast for WorldItems<T> {
    impl_entity_cast!(cast WorldItems<T>);
}

impl<T: WorldItemsNames> CreateEntity for WorldItems<T> {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base,
            item_type: 0,
            items: T::create(engine),
        }
    }
}

impl<T: WorldItemsNames> Entity for WorldItems<T> {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"type" {
            self.item_type = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();
        if let Some(name) = self.items.get_class_name(self.item_type) {
            let ty = self.item_type;
            trace!("{}: create item({ty}) {name:?}", self.pretty_name());
            match create_entity(&engine, name, v.origin(), v.angles(), None) {
                Ok(item) => {
                    let item_v = item.vars();
                    item_v.set_target(v.target());
                    item_v.set_target_name(v.target_name());
                    item_v.set_spawn_flags(v.spawn_flags());
                }
                Err(_) => {
                    error!("{}: failed to create {name:?}", self.pretty_name());
                }
            }
        } else {
            warn!(
                "{}: failed to create item {}",
                self.pretty_name(),
                self.item_type
            )
        }
        v.delayed_remove();
    }
}
