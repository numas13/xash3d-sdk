pub use xash3d_shared::{ffi, render::*};

macro_rules! define_ref_parm {
    ($($(#[$attr:meta])* const $name:ident = $v:ident;)*) => {
        $($(#[$attr])* pub const $name: RefParm = RefParm::new(ffi::render::$v);)*
    };
}

define_ref_parm! {
    const PARM_DEV_OVERVIEW = ref_parm_e_PARM_DEV_OVERVIEW;
    const PARM_THIRDPERSON = ref_parm_e_PARM_THIRDPERSON;
    const PARM_QUAKE_COMPATIBLE = ref_parm_e_PARM_QUAKE_COMPATIBLE;
    const PARM_GET_CLIENT_PTR = ref_parm_e_PARM_GET_CLIENT_PTR;
    const PARM_GET_HOST_PTR = ref_parm_e_PARM_GET_HOST_PTR;
    const PARM_CONNSTATE = ref_parm_e_PARM_CONNSTATE;
    const PARM_PLAYING_DEMO = ref_parm_e_PARM_PLAYING_DEMO;
    const PARM_WATER_LEVEL = ref_parm_e_PARM_WATER_LEVEL;
    const PARM_GET_WORLD_PTR = ref_parm_e_PARM_GET_WORLD_PTR;
    const PARM_LOCAL_HEALTH = ref_parm_e_PARM_LOCAL_HEALTH;
    const PARM_LOCAL_GAME = ref_parm_e_PARM_LOCAL_GAME;
    const PARM_NUMENTITIES = ref_parm_e_PARM_NUMENTITIES;
    const PARM_GET_MOVEVARS_PTR = ref_parm_e_PARM_GET_MOVEVARS_PTR;
    const PARM_GET_PALETTE_PTR = ref_parm_e_PARM_GET_PALETTE_PTR;
    const PARM_GET_VIEWENT_PTR = ref_parm_e_PARM_GET_VIEWENT_PTR;
    const PARM_GET_TEXGAMMATABLE_PTR = ref_parm_e_PARM_GET_TEXGAMMATABLE_PTR;
    const PARM_GET_LIGHTGAMMATABLE_PTR = ref_parm_e_PARM_GET_LIGHTGAMMATABLE_PTR;
    const PARM_GET_SCREENGAMMATABLE_PTR = ref_parm_e_PARM_GET_SCREENGAMMATABLE_PTR;
    const PARM_GET_LINEARGAMMATABLE_PTR = ref_parm_e_PARM_GET_LINEARGAMMATABLE_PTR;
    const PARM_GET_LIGHTSTYLES_PTR = ref_parm_e_PARM_GET_LIGHTSTYLES_PTR;
    const PARM_GET_DLIGHTS_PTR = ref_parm_e_PARM_GET_DLIGHTS_PTR;
    const PARM_GET_ELIGHTS_PTR = ref_parm_e_PARM_GET_ELIGHTS_PTR;

    /// Returns non-null integer if filtering is enabled for texture.
    ///
    /// Pass -1 to query global filtering settings.
    const PARM_TEX_FILTERING = ref_parm_e_PARM_TEX_FILTERING;
}
