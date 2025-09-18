use core::{
    ffi::{c_char, c_int, c_uchar},
    marker::PhantomData,
    slice,
};

use csz::CStrThin;
use shared::{
    engine::net::netadr_s,
    ffi::{
        self,
        menu::{UI_EXTENDED_FUNCTIONS, UI_FUNCTIONS},
    },
};

use crate::{color::RGBA, engine::UiEngineRef};

pub use shared::export::{impl_unsync_global, UnsyncGlobal};

#[allow(unused_variables)]
pub trait UiDll: UnsyncGlobal {
    fn new(engine: UiEngineRef) -> Self;

    fn vid_init(&self) -> bool {
        true
    }

    fn redraw(&self, time: f32);

    fn set_active_menu(&self, active: bool);

    fn is_visible(&self) -> bool;

    fn key_event(&self, key: c_int, down: bool) {}

    fn mouse_move(&self, x: c_int, y: c_int) {}

    fn add_server_to_list(&self, addr: netadr_s, info: &CStrThin) {}

    fn get_cursor_pos(&self) -> (c_int, c_int) {
        (0, 0)
    }

    fn set_cursor_pos(&self, x: c_int, y: c_int) {}

    fn show_cursor(&self, show: bool) {}

    fn char_event(&self, key: c_int) {}

    fn mouse_in_rect(&self) -> bool {
        false
    }

    fn credits_active(&self) -> bool {
        false
    }

    fn final_credits(&self) {}

    fn add_touch_button_to_list(
        &self,
        name: &CStrThin,
        texture: &CStrThin,
        command: &CStrThin,
        color: RGBA,
        flags: c_int,
    ) {
    }

    fn reset_ping(&self) {}

    fn show_connection_warning(&self) {}

    fn show_update_dialog(&self, prefer_store: c_int) {}

    fn show_message_box(&self, text: &CStrThin) {}

    fn connection_progress_disconnect(&self) {}

    fn connection_progress_download(
        &self,
        file_name: &CStrThin,
        server_name: &CStrThin,
        current: c_int,
        total: c_int,
        comment: &CStrThin,
    ) {
    }

    fn connection_process_download_end(&self) {}

    fn connection_progress_precache(&self) {}

    fn connection_progress_connect(&self, server: &CStrThin) {}

    fn connection_progress_change_level(&self) {}

    fn connection_progress_parse_server_info(&self, server: &CStrThin) {}
}

pub fn ui_functions<T: UiDll>() -> UI_FUNCTIONS {
    Export::<T>::ui_functions()
}

pub fn ui_extended_functions<T: UiDll>() -> UI_EXTENDED_FUNCTIONS {
    Export::<T>::ui_extended_functions()
}

#[allow(clippy::missing_safety_doc)]
trait UiDllExport {
    fn ui_functions() -> UI_FUNCTIONS {
        UI_FUNCTIONS {
            pfnVidInit: Some(Self::vid_init),
            pfnInit: Some(Self::init),
            pfnShutdown: Some(Self::shutdown),
            pfnRedraw: Some(Self::redraw),
            pfnKeyEvent: Some(Self::key_event),
            pfnMouseMove: Some(Self::mouse_move),
            pfnSetActiveMenu: Some(Self::set_active_menu),
            pfnAddServerToList: Some(Self::add_server_to_list),
            pfnGetCursorPos: Some(Self::get_cursor_pos),
            pfnSetCursorPos: Some(Self::set_cursor_pos),
            pfnShowCursor: Some(Self::show_cursor),
            pfnCharEvent: Some(Self::char_event),
            pfnMouseInRect: Some(Self::mouse_in_rect),
            pfnIsVisible: Some(Self::is_visible),
            pfnCreditsActive: Some(Self::credits_active),
            pfnFinalCredits: Some(Self::final_credits),
        }
    }

    unsafe extern "C" fn init();

    unsafe extern "C" fn shutdown();

    unsafe extern "C" fn vid_init() -> c_int;

    unsafe extern "C" fn redraw(time: f32);

