use core::ffi::c_int;

use crate::macros::define_enum_for_primitive;

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum EntityType: c_int {
        #[default]
        Normal(0),
        Player(1),
        TempEntity(2),
        Beam(3),
        Fragmented(4),

        // TODO: use consts from ffi (xash3d-ffi update required)
        // Normal(ffi::common::ET_NORMAL),
        // Player(ffi::common::ET_PLAYER),
        // TempEntity(ffi::common::ET_TEMPENTITY),
        // Beam(ffi::common::ET_BEAM),
        // Fragmented(ffi::common::ET_FRAGMENTED),
    }
}
