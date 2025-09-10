use shared::ffi::{api::studio::engine_studio_api_s, common::cl_entity_s};

pub struct Studio {
    raw: engine_studio_api_s,
}

shared::export::impl_unsync_global!(Studio);

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("engine_studio_api_s.{} is null", stringify!($name)),
        }
    };
}

#[allow(dead_code)]
impl Studio {
    pub(crate) fn new(raw: &engine_studio_api_s) -> Self {
        Self { raw: *raw }
    }

    pub fn raw(&self) -> &engine_studio_api_s {
        &self.raw
    }

    // pub Mem_Calloc: Option<unsafe extern "C" fn(number: c_int, size: usize) -> *mut c_void>,
    // pub Cache_Check: Option<unsafe extern "C" fn(c: *mut cache_user_s) -> *mut c_void>,
    // pub LoadCacheFile: Option<unsafe extern "C" fn(path: *const c_char, cu: *mut cache_user_s)>,
    // pub Mod_ForName:
    //     Option<unsafe extern "C" fn(name: *const c_char, crash_if_missing: c_int) -> *mut model_s>,
    // pub Mod_Extradata: Option<unsafe extern "C" fn(mod_: *mut model_s) -> *mut c_void>,
    // pub GetModelByIndex: Option<unsafe extern "C" fn(index: c_int) -> *mut model_s>,

    pub fn get_current_entity(&self) -> *mut cl_entity_s {
        unsafe { unwrap!(self, GetCurrentEntity)() }
    }

    // pub PlayerInfo: Option<unsafe extern "C" fn(index: c_int) -> *mut player_info_s>,
    // pub GetPlayerState: Option<unsafe extern "C" fn(index: c_int) -> *mut entity_state_s>,
    // pub GetViewEntity: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    // pub GetTimes:
    //     Option<unsafe extern "C" fn(framecount: *mut c_int, current: *mut f64, old: *mut f64)>,
    // pub GetCvar: Option<unsafe extern "C" fn(name: *const c_char) -> *mut cvar_s>,
    // pub GetViewInfo: Option<
    //     unsafe extern "C" fn(origin: *mut f32, upv: *mut f32, rightv: *mut f32, vpnv: *mut f32),
    // >,
    // pub GetChromeSprite: Option<unsafe extern "C" fn() -> *mut model_s>,
    // pub GetModelCounters: Option<unsafe extern "C" fn(s: *mut *mut c_int, a: *mut *mut c_int)>,
    // pub GetAliasScale: Option<unsafe extern "C" fn(x: *mut f32, y: *mut f32)>,
    // pub StudioGetBoneTransform: Option<unsafe extern "C" fn() -> *mut *mut *mut *mut f32>,
    // pub StudioGetLightTransform: Option<unsafe extern "C" fn() -> *mut *mut *mut *mut f32>,
    // pub StudioGetAliasTransform: Option<unsafe extern "C" fn() -> *mut *mut *mut f32>,
    // pub StudioGetRotationMatrix: Option<unsafe extern "C" fn() -> *mut *mut *mut f32>,
    // pub StudioSetupModel: Option<
    //     unsafe extern "C" fn(
    //         bodypart: c_int,
    //         ppbodypart: *mut *mut c_void,
    //         ppsubmodel: *mut *mut c_void,
    //     ),
    // >,
    // pub StudioCheckBBox: Option<unsafe extern "C" fn() -> c_int>,
    // pub StudioDynamicLight:
    //     Option<unsafe extern "C" fn(ent: *mut cl_entity_s, plight: *mut alight_s)>,
    // pub StudioEntityLight: Option<unsafe extern "C" fn(plight: *mut alight_s)>,
    // pub StudioSetupLighting: Option<unsafe extern "C" fn(plighting: *mut alight_s)>,
    // pub StudioDrawPoints: Option<unsafe extern "C" fn()>,
    // pub StudioDrawHulls: Option<unsafe extern "C" fn()>,
    // pub StudioDrawAbsBBox: Option<unsafe extern "C" fn()>,
    // pub StudioDrawBones: Option<unsafe extern "C" fn()>,
    // pub StudioSetupSkin: Option<unsafe extern "C" fn(ptexturehdr: *mut c_void, index: c_int)>,
    // pub StudioSetRemapColors: Option<unsafe extern "C" fn(top: c_int, bottom: c_int)>,
    // pub SetupPlayerModel: Option<unsafe extern "C" fn(index: c_int) -> *mut model_s>,
    // pub StudioClientEvents: Option<unsafe extern "C" fn()>,
    // pub GetForceFaceFlags: Option<unsafe extern "C" fn() -> c_int>,
    // pub SetForceFaceFlags: Option<unsafe extern "C" fn(flags: c_int)>,
    // pub StudioSetHeader: Option<unsafe extern "C" fn(header: *mut c_void)>,
    // pub SetRenderModel: Option<unsafe extern "C" fn(model: *mut model_s)>,
    // pub SetupRenderer: Option<unsafe extern "C" fn(rendermode: c_int)>,
    // pub RestoreRenderer: Option<unsafe extern "C" fn()>,
    // pub SetChromeOrigin: Option<unsafe extern "C" fn()>,

    pub fn is_hardware(&self) -> bool {
        unsafe { unwrap!(self, IsHardware)() != 0 }
    }

    // pub GL_StudioDrawShadow: Option<unsafe extern "C" fn()>,
    // pub GL_SetRenderMode: Option<unsafe extern "C" fn(mode: c_int)>,
    // pub StudioSetRenderamt: Option<unsafe extern "C" fn(iRenderamt: c_int)>,
    // pub StudioSetCullState: Option<unsafe extern "C" fn(iCull: c_int)>,
    // pub StudioRenderShadow: Option<
    //     unsafe extern "C" fn(
    //         iSprite: c_int,
    //         p1: *mut f32,
    //         p2: *mut f32,
    //         p3: *mut f32,
    //         p4: *mut f32,
    //     ),
    // >,
}
