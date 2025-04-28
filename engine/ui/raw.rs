#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]

use core::{
    ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_void},
    ptr,
};

use csz::CStrArray;
use shared::{
    cvar::cvar_s,
    raw::{byte, cl_entity_s, con_nprint_s, net_api_s, netadr_s, ref_viewpass_s, wrect_s},
};

pub type HIMAGE = c_int;

pub const MENU_EXTENDED_API_VERSION: c_int = 1;

pub const GAMEINFO_VERSION: c_int = 2;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GAMEINFO {
    pub gamefolder: CStrArray<64>,
    pub startmap: CStrArray<64>,
    pub trainmap: CStrArray<64>,
    pub title: CStrArray<64>,
    pub version: CStrArray<14>,
    pub flags: c_short,
    pub game_url: CStrArray<256>,
    pub update_url: CStrArray<256>,
    pub type_: CStrArray<64>,
    pub date: CStrArray<64>,
    pub size: CStrArray<64>,
    pub gamemode: c_int,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum GameType {
    Normal = 0,
    SingleplayerOnly = 1,
    MultiplayerOnly = 2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct gameinfo2_s {
    pub gi_version: c_int,
    pub gamefolder: CStrArray<64>,
    pub startmap: CStrArray<64>,
    pub trainmap: CStrArray<64>,
    pub demomap: CStrArray<64>,
    pub title: CStrArray<64>,
    pub iconpath: CStrArray<64>,
    pub version: CStrArray<16>,
    pub flags: u32,
    pub game_url: CStrArray<256>,
    pub update_url: CStrArray<256>,
    pub type_: CStrArray<64>,
    pub date: CStrArray<64>,
    pub size: u64,
    pub gamemode: GameType,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ui_globalvars_s {
    pub time: f32,
    pub frametime: f32,
    pub scrWidth: c_int,
    pub scrHeight: c_int,
    pub maxClients: c_int,
    pub developer: c_int,
    pub demoplayback: c_int,
    pub demorecording: c_int,
    pub demoname: CStrArray<64>,
    pub maptitle: CStrArray<64>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ui_enginefuncs_s {
    pub pfnPIC_Load: Option<
        unsafe extern "C" fn(
            szPicName: *const c_char,
            ucRawImage: *const byte,
            ulRawImageSize: c_int,
            flags: c_int,
        ) -> HIMAGE,
    >,
    pub pfnPIC_Free: Option<unsafe extern "C" fn(szPicName: *const c_char)>,
    pub pfnPIC_Width: Option<unsafe extern "C" fn(hPic: HIMAGE) -> c_int>,
    pub pfnPIC_Height: Option<unsafe extern "C" fn(hPic: HIMAGE) -> c_int>,
    pub pfnPIC_Set:
        Option<unsafe extern "C" fn(hPic: HIMAGE, r: c_int, g: c_int, b: c_int, a: c_int)>,
    pub pfnPIC_Draw: Option<
        unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int, prc: *const wrect_s),
    >,
    pub pfnPIC_DrawHoles: Option<
        unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int, prc: *const wrect_s),
    >,
    pub pfnPIC_DrawTrans: Option<
        unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int, prc: *const wrect_s),
    >,
    pub pfnPIC_DrawAdditive: Option<
        unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int, prc: *const wrect_s),
    >,
    pub pfnPIC_EnableScissor:
        Option<unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int)>,
    pub pfnPIC_DisableScissor: Option<unsafe extern "C" fn()>,
    pub pfnFillRGBA: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            width: c_int,
            height: c_int,
            r: c_int,
            g: c_int,
            b: c_int,
            a: c_int,
        ),
    >,
    pub pfnRegisterVariable: Option<
        unsafe extern "C" fn(
            szName: *const c_char,
            szValue: *const c_char,
            flags: c_int,
        ) -> *mut cvar_s,
    >,
    pub pfnGetCvarFloat: Option<unsafe extern "C" fn(szName: *const c_char) -> f32>,
    pub pfnGetCvarString: Option<unsafe extern "C" fn(szName: *const c_char) -> *const c_char>,
    pub pfnCvarSetString:
        Option<unsafe extern "C" fn(szName: *const c_char, szValue: *const c_char)>,
    pub pfnCvarSetValue: Option<unsafe extern "C" fn(szName: *const c_char, flValue: f32)>,
    pub pfnAddCommand: Option<
        unsafe extern "C" fn(
            cmd_name: *const c_char,
            function: Option<unsafe extern "C" fn()>,
        ) -> c_int,
    >,
    pub pfnClientCmd: Option<unsafe extern "C" fn(execute_now: c_int, szCmdString: *const c_char)>,
    pub pfnDelCommand: Option<unsafe extern "C" fn(cmd_name: *const c_char)>,
    pub pfnCmdArgc: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnCmdArgv: Option<unsafe extern "C" fn(argc: c_int) -> *const c_char>,
    pub pfnCmd_Args: Option<unsafe extern "C" fn() -> *const c_char>,
    pub Con_Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_DPrintf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub Con_NPrintf: Option<unsafe extern "C" fn(pos: c_int, fmt: *const c_char, ...)>,
    pub Con_NXPrintf:
        Option<unsafe extern "C" fn(info: *mut con_nprint_s, fmt: *const c_char, ...)>,
    pub pfnPlayLocalSound: Option<unsafe extern "C" fn(szSound: *const c_char)>,
    pub pfnDrawLogo: Option<
        unsafe extern "C" fn(filename: *const c_char, x: f32, y: f32, width: f32, height: f32),
    >,
    pub pfnGetLogoWidth: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetLogoHeight: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetLogoLength: Option<unsafe extern "C" fn() -> f32>,
    pub pfnDrawCharacter: Option<
        unsafe extern "C" fn(
            x: c_int,
            y: c_int,
            width: c_int,
            height: c_int,
            ch: c_int,
            ulRGBA: c_int,
            hFont: HIMAGE,
        ),
    >,
    pub pfnDrawConsoleString:
        Option<unsafe extern "C" fn(x: c_int, y: c_int, string: *const c_char) -> c_int>,
    pub pfnDrawSetTextColor:
        Option<unsafe extern "C" fn(r: c_int, g: c_int, b: c_int, alpha: c_int)>,
    pub pfnDrawConsoleStringLen:
        Option<unsafe extern "C" fn(string: *const c_char, length: *mut c_int, height: *mut c_int)>,
    pub pfnSetConsoleDefaultColor: Option<unsafe extern "C" fn(r: c_int, g: c_int, b: c_int)>,
    pub pfnGetPlayerModel: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    pub pfnSetModel: Option<unsafe extern "C" fn(ed: *mut cl_entity_s, path: *const c_char)>,
    pub pfnClearScene: Option<unsafe extern "C" fn()>,
    pub pfnRenderScene: Option<unsafe extern "C" fn(rvp: *const ref_viewpass_s)>,
    pub CL_CreateVisibleEntity:
        Option<unsafe extern "C" fn(type_: c_int, ent: *mut cl_entity_s) -> c_int>,
    pub pfnHostError: Option<unsafe extern "C" fn(szFmt: *const c_char, ...)>,
    pub pfnFileExists:
        Option<unsafe extern "C" fn(filename: *const c_char, gamedironly: c_int) -> c_int>,
    pub pfnGetGameDir: Option<unsafe extern "C" fn(szGetGameDir: *mut c_char)>,
    pub pfnCreateMapsList: Option<unsafe extern "C" fn(fRefresh: c_int) -> c_int>,
    pub pfnClientInGame: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnClientJoin: Option<unsafe extern "C" fn(adr: netadr_s)>,
    pub COM_LoadFile:
        Option<unsafe extern "C" fn(filename: *const c_char, pLength: *mut c_int) -> *mut byte>,
    pub COM_ParseFile:
        Option<unsafe extern "C" fn(data: *mut c_char, token: *mut c_char) -> *mut c_char>,
    pub COM_FreeFile: Option<unsafe extern "C" fn(buffer: *mut c_void)>,
    pub pfnKeyClearStates: Option<unsafe extern "C" fn()>,
    pub pfnSetKeyDest: Option<unsafe extern "C" fn(dest: c_int)>,
    pub pfnKeynumToString: Option<unsafe extern "C" fn(keynum: c_int) -> *const c_char>,
    pub pfnKeyGetBinding: Option<unsafe extern "C" fn(keynum: c_int) -> *const c_char>,
    pub pfnKeySetBinding: Option<unsafe extern "C" fn(keynum: c_int, binding: *const c_char)>,
    pub pfnKeyIsDown: Option<unsafe extern "C" fn(keynum: c_int) -> c_int>,
    pub pfnKeyGetOverstrikeMode: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnKeySetOverstrikeMode: Option<unsafe extern "C" fn(fActive: c_int)>,
    pub pfnKeyGetState: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,
    pub pfnMemAlloc: Option<
        unsafe extern "C" fn(cb: usize, filename: *const c_char, fileline: c_int) -> *mut c_void,
    >,
    pub pfnMemFree:
        Option<unsafe extern "C" fn(mem: *mut c_void, filename: *const c_char, fileline: c_int)>,
    pub pfnGetGameInfo: Option<unsafe extern "C" fn(pgameinfo: *mut GAMEINFO) -> c_int>,
    pub pfnGetGamesList: Option<unsafe extern "C" fn(numGames: *mut c_int) -> *mut *mut GAMEINFO>,
    pub pfnGetFilesList: Option<
        unsafe extern "C" fn(
            pattern: *const c_char,
            numFiles: *mut c_int,
            gamedironly: c_int,
        ) -> *mut *mut c_char,
    >,
    pub pfnGetSaveComment:
        Option<unsafe extern "C" fn(savename: *const c_char, comment: *mut c_char) -> c_int>,
    pub pfnGetDemoComment:
        Option<unsafe extern "C" fn(demoname: *const c_char, comment: *mut c_char) -> c_int>,
    pub pfnCheckGameDll: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnGetClipboardData: Option<unsafe extern "C" fn() -> *mut c_char>,
    pub pfnShellExecute:
        Option<unsafe extern "C" fn(name: *const c_char, args: *const c_char, closeEngine: c_int)>,
    pub pfnWriteServerConfig: Option<unsafe extern "C" fn(name: *const c_char)>,
    pub pfnChangeInstance:
        Option<unsafe extern "C" fn(newInstance: *const c_char, szFinalMessage: *const c_char)>,
    pub pfnPlayBackgroundTrack:
        Option<unsafe extern "C" fn(introName: *const c_char, loopName: *const c_char)>,
    pub pfnHostEndGame: Option<unsafe extern "C" fn(szFinalMessage: *const c_char)>,
    pub pfnRandomFloat: Option<unsafe extern "C" fn(flLow: f32, flHigh: f32) -> f32>,
    pub pfnRandomLong: Option<unsafe extern "C" fn(lLow: c_int, lHigh: c_int) -> c_int>,
    pub pfnSetCursor: Option<unsafe extern "C" fn(hCursor: *mut c_void)>,
    pub pfnIsMapValid: Option<unsafe extern "C" fn(filename: *const c_char) -> c_int>,
    pub pfnProcessImage: Option<
        unsafe extern "C" fn(texnum: c_int, gamma: f32, topColor: c_int, bottomColor: c_int),
    >,
    pub pfnCompareFileTime: Option<
        unsafe extern "C" fn(
            filename1: *const c_char,
            filename2: *const c_char,
            iCompare: *mut c_int,
        ) -> c_int,
    >,
    pub pfnGetModeString: Option<unsafe extern "C" fn(vid_mode: c_int) -> *const c_char>,
    pub COM_SaveFile: Option<
        unsafe extern "C" fn(filename: *const c_char, data: *const c_void, len: c_int) -> c_int,
    >,
    pub COM_RemoveFile: Option<unsafe extern "C" fn(filepath: *const c_char) -> c_int>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UI_FUNCTIONS {
    pub pfnVidInit: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnInit: Option<unsafe extern "C" fn()>,
    pub pfnShutdown: Option<unsafe extern "C" fn()>,
    pub pfnRedraw: Option<unsafe extern "C" fn(flTime: f32)>,
    pub pfnKeyEvent: Option<unsafe extern "C" fn(key: c_int, down: c_int)>,
    pub pfnMouseMove: Option<unsafe extern "C" fn(x: c_int, y: c_int)>,
    pub pfnSetActiveMenu: Option<unsafe extern "C" fn(active: c_int)>,
    pub pfnAddServerToList: Option<unsafe extern "C" fn(adr: netadr_s, info: *const c_char)>,
    pub pfnGetCursorPos: Option<unsafe extern "C" fn(pos_x: *mut c_int, pos_y: *mut c_int)>,
    pub pfnSetCursorPos: Option<unsafe extern "C" fn(pos_x: c_int, pos_y: c_int)>,
    pub pfnShowCursor: Option<unsafe extern "C" fn(show: c_int)>,
    pub pfnCharEvent: Option<unsafe extern "C" fn(key: c_int)>,
    pub pfnMouseInRect: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnIsVisible: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnCreditsActive: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnFinalCredits: Option<unsafe extern "C" fn()>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ui_extendedfuncs_s {
    pub pfnEnableTextInput: Option<unsafe extern "C" fn(enable: c_int)>,
    pub pfnUtfProcessChar: Option<unsafe extern "C" fn(ch: c_int) -> c_int>,
    pub pfnUtfMoveLeft: Option<unsafe extern "C" fn(str_: *mut c_char, pos: c_int) -> c_int>,
    pub pfnUtfMoveRight:
        Option<unsafe extern "C" fn(str_: *mut c_char, pos: c_int, length: c_int) -> c_int>,
    pub pfnGetRenderers: Option<
        unsafe extern "C" fn(
            num: c_uint,
            shortName: *mut c_char,
            size1: usize,
            readableName: *mut c_char,
            size2: usize,
        ) -> c_int,
    >,
    pub pfnDoubleTime: Option<unsafe extern "C" fn() -> f64>,
    pub pfnParseFile: Option<
        unsafe extern "C" fn(
            data: *mut c_char,
            buf: *mut c_char,
            size: c_int,
            flags: c_uint,
            len: *mut c_int,
        ) -> *mut c_char,
    >,
    pub pfnAdrToString: Option<unsafe extern "C" fn(a: netadr_s) -> *const c_char>,
    pub pfnCompareAdr: Option<unsafe extern "C" fn(a: *const c_void, b: *const c_void) -> c_int>,
    pub pfnGetNativeObject: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,
    pub pNetAPI: *mut net_api_s,
    pub pfnGetGameInfo: Option<unsafe extern "C" fn(gi_version: c_int) -> *mut gameinfo2_s>,
    pub pfnGetModInfo:
        Option<unsafe extern "C" fn(gi_version: c_int, mod_index: c_int) -> *mut gameinfo2_s>,
}

impl Default for ui_extendedfuncs_s {
    fn default() -> Self {
        Self {
            pfnEnableTextInput: None,
            pfnUtfProcessChar: None,
            pfnUtfMoveLeft: None,
            pfnUtfMoveRight: None,
            pfnGetRenderers: None,
            pfnDoubleTime: None,
            pfnParseFile: None,
            pfnAdrToString: None,
            pfnCompareAdr: None,
            pfnGetNativeObject: None,
            pNetAPI: ptr::null_mut(),
            pfnGetGameInfo: None,
            pfnGetModInfo: None,
        }
    }
}

pub type ADDTOUCHBUTTONTOLIST = Option<
    unsafe extern "C" fn(
        name: *const c_char,
        texture: *const c_char,
        command: *const c_char,
        color: *mut c_uchar,
        flags: c_int,
    ),
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UI_EXTENDED_FUNCTIONS {
    pub pfnAddTouchButtonToList: ADDTOUCHBUTTONTOLIST,
    pub pfnResetPing: Option<unsafe extern "C" fn()>,
    pub pfnShowConnectionWarning: Option<unsafe extern "C" fn()>,
    pub pfnShowUpdateDialog: Option<unsafe extern "C" fn(preferStore: c_int)>,
    pub pfnShowMessageBox: Option<unsafe extern "C" fn(text: *const c_char)>,
    pub pfnConnectionProgress_Disconnect: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_Download: Option<
        unsafe extern "C" fn(
            pszFileName: *const c_char,
            pszServerName: *const c_char,
            iCurrent: c_int,
            iTotal: c_int,
            comment: *const c_char,
        ),
    >,
    pub pfnConnectionProgress_DownloadEnd: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_Precache: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_Connect: Option<unsafe extern "C" fn(server: *const c_char)>,
    pub pfnConnectionProgress_ChangeLevel: Option<unsafe extern "C" fn()>,
    pub pfnConnectionProgress_ParseServerInfo: Option<unsafe extern "C" fn(server: *const c_char)>,
}

pub type MENUAPI = Option<
    unsafe extern "C" fn(
        pFunctionTable: *mut UI_FUNCTIONS,
        engfuncs: *mut ui_enginefuncs_s,
        pGlobals: *mut ui_globalvars_s,
    ) -> c_int,
>;

pub type UIEXTENEDEDAPI = Option<
    unsafe extern "C" fn(
        version: c_int,
        pFunctionTable: *mut UI_EXTENDED_FUNCTIONS,
        engfuncs: *mut ui_extendedfuncs_s,
    ) -> c_int,
>;

pub type UITEXTAPI = Option<unsafe extern "C" fn(engfuncs: *mut ui_extendedfuncs_s) -> c_int>;
