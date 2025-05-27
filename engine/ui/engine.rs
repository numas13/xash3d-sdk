use core::{
    ffi::{c_char, c_int, c_uint, c_void, CStr},
    fmt::{self, Write},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr, slice,
};

use csz::{CStrArray, CStrThin};
use shared::borrow::{BorrowRef, Ref};

use crate::{
    color::RGBA,
    consts::{MAX_STRING, MAX_SYSPATH},
    cvar::{CVarFlags, CVarPtr},
    raw::{self, kbutton_t, net_api_s, netadr_s, wrect_s, HIMAGE},
    utils::str::{AsPtr, ToEngineStr},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: c_int,
    pub y: c_int,
}

impl Point {
    pub const fn new(x: c_int, y: c_int) -> Self {
        Self { x, y }
    }

    pub const fn components(&self) -> (c_int, c_int) {
        (self.x, self.y)
    }
}

impl From<Size> for Point {
    fn from(size: Size) -> Self {
        Self::new(size.width, size.height)
    }
}

impl From<(c_int, c_int)> for Point {
    fn from((x, y): (c_int, c_int)) -> Self {
        Self::new(x, y)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Size {
    pub width: c_int,
    pub height: c_int,
}

impl Size {
    pub const fn new(width: c_int, height: c_int) -> Self {
        Self { width, height }
    }

    pub const fn components(&self) -> (c_int, c_int) {
        (self.width, self.height)
    }
}

impl From<Point> for Size {
    fn from(point: Point) -> Self {
        Self::new(point.x, point.y)
    }
}

impl From<(c_int, c_int)> for Size {
    fn from((w, h): (c_int, c_int)) -> Self {
        Self::new(w, h)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum ActiveMenu {
    Console,
    Game,
    Menu,
}

impl TryFrom<c_int> for ActiveMenu {
    type Error = ();

    fn try_from(value: c_int) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Console),
            1 => Ok(Self::Game),
            2 => Ok(Self::Menu),
            _ => Err(()),
        }
    }
}

#[derive(Default)]
struct Borrows {
    keynum_to_str: BorrowRef,
    addr_to_string: BorrowRef,
}

pub struct Engine {
    raw: raw::ui_enginefuncs_s,
    ext: raw::ui_extendedfuncs_s,
    borrows: Borrows,
}

macro_rules! unwrap {
    ($self:expr, ext.net.$name:ident) => {
        match $self.net_api().unwrap().$name {
            Some(func) => func,
            None => panic!("ui_extendedfuncs_s.net_api.{} is null", stringify!($name)),
        }
    };
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

impl Engine {
    pub(crate) fn new(raw: raw::ui_enginefuncs_s) -> Self {
        Self {
            raw,
            ext: Default::default(),
            borrows: Default::default(),
        }
    }

    pub(crate) fn set_extended(&mut self, ext: raw::ui_extendedfuncs_s) {
        self.ext = ext;
    }

    pub fn pic_load(&self, path: impl ToEngineStr, buf: Option<&[u8]>, flags: u32) -> HIMAGE {
        let path = path.to_engine_str();
        let (buf, len) = buf
            .map(|i| (i.as_ptr(), i.len()))
            .unwrap_or((ptr::null(), 0));
        unsafe { unwrap!(self, pfnPIC_Load)(path.as_ptr(), buf, len as c_int, flags as c_int) }
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

    // pub pfnPIC_EnableScissor:
    //     Option<unsafe extern "C" fn(x: c_int, y: c_int, width: c_int, height: c_int)>,
    // pub pfnPIC_DisableScissor: Option<unsafe extern "C" fn()>,

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
            unwrap!(self, pfnRegisterVariable)(name.as_ptr(), value.as_ptr(), flags.bits())
        };
        if !ptr.is_null() {
            Some(CVarPtr::from_ptr(ptr))
        } else {
            None
        }
    }

    pub fn get_cvar_float(&self, name: impl ToEngineStr) -> f32 {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, pfnGetCvarFloat)(name.as_ptr()) }
    }

    pub fn get_cvar_string(&self, name: impl ToEngineStr) -> &CStrThin {
        let name = name.to_engine_str();
        // FIXME: A lifetime of returned string is valid till
        // set_cvar_string call with the same name and can be
        // changed anywhere in C code.
        let ptr = unsafe { unwrap!(self, pfnGetCvarString)(name.as_ptr()) };
        assert!(!ptr.is_null());
        unsafe { CStrThin::from_ptr(ptr) }
    }

    pub fn cvar<T: CVar>(&self, name: &CStr) -> T {
        CVar::get(self, name)
    }

    pub fn cvar_set<T: CVar>(&self, name: &CStr, value: T) {
        CVar::set(&value, self, name)
    }

    pub fn set_cvar_string(&self, name: impl ToEngineStr, value: impl ToEngineStr) {
        let name = name.to_engine_str();
        let value = value.to_engine_str();
        unsafe {
            unwrap!(self, pfnCvarSetString)(name.as_ptr(), value.as_ptr());
        }
    }

    pub fn set_cvar_float(&self, name: impl ToEngineStr, value: f32) {
        let name = name.to_engine_str();
        unsafe {
            unwrap!(self, pfnCvarSetValue)(name.as_ptr(), value);
        }
    }

    pub fn add_command(
        &self,
        cmd_name: impl ToEngineStr,
        function: Option<unsafe extern "C" fn()>,
    ) -> c_int {
        unsafe { unwrap!(self, pfnAddCommand)(cmd_name.to_engine_str().as_ptr(), function) }
    }

    pub fn client_cmd(&self, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnClientCmd)(0, cmd.as_ptr()) }
    }

    pub fn client_cmd_now(&self, cmd: impl ToEngineStr) {
        let cmd = cmd.to_engine_str();
        unsafe { unwrap!(self, pfnClientCmd)(1, cmd.as_ptr()) }
    }

    fn client_cmdf_impl(&self, now: bool, cmd: fmt::Arguments) {
        let mut buf = CStrArray::<1024>::new();
        buf.cursor().write_fmt(cmd).unwrap();
        unsafe { unwrap!(self, pfnClientCmd)(now as c_int, buf.as_ptr()) }
    }

    pub fn client_cmdf(&self, cmd: fmt::Arguments) {
        self.client_cmdf_impl(false, cmd);
    }

    pub fn client_cmdf_now(&self, cmd: fmt::Arguments) {
        self.client_cmdf_impl(true, cmd);
    }

    pub fn delete_command(&self, cmd_name: impl ToEngineStr) {
        let cmd_name = cmd_name.to_engine_str();
        unsafe {
            unwrap!(self, pfnDelCommand)(cmd_name.as_ptr());
        }
    }

    pub fn cmd_argc(&self) -> usize {
        unsafe { unwrap!(self, pfnCmdArgc)() as usize }
    }

    pub fn cmd_argv(&self, index: usize) -> &CStrThin {
        let ptr = unsafe { unwrap!(self, pfnCmdArgv)(index as c_int) };
        assert!(!ptr.is_null());
        unsafe { CStrThin::from_ptr(ptr) }
    }

    pub fn cmd_args(&self) -> Option<&CStrThin> {
        let ptr = unsafe { unwrap!(self, pfnCmd_Args)() };
        if !ptr.is_null() {
            Some(unsafe { CStrThin::from_ptr(ptr) })
        } else {
            None
        }
    }

    pub fn con_print(&self, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, Con_Printf)(c"%s".as_ptr(), msg.as_ptr());
        }
    }

    pub fn con_printf(&self, args: fmt::Arguments<'_>) -> fmt::Result {
        let mut buf = CStrArray::<8192>::new();
        buf.cursor().write_fmt(args)?;
        self.con_print(buf.as_thin());
        Ok(())
    }

    pub fn con_dprint(&self, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, Con_DPrintf)(c"%s".as_ptr(), msg.as_ptr());
        }
    }

    pub fn con_dprintf(&self, args: fmt::Arguments<'_>) -> fmt::Result {
        let mut buf = CStrArray::<8192>::new();
        buf.cursor().write_fmt(args)?;
        self.con_dprint(buf.as_thin());
        Ok(())
    }

    pub fn con_nprint(&self, pos: c_int, msg: impl ToEngineStr) {
        let msg = msg.to_engine_str();
        unsafe {
            unwrap!(self, Con_NPrintf)(pos, c"%s".as_ptr(), msg.as_ptr());
        }
    }

    pub fn con_nprintf(&self, pos: c_int, args: fmt::Arguments<'_>) -> fmt::Result {
        let mut buf = CStrArray::<8192>::new();
        buf.cursor().write_fmt(args)?;
        self.con_nprint(pos, buf.as_thin());
        Ok(())
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

    pub fn draw_console_string(&self, x: c_int, y: c_int, text: impl ToEngineStr) -> c_int {
        let text = text.to_engine_str();
        unsafe { unwrap!(self, pfnDrawConsoleString)(x, y, text.as_ptr()) }
    }

    // pub pfnDrawSetTextColor:
    //     Option<unsafe extern "C" fn(r: c_int, g: c_int, b: c_int, alpha: c_int)>,
    // pub pfnDrawConsoleStringLen:
    //     Option<unsafe extern "C" fn(string: *const c_char, length: *mut c_int, height: *mut c_int)>,
    // pub pfnSetConsoleDefaultColor: Option<unsafe extern "C" fn(r: c_int, g: c_int, b: c_int)>,
    // pub pfnGetPlayerModel: Option<unsafe extern "C" fn() -> *mut cl_entity_s>,
    // pub pfnSetModel: Option<unsafe extern "C" fn(ed: *mut cl_entity_s, path: *const c_char)>,
    // pub pfnClearScene: Option<unsafe extern "C" fn()>,
    // pub pfnRenderScene: Option<unsafe extern "C" fn(rvp: *const ffi::ref_viewpass_s)>,
    // pub CL_CreateVisibleEntity:
    //     Option<unsafe extern "C" fn(type_: c_int, ent: *mut cl_entity_s) -> c_int>,

    pub fn host_error(&self, args: fmt::Arguments<'_>) -> ! {
        let mut buf = CStrArray::<4096>::new();
        buf.cursor().write_fmt(args).unwrap();
        unsafe {
            unwrap!(self, pfnHostError)(c"%s".as_ptr(), buf.as_ptr());
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
        self.client_in_game() && self.cvar::<f32>(c"cl_background") == 0.0
    }

    // pub pfnClientJoin: Option<unsafe extern "C" fn(adr: netadr_s)>,

    pub fn load_file(&self, path: impl ToEngineStr) -> Option<File> {
        let path = path.to_engine_str();
        let mut len = 0;
        let data = unsafe { unwrap!(self, COM_LoadFile)(path.as_ptr(), &mut len) };
        if !data.is_null() {
            Some(File {
                data: data.cast(),
                len: len as usize,
            })
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
    /// Buffer must be allocated with [load_file].
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

    pub fn keynum_to_str(&self, keynum: c_int) -> Ref<CStrThin> {
        // SAFETY: The returned string is allocated in a private static buffer
        // in that function. Never returns a null pointer.
        unsafe {
            let s = unwrap!(self, pfnKeynumToString)(keynum);
            self.borrows.keynum_to_str.borrow(s as *mut CStrThin)
        }
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
            Some(GameInfo {
                raw: unsafe { raw.assume_init() },
            })
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

    pub fn get_files_list(&self, pattern: impl ToEngineStr, gamedironly: bool) -> FileList {
        let pattern = pattern.to_engine_str();
        let mut len = 0;
        let func = unwrap!(self, pfnGetFilesList);
        let data = unsafe { func(pattern.as_ptr(), &mut len, gamedironly as c_int) };
        let raw = if !data.is_null() {
            unsafe { slice::from_raw_parts(data.cast(), len as usize) }
        } else {
            &[]
        };
        FileList { raw }
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

    pub fn rand_f32(&self, start: f32, end: f32) -> f32 {
        unsafe { unwrap!(self, pfnRandomFloat)(start, end) }
    }

    pub fn rand_int(&self, start: c_int, end: c_int) -> c_int {
        unsafe { unwrap!(self, pfnRandomLong)(start, end) }
    }

    // pub pfnSetCursor: Option<unsafe extern "C" fn(hCursor: *mut c_void)>,

    pub fn is_map_valid(&self, filename: impl ToEngineStr) -> bool {
        let filename = filename.to_engine_str();
        unsafe { unwrap!(self, pfnIsMapValid)(filename.as_ptr()) != 0 }
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

    pub fn time_f64(&self) -> f64 {
        unsafe { unwrap!(self, ext.pfnDoubleTime)() }
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

    pub fn addr_to_string(&self, addr: netadr_s) -> Ref<CStrThin> {
        // SAFETY: The returned string is allocated in a private static buffer
        // in that function. Never returns a null pointer.
        unsafe {
            let s = unwrap!(self, ext.pfnAdrToString)(addr);
            self.borrows.addr_to_string.borrow(s as *mut CStrThin)
        }
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

    fn net_api(&self) -> Option<&net_api_s> {
        if !self.ext.pNetAPI.is_null() {
            Some(unsafe { &*self.ext.pNetAPI })
        } else {
            None
        }
    }

    // pub InitNetworking: Option<unsafe extern "C" fn()>,
    // pub Status: Option<unsafe extern "C" fn(status: *mut net_status_s)>,
    // pub SendRequest: Option<
    //     unsafe extern "C" fn(
    //         context: c_int,
    //         request: c_int,
    //         flags: c_int,
    //         timeout: f64,
    //         remote_address: *mut netadr_s,
    //         response: net_api_response_func_t,
    //     ),
    // >,
    // pub CancelRequest: Option<unsafe extern "C" fn(context: c_int)>,
    // pub CancelAllRequests: Option<unsafe extern "C" fn()>,

    pub fn addr_to_string_ref(&self, addr: &netadr_s) -> Ref<CStrThin> {
        // SAFETY: The returned string is allocated in a private static buffer
        // in that function. Never returns a null pointer.
        unsafe {
            // XXX: uses pfnAdrToString under the hood
            let s = unwrap!(self, ext.net.AdrToString)(addr);
            self.borrows.addr_to_string.borrow(s as *mut CStrThin)
        }
    }

    pub fn compare_addr(&self, a: &netadr_s, b: &netadr_s) -> bool {
        unsafe { unwrap!(self, ext.net.CompareAdr)(a, b) != 0 }
    }

    pub fn string_to_addr(&self, s: impl ToEngineStr) -> Option<netadr_s> {
        let s = s.to_engine_str();
        let mut netadr_s = MaybeUninit::uninit();
        let res = unsafe { unwrap!(self, ext.net.StringToAdr)(s.as_ptr(), netadr_s.as_mut_ptr()) };
        if res != 0 {
            Some(unsafe { netadr_s.assume_init() })
        } else {
            None
        }
    }

    // pub ValueForKey:
    //     Option<unsafe extern "C" fn(s: *const c_char, key: *const c_char) -> *const c_char>,
    // pub RemoveKey: Option<unsafe extern "C" fn(s: *mut c_char, key: *const c_char)>,
    // pub SetValueForKey: Option<
    //     unsafe extern "C" fn(
    //         s: *mut c_char,
    //         key: *const c_char,
    //         value: *const c_char,
    //         maxsize: c_int,
    //     ),
    // >,
}

pub struct File {
    data: *mut u8,
    len: usize,
}

impl File {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }

    pub fn cursor(&mut self) -> Cursor {
        unsafe { Cursor::from_ptr(self.data) }
    }
}

impl Deref for File {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for File {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            super::engine().free_file(self.data.cast());
        }
    }
}

pub struct Cursor<'a> {
    data: *mut u8,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> Cursor<'a> {
    /// Creates cursor from a raw pointer.
    ///
    /// # Safety
    ///
    /// `data` must contain a nul-byte at the end.
    pub unsafe fn from_ptr(data: *mut u8) -> Self {
        Self {
            data,
            phantom: PhantomData,
        }
    }

    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        assert_eq!(slice.last(), Some(&0));
        unsafe { Self::from_ptr(slice.as_mut_ptr()) }
    }
}

pub struct FileList<'a> {
    raw: &'a [*const c_char],
}

impl<'a> FileList<'a> {
    fn as_slice(&self) -> &'a [*const c_char] {
        self.raw
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a CStrThin> {
        self.as_slice()
            .iter()
            .map(|i| unsafe { CStrThin::from_ptr(*i) })
    }
}

#[derive(Clone)]
pub struct GameInfo {
    raw: raw::GAMEINFO,
}

impl GameInfo {
    pub fn game_folder(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.raw.gamefolder.as_ptr())
                .to_str()
                .unwrap()
        }
    }

    pub fn start_map(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.startmap.as_ptr()).to_str().unwrap() }
    }

    pub fn train_map(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.trainmap.as_ptr()).to_str().unwrap() }
    }

    pub fn title(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.title.as_ptr()).to_str().unwrap() }
    }

    pub fn version(&self) -> Option<&str> {
        self.raw.version.to_str().ok()
    }

    #[inline(always)]
    pub fn flags(&self) -> u16 {
        self.raw.flags as u16
    }

    pub fn game_url(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.game_url.as_ptr()).to_str().unwrap() }
    }

    pub fn update_url(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.raw.update_url.as_ptr())
                .to_str()
                .unwrap()
        }
    }

    pub fn type_(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.type_.as_ptr()).to_str().unwrap() }
    }

    pub fn date(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.date.as_ptr()).to_str().unwrap() }
    }

    pub fn size(&self) -> &str {
        unsafe { CStr::from_ptr(self.raw.size.as_ptr()).to_str().unwrap() }
    }

    #[inline(always)]
    pub fn game_mode(&self) -> u32 {
        self.raw.gamemode as u32
    }
}

