use core::ffi::{c_char, c_int, c_uint, c_void};

use shared::raw::{byte, qboolean, vec2_t};

pub const VGUI_DefaultCursor_dc_user: VGUI_DefaultCursor = 0;
pub const VGUI_DefaultCursor_dc_none: VGUI_DefaultCursor = 1;
pub const VGUI_DefaultCursor_dc_arrow: VGUI_DefaultCursor = 2;
pub const VGUI_DefaultCursor_dc_ibeam: VGUI_DefaultCursor = 3;
pub const VGUI_DefaultCursor_dc_hourglass: VGUI_DefaultCursor = 4;
pub const VGUI_DefaultCursor_dc_crosshair: VGUI_DefaultCursor = 5;
pub const VGUI_DefaultCursor_dc_up: VGUI_DefaultCursor = 6;
pub const VGUI_DefaultCursor_dc_sizenwse: VGUI_DefaultCursor = 7;
pub const VGUI_DefaultCursor_dc_sizenesw: VGUI_DefaultCursor = 8;
pub const VGUI_DefaultCursor_dc_sizewe: VGUI_DefaultCursor = 9;
pub const VGUI_DefaultCursor_dc_sizens: VGUI_DefaultCursor = 10;
pub const VGUI_DefaultCursor_dc_sizeall: VGUI_DefaultCursor = 11;
pub const VGUI_DefaultCursor_dc_no: VGUI_DefaultCursor = 12;
pub const VGUI_DefaultCursor_dc_hand: VGUI_DefaultCursor = 13;
pub const VGUI_DefaultCursor_dc_last: VGUI_DefaultCursor = 14;
pub type VGUI_DefaultCursor = c_uint;

#[repr(C)]
pub struct vpoint_t {
    pub point: vec2_t,
    pub coord: vec2_t,
}

pub const VGUI_MouseCode_MOUSE_LEFT: VGUI_MouseCode = 0;
pub const VGUI_MouseCode_MOUSE_RIGHT: VGUI_MouseCode = 1;
pub const VGUI_MouseCode_MOUSE_MIDDLE: VGUI_MouseCode = 2;
pub const VGUI_MouseCode_MOUSE_LAST: VGUI_MouseCode = 3;
pub type VGUI_MouseCode = c_uint;

