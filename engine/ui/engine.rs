use core::{
    ffi::{c_char, c_int, c_uint, c_void},
    fmt,
    mem::{self, MaybeUninit},
    ptr, slice,
    str::FromStr,
};

use csz::{CStrArray, CStrSlice, CStrThin};
use shared::{
    borrow::{BorrowRef, Ref},
    engine::net::{netadr_s, NetApi},
    export::impl_unsync_global,
    ffi::menu::{ui_enginefuncs_s, ui_extendedfuncs_s},
    str::{AsCStrPtr, ToEngineStr},
};

use crate::{
    color::{RGB, RGBA},
    consts::{MAX_STRING, MAX_SYSPATH},
    cvar::{CVarFlags, CVarPtr},
    engine_types::{ActiveMenu, Point, Size},
    file::{Cursor, File, FileList},
    game_info::GameInfo,
    raw::{self, kbutton_t, wrect_s, HIMAGE},
};

pub use shared::engine::{net, AddCmdError, BufferError};

pub(crate) mod prelude {
    pub use shared::engine::{
        net::EngineNet, EngineCmd, EngineCmdArgsRaw, EngineConsole, EngineCvar,
        EngineDrawConsoleString, EngineRng, EngineSystemTime,
    };
}

pub use self::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Protocol {
    GoldSrc,
    Xash48,
    /// Same as [Protocol::Xash48].
    Legacy,
    Xash49,
    /// Same as [Protocol::Xash49].
    Current,
}

impl Protocol {
    pub fn is_current(&self) -> bool {
        matches!(self, Self::Current | Self::Xash49)
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::Legacy | Self::Xash48)
    }

    pub fn is_goldsrc(&self) -> bool {
        matches!(self, Self::GoldSrc)
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Self::Current
    }
}

pub struct InvalidProtocolError;

impl FromStr for Protocol {
    type Err = InvalidProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gs" | "goldsrc" => Ok(Self::GoldSrc),
            "48" => Ok(Self::Xash48),
            "legacy" => Ok(Self::Legacy),
            "49" => Ok(Self::Xash49),
            "current" => Ok(Self::Current),
            _ => Err(InvalidProtocolError),
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GoldSrc => "gs".fmt(f),
            Self::Xash48 => "48".fmt(f),
            Self::Legacy => "legacy".fmt(f),
            Self::Xash49 => "49".fmt(f),
            Self::Current => "current".fmt(f),
        }
    }
}

#[derive(Default)]
struct Borrows {
    keynum_to_str: BorrowRef,
}

pub struct UiEngine {
    raw: ui_enginefuncs_s,
    ext: ui_extendedfuncs_s,
    net_api: NetApi,
    borrows: Borrows,
}

impl_unsync_global!(UiEngine);

macro_rules! unwrap {
    ($self:expr, ext.$name:ident) => {
        match $self.ext.$name {
            Some(func) => func,
            None => panic!("ui_extendedfuncs_s.{} is null", stringify!($name)),
        }
    };
    ($self:expr, $name:ident) => {
        match $self.raw.$name {
            Some(func) => func,
            None => panic!("ui_enginefuncs_s.{} is null", stringify!($name)),
        }
    };
}

impl UiEngine {
    pub(crate) fn new(raw: &ui_enginefuncs_s) -> Self {
        Self {
            raw: *raw,
            ext: unsafe { mem::zeroed() },
            borrows: Default::default(),
            net_api: NetApi::new(ptr::null_mut()),
        }
    }

    pub(crate) fn set_extended(&mut self, ext: ui_extendedfuncs_s) {
        self.net_api = NetApi::new(ext.pNetAPI);
        self.ext = ext;
    }

    pub fn raw(&self) -> &ui_enginefuncs_s {
        &self.raw
    }

    pub fn raw_ext(&self) -> &ui_extendedfuncs_s {
        &self.ext
    }

    pub fn pic_load(
        &self,
        path: impl ToEngineStr,
        buf: Option<&[u8]>,
        flags: u32,
    ) -> Option<HIMAGE> {
        let path = path.to_engine_str();
        let (buf, len) = buf
            .map(|i| (i.as_ptr(), i.len()))
            .unwrap_or((ptr::null(), 0));
        let pic =
            unsafe { unwrap!(self, pfnPIC_Load)(path.as_ptr(), buf, len as c_int, flags as c_int) };
        (pic != 0).then_some(pic)
    }

