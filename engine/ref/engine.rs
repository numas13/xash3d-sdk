use core::{
    cell::{Ref, RefCell, RefMut},
    ffi::{c_char, c_int, c_void},
    fmt::{self, Write},
    ops::{Deref, DerefMut},
    ptr, slice,
};

use csz::{CStrArray, CStrThin};
use shared::{
    consts::RefParm,
    raw::{cl_entity_s, decallist_s, model_s, ref_viewpass_s, vec2_t},
};
use utils::str::{AsPtr, ToEngineStr};

use crate::{
    cell::SyncOnceCell,
    raw::{
        convar_s, ilFlags_t, ref_api_s, ref_globals_s, render_interface_t, rgbdata_t, GraphicApi,
        ImageFlags,
    },
};

pub enum Renderer {
    Engine,
    Client,
}

pub struct Draw<'a> {
    raw: &'a render_interface_t,
}

impl Draw<'_> {
    fn new(raw: &render_interface_t) -> Draw {
        Draw { raw }
    }

    pub fn version(&self) -> c_int {
        self.raw.version
    }

    pub fn gl_render_frame(&self, rvp: &ref_viewpass_s) -> Option<Renderer> {
        self.raw.GL_RenderFrame.map(|f| match unsafe { f(rvp) } {
            0 => Renderer::Engine,
            1 => Renderer::Client,
            n => {
                error!("expected GL_RenderFrame result {n}");
                Renderer::Engine
            }
        })
    }

    pub fn gl_build_lightmaps(&self) -> Option<()> {
        self.raw.GL_BuildLightmaps.map(|f| unsafe { f() })
    }

    pub fn gl_ortho_bounds(&self, mins: vec2_t, maxs: vec2_t) -> Option<()> {
        self.raw
            .GL_OrthoBounds
            .map(|f| unsafe { f(mins.as_ptr(), maxs.as_ptr()) })
    }

    pub fn r_create_studio_decal_list(&self, list: &mut [decallist_s]) -> Option<usize> {
        self.raw
            .R_CreateStudioDecalList
            .map(|f| unsafe { f(list.as_mut_ptr(), list.len() as c_int) as usize })
    }

    pub fn r_clear_studio_decals(&self) -> Option<()> {
        self.raw.R_ClearStudioDecals.map(|f| unsafe { f() })
    }

    pub fn r_speeds_message(&self, out: &mut [c_char]) -> Option<bool> {
        self.raw
            .R_SpeedsMessage
            .map(|f| unsafe { f(out.as_mut_ptr(), out.len()).to_bool() })
    }

    // XXX: temporary silence clippy
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn mod_process_user_data(
        &self,
        model: &mut model_s,
        create: bool,
        buffer: *const u8,
    ) -> Option<()> {
        self.raw
            .Mod_ProcessUserData
            .map(|f| unsafe { f(model, create.into(), buffer) })
    }

    pub fn r_process_ent_data(&self, allocate: bool) -> Option<()> {
        self.raw
            .R_ProcessEntData
            .map(|f| unsafe { f(allocate.into()) })
    }

    pub fn mod_get_current_vis(&self) -> Option<*mut u8> {
        self.raw.Mod_GetCurrentVis.map(|f| unsafe { f() })
    }

    pub fn r_new_map(&self) -> Option<()> {
        self.raw.R_NewMap.map(|f| unsafe { f() })
    }

    pub fn r_clear_scene(&self) -> Option<()> {
        self.raw.R_ClearScene.map(|f| unsafe { f() })
    }

    pub fn cl_update_latched_vars(&self, ent: &mut cl_entity_s, reset: bool) -> Option<()> {
        self.raw
            .CL_UpdateLatchedVars
            .map(|f| unsafe { f(ent, reset.into()) })
    }
}

#[derive(Default)]
pub struct SwBuffer {
    width: c_int,
    height: c_int,
    stride: u32,
    bpp: u32,
    r_mask: u32,
    g_mask: u32,
    b_mask: u32,
}

impl SwBuffer {
    pub fn width(&self) -> usize {
        self.width as usize
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }

    pub fn stride(&self) -> usize {
        self.stride as usize
    }

