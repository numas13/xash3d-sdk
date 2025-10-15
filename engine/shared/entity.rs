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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EntityIndex(u16);

impl Default for EntityIndex {
    fn default() -> Self {
        Self::WORLD_SPAWN
    }
}

impl EntityIndex {
    #[deprecated(note = "use WORLD_SPAWN instead")]
    pub const ZERO: Self = Self(0);

    /// The world spawn entity index.
    pub const WORLD_SPAWN: Self = Self(0);

    pub const fn new(index: u16) -> Option<Self> {
        if index < 0x1000 {
            Some(Self(index))
        } else {
            None
        }
    }

    /// Creates `EntityIndex` from a raw index value.
    ///
    /// # Safety
    ///
    /// The index value must be less than `0x1000`.
    pub const unsafe fn new_unchecked(index: u16) -> Self {
        Self(index)
    }

    pub const fn to_u16(self) -> u16 {
        self.0
    }

    pub const fn to_i32(self) -> i32 {
        self.0 as i32
    }

    #[deprecated(note = "use is_world_spawn instead")]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if the index is for the world spawn entity.
    pub const fn is_world_spawn(&self) -> bool {
        self.0 == Self::WORLD_SPAWN.0
    }
}

impl From<BeamEntity> for EntityIndex {
    fn from(value: BeamEntity) -> Self {
        value.index()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BeamEntity(u16);

impl Default for BeamEntity {
    fn default() -> Self {
        Self::new(EntityIndex::default())
    }
}

impl BeamEntity {
    pub const fn new(index: EntityIndex) -> BeamEntity {
        BeamEntity(index.to_u16())
    }

    /// Creates `BeamEntity` from the given values.
    ///
    /// Returns `None` if the attachment is not less than `0x10`.
    pub const fn with_attachment(index: EntityIndex, attachment: u16) -> Option<BeamEntity> {
        if attachment < 0x10 {
            Some(unsafe { Self::new_unchecked(index.to_u16(), attachment) })
        } else {
            None
        }
    }

    /// Creates `BeamEntity` without checking whether arguments are valid. This results
    /// in undefined behavior if arguments is not valid.
    ///
    /// # Safety
    ///
    /// * `index` must be less than `0x1000`.
    /// * `attachment` must be less than `0x10`.
    pub const unsafe fn new_unchecked(index: u16, attachment: u16) -> BeamEntity {
        Self((attachment << 12) | index)
    }

    /// Creates `BeamEntity` from a raw value.
    pub const fn from_bits(bits: u16) -> BeamEntity {
        BeamEntity(bits)
    }

    /// Returns the underlying bits value.
    pub const fn bits(&self) -> u16 {
        self.0
    }

    /// Returns the entity index.
    pub const fn index(&self) -> EntityIndex {
        unsafe { EntityIndex::new_unchecked(self.0 & 0xfff) }
    }

    /// Returns the attachment.
    pub const fn attachment(&self) -> u16 {
        self.0 >> 12
    }
}

impl From<EntityIndex> for BeamEntity {
    fn from(value: EntityIndex) -> Self {
        Self::new(value)
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

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct DamageFlags: u32 {
        const GENERIC = 0;

        const CRUSH = 1 << 0;
        const BULLET = 1 << 1;
        const SLASH = 1 << 2;
        const BURN = 1 << 3;
        const FREEZE = 1 << 4;
        const FALL = 1 << 5;
        const BLAST = 1 << 6;
        const CLUB = 1 << 7;
        const SHOCK = 1 << 8;
        const SONIC = 1 << 9;
        const ENERGYBEAM = 1 << 10;
        const NEVERGIB = 1 << 12;
        const ALWAYSGIB = 1 << 13;

        const TIMEBASED = !0xff003fff;

        const DROWN = 1 << 14;
        const FIRSTTIMEBASED = Self::DROWN.bits();

        const PARALYZE = 1 << 15;
        const NERVEGAS = 1 << 16;
        const POISON = 1 << 17;
        const RADIATION = 1 << 18;
        const DROWNRECOVER = 1 << 19;
        const ACID = 1 << 20;
        const SLOWBURN = 1 << 21;
        const SLOWFREEZE = 1 << 22;
        const MORTAR = 1 << 23;
    }
}

bitflags! {
    /// Buttons the player is currently pressing.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Buttons: i32 {
        const NONE          = 0;
        const ATTACK        = ffi::common::IN_ATTACK;
        const JUMP          = ffi::common::IN_JUMP;
        const DUCK          = ffi::common::IN_DUCK;
        const FORWARD       = ffi::common::IN_FORWARD;
        const BACK          = ffi::common::IN_BACK;
        const USE           = ffi::common::IN_USE;
        const CANCEL        = ffi::common::IN_CANCEL;
        const LEFT          = ffi::common::IN_LEFT;
        const RIGHT         = ffi::common::IN_RIGHT;
        const MOVELEFT      = ffi::common::IN_MOVELEFT;
        const MOVERIGHT     = ffi::common::IN_MOVERIGHT;
        const ATTACK2       = ffi::common::IN_ATTACK2;
        const RUN           = ffi::common::IN_RUN;
        const RELOAD        = ffi::common::IN_RELOAD;
        const ALT1          = ffi::common::IN_ALT1;
        // Used by client for when scoreboard is held down.
        const SCORE         = ffi::common::IN_SCORE;
    }
}

macro_rules! impl_buttons_is {
    ($( fn $meth:ident = $name:ident ),* $(,)?) => {
        $(
            /// Returns true if
            #[doc = concat!("[", stringify!($name), "](Self::", stringify!($name), ")")]
            /// is pressed.
            pub fn $meth(&self) -> bool {
                self.intersects(Self::$name)
            }
        )*
    };
}

impl Buttons {
    impl_buttons_is! {
        fn is_attack        = ATTACK,
        fn is_jump          = JUMP,
        fn is_duck          = DUCK,
        fn is_forward       = FORWARD,
        fn is_back          = BACK,
        fn is_use           = USE,
        fn is_cancel        = CANCEL,
        fn is_left          = LEFT,
        fn is_right         = RIGHT,
        fn is_move_left     = MOVELEFT,
        fn is_move_right    = MOVERIGHT,
        fn is_attack2       = ATTACK2,
        fn is_run           = RUN,
        fn is_reload        = RELOAD,
        fn is_alt1          = ALT1,
        fn is_score         = SCORE,
    }
}
