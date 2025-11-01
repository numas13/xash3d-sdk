use xash3d_server::{
    entity::{delegate_entity, BaseEntity, Solid},
    prelude::*,
    private::impl_private,
    utils::LinearMove,
};

use crate::base_door::{BaseDoor, SpawnFlags};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Door {
    pub(crate) base: BaseDoor<LinearMove>,
}

impl CreateEntity for Door {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseDoor::create(base),
        }
    }
}

impl Entity for Door {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.precache();

        let spawn_flags = self.base.spawn_flags();
        let v = self.base.vars();
        v.set_move_dir_from_angles();

        if v.skin() == 0 {
            // normal door
            if spawn_flags.intersects(SpawnFlags::PASSABLE) {
                v.set_solid(Solid::Not);
            } else {
                v.set_solid(Solid::Bsp);
            }
        } else {
            // special contents
            v.set_solid(Solid::Not);
            v.with_spawn_flags(|f| f | SpawnFlags::SILENT.bits());
        }

        self.base.spawn();
    }
}

impl_private!(Door {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_func_door {
    () => {
        $crate::export_entity!(func_door, $crate::func_door::Door);
    };
}
#[doc(inline)]
pub use export_func_door as export;