    pub fn bpp(&self) -> usize {
        self.bpp as usize
    }

    pub fn r_mask(&self) -> u32 {
        self.r_mask
    }

    pub fn g_mask(&self) -> u32 {
        self.g_mask
    }

    pub fn b_mask(&self) -> u32 {
        self.b_mask
    }

    pub fn stride_bytes(&self) -> usize {
        self.stride() * self.bpp()
    }

    pub fn row_bytes(&self) -> usize {
        self.width() * self.bpp()
    }

    pub fn len_bytes(&self) -> usize {
        self.stride_bytes() * self.height()
    }

    pub fn is_empty(&self) -> bool {
        self.stride == 0 || self.width == 0 || self.height == 0
    }

    pub fn len(&self) -> usize {
        self.stride() * self.height()
    }

    pub fn lock(&mut self, width: c_int, height: c_int) -> Option<SwBufferLock> {
        let engine = engine();
        let data = unsafe { engine.sw_lock_buffer() };
        if !data.is_null() && width == self.width && height == self.height {
            Some(SwBufferLock { buf: self, data })
        } else {
            None
        }
    }
}

pub struct SwBufferLock<'a> {
    buf: &'a mut SwBuffer,
    data: *mut c_void,
}

impl SwBufferLock<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.cast(), self.len_bytes()) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data.cast(), self.len_bytes()) }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.cast()
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.data.cast()
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        let stride = self.stride_bytes();
        let row_len = self.row_bytes();
        self.as_bytes_mut()
            .chunks_exact_mut(stride)
            .map(move |row| &mut row[..row_len])
    }
}

impl Deref for SwBufferLock<'_> {
    type Target = SwBuffer;

    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

impl DerefMut for SwBufferLock<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf
    }
}

impl Drop for SwBufferLock<'_> {
    fn drop(&mut self) {
        unsafe {
            engine().sw_unlock_buffer();
        }
    }
}

pub struct RgbData {
    raw: *mut rgbdata_t,
}

impl Clone for RgbData {
    fn clone(&self) -> Self {
        let raw = unsafe { engine().fs_copy_image(self.raw) };
        assert!(!raw.is_null());
        Self { raw }
    }
}

impl Drop for RgbData {
    fn drop(&mut self) {
        unsafe {
            engine().fs_free_image(self.raw);
        }
    }
}

impl Deref for RgbData {
    type Target = rgbdata_t;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for RgbData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}

pub struct SaveImageError(());

impl fmt::Display for SaveImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("failed to save an image")
    }
}

impl fmt::Debug for SaveImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SaveImageError").finish()
    }
}

pub struct Engine {
    raw: ref_api_s,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("ref_api_s.{} is null", stringify!($name)),
        }
    };
}

impl Engine {
    pub fn raw(&self) -> &ref_api_s {
        &self.raw
    }

    pub fn get_parm(&self, parm: RefParm, arg: c_int) -> isize {
        unsafe { unwrap!(self, EngineGetParm)(parm.as_raw(), arg) }
    }

    // pub Cvar_Get: Option<
    //     unsafe extern "C" fn(
    //         szName: *const c_char,
    //         szValue: *const c_char,
    //         flags: c_int,
    //         description: *const c_char,
    //     ) -> *mut cvar_s,
    // >,
    // pub pfnGetCvarPointer:
    //     Option<unsafe extern "C" fn(name: *const c_char, ignore_flags: c_int) -> *mut cvar_s>,

