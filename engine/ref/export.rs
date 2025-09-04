use core::{
    ffi::{c_char, c_int, c_uchar, c_uint, c_void},
    marker::PhantomData,
};

use shared::raw::{
    byte, cl_entity_s, colorVec, decal_s, decallist_s, lightstyle_t, model_s, msurface_s,
    particle_s, qboolean, ref_viewpass_s, vec3_t, TextureFlags, BEAM, TRICULLSTYLE,
};

use crate::{
    engine::RefEngineFunctions,
    globals::RefGlobalsRaw,
    raw::{mstudioseqdesc_t, mstudiotex_s, ref_screen_rotation_t, rgbdata_t},
};

pub use shared::export::{impl_unsync_global, UnsyncGlobal};

#[allow(unused_variables)]
pub trait RefDll: UnsyncGlobal {
    // TODO:
}

#[allow(non_camel_case_types)]
pub type ref_interface_s = RefDllFunctions;

#[allow(non_snake_case)]
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct RefDllFunctions {
    pub R_Init: Option<unsafe extern "C" fn() -> qboolean>,
    pub R_Shutdown: Option<unsafe extern "C" fn()>,
    pub R_GetConfigName: Option<unsafe extern "C" fn() -> *const c_char>,
    pub R_SetDisplayTransform: Option<
        unsafe extern "C" fn(
            rotate: ref_screen_rotation_t,
            x: c_int,
            y: c_int,
            scale_x: f32,
            scale_y: f32,
        ) -> qboolean,
    >,
    pub GL_SetupAttributes: Option<unsafe extern "C" fn(safegl: c_int)>,
    pub GL_InitExtensions: Option<unsafe extern "C" fn()>,
    pub GL_ClearExtensions: Option<unsafe extern "C" fn()>,
    pub R_GammaChanged: Option<unsafe extern "C" fn(do_reset_gamma: qboolean)>,
    pub R_BeginFrame: Option<unsafe extern "C" fn(clearScene: qboolean)>,
    pub R_RenderScene: Option<unsafe extern "C" fn()>,
    pub R_EndFrame: Option<unsafe extern "C" fn()>,
    pub R_PushScene: Option<unsafe extern "C" fn()>,
    pub R_PopScene: Option<unsafe extern "C" fn()>,
    pub GL_BackendStartFrame: Option<unsafe extern "C" fn()>,
    pub GL_BackendEndFrame: Option<unsafe extern "C" fn()>,
    pub R_ClearScreen: Option<unsafe extern "C" fn()>,
    pub R_AllowFog: Option<unsafe extern "C" fn(allow: qboolean)>,
    pub GL_SetRenderMode: Option<unsafe extern "C" fn(renderMode: c_int)>,
    pub R_AddEntity:
        Option<unsafe extern "C" fn(clent: *mut cl_entity_s, type_: c_int) -> qboolean>,
    pub CL_AddCustomBeam: Option<unsafe extern "C" fn(pEnvBeam: *mut cl_entity_s)>,
    pub R_ProcessEntData: Option<
        unsafe extern "C" fn(allocate: qboolean, entities: *mut cl_entity_s, max_entities: c_uint),
    >,
    pub R_Flush: Option<unsafe extern "C" fn(flush_flags: c_uint)>,
    pub R_ShowTextures: Option<unsafe extern "C" fn()>,
    pub R_GetTextureOriginalBuffer: Option<unsafe extern "C" fn(idx: c_uint) -> *const byte>,
    pub GL_LoadTextureFromBuffer: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            pic: *mut rgbdata_t,
            flags: TextureFlags,
            update: qboolean,
        ) -> c_int,
    >,
    pub GL_ProcessTexture: Option<
        unsafe extern "C" fn(texnum: c_int, gamma: f32, topColor: c_int, bottomColor: c_int),
    >,
    pub R_SetupSky: Option<unsafe extern "C" fn(skyboxTextures: *mut c_int)>,
    pub R_Set2DMode: Option<unsafe extern "C" fn(enable: qboolean)>,
    pub R_DrawStretchRaw: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            cols: c_int,
            rows: c_int,
            data: *const byte,
            dirty: qboolean,
        ),
    >,
    pub R_DrawStretchPic: Option<
        unsafe extern "C" fn(
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            s1: f32,
            t1: f32,
            s2: f32,
            t2: f32,
            texnum: c_int,
        ),
    >,
    pub FillRGBA: Option<
        unsafe extern "C" fn(
            rendermode: c_int,
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            r: byte,
            g: byte,
            b: byte,
            a: byte,
        ),
    >,
    pub WorldToScreen:
        Option<unsafe extern "C" fn(world: *const vec3_t, screen: *mut vec3_t) -> c_int>,
    pub VID_ScreenShot:
        Option<unsafe extern "C" fn(filename: *const c_char, shot_type: c_int) -> qboolean>,
    pub VID_CubemapShot: Option<
        unsafe extern "C" fn(
            base: *const c_char,
            size: c_uint,
            vieworg: *const f32,
            skyshot: qboolean,
        ) -> qboolean,
    >,
    pub R_LightPoint: Option<unsafe extern "C" fn(p: *const f32) -> colorVec>,
    pub R_DecalShoot: Option<
        unsafe extern "C" fn(
            textureIndex: c_int,
            entityIndex: c_int,
            modelIndex: c_int,
            pos: *mut vec3_t,
            flags: c_int,
            scale: f32,
        ),
    >,
    pub R_DecalRemoveAll: Option<unsafe extern "C" fn(texture: c_int)>,
    pub R_CreateDecalList: Option<unsafe extern "C" fn(pList: *mut decallist_s) -> c_int>,
    pub R_ClearAllDecals: Option<unsafe extern "C" fn()>,
    pub R_StudioEstimateFrame: Option<
        unsafe extern "C" fn(
            e: *mut cl_entity_s,
            pseqdesc: *mut mstudioseqdesc_t,
            time: f64,
        ) -> f32,
    >,
    pub R_StudioLerpMovement: Option<
        unsafe extern "C" fn(
            e: *mut cl_entity_s,
            time: f64,
            origin: *mut vec3_t,
            angles: *mut vec3_t,
        ),
    >,
    pub CL_InitStudioAPI: Option<unsafe extern "C" fn()>,
    pub R_SetSkyCloudsTextures:
        Option<unsafe extern "C" fn(solidskyTexture: c_int, alphaskyTexture: c_int)>,
    pub GL_SubdivideSurface: Option<unsafe extern "C" fn(mod_: *mut model_s, fa: *mut msurface_s)>,
    pub CL_RunLightStyles: Option<unsafe extern "C" fn(ls: *mut lightstyle_t)>,
    pub R_GetSpriteParms: Option<
        unsafe extern "C" fn(
            frameWidth: *mut c_int,
            frameHeight: *mut c_int,
            numFrames: *mut c_int,
            currentFrame: c_int,
            pSprite: *const model_s,
        ),
    >,
    pub R_GetSpriteTexture:
        Option<unsafe extern "C" fn(m_pSpriteModel: *const model_s, frame: c_int) -> c_int>,
    pub Mod_ProcessRenderData: Option<
        unsafe extern "C" fn(mod_: *mut model_s, create: qboolean, buffer: *const byte) -> qboolean,
    >,
    pub Mod_StudioLoadTextures: Option<unsafe extern "C" fn(mod_: *mut model_s, data: *mut c_void)>,
    pub CL_DrawParticles:
        Option<unsafe extern "C" fn(frametime: f64, particles: *mut particle_s, partsize: f32)>,
    pub CL_DrawTracers: Option<unsafe extern "C" fn(frametime: f64, tracers: *mut particle_s)>,
    pub CL_DrawBeams: Option<unsafe extern "C" fn(fTrans: c_int, beams: *mut BEAM)>,
    pub R_BeamCull: Option<
        unsafe extern "C" fn(
            start: *const vec3_t,
            end: *const vec3_t,
            pvsOnly: qboolean,
        ) -> qboolean,
    >,
    pub RefGetParm: Option<unsafe extern "C" fn(parm: c_int, arg: c_int) -> c_int>,
    pub GetDetailScaleForTexture:
        Option<unsafe extern "C" fn(texture: c_int, xScale: *mut f32, yScale: *mut f32)>,
    pub GetExtraParmsForTexture: Option<
        unsafe extern "C" fn(
            texture: c_int,
            red: *mut byte,
            green: *mut byte,
            blue: *mut byte,
            alpha: *mut byte,
        ),
    >,
    pub GetFrameTime: Option<unsafe extern "C" fn() -> f32>,
    pub R_SetCurrentEntity: Option<unsafe extern "C" fn(ent: *mut cl_entity_s)>,
    pub R_SetCurrentModel: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    pub GL_FindTexture: Option<unsafe extern "C" fn(name: *const c_char) -> c_int>,
    pub GL_TextureName: Option<unsafe extern "C" fn(texnum: c_uint) -> *const c_char>,
    pub GL_TextureData: Option<unsafe extern "C" fn(texnum: c_uint) -> *const byte>,
    pub GL_LoadTexture: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            buf: *const byte,
            size: usize,
            flags: c_int,
        ) -> c_int,
    >,
    pub GL_CreateTexture: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            width: c_int,
            height: c_int,
            buffer: *const c_void,
            flags: TextureFlags,
        ) -> c_int,
    >,
    pub GL_LoadTextureArray:
        Option<unsafe extern "C" fn(names: *mut *const c_char, flags: c_int) -> c_int>,
    pub GL_CreateTextureArray: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            width: c_int,
            height: c_int,
            depth: c_int,
            buffer: *const c_void,
            flags: TextureFlags,
        ) -> c_int,
    >,
    pub GL_FreeTexture: Option<unsafe extern "C" fn(texnum: c_uint)>,
    pub R_OverrideTextureSourceSize:
        Option<unsafe extern "C" fn(texnum: c_uint, srcWidth: c_uint, srcHeight: c_uint)>,
    pub DrawSingleDecal: Option<unsafe extern "C" fn(pDecal: *mut decal_s, fa: *mut msurface_s)>,
    pub R_DecalSetupVerts: Option<
        unsafe extern "C" fn(
            pDecal: *mut decal_s,
            surf: *mut msurface_s,
            texture: c_int,
            outCount: *mut c_int,
        ) -> *mut f32,
    >,
    pub R_EntityRemoveDecals: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    pub AVI_UploadRawFrame: Option<
        unsafe extern "C" fn(
            texture: c_int,
            cols: c_int,
            rows: c_int,
            width: c_int,
            height: c_int,
            data: *const byte,
        ),
    >,
    pub GL_Bind: Option<unsafe extern "C" fn(tmu: c_int, texnum: c_uint)>,
    pub GL_SelectTexture: Option<unsafe extern "C" fn(tmu: c_int)>,
    pub GL_LoadTextureMatrix: Option<unsafe extern "C" fn(glmatrix: *const f32)>,
    pub GL_TexMatrixIdentity: Option<unsafe extern "C" fn()>,
    pub GL_CleanUpTextureUnits: Option<unsafe extern "C" fn(last: c_int)>,
    pub GL_TexGen: Option<unsafe extern "C" fn(coord: c_uint, mode: c_uint)>,
    pub GL_TextureTarget: Option<unsafe extern "C" fn(target: c_uint)>,
    pub GL_TexCoordArrayMode: Option<unsafe extern "C" fn(texmode: c_uint)>,
    pub GL_UpdateTexSize:
        Option<unsafe extern "C" fn(texnum: c_int, width: c_int, height: c_int, depth: c_int)>,
    pub GL_Reserved0: Option<unsafe extern "C" fn()>,
    pub GL_Reserved1: Option<unsafe extern "C" fn()>,
    pub GL_DrawParticles: Option<
        unsafe extern "C" fn(rvp: *const ref_viewpass_s, trans_pass: qboolean, frametime: f32),
    >,
    pub LightVec: Option<
        unsafe extern "C" fn(
            start: *const f32,
            end: *const f32,
            lightspot: *mut f32,
            lightvec: *mut f32,
        ) -> colorVec,
    >,
    pub StudioGetTexture: Option<unsafe extern "C" fn(e: *mut cl_entity_s) -> *mut mstudiotex_s>,
    pub GL_RenderFrame: Option<unsafe extern "C" fn(rvp: *const ref_viewpass_s)>,
    pub GL_OrthoBounds: Option<unsafe extern "C" fn(mins: *const f32, maxs: *const f32)>,
    pub R_SpeedsMessage: Option<unsafe extern "C" fn(out: *mut c_char, size: usize) -> qboolean>,
    pub Mod_GetCurrentVis: Option<unsafe extern "C" fn() -> *mut byte>,
    pub R_NewMap: Option<unsafe extern "C" fn()>,
    pub R_ClearScene: Option<unsafe extern "C" fn()>,
    pub R_GetProcAddress: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,
    pub TriRenderMode: Option<unsafe extern "C" fn(mode: c_int)>,
    pub Begin: Option<unsafe extern "C" fn(primitiveCode: c_int)>,
    pub End: Option<unsafe extern "C" fn()>,
    pub Color4f: Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32)>,
    pub Color4ub: Option<unsafe extern "C" fn(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar)>,
    pub TexCoord2f: Option<unsafe extern "C" fn(u: f32, v: f32)>,
    pub Vertex3fv: Option<unsafe extern "C" fn(worldPnt: *const f32)>,
    pub Vertex3f: Option<unsafe extern "C" fn(x: f32, y: f32, z: f32)>,
    pub Fog: Option<
        unsafe extern "C" fn(flFogColor: *mut [f32; 3], flStart: f32, flEnd: f32, bOn: c_int),
    >,
    pub ScreenToWorld: Option<unsafe extern "C" fn(screen: *const f32, world: *mut f32)>,
    pub GetMatrix: Option<unsafe extern "C" fn(pname: c_int, matrix: *mut f32)>,
    pub FogParams: Option<unsafe extern "C" fn(flDensity: f32, iFogSkybox: c_int)>,
    pub CullFace: Option<unsafe extern "C" fn(mode: TRICULLSTYLE)>,
    pub VGUI_SetupDrawing: Option<unsafe extern "C" fn(rect: qboolean)>,
    pub VGUI_UploadTextureBlock: Option<
        unsafe extern "C" fn(
            drawX: c_int,
            drawY: c_int,
            rgba: *const byte,
            blockWidth: c_int,
            blockHeight: c_int,
        ),
    >,
}