pub const VGUI_KeyCode_KEY_0: VGUI_KeyCode = 0;
pub const VGUI_KeyCode_KEY_1: VGUI_KeyCode = 1;
pub const VGUI_KeyCode_KEY_2: VGUI_KeyCode = 2;
pub const VGUI_KeyCode_KEY_3: VGUI_KeyCode = 3;
pub const VGUI_KeyCode_KEY_4: VGUI_KeyCode = 4;
pub const VGUI_KeyCode_KEY_5: VGUI_KeyCode = 5;
pub const VGUI_KeyCode_KEY_6: VGUI_KeyCode = 6;
pub const VGUI_KeyCode_KEY_7: VGUI_KeyCode = 7;
pub const VGUI_KeyCode_KEY_8: VGUI_KeyCode = 8;
pub const VGUI_KeyCode_KEY_9: VGUI_KeyCode = 9;
pub const VGUI_KeyCode_KEY_A: VGUI_KeyCode = 10;
pub const VGUI_KeyCode_KEY_B: VGUI_KeyCode = 11;
pub const VGUI_KeyCode_KEY_C: VGUI_KeyCode = 12;
pub const VGUI_KeyCode_KEY_D: VGUI_KeyCode = 13;
pub const VGUI_KeyCode_KEY_E: VGUI_KeyCode = 14;
pub const VGUI_KeyCode_KEY_F: VGUI_KeyCode = 15;
pub const VGUI_KeyCode_KEY_G: VGUI_KeyCode = 16;
pub const VGUI_KeyCode_KEY_H: VGUI_KeyCode = 17;
pub const VGUI_KeyCode_KEY_I: VGUI_KeyCode = 18;
pub const VGUI_KeyCode_KEY_J: VGUI_KeyCode = 19;
pub const VGUI_KeyCode_KEY_K: VGUI_KeyCode = 20;
pub const VGUI_KeyCode_KEY_L: VGUI_KeyCode = 21;
pub const VGUI_KeyCode_KEY_M: VGUI_KeyCode = 22;
pub const VGUI_KeyCode_KEY_N: VGUI_KeyCode = 23;
pub const VGUI_KeyCode_KEY_O: VGUI_KeyCode = 24;
pub const VGUI_KeyCode_KEY_P: VGUI_KeyCode = 25;
pub const VGUI_KeyCode_KEY_Q: VGUI_KeyCode = 26;
pub const VGUI_KeyCode_KEY_R: VGUI_KeyCode = 27;
pub const VGUI_KeyCode_KEY_S: VGUI_KeyCode = 28;
pub const VGUI_KeyCode_KEY_T: VGUI_KeyCode = 29;
pub const VGUI_KeyCode_KEY_U: VGUI_KeyCode = 30;
pub const VGUI_KeyCode_KEY_V: VGUI_KeyCode = 31;
pub const VGUI_KeyCode_KEY_W: VGUI_KeyCode = 32;
pub const VGUI_KeyCode_KEY_X: VGUI_KeyCode = 33;
pub const VGUI_KeyCode_KEY_Y: VGUI_KeyCode = 34;
pub const VGUI_KeyCode_KEY_Z: VGUI_KeyCode = 35;
pub const VGUI_KeyCode_KEY_PAD_0: VGUI_KeyCode = 36;
pub const VGUI_KeyCode_KEY_PAD_1: VGUI_KeyCode = 37;
pub const VGUI_KeyCode_KEY_PAD_2: VGUI_KeyCode = 38;
pub const VGUI_KeyCode_KEY_PAD_3: VGUI_KeyCode = 39;
pub const VGUI_KeyCode_KEY_PAD_4: VGUI_KeyCode = 40;
pub const VGUI_KeyCode_KEY_PAD_5: VGUI_KeyCode = 41;
pub const VGUI_KeyCode_KEY_PAD_6: VGUI_KeyCode = 42;
pub const VGUI_KeyCode_KEY_PAD_7: VGUI_KeyCode = 43;
pub const VGUI_KeyCode_KEY_PAD_8: VGUI_KeyCode = 44;
pub const VGUI_KeyCode_KEY_PAD_9: VGUI_KeyCode = 45;
pub const VGUI_KeyCode_KEY_PAD_DIVIDE: VGUI_KeyCode = 46;
pub const VGUI_KeyCode_KEY_PAD_MULTIPLY: VGUI_KeyCode = 47;
pub const VGUI_KeyCode_KEY_PAD_MINUS: VGUI_KeyCode = 48;
pub const VGUI_KeyCode_KEY_PAD_PLUS: VGUI_KeyCode = 49;
pub const VGUI_KeyCode_KEY_PAD_ENTER: VGUI_KeyCode = 50;
pub const VGUI_KeyCode_KEY_PAD_DECIMAL: VGUI_KeyCode = 51;
pub const VGUI_KeyCode_KEY_LBRACKET: VGUI_KeyCode = 52;
pub const VGUI_KeyCode_KEY_RBRACKET: VGUI_KeyCode = 53;
pub const VGUI_KeyCode_KEY_SEMICOLON: VGUI_KeyCode = 54;
pub const VGUI_KeyCode_KEY_APOSTROPHE: VGUI_KeyCode = 55;
pub const VGUI_KeyCode_KEY_BACKQUOTE: VGUI_KeyCode = 56;
pub const VGUI_KeyCode_KEY_COMMA: VGUI_KeyCode = 57;
pub const VGUI_KeyCode_KEY_PERIOD: VGUI_KeyCode = 58;
pub const VGUI_KeyCode_KEY_SLASH: VGUI_KeyCode = 59;
pub const VGUI_KeyCode_KEY_BACKSLASH: VGUI_KeyCode = 60;
pub const VGUI_KeyCode_KEY_MINUS: VGUI_KeyCode = 61;
pub const VGUI_KeyCode_KEY_EQUAL: VGUI_KeyCode = 62;
pub const VGUI_KeyCode_KEY_ENTER: VGUI_KeyCode = 63;
pub const VGUI_KeyCode_KEY_SPACE: VGUI_KeyCode = 64;
pub const VGUI_KeyCode_KEY_BACKSPACE: VGUI_KeyCode = 65;
pub const VGUI_KeyCode_KEY_TAB: VGUI_KeyCode = 66;
pub const VGUI_KeyCode_KEY_CAPSLOCK: VGUI_KeyCode = 67;
pub const VGUI_KeyCode_KEY_NUMLOCK: VGUI_KeyCode = 68;
pub const VGUI_KeyCode_KEY_ESCAPE: VGUI_KeyCode = 69;
pub const VGUI_KeyCode_KEY_SCROLLLOCK: VGUI_KeyCode = 70;
pub const VGUI_KeyCode_KEY_INSERT: VGUI_KeyCode = 71;
pub const VGUI_KeyCode_KEY_DELETE: VGUI_KeyCode = 72;
pub const VGUI_KeyCode_KEY_HOME: VGUI_KeyCode = 73;
pub const VGUI_KeyCode_KEY_END: VGUI_KeyCode = 74;
pub const VGUI_KeyCode_KEY_PAGEUP: VGUI_KeyCode = 75;
pub const VGUI_KeyCode_KEY_PAGEDOWN: VGUI_KeyCode = 76;
pub const VGUI_KeyCode_KEY_BREAK: VGUI_KeyCode = 77;
pub const VGUI_KeyCode_KEY_LSHIFT: VGUI_KeyCode = 78;
pub const VGUI_KeyCode_KEY_RSHIFT: VGUI_KeyCode = 79;
pub const VGUI_KeyCode_KEY_LALT: VGUI_KeyCode = 80;
pub const VGUI_KeyCode_KEY_RALT: VGUI_KeyCode = 81;
pub const VGUI_KeyCode_KEY_LCONTROL: VGUI_KeyCode = 82;
pub const VGUI_KeyCode_KEY_RCONTROL: VGUI_KeyCode = 83;
pub const VGUI_KeyCode_KEY_LWIN: VGUI_KeyCode = 84;
pub const VGUI_KeyCode_KEY_RWIN: VGUI_KeyCode = 85;
pub const VGUI_KeyCode_KEY_APP: VGUI_KeyCode = 86;
pub const VGUI_KeyCode_KEY_UP: VGUI_KeyCode = 87;
pub const VGUI_KeyCode_KEY_LEFT: VGUI_KeyCode = 88;
pub const VGUI_KeyCode_KEY_DOWN: VGUI_KeyCode = 89;
pub const VGUI_KeyCode_KEY_RIGHT: VGUI_KeyCode = 90;
pub const VGUI_KeyCode_KEY_F1: VGUI_KeyCode = 91;
pub const VGUI_KeyCode_KEY_F2: VGUI_KeyCode = 92;
pub const VGUI_KeyCode_KEY_F3: VGUI_KeyCode = 93;
pub const VGUI_KeyCode_KEY_F4: VGUI_KeyCode = 94;
pub const VGUI_KeyCode_KEY_F5: VGUI_KeyCode = 95;
pub const VGUI_KeyCode_KEY_F6: VGUI_KeyCode = 96;
pub const VGUI_KeyCode_KEY_F7: VGUI_KeyCode = 97;
pub const VGUI_KeyCode_KEY_F8: VGUI_KeyCode = 98;
pub const VGUI_KeyCode_KEY_F9: VGUI_KeyCode = 99;
pub const VGUI_KeyCode_KEY_F10: VGUI_KeyCode = 100;
pub const VGUI_KeyCode_KEY_F11: VGUI_KeyCode = 101;
pub const VGUI_KeyCode_KEY_F12: VGUI_KeyCode = 102;
pub const VGUI_KeyCode_KEY_LAST: VGUI_KeyCode = 103;
pub type VGUI_KeyCode = c_uint;