    pub fn get_cvar_float(&self, name: impl ToEngineStr) -> f32 {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnGetCvarFloat)(name.as_ptr()) }
    }

    pub fn get_cvar_string(&self, name: impl ToEngineStr) -> &CStrThin {
        let name = name.to_engine_str();
        // FIXME: The lifetime of the returned string is only valid until the cvar is modified.
        let ret = unsafe { unwrap!(self, pfnGetCvarString)(name.as_ptr()) };
        // The engine returns an empty string if cvar is not defined.
        assert!(!ret.is_null());
        unsafe { CStrThin::from_ptr(ret) }
    }

    // pub Cvar_SetValue: Option<unsafe extern "C" fn(name: *const c_char, value: f32)>,
    // pub Cvar_Set: Option<unsafe extern "C" fn(name: *const c_char, value: *const c_char)>,

    pub fn cvar_register(&self, var: &'static mut convar_s) {
        unsafe { unwrap!(self, Cvar_RegisterVariable)(var) }
    }

    // pub Cvar_FullSet:
    //     Option<unsafe extern "C" fn(var_name: *const c_char, value: *const c_char, flags: c_int)>,
    // pub Cmd_AddCommand: Option<
    //     unsafe extern "C" fn(
    //         cmd_name: *const c_char,
    //         function: Option<unsafe extern "C" fn()>,
    //         description: *const c_char,
    //     ) -> c_int,
    // >,
    // pub Cmd_RemoveCommand: Option<unsafe extern "C" fn(cmd_name: *const c_char)>,
    // pub Cmd_Argc: Option<unsafe extern "C" fn() -> c_int>,
    // pub Cmd_Argv: Option<unsafe extern "C" fn(arg: c_int) -> *const c_char>,
    // pub Cmd_Args: Option<unsafe extern "C" fn() -> *const c_char>,
    // pub Cbuf_AddText: Option<unsafe extern "C" fn(commands: *const c_char)>,
    // pub Cbuf_InsertText: Option<unsafe extern "C" fn(commands: *const c_char)>,
    // pub Cbuf_Execute: Option<unsafe extern "C" fn()>,

    pub fn con_print(&self, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, Con_Printf)(c"%s".as_ptr(), msg.as_ptr());
        }
    }

    pub fn con_printf(&self, args: fmt::Arguments) -> fmt::Result {
        let mut buf = CStrArray::<8192>::new();
        buf.cursor().write_fmt(args)?;
        self.con_print(buf.as_thin());
        Ok(())
    }

    // pub Con_DPrintf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    // pub Con_Reportf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    // pub Con_NPrintf: Option<unsafe extern "C" fn(pos: c_int, fmt: *const c_char, ...)>,
    // pub Con_NXPrintf:
    //     Option<unsafe extern "C" fn(info: *mut con_nprint_s, fmt: *const c_char, ...)>,
    // pub CL_CenterPrint: Option<unsafe extern "C" fn(s: *const c_char, y: f32)>,
    // pub Con_DrawStringLen:
    //     Option<unsafe extern "C" fn(pText: *const c_char, length: *mut c_int, height: *mut c_int)>,
    // pub Con_DrawString: Option<
    //     unsafe extern "C" fn(
    //         x: c_int,
    //         y: c_int,
    //         string: *const c_char,
    //         setColor: *const rgba_t,
    //     ) -> c_int,
    // >,
    // pub CL_DrawCenterPrint: Option<unsafe extern "C" fn()>,
    // pub R_BeamGetEntity: Option<unsafe extern "C" fn(index: c_int) -> *mut cl_entity_s>,
    // pub CL_GetWaterEntity: Option<unsafe extern "C" fn(p: *const vec3_t) -> *mut cl_entity_s>,
    // pub CL_AddVisibleEntity:
    //     Option<unsafe extern "C" fn(ent: *mut cl_entity_s, entityType: c_int) -> qboolean>,
    // pub Mod_SampleSizeForFace: Option<unsafe extern "C" fn(surf: *const msurface_s) -> c_int>,
    // pub Mod_BoxVisible: Option<
    //     unsafe extern "C" fn(
    //         mins: *const vec3_t,
    //         maxs: *const vec3_t,
    //         visbits: *const byte,
    //     ) -> qboolean,
    // >,
    // pub Mod_PointInLeaf:
    //     Option<unsafe extern "C" fn(p: *const vec3_t, node: *mut mnode_s) -> *mut mleaf_s>,
    // pub R_DrawWorldHull: Option<unsafe extern "C" fn()>,
    // pub R_DrawModelHull: Option<unsafe extern "C" fn(mod_: *mut model_s)>,
    // pub R_StudioGetAnim: Option<
    //     unsafe extern "C" fn(
    //         m_pStudioHeader: *mut studiohdr_s,
    //         m_pSubModel: *mut model_s,
    //         pseqdesc: *mut mstudioseqdesc_t,
    //     ) -> *mut c_void,
    // >,
    // pub pfnStudioEvent:
    //     Option<unsafe extern "C" fn(event: *const mstudioevent_s, entity: *const cl_entity_s)>,
    // pub CL_DrawEFX: Option<unsafe extern "C" fn(time: f32, fTrans: qboolean)>,
    // pub CL_ThinkParticle: Option<unsafe extern "C" fn(frametime: f64, p: *mut particle_s)>,
    // pub R_FreeDeadParticles: Option<unsafe extern "C" fn(ppparticles: *mut *mut particle_s)>,
    // pub CL_AllocParticleFast: Option<unsafe extern "C" fn() -> *mut particle_s>,
    // pub CL_AllocElight: Option<unsafe extern "C" fn(key: c_int) -> *mut dlight_s>,
    // pub GetDefaultSprite: Option<unsafe extern "C" fn(spr: ref_defaultsprite_e) -> *mut model_s>,
    // pub R_StoreEfrags: Option<unsafe extern "C" fn(ppefrag: *mut *mut efrag_s, framecount: c_int)>,
    // pub Mod_ForName: Option<
    //     unsafe extern "C" fn(
    //         name: *const c_char,
    //         crash: qboolean,
    //         trackCRC: qboolean,
    //     ) -> *mut model_s,
    // >,
    // pub Mod_Extradata:
    //     Option<unsafe extern "C" fn(type_: c_int, model: *mut model_s) -> *mut c_void>,
    // pub CL_EntitySetRemapColors: Option<
    //     unsafe extern "C" fn(
    //         e: *mut cl_entity_s,
    //         mod_: *mut model_s,
    //         top: c_int,
    //         bottom: c_int,
    //     ) -> qboolean,
    // >,
    // pub CL_GetRemapInfoForEntity:
    //     Option<unsafe extern "C" fn(e: *mut cl_entity_s) -> *mut remap_info_s>,
    // pub CL_ExtraUpdate: Option<unsafe extern "C" fn()>,
    // pub Host_Error: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    // pub COM_SetRandomSeed: Option<unsafe extern "C" fn(lSeed: c_int)>,
    // pub COM_RandomFloat: Option<unsafe extern "C" fn(rmin: f32, rmax: f32) -> f32>,
    // pub COM_RandomLong: Option<unsafe extern "C" fn(rmin: c_int, rmax: c_int) -> c_int>,
    // pub GetScreenFade: Option<unsafe extern "C" fn() -> *mut screenfade_s>,
    // pub CL_GetScreenInfo: Option<unsafe extern "C" fn(width: *mut c_int, height: *mut c_int)>,
    // pub SetLocalLightLevel: Option<unsafe extern "C" fn(level: c_int)>,
    // pub Sys_CheckParm: Option<unsafe extern "C" fn(flag: *const c_char) -> c_int>,
    // pub pfnPlayerInfo: Option<unsafe extern "C" fn(index: c_int) -> *mut player_info_s>,
    // pub pfnGetPlayerState: Option<unsafe extern "C" fn(index: c_int) -> *mut entity_state_s>,
    // pub Mod_CacheCheck: Option<unsafe extern "C" fn(c: *mut cache_user_s) -> *mut c_void>,
    // pub Mod_LoadCacheFile: Option<unsafe extern "C" fn(path: *const c_char, cu: *mut cache_user_s)>,
    // pub Mod_Calloc: Option<unsafe extern "C" fn(number: c_int, size: usize) -> *mut c_void>,
    // pub pfnGetStudioModelInterface: Option<
    //     unsafe extern "C" fn(
    //         version: c_int,
    //         ppinterface: *mut *mut r_studio_interface_s,
    //         pstudio: *mut engine_studio_api_s,
    //     ) -> c_int,
    // >,
    // pub _Mem_AllocPool: Option<
    //     unsafe extern "C" fn(
    //         name: *const c_char,
    //         filename: *const c_char,
    //         fileline: c_int,
    //     ) -> poolhandle_t,
    // >,
    // pub _Mem_FreePool: Option<
    //     unsafe extern "C" fn(poolptr: *mut poolhandle_t, filename: *const c_char, fileline: c_int),
    // >,
    // pub _Mem_Alloc: Option<
    //     unsafe extern "C" fn(
    //         poolptr: poolhandle_t,
    //         size: usize,
    //         clear: qboolean,
    //         filename: *const c_char,
    //         fileline: c_int,
    //     ) -> *mut c_void,
    // >,
    // pub _Mem_Realloc: Option<
    //     unsafe extern "C" fn(
    //         poolptr: poolhandle_t,
    //         memptr: *mut c_void,
    //         size: usize,
    //         clear: qboolean,
    //         filename: *const c_char,
    //         fileline: c_int,
    //     ) -> *mut c_void,
    // >,
    // pub _Mem_Free:
    //     Option<unsafe extern "C" fn(data: *mut c_void, filename: *const c_char, fileline: c_int)>,
    // pub COM_LoadLibrary: Option<
    //     unsafe extern "C" fn(
    //         name: *const c_char,
    //         build_ordinals_table: c_int,
    //         directpath: qboolean,
    //     ) -> *mut c_void,
    // >,
    // pub COM_FreeLibrary: Option<unsafe extern "C" fn(handle: *mut c_void)>,
    // pub COM_GetProcAddress:
    //     Option<unsafe extern "C" fn(handle: *mut c_void, name: *const c_char) -> *mut c_void>,

    pub fn r_init_video(&self, api: GraphicApi) -> bool {
        unsafe { unwrap!(self, R_Init_Video)(api as c_int).to_bool() }
    }

    pub fn r_free_video(&self) {
        unsafe { unwrap!(self, R_Free_Video)() }
    }

    // pub GL_SetAttribute: Option<unsafe extern "C" fn(attr: c_int, value: c_int) -> c_int>,
    // pub GL_GetAttribute: Option<unsafe extern "C" fn(attr: c_int, value: *mut c_int) -> c_int>,
    // pub GL_GetProcAddress: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,
    // pub GL_SwapBuffers: Option<unsafe extern "C" fn()>,

    pub fn sw_create_buffer(&self, width: c_int, height: c_int) -> Option<SwBuffer> {
        let mut buffer = SwBuffer {
            width,
            height,
            ..SwBuffer::default()
        };
        let res = unsafe {
            unwrap!(self, SW_CreateBuffer)(
                width,
                height,
                &mut buffer.stride,
                &mut buffer.bpp,
                &mut buffer.r_mask,
                &mut buffer.g_mask,
                &mut buffer.b_mask,
            )
        };
        res.to_bool().then_some(buffer)
    }

    unsafe fn sw_lock_buffer(&self) -> *mut c_void {
        unsafe { unwrap!(self, SW_LockBuffer)() }
    }

    unsafe fn sw_unlock_buffer(&self) {
        unsafe { unwrap!(self, SW_UnlockBuffer)() }
    }

    // pub R_FatPVS: Option<
    //     unsafe extern "C" fn(
    //         org: *const f32,
    //         radius: f32,
    //         visbuffer: *mut byte,
    //         merge: qboolean,
    //         fullvis: qboolean,
    //     ) -> c_int,
    // >,
    // pub GetOverviewParms: Option<unsafe extern "C" fn() -> *const ref_overview_s>,
    // pub pfnTime: Option<unsafe extern "C" fn() -> f64>,
    // pub EV_GetPhysent: Option<unsafe extern "C" fn(idx: c_int) -> *mut physent_s>,
    // pub EV_TraceSurface: Option<
    //     unsafe extern "C" fn(ground: c_int, vstart: *mut f32, vend: *mut f32) -> *mut msurface_s,
    // >,
    // pub PM_TraceLine: Option<
    //     unsafe extern "C" fn(
    //         start: *mut f32,
    //         end: *mut f32,
    //         flags: c_int,
    //         usehull: c_int,
    //         ignore_pe: c_int,
    //     ) -> *mut pmtrace_s,
    // >,
    // pub EV_VisTraceLine: Option<
    //     unsafe extern "C" fn(start: *mut f32, end: *mut f32, flags: c_int) -> *mut pmtrace_s,
    // >,
    // pub CL_TraceLine: Option<
    //     unsafe extern "C" fn(start: *mut vec3_t, end: *mut vec3_t, flags: c_int) -> pmtrace_s,
    // >,
    // pub Image_AddCmdFlags: Option<unsafe extern "C" fn(flags: c_uint)>,

    pub fn image_set_force_flags(&self, flags: ilFlags_t) {
        unsafe { unwrap!(self, Image_SetForceFlags)(flags.bits()) }
    }

    // pub Image_ClearForceFlags: Option<unsafe extern "C" fn()>,
    // pub Image_CustomPalette: Option<unsafe extern "C" fn() -> qboolean>,

    pub fn image_process(
        &self,
        pic: &mut RgbData,
        width: c_int,
        height: c_int,
        flags: ImageFlags,
    ) -> bool {
        unsafe {
            unwrap!(self, Image_Process)(&mut pic.raw, width, height, flags.bits(), 0.0).into()
        }
    }

    pub fn fs_load_image(
        &self,
        filename: impl ToEngineStr,
        buffer: Option<&[u8]>,
    ) -> Option<RgbData> {
        let filename = filename.to_engine_str();
        let (buffer, size) = buffer
            .map(|i| (i.as_ptr(), i.len()))
            .unwrap_or((ptr::null_mut(), 0));
        let raw = unsafe { unwrap!(self, FS_LoadImage)(filename.as_ptr(), buffer, size) };
        if !raw.is_null() {
            Some(RgbData { raw })
        } else {
            None
        }
    }

    pub fn fs_save_image(
        &self,
        filename: impl ToEngineStr,
        pic: &RgbData,
    ) -> Result<(), SaveImageError> {
        let filename = filename.to_engine_str();
        let res = unsafe { unwrap!(self, FS_SaveImage)(filename.as_ptr(), pic.raw) };
        if res.to_bool() {
            Ok(())
        } else {
            Err(SaveImageError(()))
        }
    }

    unsafe fn fs_copy_image(&self, src: *mut rgbdata_t) -> *mut rgbdata_t {
        unsafe { unwrap!(self, FS_CopyImage)(src) }
    }

    unsafe fn fs_free_image(&self, pack: *mut rgbdata_t) {
        unsafe { unwrap!(self, FS_FreeImage)(pack) }
    }

    // pub Image_SetMDLPointer: Option<unsafe extern "C" fn(p: *mut byte)>,
    // pub Image_GetPFDesc: Option<unsafe extern "C" fn(idx: c_int) -> *const bpc_desc_s>,
    // pub pfnDrawNormalTriangles: Option<unsafe extern "C" fn()>,
    // pub pfnDrawTransparentTriangles: Option<unsafe extern "C" fn()>,

    pub fn draw(&self) -> Draw {
        debug_assert!(!self.raw.drawFuncs.is_null());
        Draw::new(unsafe { &*self.raw.drawFuncs })
    }

    // pub fsapi: *mut fs_api_t,
}

