use xash3d_server::{
    entity::{delegate_entity, BaseEntity, KeyValue},
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
};

use crate::func_plat::Platform;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct RotatingPlatform {
    pub(crate) base: Platform,
}

impl CreateEntity for RotatingPlatform {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Platform::create(base),
        }
    }
}

impl RotatingPlatform {
    const SF_ROTATE_Z: u32 = 1 << 6;
    const SF_ROTATE_X: u32 = 1 << 7;
}

impl Entity for RotatingPlatform {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"rotation" {
            self.base.rotation = data.parse_or_default();
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        self.base.spawn();

        let v = self.base.base.vars();
        if self.base.rotation != 0.0 {
            v.set_move_dir_from_spawn_flags(Self::SF_ROTATE_X, Self::SF_ROTATE_Z);
            self.base.start_angles = v.angles() + v.move_dir() * self.base.rotation;
            self.base.end_angles = v.angles();
        } else {
            self.base.start_angles = vec3_t::ZERO;
            self.base.end_angles = vec3_t::ZERO;
        }

        if v.target_name().is_some() {
            v.set_angles(self.base.start_angles);
        }
    }
}

impl_private!(RotatingPlatform {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_func_platrot {
    () => {
        $crate::export_entity!(func_platrot, $crate::func_platrot::RotatingPlatform);
    };
}
#[doc(inline)]
pub use export_func_platrot as export;