pub const VGUI_KeyAction_KA_TYPED: VGUI_KeyAction = 0;
pub const VGUI_KeyAction_KA_PRESSED: VGUI_KeyAction = 1;
pub const VGUI_KeyAction_KA_RELEASED: VGUI_KeyAction = 2;
pub type VGUI_KeyAction = c_uint;

pub const VGUI_MouseAction_MA_PRESSED: VGUI_MouseAction = 0;
pub const VGUI_MouseAction_MA_RELEASED: VGUI_MouseAction = 1;
pub const VGUI_MouseAction_MA_DOUBLE: VGUI_MouseAction = 2;
pub const VGUI_MouseAction_MA_WHEEL: VGUI_MouseAction = 3;
pub type VGUI_MouseAction = c_uint;

pub const key_modifier_t_KeyModifier_None: key_modifier_t = 0;
pub const key_modifier_t_KeyModifier_LeftShift: key_modifier_t = 1;
pub const key_modifier_t_KeyModifier_RightShift: key_modifier_t = 2;
pub const key_modifier_t_KeyModifier_LeftCtrl: key_modifier_t = 4;
pub const key_modifier_t_KeyModifier_RightCtrl: key_modifier_t = 8;
pub const key_modifier_t_KeyModifier_LeftAlt: key_modifier_t = 16;
pub const key_modifier_t_KeyModifier_RightAlt: key_modifier_t = 32;
pub const key_modifier_t_KeyModifier_LeftSuper: key_modifier_t = 64;
pub const key_modifier_t_KeyModifier_RightSuper: key_modifier_t = 128;
pub const key_modifier_t_KeyModifier_NumLock: key_modifier_t = 256;
pub const key_modifier_t_KeyModifier_CapsLock: key_modifier_t = 512;
pub type key_modifier_t = c_uint;

