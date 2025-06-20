use core::{ffi::c_int, mem::MaybeUninit, ptr};

use csz::CStrThin;

use crate::{
    cell::SyncOnceCell,
    color::{RGB, RGBA},
    cvar::{CVarFlags, CVarPtr},
    efx::EfxApi,
    event::EventApi,
    math::vec3_t,
    raw::{self, cl_entity_s, wrect_s},
    sprite::{SpriteHandle, SpriteList},
    utils::str::{AsPtr, ToEngineStr},
};

pub struct Engine {
    raw: raw::cl_enginefuncs_s,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("cl_enginefuncs_s.{} is null", stringify!($name)),
        }
    };
}

#[allow(dead_code)]
impl Engine {
    pub fn raw(&self) -> &raw::cl_enginefuncs_s {
        &self.raw
    }

    // pub pTriAPI: *mut triangleapi_s,

    pub fn efx_api(&self) -> EfxApi {
        EfxApi::new(unsafe { &*self.raw.pEfxAPI })
    }

    pub fn event_api(&self) -> EventApi {
        EventApi::new(unsafe { &*self.raw.pEventAPI })
    }

    // pub pDemoAPI: *mut demo_api_s,

    // pub pNetAPI: *mut net_api_s,

    pub fn spr_load(&self, pic_name: impl ToEngineStr) -> Option<SpriteHandle> {
        let pic_name = pic_name.to_engine_str();
        let raw = unsafe { unwrap!(self, pfnSPR_Load)(pic_name.as_ptr()) };
        SpriteHandle::new(raw)
    }

    pub fn spr_frames(&self, pic: SpriteHandle) -> c_int {
        unsafe { unwrap!(self, pfnSPR_Frames)(pic.raw()) }
    }

    pub fn spr_height(&self, pic: SpriteHandle, frame: c_int) -> c_int {
        unsafe { unwrap!(self, pfnSPR_Height)(pic.raw(), frame) }
    }

    pub fn spr_width(&self, pic: SpriteHandle, frame: c_int) -> c_int {
        unsafe { unwrap!(self, pfnSPR_Width)(pic.raw(), frame) }
    }

    pub fn spr_size(&self, pic: SpriteHandle, frame: c_int) -> (c_int, c_int) {
        let w = self.spr_width(pic, frame);
        let h = self.spr_height(pic, frame);
        (w, h)
    }

    pub fn spr_set(&self, pic: SpriteHandle, color: RGB) {
        let [r, g, b] = color.into();
        unsafe { unwrap!(self, pfnSPR_Set)(pic.raw(), r, g, b) }
    }

    pub fn spr_draw(&self, frame: c_int, x: c_int, y: c_int) {
        unsafe { unwrap!(self, pfnSPR_Draw)(frame, x, y, None) }
    }

    pub fn spr_draw_rect(&self, frame: c_int, x: c_int, y: c_int, rect: wrect_s) {
        unsafe { unwrap!(self, pfnSPR_Draw)(frame, x, y, Some(&rect)) }
    }

    pub fn spr_draw_holes(&self, frame: c_int, x: c_int, y: c_int) {
        unsafe { unwrap!(self, pfnSPR_DrawHoles)(frame, x, y, None) }
    }

    pub fn spr_draw_holes_rect(&self, frame: c_int, x: c_int, y: c_int, rect: wrect_s) {
        unsafe { unwrap!(self, pfnSPR_DrawHoles)(frame, x, y, Some(&rect)) }
    }

    pub fn spr_draw_additive(&self, frame: c_int, x: c_int, y: c_int) {
        unsafe { unwrap!(self, pfnSPR_DrawAdditive)(frame, x, y, None) }
    }

    pub fn spr_draw_additive_rect(&self, frame: c_int, x: c_int, y: c_int, rect: wrect_s) {
        unsafe { unwrap!(self, pfnSPR_DrawAdditive)(frame, x, y, Some(&rect)) }
    }