    pub fn pic_free(&self, path: impl ToEngineStr) {
        let path = path.to_engine_str();
        unsafe {
            unwrap!(self, pfnPIC_Free)(path.as_ptr());
        }
    }

    pub fn pic_width(&self, pic: HIMAGE) -> c_int {
        unsafe { unwrap!(self, pfnPIC_Width)(pic) }
    }

    pub fn pic_height(&self, pic: HIMAGE) -> c_int {
        unsafe { unwrap!(self, pfnPIC_Height)(pic) }
    }

    pub fn pic_size(&self, pic: HIMAGE) -> Size {
        Size::new(self.pic_width(pic), self.pic_height(pic))
    }

    pub fn pic_set<C: Into<RGBA>>(&self, pic: HIMAGE, color: C) {
        let [r, g, b, a] = color.into().into();
        unsafe {
            unwrap!(self, pfnPIC_Set)(pic, r, g, b, a);
        }
    }

    pub fn pic_draw<P, S>(&self, pos: P, size: S, rect: Option<&wrect_s>)
    where
        P: Into<Point>,
        S: Into<Size>,
    {
        let (x, y) = pos.into().components();
        let (w, h) = size.into().components();
        let r = rect.map(|i| i as *const _).unwrap_or(ptr::null());
        unsafe {
            unwrap!(self, pfnPIC_Draw)(x, y, w, h, r);
        }
    }

    pub fn pic_draw_holes<P, S>(&self, pos: P, size: S, rect: Option<&wrect_s>)
    where
        P: Into<Point>,
        S: Into<Size>,
    {
        let (x, y) = pos.into().components();
        let (w, h) = size.into().components();
        let p = rect.map(|i| i as *const _).unwrap_or(ptr::null());
        unsafe {
            unwrap!(self, pfnPIC_DrawHoles)(x, y, w, h, p);
        }
    }

    pub fn pic_draw_trans<P, S>(&self, pos: P, size: S, rect: Option<&wrect_s>)
    where
        P: Into<Point>,
        S: Into<Size>,
    {
        let (x, y) = pos.into().components();
        let (w, h) = size.into().components();
        let p = rect.map(|i| i as *const _).unwrap_or(ptr::null());
        unsafe {
            unwrap!(self, pfnPIC_DrawTrans)(x, y, w, h, p);
        }
    }

    pub fn pic_draw_additive<P, S>(&self, pos: P, size: S, rect: Option<&wrect_s>)
    where
        P: Into<Point>,
        S: Into<Size>,
    {
        let (x, y) = pos.into().components();
        let (w, h) = size.into().components();
        let p = rect.map(|i| i as *const _).unwrap_or(ptr::null());
        unsafe {
            unwrap!(self, pfnPIC_DrawAdditive)(x, y, w, h, p);
        }
    }

    pub fn pic_enable_scissor(&self, x: c_int, y: c_int, width: c_int, height: c_int) {
        unsafe { unwrap!(self, pfnPIC_EnableScissor)(x, y, width, height) }
    }

    pub fn pic_disable_scissor(&self) {
        unsafe { unwrap!(self, pfnPIC_DisableScissor)() }
    }

    pub fn fill_rgba<P, S, C>(&self, pos: P, size: S, color: C)
    where
        P: Into<Point>,
        S: Into<Size>,
        C: Into<RGBA>,
    {
        let (x, y) = pos.into().components();
        let (w, h) = size.into().components();
        let [r, g, b, a] = color.into().into();
        unsafe {
            unwrap!(self, pfnFillRGBA)(x, y, w, h, r, g, b, a);
        }
    }

    pub fn register_variable(
        &self,
        name: impl ToEngineStr,
        value: impl ToEngineStr,
        flags: CVarFlags,
    ) -> Option<CVarPtr> {
        let name = name.to_engine_str();
        let value = value.to_engine_str();
        let ptr = unsafe {
            unwrap!(self, pfnRegisterVariable)(name.as_ptr(), value.as_ptr(), flags.bits() as c_int)
        };
        if !ptr.is_null() {
            Some(CVarPtr::from_ptr(ptr.cast()))
        } else {
            None
        }
    }

