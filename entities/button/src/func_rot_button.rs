use bitflags::bitflags;
use xash3d_server::{
    entity::{BaseEntity, MoveType, Solid, delegate_entity},
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
    utils::{AngularMove, Move},
};

use crate::base_button::BaseButton;

bitflags! {
    #[derive(Copy, Clone, Debug)]
    struct SpawnFlags: u32 {
        const NOT_SOLID         = 1 << 0;
        const ROTATE_BACKWARDS  = 1 << 1;
        const TOGGLE            = 1 << 5;
        const ROTATE_Z          = 1 << 6;
        const ROTATE_X          = 1 << 7;
        const TOUCH_ONLY        = 1 << 8;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct RotatingButton {
    base: BaseButton<AngularMove>,
}

impl CreateEntity for RotatingButton {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseButton::create(base),
        }
    }
}

impl RotatingButton {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_move_dir_from_spawn_flags(&self) {
        let v = self.vars();
        let flags = self.spawn_flags();
        if flags.intersects(SpawnFlags::ROTATE_Z) {
            v.set_move_dir(vec3_t::Z);
        } else if flags.intersects(SpawnFlags::ROTATE_X) {
            v.set_move_dir(vec3_t::X);
        } else {
            v.set_move_dir(vec3_t::Y);
        }
    }
}

impl Entity for RotatingButton {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.base.spawn();
        self.precache();

        let sf = self.spawn_flags();
        let v = self.base.base.vars();

        self.set_move_dir_from_spawn_flags();
        if sf.intersects(SpawnFlags::ROTATE_BACKWARDS) {
            v.with_move_dir(|x| -x);
        }
        v.set_move_type(MoveType::Push);

        if sf.intersects(SpawnFlags::NOT_SOLID) {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }
        v.reload_model();

        self.base.button_move.init(v);
    }
}

impl_private!(RotatingButton {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_func_rot_button {
    () => {
        $crate::export_entity!(func_rot_button, $crate::func_rot_button::RotatingButton);
    };
}
#[doc(inline)]
pub use export_func_rot_button as export;
