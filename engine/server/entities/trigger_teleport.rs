use bitflags::bitflags;
use xash3d_shared::{entity::EdictFlags, ffi::common::vec3_t};

use crate::{
    entities::{point_entity::PointEntity, trigger::Trigger},
    entity::{delegate_entity, impl_entity_cast, BaseEntity, KeyValue},
    export::export_entity_default,
    prelude::*,
    str::MapString,
    utils,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct SpawnFlags: u32 {
        /// Monsters allowed to fire this trigger.
        const ALLOW_MONSTERS = 1 << 0;
        /// Players not allowed to fire this trigger.
        const NO_CLIENTS = 1 << 1;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerTeleport {
    base: Trigger,
    master: Option<MapString>,
}

impl_entity_cast!(TriggerTeleport);

impl CreateEntity for TriggerTeleport {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
            master: None,
        }
    }
}

impl TriggerTeleport {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }
}

impl Entity for TriggerTeleport {
    delegate_entity!(base not { key_value, touched });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"master" {
            self.master = Some(self.engine().new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn touched(&self, other: &dyn Entity) {
        let engine = self.engine();
        let spawn_flags = self.spawn_flags();
        let v = self.vars();
        let other_v = other.vars();

        if !other_v
            .flags()
            .intersects(EdictFlags::CLIENT | EdictFlags::MONSTER)
        {
            return;
        }
        if !utils::is_master_triggered(&engine, self.master, Some(other)) {
            return;
        }

        if !spawn_flags.intersects(SpawnFlags::ALLOW_MONSTERS)
            && other_v.flags().intersects(EdictFlags::MONSTER)
        {
            return;
        }

        if spawn_flags.intersects(SpawnFlags::NO_CLIENTS)
            && other_v.flags().intersects(EdictFlags::CLIENT)
        {
            return;
        }

        let Some(target) = v.target() else {
            return;
        };
        let Some(target) = engine.entities().by_target_name(target).first() else {
            trace!("{}: target {target} is not found", self.pretty_name());
            return;
        };
        let target_v = target.vars();

        let mut teleport_target = target_v.origin();
        if other.is_player() {
            teleport_target.z -= other_v.min_size().z;
        }
        teleport_target.z += 1.0;

        trace!(
            "{}: teleport {} to {}",
            self.pretty_name(),
            other_v.pretty_name(),
            target_v.pretty_name(),
        );

        other_v.with_flags(|f| f.difference(EdictFlags::ONGROUND));
        other_v.set_origin_and_link(teleport_target);
        other_v.set_angles(target_v.angles());

        if other.is_player() {
            other_v.set_view_angle(target_v.angles());
        }

        other_v.set_fix_angle(1);
        other_v.set_velocity(vec3_t::ZERO);
        other_v.set_base_velocity(vec3_t::ZERO);
    }
}

export_entity_default!("export-trigger_teleport", trigger_teleport, TriggerTeleport);
export_entity_default!(
    "export-trigger_teleport",
    info_teleport_destination,
    PointEntity,
);