impl RefDllFunctions {
    pub const VERSION: c_int = 10;

    pub fn new<T: RefDll + Default>() -> Self {
        Export::<T>::ref_functions()
    }
}

#[allow(clippy::missing_safety_doc)]
trait RefDllExport {
    fn ref_functions() -> RefDllFunctions {
        todo!()
    }
}

struct Export<T> {
    phantom: PhantomData<T>,
}

impl<T: RefDll + Default> RefDllExport for Export<T> {
    // TODO:
}

/// Initialize the global engine instance and returns exported functions.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn get_ref_api<T: RefDll + Default>(
    version: c_int,
    dll_funcs: Option<&mut RefDllFunctions>,
    eng_funcs: Option<&RefEngineFunctions>,
    globals: *mut RefGlobalsRaw,
) -> c_int {
    if version != RefDllFunctions::VERSION {
        return 0;
    }
    let Some(dll_funcs) = dll_funcs else { return 0 };
    let Some(eng_funcs) = eng_funcs else { return 0 };
    unsafe {
        crate::instance::init_engine(eng_funcs, globals);
    }
    *dll_funcs = RefDllFunctions::new::<T>();
    RefDllFunctions::VERSION
}

// pub const GET_REF_API_FN: &CStr = c"GetRefAPI";

pub type GetRefApiFn = Option<
    unsafe extern "C" fn(
        version: c_int,
        dll_funcs: &mut RefDllFunctions,
        eng_funcs: &RefEngineFunctions,
        globals: *mut RefGlobalsRaw,
    ) -> c_int,
>;

#[doc(hidden)]
#[macro_export]
macro_rules! export_dll {
    ($ref_dll:ty $($init:block)?) => {
        #[no_mangle]
        unsafe extern "C" fn GetRefAPI(
            version: core::ffi::c_int,
            dll_funcs: Option<&mut $crate::export::RefDllFunctions>,
            eng_funcs: Option<&$crate::engine::RefEngineFunctions>,
            globals: *mut $crate::globals::RefGlobalsRaw,
        ) -> core::ffi::c_int {
            let result = unsafe {
                $crate::export::get_ref_api::<$ref_dll>(dll_funcs, eng_funcs, globals)
            };
            if result != 0 {
                $($init)?
            }
            result
        }
    };
}
#[doc(inline)]
pub use export_dll;