    unsafe extern "C" fn key_event(key: c_int, down: c_int);

    unsafe extern "C" fn mouse_move(x: c_int, y: c_int);

    unsafe extern "C" fn set_active_menu(active: c_int);

    unsafe extern "C" fn add_server_to_list(adr: netadr_s, info: *const c_char);

    unsafe extern "C" fn get_cursor_pos(x: *mut c_int, y: *mut c_int);

    unsafe extern "C" fn set_cursor_pos(x: c_int, y: c_int);

    unsafe extern "C" fn show_cursor(show: c_int);

    unsafe extern "C" fn char_event(key: c_int);

    unsafe extern "C" fn mouse_in_rect() -> c_int;

    unsafe extern "C" fn is_visible() -> c_int;

    unsafe extern "C" fn credits_active() -> c_int;

    unsafe extern "C" fn final_credits();

    fn ui_extended_functions() -> UI_EXTENDED_FUNCTIONS {
        UI_EXTENDED_FUNCTIONS {
            pfnAddTouchButtonToList: Some(Self::add_touch_button_to_list),
            pfnResetPing: Some(Self::reset_ping),
            pfnShowConnectionWarning: Some(Self::show_connection_warning),
            pfnShowUpdateDialog: Some(Self::show_update_dialog),
            pfnShowMessageBox: Some(Self::show_message_box),
            pfnConnectionProgress_Disconnect: Some(Self::connection_progress_disconnect),
            pfnConnectionProgress_Download: Some(Self::connection_progress_download),
            pfnConnectionProgress_DownloadEnd: Some(Self::connection_process_download_end),
            pfnConnectionProgress_Precache: Some(Self::connection_progress_precache),
            pfnConnectionProgress_Connect: Some(Self::connection_progress_connect),
            pfnConnectionProgress_ChangeLevel: Some(Self::connection_progress_change_level),
            pfnConnectionProgress_ParseServerInfo: Some(
                Self::connection_progress_parse_server_info,
            ),
        }
    }

    unsafe extern "C" fn add_touch_button_to_list(
        name: *const c_char,
        texture: *const c_char,
        command: *const c_char,
        color: *mut c_uchar,
        flags: c_int,
    );

    unsafe extern "C" fn reset_ping();

    unsafe extern "C" fn show_connection_warning();

    unsafe extern "C" fn show_update_dialog(prefer_store: c_int);

    unsafe extern "C" fn show_message_box(text: *const c_char);

    unsafe extern "C" fn connection_progress_disconnect();

    unsafe extern "C" fn connection_progress_download(
        file_name: *const c_char,
        server_name: *const c_char,
        current: c_int,
        total: c_int,
        comment: *const c_char,
    );

    unsafe extern "C" fn connection_process_download_end();

    unsafe extern "C" fn connection_progress_precache();

    unsafe extern "C" fn connection_progress_connect(server: *const c_char);

    unsafe extern "C" fn connection_progress_change_level();

    unsafe extern "C" fn connection_progress_parse_server_info(server: *const c_char);
}

struct Export<T> {
    phantom: PhantomData<T>,
}

impl<T: UiDll> UiDllExport for Export<T> {
    unsafe extern "C" fn init() {
        unsafe {
            let engine = UiEngineRef::new();
            (&mut *T::global_as_mut_ptr()).write(T::new(engine));
        }
    }

    unsafe extern "C" fn shutdown() {
        unsafe {
            (&mut *T::global_as_mut_ptr()).assume_init_drop();
        }
    }

    unsafe extern "C" fn vid_init() -> c_int {
        unsafe { T::global_assume_init_ref() }.vid_init() as c_int
    }

    unsafe extern "C" fn redraw(time: f32) {
        unsafe { T::global_assume_init_ref() }.redraw(time);
    }

    unsafe extern "C" fn key_event(key: c_int, down: c_int) {
        unsafe { T::global_assume_init_ref() }.key_event(key, down != 0);
    }

    unsafe extern "C" fn mouse_move(x: c_int, y: c_int) {
        unsafe { T::global_assume_init_ref() }.mouse_move(x, y);
    }