    pub fn spr_scissor_enable(&self, x: c_int, y: c_int, width: c_int, height: c_int) {
        unsafe { unwrap!(self, pfnSPR_EnableScissor)(x, y, width, height) }
    }

    pub fn spr_scissor_disable(&self) {
        unsafe { unwrap!(self, pfnSPR_DisableScissor)() }
    }

    pub fn spr_get_list(&self, path: impl ToEngineStr) -> SpriteList {
        let path = path.to_engine_str();
        unsafe {
            let mut len = MaybeUninit::uninit();
            let data = unwrap!(self, pfnSPR_GetList)(path.as_ptr(), len.as_mut_ptr());
            SpriteList::new(data, len.assume_init() as usize)
        }
    }

    pub fn fill_rgba(&self, x: c_int, y: c_int, width: c_int, height: c_int, color: RGBA) {
        unsafe {
            let [r, g, b, a] = color.into();
            unwrap!(self, pfnFillRGBA)(x, y, width, height, r, g, b, a)
        }
    }

    pub fn get_screen_info(&self) -> raw::SCREENINFO {
        unsafe {
            let mut info = raw::SCREENINFO::default();
            assert_eq!(unwrap!(self, pfnGetScreenInfo)(&mut info), 1);
            info
        }
    }

    pub fn unset_crosshair(&self) {
        let rect = wrect_s::default();
        unsafe { unwrap!(self, pfnSetCrosshair)(0, rect, 0, 0, 0) }
    }

    pub fn set_crosshair(&self, sprite: SpriteHandle, rect: wrect_s, color: RGB) {
        let [r, g, b] = color.into();
        unsafe { unwrap!(self, pfnSetCrosshair)(sprite.raw(), rect, r, g, b) }
    }

    pub fn register_variable(
        &self,
        name: impl ToEngineStr,
        value: impl ToEngineStr,
        flags: CVarFlags,
    ) -> Option<CVarPtr> {
        let name = name.to_engine_str();
        let value = value.to_engine_str();
        unsafe {
            let raw =
                unwrap!(self, pfnRegisterVariable)(name.as_ptr(), value.as_ptr(), flags.bits());
            if !raw.is_null() {
                Some(CVarPtr::from_ptr(raw))
            } else {
                None
            }
        }
    }