    pub fn client_cmd(&self, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnClientCmd)(0, cmd.as_ptr()) }
    }

    pub fn client_cmd_now(&self, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnClientCmd)(1, cmd.as_ptr()) }
    }

    pub fn delete_command(&self, cmd_name: impl ToEngineStr) {
        let cmd_name = cmd_name.to_engine_str();
        unsafe {
            unwrap!(self, pfnDelCommand)(cmd_name.as_ptr());
        }
    }

    pub fn con_dprint(&self, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, Con_DPrintf)(c"%s".as_ptr(), msg.as_ptr());
        }
    }

    pub fn con_nprint(&self, pos: c_int, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, Con_NPrintf)(pos, c"%s".as_ptr(), msg.as_ptr());
        }
    }

    // pub Con_NXPrintf:
    //     Option<unsafe extern "C" fn(info: *mut ffi::con_nprint_s, fmt: *const c_char, ...)>,

    pub fn play_sound(&self, path: impl ToEngineStr) {
        let path = path.to_engine_str();
        unsafe {
            unwrap!(self, pfnPlayLocalSound)(path.as_ptr());
        }
    }

    pub fn draw_logo(&self, filename: impl ToEngineStr, x: f32, y: f32, w: f32, h: f32) {
        let filename = filename.to_engine_str();
        unsafe {
            unwrap!(self, pfnDrawLogo)(filename.as_ptr(), x, y, w, h);
        }
    }

    pub fn get_logo_width(&self) -> c_int {
        unsafe { unwrap!(self, pfnGetLogoWidth)() }
    }

    pub fn get_logo_height(&self) -> c_int {
        unsafe { unwrap!(self, pfnGetLogoHeight)() }
    }

    pub fn get_logo_size(&self) -> (c_int, c_int) {
        (self.get_logo_width(), self.get_logo_height())
    }

    pub fn get_logo_length(&self) -> f32 {
        unsafe { unwrap!(self, pfnGetLogoLength)() }
    }

    // pub pfnDrawCharacter: Option<
    //     unsafe extern "C" fn(
    //         x: c_int,
    //         y: c_int,
    //         width: c_int,
    //         height: c_int,
    //         ch: c_int,
    //         ulRGBA: c_int,
    //         hFont: HIMAGE,
    //     ),
    // >,

    pub fn draw_set_text_color_with_alpha(&self, color: impl Into<RGBA>) {
        let color = color.into();
        unsafe {
            unwrap!(self, pfnDrawSetTextColor)(
                color.r() as c_int,
                color.g() as c_int,
                color.b() as c_int,
                color.a() as c_int,
            );
        }
    }

    pub fn set_console_default_color(&self, color: impl Into<RGB>) {
        let color = color.into();
        unsafe {
            unwrap!(self, pfnSetConsoleDefaultColor)(
                color.r() as c_int,
                color.g() as c_int,
                color.b() as c_int,
            );
        }
    }

    // pub pfnGetPlayerModel: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    // pub pfnSetModel: Option<unsafe extern "C" fn(ed: *mut cl_entity_s, path: *const c_char)>,
    // pub pfnClearScene: Option<unsafe extern "C" fn()>,
    // pub pfnRenderScene: Option<unsafe extern "C" fn(rvp: *const ffi::ref_viewpass_s)>,
    // pub CL_CreateVisibleEntity:
    //     Option<unsafe extern "C" fn(type_: c_int, ent: *mut cl_entity_s) -> c_int>,

    pub fn host_error(&self, msg: impl ToEngineStr) -> ! {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, pfnHostError)(c"%s".as_ptr(), msg.as_ptr());
        }
        unreachable!()
    }

    pub fn file_exists(&self, filename: impl ToEngineStr, gamedironly: bool) -> bool {
        let filename = filename.to_engine_str();
        unsafe { unwrap!(self, pfnFileExists)(filename.as_ptr(), gamedironly as c_int) != 0 }
    }

    pub fn get_game_dir(&self) -> CStrArray<MAX_SYSPATH> {
        let mut buf = CStrArray::new();
        unsafe {
            unwrap!(self, pfnGetGameDir)(buf.as_mut_ptr());
        }
        buf
    }

    pub fn create_maps_list(&self, refresh: bool) -> bool {
        unsafe { unwrap!(self, pfnCreateMapsList)(refresh as c_int) != 0 }
    }

    pub fn client_in_game(&self) -> bool {
        unsafe { unwrap!(self, pfnClientInGame)() != 0 }
    }

    pub fn client_is_active(&self) -> bool {
        self.client_in_game() && !self.get_cvar::<bool>(c"cl_background")
    }

    pub fn client_join(&self, address: netadr_s, protocol: Protocol) {
        match protocol {
            Protocol::Current => unsafe { unwrap!(self, pfnClientJoin)(address) },
            _ => {
                let address = self.addr_to_string(address);
                self.client_cmd(format_args!("connect {address} {protocol}"));
            }
        }
    }

    pub fn load_file(&self, path: impl ToEngineStr) -> Option<File> {
        let path = path.to_engine_str();
        let mut len = 0;
        let data = unsafe { unwrap!(self, COM_LoadFile)(path.as_ptr(), &mut len) };
        if !data.is_null() {
            Some(unsafe { File::new(data.cast(), len as usize) })
        } else {
            None
        }
    }

    // pub COM_ParseFile:
    //     Option<unsafe extern "C" fn(data: *mut c_char, token: *mut c_char) -> *mut c_char>,

    /// Free file.
    ///
    /// # Safety
    ///
    /// Buffer must be allocated with [load_file](Self::load_file).
    pub unsafe fn free_file(&self, buffer: *mut c_void) {
        unsafe { unwrap!(self, COM_FreeFile)(buffer) }
    }

    pub fn key_clear_states(&self) {
        unsafe { unwrap!(self, pfnKeyClearStates)() }
    }

    pub fn set_key_dest(&self, dest: ActiveMenu) {
        unsafe {
            unwrap!(self, pfnSetKeyDest)(dest as c_int);
        }
    }

    #[deprecated(note = "use keynum_to_str_buffer instead")]
    pub fn keynum_to_str(&self, keynum: c_int) -> Ref<'_, CStrThin> {
        // SAFETY: The returned string is allocated in a private static buffer
        // in that function. Never returns a null pointer.
        unsafe {
            let s = unwrap!(self, pfnKeynumToString)(keynum);
            self.borrows.keynum_to_str.borrow(s as *mut CStrThin)
        }
    }

    /// Returns a string for the given key number. The buffer is used as storage for the string.
    ///
    /// Returns an error if the string length is greater than the buffer capacity.
    pub fn keynum_to_str_buffer<'a>(
        &self,
        keynum: c_int,
        buffer: &'a mut CStrSlice,
    ) -> Result<&'a CStrThin, BufferError> {
        let s = unsafe { unwrap!(self, pfnKeynumToString)(keynum) };
        assert!(!s.is_null());
        let s = unsafe { CStrThin::from_ptr(s) }.to_bytes();
        buffer.cursor().write_bytes(s).map_err(|_| BufferError)?;
        Ok(buffer.as_thin())
    }

    pub fn key_get_binding(&self, keynum: c_int) -> Option<&CStrThin> {
        // FIXME: engine returns cstr on heap, can be freed at any time
        let s = unsafe { unwrap!(self, pfnKeyGetBinding)(keynum) };
        if !s.is_null() {
            Some(unsafe { CStrThin::from_ptr(s) })
        } else {
            None
        }
    }

    pub fn key_set_binding(&self, keynum: c_int, binding: impl ToEngineStr) {
        let binding = binding.to_engine_str();
        unsafe { unwrap!(self, pfnKeySetBinding)(keynum, binding.as_ptr()) }
    }

    pub fn key_is_down(&self, keynum: c_int) -> bool {
        unsafe { unwrap!(self, pfnKeyIsDown)(keynum) != 0 }
    }

    pub fn ket_get_overstrike_mode(&self) -> c_int {
        unsafe { unwrap!(self, pfnKeyGetOverstrikeMode)() }
    }

    pub fn key_set_overstrike_mode(&self, active: c_int) {
        unsafe { unwrap!(self, pfnKeySetOverstrikeMode)(active) }
    }

    pub fn key_get_state(&self, name: impl ToEngineStr) -> Option<&'static kbutton_t> {
        let p = unsafe { unwrap!(self, pfnKeyGetState)(name.to_engine_str().as_ptr()) };
        if !p.is_null() {
            Some(unsafe { &*p.cast() })
        } else {
            None
        }
    }

    // pub pfnMemAlloc: Option<
    //     unsafe extern "C" fn(cb: usize, filename: *const c_char, fileline: c_int) -> *mut c_void,
    // >,
    // pub pfnMemFree:
    //     Option<unsafe extern "C" fn(mem: *mut c_void, filename: *const c_char, fileline: c_int)>,

    pub fn get_game_info(&self) -> Option<GameInfo> {
        let mut raw = MaybeUninit::uninit();
        let res = unsafe { unwrap!(self, pfnGetGameInfo)(raw.as_mut_ptr()) };
        if res != 0 {
            Some(GameInfo::new(unsafe { raw.assume_init() }))
        } else {
            None
        }
    }

    pub fn get_games_list(&self) -> &[&raw::GAMEINFO] {
        let mut len = 0;
        let data = unsafe { unwrap!(self, pfnGetGamesList)(&mut len) };
        if !data.is_null() {
            unsafe { slice::from_raw_parts(data.cast(), len as usize) }
        } else {
            &[]
        }
    }

    pub fn get_files_list(&self, pattern: impl ToEngineStr, gamedironly: bool) -> FileList<'_> {
        let pattern = pattern.to_engine_str();
        let mut len = 0;
        let func = unwrap!(self, pfnGetFilesList);
        let data = unsafe { func(pattern.as_ptr(), &mut len, gamedironly as c_int) };
        let raw = if !data.is_null() {
            unsafe { slice::from_raw_parts(data.cast(), len as usize) }
        } else {
            &[]
        };
        unsafe { FileList::new(raw) }
    }

    pub fn get_save_comment(&self, savename: impl ToEngineStr, comment: &mut [u8; 256]) -> bool {
        let savename = savename.to_engine_str();
        let func = unwrap!(self, pfnGetSaveComment);
        unsafe { func(savename.as_ptr(), comment.as_mut_ptr().cast()) != 0 }
    }

    pub fn get_demo_comment(
        &self,
        demoname: impl ToEngineStr,
        buf: &mut CStrArray<MAX_STRING>,
    ) -> bool {
        let demoname = demoname.to_engine_str();
        unsafe { unwrap!(self, pfnGetDemoComment)(demoname.as_ptr(), buf.as_mut_ptr()) != 0 }
    }

    pub fn check_game_dll(&self) -> bool {
        unsafe { unwrap!(self, pfnCheckGameDll)() != 0 }
    }

    pub fn get_clipboard_data(&self) -> Option<&CStrThin> {
        let p = unsafe { unwrap!(self, pfnGetClipboardData)() };
        if !p.is_null() {
            Some(unsafe { CStrThin::from_ptr(p) })
        } else {
            None
        }
    }

    pub fn shell_execute(
        &self,
        name: impl ToEngineStr,
        args: impl ToEngineStr,
        close_engine: bool,
    ) {
        let name = name.to_engine_str();
        let args = args.to_engine_str();
        unsafe {
            unwrap!(self, pfnShellExecute)(name.as_ptr(), args.as_ptr(), close_engine as c_int);
        }
    }

    pub fn write_server_config(&self, name: impl ToEngineStr) {
        let name = name.to_engine_str();
        unsafe {
            unwrap!(self, pfnWriteServerConfig)(name.as_ptr());
        }
    }

    // pub pfnChangeInstance:
    //     Option<unsafe extern "C" fn(newInstance: *const c_char, szFinalMessage: *const c_char)>,

    pub fn play_background_track(
        &self,
        intro_name: Option<impl ToEngineStr>,
        loop_name: Option<impl ToEngineStr>,
    ) {
        let intro_name = intro_name.map(|i| i.to_engine_str());
        let loop_name = loop_name.map(|i| i.to_engine_str());
        let intro_name = intro_name.map_or(ptr::null(), |i| i.as_ptr());
        let loop_name = loop_name.map_or(ptr::null(), |i| i.as_ptr());
        unsafe { unwrap!(self, pfnPlayBackgroundTrack)(intro_name, loop_name) }
    }

    pub fn stop_background_track(&self) {
        unsafe { unwrap!(self, pfnPlayBackgroundTrack)(ptr::null(), ptr::null()) }
    }

    pub fn host_end_game(&self, final_message: impl ToEngineStr) {
        let final_message = final_message.to_engine_str();
        unsafe {
            unwrap!(self, pfnHostEndGame)(final_message.as_ptr());
        }
    }

    // pub pfnSetCursor: Option<unsafe extern "C" fn(hCursor: *mut c_void)>,

    pub fn is_map_valid(&self, filename: impl ToEngineStr) -> bool {
        let filename = filename.to_engine_str();
        // FIXME: ffi: why filename is mutable?
        unsafe { unwrap!(self, pfnIsMapValid)(filename.as_ptr().cast_mut()) != 0 }
    }

    pub fn process_image(&self, texnum: c_int, gamma: f32, top_color: c_int, bottom_color: c_int) {
        unsafe {
            unwrap!(self, pfnProcessImage)(texnum, gamma, top_color, bottom_color);
        }
    }

    // pub pfnCompareFileTime: Option<
    //     unsafe extern "C" fn(
    //         filename1: *const c_char,
    //         filename2: *const c_char,
    //         iCompare: *mut c_int,
    //     ) -> c_int,
    // >,

    pub fn get_mode_string(&self, index: usize) -> Option<&'static CStrThin> {
        let p = unsafe { unwrap!(self, pfnGetModeString)(index as c_int) };
        if !p.is_null() {
            Some(unsafe { CStrThin::from_ptr(p) })
        } else {
            None
        }
    }

    pub fn get_mode_iter(&self) -> impl Iterator<Item = &'static CStrThin> + '_ {
        (0..).map_while(|i| self.get_mode_string(i))
    }

    pub fn save_file(&self, path: impl ToEngineStr, data: &[u8]) -> bool {
        let path = path.to_engine_str();
        let ptr = data.as_ptr().cast();
        let len = data.len() as c_int;
        unsafe { unwrap!(self, COM_SaveFile)(path.as_ptr(), ptr, len) != 0 }
    }

    pub fn remove_file(&self, path: impl ToEngineStr) -> bool {
        let path = path.to_engine_str();
        unsafe { unwrap!(self, COM_RemoveFile)(path.as_ptr()) != 0 }
    }

    pub fn enable_text_input(&self, enable: bool) {
        unsafe {
            unwrap!(self, ext.pfnEnableTextInput)(enable as c_int);
        }
    }

    // pub pfnUtfProcessChar: Option<unsafe extern "C" fn(ch: c_int) -> c_int>,
    // pub pfnUtfMoveLeft: Option<unsafe extern "C" fn(str_: *mut c_char, pos: c_int) -> c_int>,
    // pub pfnUtfMoveRight:
    //     Option<unsafe extern "C" fn(str_: *mut c_char, pos: c_int, length: c_int) -> c_int>,

    pub fn get_renderer(
        &self,
        index: c_uint,
        short_name: Option<&mut CStrArray<MAX_STRING>>,
        readable_name: Option<&mut CStrArray<MAX_STRING>>,
    ) -> bool {
        let default = (ptr::null_mut(), 0);
        let (s1, l1) = short_name.map_or(default, |i| (i.as_mut_ptr(), i.capacity()));
        let (s2, l2) = readable_name.map_or(default, |i| (i.as_mut_ptr(), i.capacity()));
        unsafe { unwrap!(self, ext.pfnGetRenderers)(index, s1, l1, s2, l2) != 0 }
    }

    #[deprecated(note = "use EngineSystemTime::system_time_f64 instead")]
    pub fn time_f64(&self) -> f64 {
        self.system_time_f64()
    }

    pub fn parse_file<'a>(
        &self,
        cursor: &mut Cursor,
        buf: &'a mut [u8],
        flags: c_uint,
    ) -> Option<&'a [u8]> {
        let mut len = 0;
        let data = unsafe {
            unwrap!(self, ext.pfnParseFile)(
                cursor.data.cast(),
                buf.as_mut_ptr().cast(),
                buf.len() as c_int,
                flags,
                &mut len,
            )
        };
        if !data.is_null() {
            cursor.data = data.cast();
            Some(&buf[..len as usize])
        } else {
            None
        }
    }

    pub fn addr_to_string(&self, addr: netadr_s) -> Ref<'_, CStrThin> {
        self.addr_to_string_ref(&addr)
    }

    // pub pfnCompareAdr: Option<unsafe extern "C" fn(a: *const c_void, b: *const c_void) -> c_int>,
    // pub pfnGetNativeObject: Option<unsafe extern "C" fn(name: *const c_char) -> *mut c_void>,

    pub fn get_game_info_2(&self) -> Option<&raw::gameinfo2_s> {
        let info = unsafe { unwrap!(self, ext.pfnGetGameInfo)(raw::GAMEINFO_VERSION) };
        if !info.is_null() {
            Some(unsafe { &*info })
        } else {
            None
        }
    }

    pub fn get_mod_info(&self, mod_index: c_int) -> Option<&raw::gameinfo2_s> {
        let info = unsafe { unwrap!(self, ext.pfnGetModInfo)(raw::GAMEINFO_VERSION, mod_index) };
        if !info.is_null() {
            Some(unsafe { &*info })
        } else {
            None
        }
    }

    pub fn get_mod_info_iter(&self) -> impl Iterator<Item = &raw::gameinfo2_s> {
        (0..).map_while(|i| self.get_mod_info(i))
    }
}

