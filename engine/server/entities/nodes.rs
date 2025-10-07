use xash3d_shared::{consts::SOLID_NOT, entity::MoveType};

use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, ObjectCaps,
    },
    save::{Restore, Save},
};

#[derive(Save, Restore)]
pub struct NodeEntity {
    base: BaseEntity,
    #[save(skip)]
    hint_type: u16,
    #[save(skip)]
    hint_activity: u16,
}

impl_entity_cast!(NodeEntity);

impl CreateEntity for NodeEntity {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            hint_type: 0,
            hint_activity: 0,
        }
    }
}

impl Entity for NodeEntity {
    delegate_entity!(base not { object_caps, key_value, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        let value = data.value_str();
        match data.key_name().to_bytes() {
            b"hinttype" => self.hint_type = value.parse().unwrap_or(0),
            b"activity" => self.hint_activity = value.parse().unwrap_or(0),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let ev = self.vars_mut().as_raw_mut();
        ev.movetype = MoveType::None.into();
        ev.solid = SOLID_NOT;

        // TODO: add node entity to the world graph
        warn!("spawn {} is not implemented", self.classname());

        self.vars_mut().delayed_remove();
    }
}

// TODO: add the world graph

#[cfg(feature = "export-default-entities")]
mod exports {
    use super::NodeEntity;
    use crate::{entity::Private, export::export_entity};

    export_entity!(info_node, Private<NodeEntity>);
    export_entity!(info_node_air, Private<NodeEntity>);
}
