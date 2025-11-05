use xash3d_server::{
    entity::{delegate_entity, BaseEntity, KeyValue, MoveType, ObjectCaps, Solid},
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct InfoNode {
    base: BaseEntity,
    #[cfg_attr(feature = "save", save(skip))]
    hint_type: u16,
    #[cfg_attr(feature = "save", save(skip))]
    hint_activity: u16,
}

impl CreateEntity for InfoNode {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            hint_type: 0,
            hint_activity: 0,
        }
    }
}

impl Entity for InfoNode {
    delegate_entity!(base not { object_caps, key_value, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"hinttype" => self.hint_type = data.parse_or_default(),
            b"activity" => self.hint_activity = data.parse_or_default(),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let v = self.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);

        // TODO: add node entity to the world graph
        let name = self.pretty_name();
        warn!("{name}: spawn is not implemented");

        self.vars().delayed_remove();
    }
}

impl_private!(InfoNode {});

// TODO: add the world graph

define_export! {
    export_info_node as export if "info-node" {
        info_node = info_node::InfoNode,
    }
}
