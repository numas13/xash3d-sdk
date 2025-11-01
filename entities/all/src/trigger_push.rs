use bitflags::bitflags;
use xash3d_server::{
    entities::trigger::Trigger,
    entity::{EdictFlags, MoveType, delegate_entity, BaseEntity, Solid, UseType},
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    struct SpawnFlags: u32 {
        const PUSH_ONCE = 1 << 0;
        // spawnflag that makes trigger_push spawn turned OFF
        const START_OFF = 1 << 1;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerPush {
    base: Trigger,
}

impl CreateEntity for TriggerPush {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
        }
    }
}

impl Entity for TriggerPush {
    delegate_entity!(base not { spawn, used, touched });

    fn spawn(&mut self) {
        let v = self.base.vars();
        if v.angles() == vec3_t::ZERO {
            v.with_angles(|v| v.with_y(360.0));
        }
        self.base.spawn();

        let v = self.base.vars();
        if v.speed() == 0.0 {
            v.set_speed(100.0);
        }

        let spawn_flags = SpawnFlags::from_bits_retain(v.spawn_flags());
        if spawn_flags.intersects(SpawnFlags::START_OFF) {
            v.set_solid(Solid::Not);
        }

        v.link();
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        self.base.toggle_use();
    }

    fn touched(&self, other: &dyn Entity) {
        let other_v = other.vars();
        if let MoveType::None | MoveType::Push | MoveType::NoClip | MoveType::Follow =
            other_v.move_type()
        {
            return;
        }
        if let Solid::Not | Solid::Bsp = other_v.solid() {
            return;
        }

        let v = self.base.vars();
        let push_vec = v.move_dir() * v.speed();
        let spawn_flags = SpawnFlags::from_bits_retain(self.vars().spawn_flags());
        if spawn_flags.intersects(SpawnFlags::PUSH_ONCE) {
            other_v.with_velocity(|v| v + push_vec);
            if other_v.velocity().z > 0.0 {
                other_v.with_flags(|f| f.difference(EdictFlags::ONGROUND));
            }
            self.remove_from_world();
        } else if other_v.flags().intersects(EdictFlags::BASEVELOCITY) {
            other_v.with_base_velocity(|v| v + push_vec);
        } else {
            other_v.with_flags(|f| f | EdictFlags::BASEVELOCITY);
            other_v.set_base_velocity(push_vec);
        }
    }
}

impl_private!(TriggerPush {});

define_export! {
    export_trigger_push as export if "trigger-push" {
        trigger_push = trigger_push::TriggerPush,
    }
}
