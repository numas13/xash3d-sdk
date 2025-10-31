use bitflags::bitflags;

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, KeyValue},
    export::export_entity_default,
    prelude::*,
};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct SpawnFlags: u32 {
        const WAIT_FOR_RETRIGGER    = 1 << 0;
        const TELEPORT              = 1 << 1;
        const FIRE_ONCE             = 1 << 2;
    }
}

impl SpawnFlags {
    pub fn has_wait_for_retrigger(&self) -> bool {
        self.intersects(Self::WAIT_FOR_RETRIGGER)
    }

    pub fn has_teleport(&self) -> bool {
        self.intersects(Self::TELEPORT)
    }

    pub fn has_fire_once(&self) -> bool {
        self.intersects(Self::FIRE_ONCE)
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct PathCorner {
    base: PointEntity,
    wait: f32,
}

impl CreateEntity for PathCorner {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            wait: 0.0,
        }
    }
}

impl PathCorner {
    pub fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    pub fn delay(&self) -> f32 {
        self.wait
    }
}

impl Entity for PathCorner {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"wait" {
            self.wait = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        if self.vars().target_name().is_none() {
            error!("{}: without a target name", self.pretty_name());
        }
    }
}

export_entity_default!("export-path_corner", path_corner, PathCorner {});
