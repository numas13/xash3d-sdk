use xash3d_server::{
    entity::{BaseEntity, Solid, delegate_entity},
    prelude::*,
    private::impl_private,
    utils::AngularMove,
};

use crate::base_door::{BaseDoor, SpawnFlags};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct RotatingDoor {
    pub(crate) base: BaseDoor<AngularMove>,
}

impl CreateEntity for RotatingDoor {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseDoor::create(base),
        }
    }
}

impl Entity for RotatingDoor {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.precache();

        let sf = self.base.spawn_flags();
        let v = self.base.vars();

        v.set_move_dir_from_spawn_flags(SpawnFlags::ROTATE_X.bits(), SpawnFlags::ROTATE_Z.bits());

        if sf.intersects(SpawnFlags::ROTATE_BACKWARDS) {
            v.with_move_dir(|x| -x);
        }

        if sf.intersects(SpawnFlags::PASSABLE) {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }

        self.base.spawn();
    }
}

impl_private!(RotatingDoor {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_func_door_rotating {
    () => {
        $crate::export_entity!(func_door_rotating, $crate::func_door_rotating::RotatingDoor);
    };
}
#[doc(inline)]
pub use export_func_door_rotating as export;
