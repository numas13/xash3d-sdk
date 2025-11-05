use std::{cell::Cell, ffi::c_int};

use xash3d_ui::{
    color::RGBA,
    consts::keys,
    engine::{
        ActiveMenu::{Console, Menu},
        UiEngineRef,
    },
    export::{UiDll, export_dll, impl_unsync_global},
    prelude::*,
};

pub struct Dll {
    engine: UiEngineRef,
    active: Cell<bool>,
}

impl_unsync_global!(Dll);

impl UiDll for Dll {
    fn new(engine: UiEngineRef) -> Self {
        Self {
            engine,
            active: Cell::new(false),
        }
    }

    fn set_active_menu(&self, active: bool) {
        self.active.set(active);
        self.engine.set_key_dest([Console, Menu][active as usize]);
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
            keys::K_Q => self.engine.client_cmd(c"quit"),
            _ => {}
        }
    }

    fn redraw(&self, _time: f32) {
        if !self.is_visible() {
            return;
        }
        let engine = self.engine;
        let globals = &engine.globals;
        let area = globals.screen_area();
        engine.fill_rgba(RGBA::BLACK, area);
        let text = c"Press Q to exit";
        let (w, h) = engine.console_string_size(text);
        let x = (area.width as c_int - w) / 2;
        let y = (area.height as c_int - h) / 2;
        engine.draw_console_string(x, y, text);
    }
}

export_dll!(Dll);