    unsafe extern "C" fn set_active_menu(active: c_int) {
        unsafe { T::global_assume_init_ref() }.set_active_menu(active != 0);
    }

    unsafe extern "C" fn add_server_to_list(adr: netadr_s, info: *const c_char) {
        assert!(!info.is_null());
        let info = unsafe { CStrThin::from_ptr(info) };
        unsafe { T::global_assume_init_ref() }.add_server_to_list(adr, info);
    }

    unsafe extern "C" fn get_cursor_pos(x: *mut c_int, y: *mut c_int) {
        let ret = unsafe { T::global_assume_init_ref() }.get_cursor_pos();
        if !x.is_null() {
            unsafe {
                *x = ret.0;
            }
        }
        if !y.is_null() {
            unsafe {
                *y = ret.1;
            }
        }
    }

    unsafe extern "C" fn set_cursor_pos(x: c_int, y: c_int) {
        unsafe { T::global_assume_init_ref() }.set_cursor_pos(x, y)
    }

    unsafe extern "C" fn show_cursor(show: c_int) {
        unsafe { T::global_assume_init_ref() }.show_cursor(show != 0);
    }

    unsafe extern "C" fn char_event(key: c_int) {
        unsafe { T::global_assume_init_ref() }.char_event(key);
    }

    unsafe extern "C" fn mouse_in_rect() -> c_int {
        unsafe { T::global_assume_init_ref() }.mouse_in_rect() as c_int
    }

    unsafe extern "C" fn is_visible() -> c_int {
        unsafe { T::global_assume_init_ref() }.is_visible() as c_int
    }

    unsafe extern "C" fn credits_active() -> c_int {
        unsafe { T::global_assume_init_ref() }.credits_active() as c_int
    }

    unsafe extern "C" fn final_credits() {
        unsafe { T::global_assume_init_ref() }.final_credits();
    }

    unsafe extern "C" fn add_touch_button_to_list(
        name: *const c_char,
        texture: *const c_char,
        command: *const c_char,
        color: *mut c_uchar,
        flags: c_int,
    ) {
        assert!(!name.is_null() && !texture.is_null() && !command.is_null() && !color.is_null());
        let name = unsafe { CStrThin::from_ptr(name) };
        let texture = unsafe { CStrThin::from_ptr(texture) };
        let command = unsafe { CStrThin::from_ptr(command) };
        let color = unsafe { slice::from_raw_parts(color as *const u8, 4) };
        let color = RGBA::new(color[0], color[1], color[2], color[3]);
        unsafe { T::global_assume_init_ref() }
            .add_touch_button_to_list(name, texture, command, color, flags);
    }

    unsafe extern "C" fn reset_ping() {
        unsafe { T::global_assume_init_ref() }.reset_ping();
    }

    unsafe extern "C" fn show_connection_warning() {
        unsafe { T::global_assume_init_ref() }.show_connection_warning();
    }

    unsafe extern "C" fn show_update_dialog(prefer_store: c_int) {
        unsafe { T::global_assume_init_ref() }.show_update_dialog(prefer_store);
    }

    unsafe extern "C" fn show_message_box(text: *const c_char) {
        assert!(!text.is_null());
        let text = unsafe { CStrThin::from_ptr(text) };
        unsafe { T::global_assume_init_ref() }.show_message_box(text)
    }

    unsafe extern "C" fn connection_progress_disconnect() {
        unsafe { T::global_assume_init_ref() }.connection_progress_disconnect();
    }

    unsafe extern "C" fn connection_progress_download(
        file_name: *const c_char,
        server_name: *const c_char,
        current: c_int,
        total: c_int,
        comment: *const c_char,
    ) {
        assert!(!file_name.is_null() && !server_name.is_null() && !comment.is_null());
        let file_name = unsafe { CStrThin::from_ptr(file_name) };
        let server_name = unsafe { CStrThin::from_ptr(server_name) };
        let comment = unsafe { CStrThin::from_ptr(comment) };
        unsafe { T::global_assume_init_ref() }.connection_progress_download(
            file_name,
            server_name,
            current,
            total,
            comment,
        )
    }

