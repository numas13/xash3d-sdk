use core::ffi::c_int;

use crate::{ffi, macros::define_enum_for_primitive};

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

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum MoveType: c_int {
        /// Never moves.
        #[default]
        None(ffi::common::MOVETYPE_NONE),
        /// Moving on the ground (player only).
        Walk(ffi::common::MOVETYPE_WALK),
        /// Gravity, special edge handling (monsters use this).
        Step(ffi::common::MOVETYPE_STEP),
        /// No gravity, but still collides with stuff.
        Fly(ffi::common::MOVETYPE_FLY),
        /// Gravity/collisions.
        Toss(ffi::common::MOVETYPE_TOSS),
        /// No clip to world, push and crush.
        Push(ffi::common::MOVETYPE_PUSH),
        /// No gravity, no collisions, still do velocity/avelocity.
        NoClip(ffi::common::MOVETYPE_NOCLIP),
        /// Extra size to monsters.
        FlyMissile(ffi::common::MOVETYPE_FLYMISSILE),
        /// Just like Toss, but reflect velocity when contacting surfaces.
        Bounce(ffi::common::MOVETYPE_BOUNCE),
        /// Bounce w/o gravity.
        BounceMissile(ffi::common::MOVETYPE_BOUNCEMISSILE),
        /// Track movement of aim entity.
        Follow(ffi::common::MOVETYPE_FOLLOW),
        /// BSP model that needs physics/world collisions (uses nearest hull for world collision).
        PushStep(ffi::common::MOVETYPE_PUSHSTEP),
        /// Glue two entities together (simple movewith).
        Compound(ffi::common::MOVETYPE_COMPOUND),
    }
}

impl MoveType {
    pub const fn is_flying(&self) -> bool {
        matches!(
            self,
            MoveType::Fly | MoveType::FlyMissile | MoveType::BounceMissile
        )
    }
}
