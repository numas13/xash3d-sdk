use xash3d_ffi as ffi;

pub const MAX_PLAYERS: usize = 64;
pub const MAX_TEAMS: usize = 64;

pub const PITCH: usize = ffi::common::PITCH as usize;
pub const YAW: usize = ffi::common::YAW as usize;
pub const ROLL: usize = ffi::common::ROLL as usize;

pub use ffi::common::{
    SOLID_BBOX, SOLID_BSP, SOLID_CUSTOM, SOLID_NOT, SOLID_PORTAL, SOLID_SLIDEBOX, SOLID_TRIGGER,
};

pub use ffi::common::{DEAD_DEAD, DEAD_DISCARDBODY, DEAD_DYING, DEAD_NO, DEAD_RESPAWNABLE};

pub use ffi::common::{DAMAGE_AIM, DAMAGE_NO, DAMAGE_YES};

pub use ffi::common::{EFLAG_FLESH_SOUND, EFLAG_SLERP};

pub use ffi::common::TE_SPRITETRAIL;

pub use ffi::common::{
    CONTENTS_CLIP, CONTENTS_CURRENT_0, CONTENTS_CURRENT_180, CONTENTS_CURRENT_270,
    CONTENTS_CURRENT_90, CONTENTS_CURRENT_DOWN, CONTENTS_CURRENT_UP, CONTENTS_EMPTY,
    CONTENTS_LADDER, CONTENTS_LAVA, CONTENTS_ORIGIN, CONTENTS_SKY, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_TRANSLUCENT, CONTENTS_WATER, CONTENT_FLYFIELD, CONTENT_FOG, CONTENT_GRAVITY_FLYFIELD,
};

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub enum Contents: i32 {
        #[default]
        None(0),
        Empty(ffi::common::CONTENTS_EMPTY),
        Solid(ffi::common::CONTENTS_SOLID),
        Water(ffi::common::CONTENTS_WATER),
        Slime(ffi::common::CONTENTS_SLIME),
        Lava(ffi::common::CONTENTS_LAVA),
        Sky(ffi::common::CONTENTS_SKY),
        Origin(ffi::common::CONTENTS_ORIGIN),
        Clip(ffi::common::CONTENTS_CLIP),
        Current0(ffi::common::CONTENTS_CURRENT_0),
        Current90(ffi::common::CONTENTS_CURRENT_90),
        Current180(ffi::common::CONTENTS_CURRENT_180),
        Current270(ffi::common::CONTENTS_CURRENT_270),
        CurrentUp(ffi::common::CONTENTS_CURRENT_UP),
        CurrentDown(ffi::common::CONTENTS_CURRENT_DOWN),
        Translucent(ffi::common::CONTENTS_TRANSLUCENT),
        Ladder(ffi::common::CONTENTS_LADDER),
        FlyField(ffi::common::CONTENT_FLYFIELD),
        GravityFlyField(ffi::common::CONTENT_GRAVITY_FLYFIELD),
        Fog(ffi::common::CONTENT_FOG),
    }
}

pub use ffi::common::{
    IN_ALT1, IN_ATTACK, IN_ATTACK2, IN_BACK, IN_CANCEL, IN_DUCK, IN_FORWARD, IN_JUMP, IN_LEFT,
    IN_MOVELEFT, IN_MOVERIGHT, IN_RELOAD, IN_RIGHT, IN_RUN, IN_SCORE, IN_USE,
};

pub use ffi::common::{TE_BOUNCE_NULL, TE_BOUNCE_SHELL, TE_BOUNCE_SHOTSHELL};

pub const MAX_STRING: usize = ffi::common::MAX_STRING as usize;
pub const MAX_SYSPATH: usize = ffi::common::MAX_SYSPATH as usize;
pub const MAX_MAP_HULLS: usize = ffi::common::MAX_MAP_HULLS as usize;

pub use ffi::common::{ENTITY_BEAM, ENTITY_NORMAL};

pub const MAX_PHYSENTS: usize = ffi::player_move::MAX_PHYSENTS as usize;
pub const MAX_MOVEENTS: usize = ffi::player_move::MAX_MOVEENTS as usize;
pub const MAX_CLIP_PLANES: usize = ffi::player_move::MAX_CLIP_PLANES as usize;

pub use ffi::player_move::{
    PM_CUSTOM_IGNORE, PM_GLASS_IGNORE, PM_NORMAL, PM_STUDIO_BOX, PM_STUDIO_IGNORE,
    PM_TRACELINE_ANYVISIBLE, PM_TRACELINE_PHYSENTSONLY, PM_WORLD_ONLY,
};