#[repr(C)]
pub struct vguiapi_s {
    pub initialized: qboolean,
    pub DrawInit: Option<unsafe extern "C" fn()>,
    pub DrawShutdown: Option<unsafe extern "C" fn()>,
    pub SetupDrawingText: Option<unsafe extern "C" fn(pColor: *mut c_int)>,
    pub SetupDrawingRect: Option<unsafe extern "C" fn(pColor: *mut c_int)>,
    pub SetupDrawingImage: Option<unsafe extern "C" fn(pColor: *mut c_int)>,
    pub BindTexture: Option<unsafe extern "C" fn(id: c_int)>,
    pub EnableTexture: Option<unsafe extern "C" fn(enable: qboolean)>,
    pub Reserved0: Option<unsafe extern "C" fn(id: c_int, width: c_int, height: c_int)>,
    pub UploadTexture:
        Option<unsafe extern "C" fn(id: c_int, buffer: *const c_char, width: c_int, height: c_int)>,
    pub Reserved1: Option<
        unsafe extern "C" fn(
            id: c_int,
            drawX: c_int,
            drawY: c_int,
            rgba: *const byte,
            blockWidth: c_int,
            blockHeight: c_int,
        ),
    >,
    pub DrawQuad: Option<unsafe extern "C" fn(ul: *const vpoint_t, lr: *const vpoint_t)>,
    pub GetTextureSizes: Option<unsafe extern "C" fn(width: *mut c_int, height: *mut c_int)>,
    pub GenerateTexture: Option<unsafe extern "C" fn() -> c_int>,
    pub EngineMalloc: Option<unsafe extern "C" fn(size: usize) -> *mut c_void>,
    pub CursorSelect: Option<unsafe extern "C" fn(cursor: VGUI_DefaultCursor)>,
    pub GetColor: Option<unsafe extern "C" fn(i: c_int, j: c_int) -> byte>,
    pub IsInGame: Option<unsafe extern "C" fn() -> qboolean>,
    pub EnableTextInput: Option<unsafe extern "C" fn(enable: qboolean, force: qboolean)>,
    pub GetCursorPos: Option<unsafe extern "C" fn(x: *mut c_int, y: *mut c_int)>,
    pub ProcessUtfChar: Option<unsafe extern "C" fn(ch: c_int) -> c_int>,
    pub GetClipboardText:
        Option<unsafe extern "C" fn(buffer: *mut c_char, bufferSize: usize) -> c_int>,
    pub SetClipboardText: Option<unsafe extern "C" fn(text: *const c_char)>,
    pub GetKeyModifiers: Option<unsafe extern "C" fn() -> key_modifier_t>,
    pub Startup: Option<unsafe extern "C" fn(width: c_int, height: c_int)>,
    pub Shutdown: Option<unsafe extern "C" fn()>,
    pub GetPanel: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub Paint: Option<unsafe extern "C" fn()>,
    pub Mouse: Option<unsafe extern "C" fn(action: VGUI_MouseAction, code: c_int)>,
    pub Key: Option<unsafe extern "C" fn(action: VGUI_KeyAction, code: VGUI_KeyCode)>,
    pub MouseMove: Option<unsafe extern "C" fn(x: c_int, y: c_int)>,
    pub TextInput: Option<unsafe extern "C" fn(text: *const c_char)>,
}
pub type vguiapi_t = vguiapi_s;
