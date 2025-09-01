use core::{
    ffi::{c_char, c_int, c_void},
    ptr,
};

use shared::str::{AsCStrPtr, ToEngineStr};

use crate::{
    consts::RefParm,
    cvar::cvar_s,
    engine_types::*,
    raw::{
        self, bsp::MAX_MAP_LEAFS_BYTES, convar_s, ilFlags_t, mleaf_s, mnode_s, rgbdata_t, vec3_t,
        GraphicApi, ImageFlags,
    },
};

pub use shared::engine::AddCmdError;

pub(crate) mod prelude {
    pub use shared::engine::{
        EngineCmd, EngineCmdArgsRaw, EngineConsole, EngineCvar, EngineRng, EngineSystemTime,
    };
}

pub use self::prelude::*;

pub struct RefEngine {
    raw: raw::ref_api_s,
}

shared::export::impl_unsync_global!(RefEngine);

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("ref_api_s.{} is null", stringify!($name)),
        }
    };
}

impl RefEngine {
    pub(crate) fn new(raw: &raw::ref_api_s) -> Self {
        Self { raw: *raw }
    }

    pub fn raw(&self) -> &raw::ref_api_s {
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

    pub fn get_cvar_ptr(&self, name: impl ToEngineStr, ignore_flags: c_int) -> *mut cvar_s {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnGetCvarPointer)(name.as_ptr(), ignore_flags) }
    }

    pub fn cvar_register(&self, var: &'static mut convar_s) {
        unsafe { unwrap!(self, Cvar_RegisterVariable)(var) }
    }

    // pub Cvar_FullSet:
    //     Option<unsafe extern "C" fn(var_name: *const c_char, value: *const c_char, flags: c_int)>,

    pub fn add_command_with_desc(
        &self,
        name: impl ToEngineStr,
        func: unsafe extern "C" fn(),
        desc: impl ToEngineStr,
    ) -> Result<(), AddCmdError> {
        let name = name.to_engine_str();
        let desc = desc.to_engine_str();
        let result = unsafe { unwrap!(self, Cmd_AddCommand)(name.as_ptr(), func, desc.as_ptr()) };
        if result == 0 {
            Err(AddCmdError)
        } else {
            Ok(())
        }
    }

    // pub Cmd_RemoveCommand: Option<unsafe extern "C" fn(cmd_name: *const c_char)>,

    // pub Cbuf_AddText: Option<unsafe extern "C" fn(commands: *const c_char)>,
    // pub Cbuf_InsertText: Option<unsafe extern "C" fn(commands: *const c_char)>,
    // pub Cbuf_Execute: Option<unsafe extern "C" fn()>,
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

    ///
    ///
    /// # Safety
    ///
    /// `node` must not be null.
    pub unsafe fn mod_point_in_leaf(&self, point: vec3_t, node: *mut mnode_s) -> *mut mleaf_s {
        unsafe { unwrap!(self, Mod_PointInLeaf)(&point, node) }
    }

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

    pub fn r_store_efrags(&self, efrags: &mut *mut raw::efrag_s, framecount: c_int) {
        unsafe { unwrap!(self, R_StoreEfrags)(efrags, framecount) }
    }

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

    pub fn cl_extra_update(&self) {
        unsafe { unwrap!(self, CL_ExtraUpdate)() }
    }

    // pub Host_Error: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,

    pub fn set_random_seed(&self, seed: c_int) {
        unsafe { unwrap!(self, COM_SetRandomSeed)(seed) }
    }

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

    pub(crate) unsafe fn sw_lock_buffer(&self) -> *mut c_void {
        unsafe { unwrap!(self, SW_LockBuffer)() }
    }

    pub(crate) unsafe fn sw_unlock_buffer(&self) {
        unsafe { unwrap!(self, SW_UnlockBuffer)() }
    }

    /// Calculates a Potentially Visible Sets (PVS) within radius of pixels of the given point.
    ///
    /// Returns an error if length of `buffer` is less than [MAX_MAP_LEAFS_BYTES].
    pub fn r_fat_pvs<'a>(
        &self,
        org: vec3_t,
        radius: f32,
        buffer: &'a mut [u8],
        merge: bool,
        fullvis: bool,
    ) -> Result<&'a mut [u8], FatPvsError> {
        if buffer.len() < MAX_MAP_LEAFS_BYTES {
            return Err(FatPvsError(()));
        }
        let bytes = unsafe {
            unwrap!(self, R_FatPVS)(
                org.as_ptr(),
                radius,
                buffer.as_mut_ptr(),
                merge.into(),
                fullvis.into(),
            )
        };
        Ok(&mut buffer[..bytes as usize])
    }

    pub fn get_overview_parms(&self) -> &raw::ref_overview_s {
        let ret = unsafe { unwrap!(self, GetOverviewParms)() };
        assert!(!ret.is_null());
        unsafe { &*ret }
    }

    #[deprecated(note = "use EngineSystemTime::system_time_f64 instead")]
    pub fn time(&self) -> f64 {
        self.system_time_f64()
    }

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

    pub(crate) unsafe fn fs_copy_image(&self, src: *mut rgbdata_t) -> *mut rgbdata_t {
        unsafe { unwrap!(self, FS_CopyImage)(src) }
    }

    pub(crate) unsafe fn fs_free_image(&self, pack: *mut rgbdata_t) {
        unsafe { unwrap!(self, FS_FreeImage)(pack) }
    }

    // pub Image_SetMDLPointer: Option<unsafe extern "C" fn(p: *mut byte)>,
    // pub Image_GetPFDesc: Option<unsafe extern "C" fn(idx: c_int) -> *const bpc_desc_s>,
    // pub pfnDrawNormalTriangles: Option<unsafe extern "C" fn()>,
    // pub pfnDrawTransparentTriangles: Option<unsafe extern "C" fn()>,

    pub fn draw(&self) -> Draw<'_> {
        debug_assert!(!self.raw.drawFuncs.is_null());
        Draw::new(unsafe { &*self.raw.drawFuncs })
    }

    // pub fsapi: *mut fs_api_t,
}

