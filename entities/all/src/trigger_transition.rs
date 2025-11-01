use xash3d_server::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, MoveType, Solid},
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

define_export! {
    export_trigger_transition as export if "trigger-transition" {
        trigger_transition = trigger_transition::TriggerVolume,
    }
}
