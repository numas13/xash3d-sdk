use xash3d_shared::entity::MoveType;

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, Solid},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerVolume {
    base: PointEntity,
}

impl CreateEntity for TriggerVolume {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl Entity for TriggerVolume {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        let v = self.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.reload_model();
        v.remove_model();
    }
}

impl_private!(TriggerVolume {});

export_entity_default!(
    "export-trigger_transition",
    trigger_transition,
    TriggerVolume
);