impl EngineCvar for UiEngine {
    fn fn_get_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char) -> f32 {
        unwrap!(self, pfnGetCvarFloat)
    }

    fn fn_set_cvar_float(&self) -> unsafe extern "C" fn(name: *const c_char, value: f32) {
        unwrap!(self, pfnCvarSetValue)
    }

    fn fn_get_cvar_string(&self) -> unsafe extern "C" fn(name: *const c_char) -> *const c_char {
        unwrap!(self, pfnGetCvarString)
    }

    fn fn_set_cvar_string(
        &self,
    ) -> unsafe extern "C" fn(name: *const c_char, value: *const c_char) {
        unwrap!(self, pfnCvarSetString)
    }
}

impl EngineRng for UiEngine {
    fn fn_random_float(&self) -> unsafe extern "C" fn(min: f32, max: f32) -> f32 {
        unwrap!(self, pfnRandomFloat)
    }

    fn fn_random_int(&self) -> unsafe extern "C" fn(min: c_int, max: c_int) -> c_int {
        unwrap!(self, pfnRandomLong)
    }
}

impl EngineConsole for UiEngine {
    fn console_print(&self, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe { unwrap!(self, Con_Printf)(c"%s".as_ptr(), msg.as_ptr()) }
    }
}

impl EngineCmd for UiEngine {
    fn fn_cmd_argc(&self) -> unsafe extern "C" fn() -> c_int {
        unwrap!(self, pfnCmdArgc)
    }