    unsafe extern "C" fn connection_process_download_end() {
        unsafe { T::global_assume_init_ref() }.connection_process_download_end();
    }

    unsafe extern "C" fn connection_progress_precache() {
        unsafe { T::global_assume_init_ref() }.connection_progress_precache();
    }

    unsafe extern "C" fn connection_progress_connect(server: *const c_char) {
        assert!(!server.is_null());
        let server = unsafe { CStrThin::from_ptr(server) };
        unsafe { T::global_assume_init_ref() }.connection_progress_connect(server)
    }

    unsafe extern "C" fn connection_progress_change_level() {
        unsafe { T::global_assume_init_ref() }.connection_progress_change_level();
    }

    unsafe extern "C" fn connection_progress_parse_server_info(server: *const c_char) {
        assert!(!server.is_null());
        let server = unsafe { CStrThin::from_ptr(server) };
        unsafe { T::global_assume_init_ref() }.connection_progress_parse_server_info(server);
    }
}

/// Initialize the global engine instance and returns exported functions.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn get_menu_api<T: UiDll>(
    dll_funcs: Option<&mut UI_FUNCTIONS>,
    eng_funcs: Option<&ffi::menu::ui_enginefuncs_s>,
    globals: *mut ffi::menu::ui_globalvars_s,
) -> c_int {
    let Some(dll_funcs) = dll_funcs else { return 0 };
    let Some(eng_funcs) = eng_funcs else { return 0 };
    if globals.is_null() {
        return 0;
    }
    unsafe {
        crate::instance::init_engine(eng_funcs, globals);
    }
    *dll_funcs = ui_functions::<T>();
    1
}

/// Initialize extended functions and returns extended exported functions.
///
/// # Safety
///
/// Must be called only once after [get_menu_api].
pub unsafe fn get_ext_api<T: UiDll>(
    version: c_int,
    dll_funcs: Option<&mut ffi::menu::UI_EXTENDED_FUNCTIONS>,
    eng_funcs: Option<&ffi::menu::ui_extendedfuncs_s>,
) -> c_int {
    let expected = ffi::menu::MENU_EXTENDED_API_VERSION as c_int;
    if version != expected {
        // FIXME: logger is not initialized yet
        error!("GetExtAPI: unsupported version (engine {version}, menu {expected})");
        return 0;
    }

    let Some(dll_funcs) = dll_funcs else { return 0 };
    let Some(eng_funcs) = eng_funcs else {
        return 0;
    };

    unsafe {
        crate::instance::init_engine_ext(eng_funcs);
    }

    *dll_funcs = ui_extended_functions::<T>();
    1
}

#[doc(hidden)]
#[macro_export]
macro_rules! export_dll {
    ($ui_dll:ty $(, pre $pre:block)? $(, post $post:block)?) => {
        #[no_mangle]
        pub unsafe extern "C" fn GetMenuAPI(
            dll_funcs: Option<&mut $crate::ffi::menu::UI_FUNCTIONS>,
            eng_funcs: Option<&$crate::ffi::menu::ui_enginefuncs_s>,
            globals: *mut $crate::ffi::menu::ui_globalvars_s,
        ) -> core::ffi::c_int {
            $($pre)?
            let result = unsafe {
                $crate::export::get_menu_api::<$ui_dll>(dll_funcs, eng_funcs, globals)
            };
            $($post)?
            result
        }

        #[no_mangle]
        pub unsafe extern "C" fn GetExtAPI(
            version: core::ffi::c_int,
            dll_funcs: Option<&mut $crate::ffi::menu::UI_EXTENDED_FUNCTIONS>,
            eng_funcs: Option<&$crate::ffi::menu::ui_extendedfuncs_s>,
        ) -> core::ffi::c_int {
            unsafe {
                $crate::export::get_ext_api::<$ui_dll>(version, dll_funcs, eng_funcs)
            }
        }
    };
}
#[doc(inline)]
pub use export_dll;