impl fmt::Debug for GameInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("GameInfo")
            .field("gamefolder", &self.game_folder())
            .field("startmap", &self.start_map())
            .field("trainmap", &self.train_map())
            .field("title", &self.title())
            .field("version", &self.version())
            .field("flags", &self.flags())
            .field("game_url", &self.game_url())
            .field("update_url", &self.update_url())
            .field("type", &self.type_())
            .field("date", &self.date())
            .field("size", &self.size())
            .field("gamemode", &self.game_mode())
            .finish()
    }
}

pub trait CVar {
    fn get(eng: &Engine, name: &CStr) -> Self;
    fn set(&self, eng: &Engine, name: &CStr);
}

impl CVar for bool {
    fn get(eng: &Engine, name: &CStr) -> Self {
        eng.get_cvar_float(name) != 0.0
    }

    fn set(&self, eng: &Engine, name: &CStr) {
        eng.set_cvar_float(name, *self as u32 as f32);
    }
}

impl CVar for usize {
    fn get(eng: &Engine, name: &CStr) -> Self {
        eng.get_cvar_float(name) as usize
    }

    fn set(&self, eng: &Engine, name: &CStr) {
        eng.set_cvar_float(name, *self as f32);
    }
}

impl CVar for f32 {
    fn get(eng: &Engine, name: &CStr) -> Self {
        eng.get_cvar_float(name)
    }

    fn set(&self, eng: &Engine, name: &CStr) {
        eng.set_cvar_float(name, *self);
    }
}

impl CVar for &CStr {
    fn get(eng: &Engine, name: &CStr) -> Self {
        eng.get_cvar_string(name).into()
    }

    fn set(&self, eng: &Engine, name: &CStr) {
        eng.set_cvar_string(name, *self)
    }
}
