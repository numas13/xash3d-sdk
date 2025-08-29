use std::{cell::Cell, ffi::c_int};

use shared::color::RGBA;
use xash3d_ui_engine::{
    consts::keys,
    engine,
    export::{export_dll, impl_unsync_global, UiDll},
    globals,
    ActiveMenu::{Console, Menu},
};

#[derive(Default)]
pub struct Instance {
    active: Cell<bool>,
}

impl_unsync_global!(Instance);

impl UiDll for Instance {
    fn set_active_menu(&self, active: bool) {
        self.active.set(active);
        engine().set_key_dest([Console, Menu][active as usize]);
    }

    fn is_visible(&self) -> bool {
        self.active.get()
    }

    fn key_event(&self, key: c_int, down: bool) {
        if !down {
            return;
        }
        match key as u8 {
            keys::K_ESCAPE => self.set_active_menu(false),
            keys::K_Q => engine().client_cmd(c"quit"),
            _ => {}
        }
    }

    fn redraw(&self, _time: f32) {
        if !self.is_visible() {
            return;
        }
        let engine = engine();
        let globals = globals();
        engine.fill_rgba((0, 0), (globals.scrWidth, globals.scrHeight), RGBA::BLACK);
        let text = c"Press Q to exit";
        let (w, h) = engine.draw_console_string_len(text);
        let x = (globals.scrWidth - w) / 2;
        let y = (globals.scrHeight - h) / 2;
        engine.draw_console_string(x, y, text);
    }
}

export_dll!(Instance);