impl EngineCvar for RefEngine {
    fn fn_get_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char) -> f32 {
        unwrap!(self, pfnGetCvarFloat)
    }

    fn fn_set_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char, value: f32) {
        unwrap!(self, Cvar_SetValue)
    }

    fn fn_get_cvar_string(&self) -> unsafe extern "C" fn(name: *const c_char) -> *const c_char {
        unwrap!(self, pfnGetCvarString)
    }

    fn fn_set_cvar_string(
        &self,
    ) -> unsafe extern "C" fn(name: *const c_char, value: *const c_char) {
        unwrap!(self, Cvar_Set)
    }
}

impl EngineRng for RefEngine {
    fn fn_random_float(&self) -> unsafe extern "C" fn(min: f32, max: f32) -> f32 {
        unwrap!(self, COM_RandomFloat)
    }

    fn fn_random_int(&self) -> unsafe extern "C" fn(min: c_int, max: c_int) -> c_int {
        unwrap!(self, COM_RandomLong)
    }
}

impl EngineConsole for RefEngine {
    fn console_print(&self, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe { unwrap!(self, Con_Printf)(c"%s".as_ptr(), msg.as_ptr()) }
    }
}

impl EngineCmd for RefEngine {
    fn fn_cmd_argc(&self) -> unsafe extern "C" fn() -> c_int {
        unwrap!(self, Cmd_Argc)
    }

    fn fn_cmd_argv(&self) -> unsafe extern "C" fn(argc: c_int) -> *const c_char {
        unwrap!(self, Cmd_Argv)
    }

    fn add_command(
        &self,
        name: impl ToEngineStr,
        func: unsafe extern "C" fn(),
    ) -> Result<(), AddCmdError> {
        self.add_command_with_desc(name, func, c"ref command")
    }
}

impl EngineCmdArgsRaw for RefEngine {
    fn fn_cmd_args_raw(&self) -> unsafe extern "C" fn() -> *const c_char {
        unwrap!(self, Cmd_Args)
    }
}

impl EngineSystemTime for RefEngine {
    fn system_time_f64(&self) -> f64 {
        unsafe { unwrap!(self, pfnTime)() }
    }
}