    fn fn_cmd_argv(&self) -> unsafe extern "C" fn(argc: c_int) -> *const c_char {
        unwrap!(self, pfnCmdArgv)
    }

    fn add_command(
        &self,
        name: impl ToEngineStr,
        func: unsafe extern "C" fn(),
    ) -> Result<(), AddCmdError> {
        let name = name.to_engine_str();
        let result = unsafe { unwrap!(self, pfnAddCommand)(name.as_ptr(), Some(func)) };
        if result == 0 {
            Err(AddCmdError)
        } else {
            Ok(())
        }
    }
}

impl EngineCmdArgsRaw for UiEngine {
    fn fn_cmd_args_raw(&self) -> unsafe extern "C" fn() -> *const c_char {
        unwrap!(self, pfnCmd_Args)
    }
}

impl EngineSystemTime for UiEngine {
    fn system_time_f64(&self) -> f64 {
        unsafe { unwrap!(self, ext.pfnDoubleTime)() }
    }
}

impl EngineDrawConsoleString for UiEngine {
    fn set_text_color(&self, color: impl Into<RGB>) {
        self.draw_set_text_color_with_alpha(color.into());
    }

    fn console_string_size(&self, text: impl ToEngineStr) -> (c_int, c_int) {
        let text = text.to_engine_str();
        let mut width = 0;
        let mut height = 0;
        unsafe {
            unwrap!(self, pfnDrawConsoleStringLen)(text.as_ptr(), &mut width, &mut height);
        }
        (width, height)
    }

    fn draw_console_string(&self, x: c_int, y: c_int, text: impl ToEngineStr) -> c_int {
        let text = text.to_engine_str();
        unsafe { unwrap!(self, pfnDrawConsoleString)(x, y, text.as_ptr()) }
    }
}

impl EngineNet for UiEngine {
    fn net_api(&self) -> &NetApi {
        &self.net_api
    }
}
