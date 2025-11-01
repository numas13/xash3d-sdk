use bitflags::bitflags;
use xash3d_shared::entity::MoveType;

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, KeyValue, Solid, UseType},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
    utils,
};

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const EVERYONE  = 1 << 0;
        // const DISRUPT   = 1 << 1;
        const IN_AIR    = 1 << 2;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Shake {
    base: PointEntity,
}

impl CreateEntity for Shake {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl Shake {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    pub fn amplitude(&self) -> f32 {
        self.vars().scale()
    }

    pub fn set_amplitude(&self, amplitude: f32) {
        self.vars().set_scale(amplitude);
    }

    pub fn frequency(&self) -> f32 {
        self.vars().damage_save()
    }

    pub fn set_frequency(&self, frequency: f32) {
        self.vars().set_damage_save(frequency);
    }

    pub fn duration(&self) -> f32 {
        self.vars().damage_take()
    }

    pub fn set_duration(&self, duration: f32) {
        self.vars().set_damage_take(duration);
    }

    pub fn radius(&self) -> f32 {
        self.vars().damage()
    }

    pub fn set_radius(&self, radius: f32) {
        self.vars().set_damage(radius);
    }
}

impl Entity for Shake {
    delegate_entity!(base not { key_value, spawn, used });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"amplitude" => self.set_amplitude(data.parse_or_default()),
            b"frequency" => self.set_frequency(data.parse_or_default()),
            b"duration" => self.set_duration(data.parse_or_default()),
            b"radius" => self.set_radius(data.parse_or_default()),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let v = self.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.remove_effects();
        v.set_frame(0.0);

        if self.spawn_flags().intersects(SpawnFlags::EVERYONE) {
            self.set_radius(0.0);
        }
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        utils::ScreenShake::new(&self.engine())
            .amplitude(self.amplitude())
            .frequency(self.frequency())
            .duration(self.duration())
            .radius(self.radius())
            .in_air(self.spawn_flags().intersects(SpawnFlags::IN_AIR))
            .emit(self.vars().origin());
    }
}

impl_private!(Shake {});

export_entity_default!("export-env_shake", env_shake, Shake);