impl From<ref_api_s> for Engine {
    fn from(raw: ref_api_s) -> Self {
        Self { raw }
    }
}

pub struct Globals {
    raw: *mut ref_globals_s,
}

impl Globals {
    fn new(raw: *mut ref_globals_s) -> Self {
        Self { raw }
    }

    pub fn screen_size(&self) -> (c_int, c_int) {
        (self.width, self.height)
    }
}

impl Deref for Globals {
    type Target = ref_globals_s;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for Globals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}

static ENGINE: SyncOnceCell<Engine> = unsafe { SyncOnceCell::new() };
static GLOBALS: SyncOnceCell<RefCell<Globals>> = unsafe { SyncOnceCell::new() };

pub fn engine<'a>() -> &'a Engine {
    ENGINE.get().unwrap()
}

pub fn globals() -> Ref<'static, Globals> {
    GLOBALS.get().unwrap().borrow()
}

pub fn globals_mut() -> RefMut<'static, Globals> {
    GLOBALS.get().unwrap().borrow_mut()
}

pub fn init(engine_funcs: &ref_api_s, globals: *mut ref_globals_s) {
    if ENGINE.set((*engine_funcs).into()).is_err()
        || GLOBALS.set(RefCell::new(Globals::new(globals))).is_err()
    {
        warn!("ref engine initialized multiple times");
    }
}
