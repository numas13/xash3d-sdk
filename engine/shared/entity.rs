use core::{ffi::c_int, mem};

use bitflags::bitflags;
use xash3d_ffi::common::entity_state_s;

use crate::{
    ffi,
    macros::define_enum_for_primitive,
    render::{RenderFx, RenderMode},
};

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum EntityType: c_int {
        #[default]
        Normal(ffi::common::ET_NORMAL),
        Player(ffi::common::ET_PLAYER),
        TempEntity(ffi::common::ET_TEMPENTITY),
        Beam(ffi::common::ET_BEAM),
        Fragmented(ffi::common::ET_FRAGMENTED),
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

bitflags! {
    /// Drawing effects for an entity.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Effects: c_int {
        const NONE                  = 0;
        /// Swirling cloud of particles.
        const BRIGHTFIELD           = ffi::common::EF_BRIGHTFIELD;
        /// Single frame ELIGHT on entity attachment 0.
        const MUZZLEFLASH           = ffi::common::EF_MUZZLEFLASH;
        /// DLIGHT centered at entity origin.
        const BRIGHTLIGHT           = ffi::common::EF_BRIGHTLIGHT;
        /// Player flashlight.
        const DIMLIGHT              = ffi::common::EF_DIMLIGHT;
        /// Get lighting from ceiling.
        const INVLIGHT              = ffi::common::EF_INVLIGHT;
        /// Do not interpolate the next frame.
        const NOINTERP              = ffi::common::EF_NOINTERP;
        /// Rocket flare glow sprite.
        const LIGHT                 = ffi::common::EF_LIGHT;
        /// Do not draw this entity.
        const NODRAW                = ffi::common::EF_NODRAW;
        /// Do not remove sides for func_water entity.
        const WATERSIDES            = ffi::common::EF_WATERSIDES;
        /// Just get fullbright.
        const FULLBRIGHT            = ffi::common::EF_FULLBRIGHT;
        /// Ignore shadow for this entity
        const NOSHADOW              = ffi::common::EF_NOSHADOW;
        /// This entity allowed to merge vis (e.g. env_sky or portal camera).
        const MERGE_VISIBILITY      = ffi::common::EF_MERGE_VISIBILITY;
        /// This entity requested phs bitvector instead of pvsbitvector in AddToFullPack calls.
        const REQUEST_PHS           = ffi::common::EF_REQUEST_PHS;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct EdictFlags: c_int {
        const NONE          = 0;
        /// Changes the SV_Movestep() behavior to not need to be on ground.
        const FLY           = ffi::common::FL_FLY;
        /// Changes the SV_Movestep() behavior to not need to be on ground (but stay in water).
        const SWIM          = ffi::common::FL_SWIM;
        const CONVEYOR      = ffi::common::FL_CONVEYOR;
        const CLIENT        = ffi::common::FL_CLIENT;
        const INWATER       = ffi::common::FL_INWATER;
        const MONSTER       = ffi::common::FL_MONSTER;
        const GODMODE       = ffi::common::FL_GODMODE;
        const NOTARGET      = ffi::common::FL_NOTARGET;
        /// Don't send entity to local host, it's predicting this entity itself.
        const SKIPLOCALHOST = ffi::common::FL_SKIPLOCALHOST;
        /// At rest / on the ground.
        const ONGROUND      = ffi::common::FL_ONGROUND;
        /// Not all corners are valid.
        const PARTIALGROUND = ffi::common::FL_PARTIALGROUND;
        /// Player jumping out of water.
        const WATERJUMP     = ffi::common::FL_WATERJUMP;
        /// Player is frozen for 3rd person camera.
        const FROZEN        = ffi::common::FL_FROZEN;
        /// JAC: fake client, simulated server side; don't send network messages to them.
        const FAKECLIENT    = ffi::common::FL_FAKECLIENT;
        /// Player flag -- Player is fully crouched.
        const DUCKING       = ffi::common::FL_DUCKING;
        /// Apply floating force to this entity when in water.
        const FLOAT         = ffi::common::FL_FLOAT;
        /// Worldgraph has this ent listed as something that blocks a connection.
        const GRAPHED       = ffi::common::FL_GRAPHED;

        // UNDONE: Do we need these?
        const IMMUNE_WATER  = ffi::common::FL_IMMUNE_WATER;
        const IMMUNE_SLIME  = ffi::common::FL_IMMUNE_SLIME;
        const IMMUNE_LAVA   = ffi::common::FL_IMMUNE_LAVA;

        /// This is a spectator proxy.
        const PROXY         = ffi::common::FL_PROXY;
        /// Brush model flag.
        ///
        /// Call think every frame regardless of nextthink - ltime (for
        /// constantly changing velocity/path).
        const ALWAYSTHINK   = ffi::common::FL_ALWAYSTHINK;
        /// Base velocity has been applied this frame.
        ///
        /// Used to convert base velocity into momentum.
        const BASEVELOCITY  = ffi::common::FL_BASEVELOCITY;
        /// Only collide in with monsters who have FL_MONSTERCLIP set.
        const MONSTERCLIP   = ffi::common::FL_MONSTERCLIP;
        /// Player is _controlling_ a train.
        ///
        /// Movement commands should be ignored on client during prediction.
        const ONTRAIN       = ffi::common::FL_ONTRAIN;
        /// Not moveable/removeable brush entity.
        ///
        /// Really part of the world, but represented as an entity for transparency or something.
        const WORLDBRUSH    = ffi::common::FL_WORLDBRUSH;
        /// This client is a spectator.
        ///
        /// Don't run touch functions, etc.
        const SPECTATOR     = ffi::common::FL_SPECTATOR;
        /// Predicted laser spot from rocket launcher.
        const LASERDOT      = ffi::common::FL_LASERDOT;

        /// This is a custom entity.
        const CUSTOMENTITY  = ffi::common::FL_CUSTOMENTITY;
        /// This entity is marked for death.
        ///
        /// This allows the engine to kill ents at the appropriate time.
        const KILLME        = ffi::common::FL_KILLME;
        /// Entity is dormant, no updates to client.
        const DORMANT       = ffi::common::FL_DORMANT as c_int;
    }
}

// TODO: add safe wrapper for entity_state_s and remove this trait
#[doc(hidden)]
pub trait EntityStateExt {
    fn renderfx(&self) -> RenderFx;

    fn rendermode(&self) -> RenderMode;

    fn effects(&self) -> &Effects;
}

impl EntityStateExt for entity_state_s {
    fn renderfx(&self) -> RenderFx {
        RenderFx::from_raw(self.renderfx).unwrap()
    }

    fn rendermode(&self) -> RenderMode {
        RenderMode::from_raw(self.rendermode).unwrap()
    }

    fn effects(&self) -> &Effects {
        const_assert_size_of_field_eq!(Effects, entity_state_s, effects);
        unsafe { mem::transmute(&self.effects) }
    }
}