    pub fn cvar_get_float(&self, name: impl ToEngineStr) -> f32 {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnGetCvarFloat)(name.as_ptr()) }
    }

    pub fn cvar_get_c_str(&self, name: impl ToEngineStr) -> &CStrThin {
        let name = name.to_engine_str();
        let ptr = unsafe { unwrap!(self, pfnGetCvarString)(name.as_ptr()) };

        // SAFETY: the engine returns an empty string if cvar is not found
        unsafe { CStrThin::from_ptr(ptr) }
    }

    pub fn add_command(&self, name: impl ToEngineStr, func: unsafe extern "C" fn()) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnAddCommand)(name.as_ptr(), func) }
    }

    pub fn hook_user_msg(&self, name: impl ToEngineStr, func: raw::pfnUserMsgHook) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnHookUserMsg)(name.as_ptr(), func) }
    }

    pub fn server_cmd(&self, cmd: impl ToEngineStr) -> c_int {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnServerCmd)(cmd.as_ptr()) }
    }

    pub fn client_cmd(&self, cmd: impl ToEngineStr) -> c_int {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnClientCmd)(cmd.as_ptr()) }
    }

    pub fn get_player_info(&self, entity: c_int) -> Option<raw::hud_player_info_s> {
        unsafe {
            let mut info = MaybeUninit::uninit();
            unwrap!(self, pfnGetPlayerInfo)(entity, info.as_mut_ptr());
            let info = info.assume_init();
            if !info.name.is_null() {
                Some(info)
            } else {
                None
            }
        }
    }

    pub fn play_sound_by_name(&self, name: impl ToEngineStr, vol: f32) {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnPlaySoundByName)(name.as_ptr(), vol) }
    }

    // pub pfnPlaySoundByIndex: Option<unsafe extern "C" fn(iSound: c_int, volume: f32)>,
    // pub pfnAngleVectors: Option<
    //     unsafe extern "C" fn(
    //         vecAngles: *const f32,
    //         forward: *mut f32,
    //         right: *mut f32,
    //         up: *mut f32,
    //     ),
    // >,

    pub fn text_message_get(
        &self,
        msg: impl ToEngineStr,
    ) -> Option<&'static raw::client_textmessage_s> {
        let msg = msg.to_engine_str();
        unsafe {
            let ret = unwrap!(self, pfnTextMessageGet)(msg.as_ptr());
            if !ret.is_null() && !(*ret).pMessage.is_null() {
                Some(&*ret)
            } else {
                None
            }
        }
    }

    pub fn draw_character(&self, x: c_int, y: c_int, number: c_int, color: RGB) -> c_int {
        let [r, g, b] = color.into();
        unsafe { unwrap!(self, pfnDrawCharacter)(x, y, number, r, g, b) }
    }

    pub fn draw_console_string(&self, x: c_int, y: c_int, s: impl ToEngineStr) -> c_int {
        let s = s.to_engine_str();
        unsafe { unwrap!(self, pfnDrawConsoleString)(x, y, s.as_ptr()) }
    }

    pub fn draw_set_text_color(&self, r: f32, g: f32, b: f32) {
        unsafe { unwrap!(self, pfnDrawSetTextColor)(r, g, b) }
    }

    pub fn draw_console_string_len(&self, s: impl ToEngineStr) -> (c_int, c_int) {
        let s = s.to_engine_str();
        unsafe {
            let mut w = 0;
            let mut h = 0;
            unwrap!(self, pfnDrawConsoleStringLen)(s.as_ptr(), &mut w, &mut h);
            (w, h)
        }
    }

    pub fn console_print(&self, s: impl ToEngineStr) {
        let s = s.to_engine_str();
        unsafe { unwrap!(self, pfnConsolePrint)(s.as_ptr()) }
    }

    // pub pfnCenterPrint: Option<unsafe extern "C" fn(string: *const c_char)>,

    pub fn get_window_center_x(&self) -> c_int {
        unsafe { unwrap!(self, GetWindowCenterX)() }
    }

    pub fn get_window_center_y(&self) -> c_int {
        unsafe { unwrap!(self, GetWindowCenterY)() }
    }

    pub fn get_window_center(&self) -> (c_int, c_int) {
        (self.get_window_center_x(), self.get_window_center_y())
    }

    pub fn get_view_angles(&self) -> vec3_t {
        unsafe {
            let mut ret = MaybeUninit::uninit();
            unwrap!(self, GetViewAngles)(ret.as_mut_ptr());
            ret.assume_init()
        }
    }

    pub fn set_view_angles(&self, angles: vec3_t) {
        unsafe { unwrap!(self, SetViewAngles)(&angles) }
    }

    pub fn get_max_clients(&self) -> c_int {
        unsafe { unwrap!(self, GetMaxClients)() }
    }

    pub fn cvar_set_value(&self, name: impl ToEngineStr, value: f32) {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, Cvar_SetValue)(name.as_ptr(), value) }
    }

    pub fn cmd_argc(&self) -> usize {
        unsafe { unwrap!(self, Cmd_Argc)() as usize }
    }

    pub fn cmd_argv(&self, n: usize) -> &CStrThin {
        let ptr = unsafe { unwrap!(self, Cmd_Argv)(n as c_int) };

        // SAFETY: the engine returns an empty string if cvar is not found
        unsafe { CStrThin::from_ptr(ptr) }
    }

    // pub Con_Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    // pub Con_DPrintf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    // pub Con_NPrintf: Option<unsafe extern "C" fn(pos: c_int, fmt: *const c_char, ...)>,
    // pub Con_NXPrintf:
    //     Option<unsafe extern "C" fn(info: *mut con_nprint_s, fmt: *const c_char, ...)>,
    // pub PhysInfo_ValueForKey: Option<unsafe extern "C" fn(key: *const c_char) -> *const c_char>,
    // pub ServerInfo_ValueForKey: Option<unsafe extern "C" fn(key: *const c_char) -> *const c_char>,

    pub fn get_client_max_speed(&self) -> f32 {
        unsafe { unwrap!(self, GetClientMaxspeed)() }
    }

    pub fn check_parm(&self, parm: impl ToEngineStr) -> c_int {
        let parm = parm.to_engine_str();
        unsafe { unwrap!(self, CheckParm)(parm.as_ptr(), ptr::null_mut()) }
    }

    pub fn key_event(&self, key: u32, down: bool) {
        unsafe {
            unwrap!(self, Key_Event)(key as c_int, down as c_int);
        }
    }

    pub fn get_mouse_position(&self) -> (c_int, c_int) {
        unsafe {
            let mut x = 0;
            let mut y = 0;
            unwrap!(self, GetMousePosition)(&mut x, &mut y);
            (x, y)
        }
    }

    pub fn is_no_clipping(&self) -> bool {
        unsafe { unwrap!(self, IsNoClipping)() != 0 }
    }

    /// Returns the entity of local player model.
    ///
    /// # SAFETY
    ///
    /// Never returns a null pointer.
    pub fn get_local_player(&self) -> *mut cl_entity_s {
        let ent = unsafe { unwrap!(self, GetLocalPlayer)() };
        assert!(!ent.is_null());
        ent
    }

    /// Returns the entity of weapon model.
    ///
    /// # SAFETY
    ///
    /// Never returns a null pointer.
    pub fn get_view_entity(&self) -> *mut cl_entity_s {
        let ent = unsafe { unwrap!(self, GetViewModel)() };
        assert!(!ent.is_null());
        ent
    }

    pub fn get_entity_by_index(&self, index: c_int) -> *mut cl_entity_s {
        unsafe { unwrap!(self, GetEntityByIndex)(index) }
    }

    pub fn get_client_time(&self) -> f32 {
        unsafe { unwrap!(self, GetClientTime)() }
    }

    pub fn calc_shake(&self) {
        unsafe { unwrap!(self, V_CalcShake)() }
    }

    pub fn apply_shake(&self, origin: &mut vec3_t, angles: &mut vec3_t, factor: f32) {
        unsafe { unwrap!(self, V_ApplyShake)(origin, angles, factor) }
    }

    pub fn pm_point_contents(&self, point: vec3_t) -> (c_int, c_int) {
        unsafe {
            let mut truecont = MaybeUninit::uninit();
            let cont = unwrap!(self, PM_PointContents)(point.as_ptr(), truecont.as_mut_ptr());
            (cont, truecont.assume_init())
        }
    }

    pub fn pm_water_entity(&self, point: vec3_t) -> c_int {
        unsafe { unwrap!(self, PM_WaterEntity)(point.as_ptr()) }
    }

    // pub PM_TraceLine: Option<
    //     unsafe extern "C" fn(
    //         start: *mut f32,
    //         end: *mut f32,
    //         flags: c_int,
    //         usehull: c_int,
    //         ignore_pe: c_int,
    //     ) -> *mut pmtrace_s,
    // >,
    // pub CL_LoadModel:
    //     Option<unsafe extern "C" fn(modelname: *const c_char, index: *mut c_int) -> *mut model_s>,
    // pub CL_CreateVisibleEntity:
    //     Option<unsafe extern "C" fn(type_: c_int, ent: *mut cl_entity_s) -> c_int>,
    // pub GetSpritePointer: Option<unsafe extern "C" fn(hSprite: HSPRITE) -> *const model_s>,
    // pub pfnPlaySoundByNameAtLocation:
    //     Option<unsafe extern "C" fn(szSound: *mut c_char, volume: f32, origin: *mut f32)>,
    // pub pfnPrecacheEvent:
    //     Option<unsafe extern "C" fn(type_: c_int, psz: *const c_char) -> c_ushort>,
    // pub pfnPlaybackEvent: Option<
    //     unsafe extern "C" fn(
    //         flags: c_int,
    //         pInvoker: *const edict_s,
    //         eventindex: c_ushort,
    //         delay: f32,
    //         origin: *mut f32,
    //         angles: *mut f32,
    //         fparam1: f32,
    //         fparam2: f32,
    //         iparam1: c_int,
    //         iparam2: c_int,
    //         bparam1: c_int,
    //         bparam2: c_int,
    //     ),
    // >,
    // pub pfnWeaponAnim: Option<unsafe extern "C" fn(iAnim: c_int, body: c_int)>,

    pub fn random_float(&self, low: f32, high: f32) -> f32 {
        unsafe { unwrap!(self, pfnRandomFloat)(low, high) }
    }

    pub fn random_int(&self, low: c_int, high: c_int) -> c_int {
        unsafe { unwrap!(self, pfnRandomLong)(low, high) }
    }

    pub fn rand(&self) -> c_int {
        self.random_int(0, c_int::MAX)
    }

    pub fn hook_event(
        &self,
        name: impl ToEngineStr,
        event: Option<unsafe extern "C" fn(args: *mut raw::event_args_s)>,
    ) {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnHookEvent)(name.as_ptr(), event) }
    }

    // pub Con_IsVisible: Option<unsafe extern "C" fn() -> c_int>,
    // pub pfnGetGameDirectory: Option<unsafe extern "C" fn() -> *const c_char>,

    pub fn get_cvar(&self, name: impl ToEngineStr) -> CVarPtr {
        let name = name.to_engine_str();
        let ptr = unsafe { unwrap!(self, pfnGetCvarPointer)(name.as_ptr()) };
        CVarPtr::from_ptr(ptr)
    }

    // pub Key_LookupBinding: Option<unsafe extern "C" fn(pBinding: *const c_char) -> *const c_char>,
    // pub pfnGetLevelName: Option<unsafe extern "C" fn() -> *const c_char>,
    // pub pfnGetScreenFade: Option<unsafe extern "C" fn(fade: *mut screenfade_s)>,
    // pub pfnSetScreenFade: Option<unsafe extern "C" fn(fade: *mut screenfade_s)>,
    // pub VGui_GetPanel: Option<unsafe extern "C" fn() -> *mut c_void>,
    // pub VGui_ViewportPaintBackground: Option<unsafe extern "C" fn(extents: *mut [c_int; 4usize])>,
    // pub COM_LoadFile: Option<
    //     unsafe extern "C" fn(path: *const c_char, usehunk: c_int, pLength: *mut c_int) -> *mut byte,
    // >,
    // pub COM_ParseFile:
    //     Option<unsafe extern "C" fn(data: *mut c_char, token: *mut c_char) -> *mut c_char>,
    // pub COM_FreeFile: Option<unsafe extern "C" fn(buffer: *mut c_void)>,
    // pub pVoiceTweak: *mut IVoiceTweak_s,

    pub fn is_spectator_only(&self) -> bool {
        unsafe { unwrap!(self, IsSpectateOnly)() != 0 }
    }

    // pub LoadMapSprite: Option<unsafe extern "C" fn(filename: *const c_char) -> *mut model_s>,
    // pub COM_AddAppDirectoryToSearchPath:
    //     Option<unsafe extern "C" fn(pszBaseDir: *const c_char, appName: *const c_char)>,
    // pub COM_ExpandFilename: Option<
    //     unsafe extern "C" fn(
    //         fileName: *const c_char,
    //         nameOutBuffer: *mut c_char,
    //         nameOutBufferSize: c_int,
    //     ) -> c_int,
    // >,
    // pub PlayerInfo_ValueForKey:
    //     Option<unsafe extern "C" fn(playerNum: c_int, key: *const c_char) -> *const c_char>,
    // pub PlayerInfo_SetValueForKey:
    //     Option<unsafe extern "C" fn(key: *const c_char, value: *const c_char)>,
    // pub GetPlayerUniqueID:
    //     Option<unsafe extern "C" fn(iPlayer: c_int, playerID: *mut [c_char; 16usize]) -> qboolean>,
    // pub GetTrackerIDForPlayer: Option<unsafe extern "C" fn(playerSlot: c_int) -> c_int>,
    // pub GetPlayerForTrackerID: Option<unsafe extern "C" fn(trackerID: c_int) -> c_int>,
    // pub pfnServerCmdUnreliable: Option<unsafe extern "C" fn(szCmdString: *mut c_char) -> c_int>,
    // pub pfnGetMousePos: Option<unsafe extern "C" fn(ppt: *mut tagPOINT)>,

    pub fn set_mouse_position(&self, x: c_int, y: c_int) {
        unsafe { unwrap!(self, pfnSetMousePos)(x, y) }
    }

    // pub pfnSetMouseEnable: Option<unsafe extern "C" fn(fEnable: qboolean)>,
    // pub pfnGetFirstCvarPtr: Option<unsafe extern "C" fn() -> *mut cvar_s>,
    // pub pfnGetFirstCmdFunctionHandle: Option<unsafe extern "C" fn() -> *mut c_void>,
    // pub pfnGetNextCmdFunctionHandle:
    //     Option<unsafe extern "C" fn(cmdhandle: *mut c_void) -> *mut c_void>,
    // pub pfnGetCmdFunctionName:
    //     Option<unsafe extern "C" fn(cmdhandle: *mut c_void) -> *const c_char>,
    // pub pfnGetClientOldTime: Option<unsafe extern "C" fn() -> f32>,
    // pub pfnGetGravity: Option<unsafe extern "C" fn() -> f32>,
    // pub pfnGetModelByIndex: Option<unsafe extern "C" fn(index: c_int) -> *mut model_s>,
    // pub pfnSetFilterMode: Option<unsafe extern "C" fn(mode: c_int)>,
    // pub pfnSetFilterColor: Option<unsafe extern "C" fn(red: f32, green: f32, blue: f32)>,
    // pub pfnSetFilterBrightness: Option<unsafe extern "C" fn(brightness: f32)>,
    // pub pfnSequenceGet: Option<
    //     unsafe extern "C" fn(fileName: *const c_char, entryName: *const c_char) -> *mut c_void,
    // >,
    // pub pfnSPR_DrawGeneric: Option<
    //     unsafe extern "C" fn(
    //         frame: c_int,
    //         x: c_int,
    //         y: c_int,
    //         prc: *const wrect_t,
    //         blendsrc: c_int,
    //         blenddst: c_int,
    //         width: c_int,
    //         height: c_int,
    //     ),
    // >,
    // pub pfnSequencePickSentence: Option<
    //     unsafe extern "C" fn(
    //         groupName: *const c_char,
    //         pickMethod: c_int,
    //         entryPicked: *mut c_int,
    //     ) -> *mut c_void,
    // >,

    pub fn draw_string(&self, x: c_int, y: c_int, s: impl ToEngineStr, color: RGB) -> c_int {
        let s = s.to_engine_str();
        let [r, g, b] = color.into();
        unsafe { unwrap!(self, pfnDrawString)(x, y, s.as_ptr(), r, g, b) }
    }

    pub fn draw_string_reverse(
        &self,
        x: c_int,
        y: c_int,
        s: impl ToEngineStr,
        color: RGB,
    ) -> c_int {
        let s = s.to_engine_str();
        let [r, g, b] = color.into();
        unsafe { unwrap!(self, pfnDrawStringReverse)(x, y, s.as_ptr(), r, g, b) }
    }

    // pub LocalPlayerInfo_ValueForKey:
    //     Option<unsafe extern "C" fn(key: *const c_char) -> *const c_char>,
    // pub pfnVGUI2DrawCharacter:
    //     Option<unsafe extern "C" fn(x: c_int, y: c_int, ch: c_int, font: c_uint) -> c_int>,
    // pub pfnVGUI2DrawCharacterAdditive: Option<
    //     unsafe extern "C" fn(
    //         x: c_int,
    //         y: c_int,
    //         ch: c_int,
    //         r: c_int,
    //         g: c_int,
    //         b: c_int,
    //         font: c_uint,
    //     ) -> c_int,
    // >,
    // pub pfnGetApproxWavePlayLen: Option<unsafe extern "C" fn(filename: *const c_char) -> c_uint>,
    // pub GetCareerGameUI: Option<unsafe extern "C" fn() -> *mut c_void>,
    // pub Cvar_Set: Option<unsafe extern "C" fn(name: *const c_char, value: *const c_char)>,
    // pub pfnIsPlayingCareerMatch: Option<unsafe extern "C" fn() -> c_int>,
    // pub pfnPlaySoundVoiceByName:
    //     Option<unsafe extern "C" fn(szSound: *mut c_char, volume: f32, pitch: c_int)>,
    // pub pfnPrimeMusicStream: Option<unsafe extern "C" fn(filename: *mut c_char, looping: c_int)>,
    // pub pfnSys_FloatTime: Option<unsafe extern "C" fn() -> f64>,
    // pub pfnProcessTutorMessageDecayBuffer:
    //     Option<unsafe extern "C" fn(buffer: *mut c_int, buflen: c_int)>,
    // pub pfnConstructTutorMessageDecayBuffer:
    //     Option<unsafe extern "C" fn(buffer: *mut c_int, buflen: c_int)>,
    // pub pfnResetTutorMessageDecayData: Option<unsafe extern "C" fn()>,
    // pub pfnPlaySoundByNameAtPitch:
    //     Option<unsafe extern "C" fn(szSound: *mut c_char, volume: f32, pitch: c_int)>,

    pub fn fill_rgba_blend(&self, x: c_int, y: c_int, width: c_int, height: c_int, color: RGBA) {
        unsafe {
            let [r, g, b, a] = color.into();
            unwrap!(self, pfnFillRGBABlend)(x, y, width, height, r, g, b, a)
        }
    }

    // pub pfnGetAppID: Option<unsafe extern "C" fn() -> c_int>,
    // pub pfnGetAliases: Option<unsafe extern "C" fn() -> *mut cmdalias_t>,
    // pub pfnVguiWrap2_GetMouseDelta: Option<unsafe extern "C" fn(x: *mut c_int, y: *mut c_int)>,
    // pub pfnFilteredClientCmd: Option<unsafe extern "C" fn(cmd: *const c_char) -> c_int>,

    pub fn is_singleplayer(&self) -> bool {
        self.get_max_clients() == 1
    }

    pub fn is_multiplayer(&self) -> bool {
        self.get_max_clients() > 1
    }
}

impl From<raw::cl_enginefuncs_s> for Engine {
    fn from(raw: raw::cl_enginefuncs_s) -> Self {
        Self { raw }
    }
}

static ENGINE: SyncOnceCell<Engine> = unsafe { SyncOnceCell::new() };

pub fn engine<'a>() -> &'a Engine {
    ENGINE.get().unwrap()
}

pub fn engine_set(engine_funcs: raw::cl_enginefuncs_s) {
    if ENGINE.set(engine_funcs.into()).is_err() {
        warn!("client engine initialized multiple times");
    }
}
