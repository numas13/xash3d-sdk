use core::ffi::c_int;

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

pub use ffi::common::{
    CHAN_AUTO, CHAN_BODY, CHAN_ITEM, CHAN_NETWORKVOICE_BASE, CHAN_NETWORKVOICE_END, CHAN_STATIC,
    CHAN_STREAM, CHAN_VOICE, CHAN_WEAPON,
};

pub use ffi::common::{ATTN_IDLE, ATTN_NONE, ATTN_NORM, ATTN_STATIC};

pub use ffi::common::{PITCH_HIGH, PITCH_LOW, PITCH_NORM};

pub use ffi::common::{
    IN_ALT1, IN_ATTACK, IN_ATTACK2, IN_BACK, IN_CANCEL, IN_DUCK, IN_FORWARD, IN_JUMP, IN_LEFT,
    IN_MOVELEFT, IN_MOVERIGHT, IN_RELOAD, IN_RIGHT, IN_RUN, IN_SCORE, IN_USE,
};

pub use ffi::common::{TE_BOUNCE_NULL, TE_BOUNCE_SHELL, TE_BOUNCE_SHOTSHELL};

pub const MAX_STRING: usize = ffi::common::MAX_STRING as usize;
pub const MAX_SYSPATH: usize = ffi::common::MAX_SYSPATH as usize;
pub const MAX_MAP_HULLS: usize = ffi::common::MAX_MAP_HULLS as usize;

pub use ffi::common::{ENTITY_BEAM, ENTITY_NORMAL};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct RefParm(c_int);

impl RefParm {
    pub const fn new(raw: c_int) -> RefParm {
        Self(raw)
    }

    pub const fn as_raw(&self) -> c_int {
        self.0
    }
}

macro_rules! define_ref_parm {
    ($($name:ident),* $(,)?) => {
        $(pub const $name: RefParm = RefParm::new(ffi::api::render::$name);)*
    };
}

define_ref_parm! {
    PARM_TEX_WIDTH,
    PARM_TEX_HEIGHT,
    PARM_TEX_SRC_WIDTH,
    PARM_TEX_SRC_HEIGHT,
    PARM_TEX_SKYBOX,
    PARM_TEX_SKYTEXNUM,
    PARM_TEX_LIGHTMAP,
    PARM_TEX_TARGET,
    PARM_TEX_TEXNUM,
    PARM_TEX_FLAGS,
    PARM_TEX_DEPTH,
    PARM_TEX_GLFORMAT,
    PARM_TEX_ENCODE,
    PARM_TEX_MIPCOUNT,
    PARM_BSP2_SUPPORTED,
    PARM_SKY_SPHERE,
    PARAM_GAMEPAUSED,
    PARM_MAP_HAS_DELUXE,
    PARM_MAX_ENTITIES,
    PARM_WIDESCREEN,
    PARM_FULLSCREEN,
    PARM_SCREEN_WIDTH,
    PARM_SCREEN_HEIGHT,
    PARM_CLIENT_INGAME,
    PARM_FEATURES,
    PARM_ACTIVE_TMU,
    PARM_LIGHTSTYLEVALUE,
    PARM_MAX_IMAGE_UNITS,
    PARM_CLIENT_ACTIVE,
    PARM_REBUILD_GAMMA,
    PARM_DEDICATED_SERVER,
    PARM_SURF_SAMPLESIZE,
    PARM_GL_CONTEXT_TYPE,
    PARM_GLES_WRAPPER,
    PARM_STENCIL_ACTIVE,
    PARM_WATER_ALPHA,
    PARM_TEX_MEMORY,
    PARM_DELUXEDATA,
    PARM_SHADOWDATA,
}

pub const MAX_PHYSENTS: usize = ffi::player_move::MAX_PHYSENTS as usize;
pub const MAX_MOVEENTS: usize = ffi::player_move::MAX_MOVEENTS as usize;
pub const MAX_CLIP_PLANES: usize = ffi::player_move::MAX_CLIP_PLANES as usize;

pub use ffi::player_move::{
    PM_CUSTOM_IGNORE, PM_GLASS_IGNORE, PM_NORMAL, PM_STUDIO_BOX, PM_STUDIO_IGNORE,
    PM_TRACELINE_ANYVISIBLE, PM_TRACELINE_PHYSENTSONLY, PM_WORLD_ONLY,
};
