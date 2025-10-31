use xash3d_shared::entity::MoveType;

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, Solid},
    export::export_entity_default,
    prelude::*,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerVolume {
    base: PointEntity,
}

impl_entity_cast!(TriggerVolume);

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

export_entity_default!(
    "export-trigger_transition",
    trigger_transition,
    TriggerVolume
);
