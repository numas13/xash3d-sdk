use xash3d_server::{
    consts::SOLID_NOT,
    entity::{delegate_entity, BaseEntity, CreateEntity, Entity, KeyValue, MoveType, ObjectCaps},
    export::export_entity,
};

use crate::{entity::Private, impl_cast};

pub struct NodeEntity {
    base: BaseEntity,
    hint_type: u16,
    hint_activity: u16,
}

impl_cast!(NodeEntity);

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

export_entity!(info_node, Private<NodeEntity>);
export_entity!(info_node_air, Private<NodeEntity>);

// TODO: add the world graph
