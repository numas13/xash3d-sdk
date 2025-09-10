use core::{cmp, ffi::c_int, fmt::Write};

use alloc::string::String;
use cl::{
    color::RGB,
    message::{hook_message, Message, MessageError},
    prelude::*,
};
use csz::CStrArray;

use crate::export::hud;

use super::{HudItem, State};

const MAX_MENU_STRING: usize = 512;

pub struct Menu {
    time: f32,
    waiting_more: bool,
    data: String,
    slots: u16,
}

impl Menu {
    pub fn new() -> Self {
        hook_message!(ShowMenu, Menu::msg_show_menu);

        Self {
            time: 0.0,
            waiting_more: false,
            data: String::with_capacity(MAX_MENU_STRING),
            slots: 0,
        }
    }

    pub fn is_displayed(&self) -> bool {
        self.time != 0.0
    }

    fn show_menu(&mut self, state: &State, slots: u16, time: u8, more: bool, data: &str) {
        if slots == 0 {
            self.waiting_more = false;
            return;
        }

        if !self.waiting_more {
            self.data.clear();
        }
        self.data.push_str(data);

        if more {
            self.waiting_more = more;
            return;
        }

        let mut buf = String::with_capacity(self.data.len());
        super::text_message::localise_string(&mut buf, &self.data);
        self.data = buf;

        self.time = if time > 0 {
            state.time + time as f32
        } else {
            0.0
        };
        self.waiting_more = false;
        self.slots = slots;
    }

    pub fn select_menu_item(&mut self, item: u32) {
        if item > 0 && self.slots & (1 << (item - 1)) != 0 {
            let mut buf = CStrArray::<128>::new();
            writeln!(buf.cursor(), "menuselect {item}").ok();
            engine().client_cmd(buf.as_c_str());
            self.time = 0.0;
        }
    }

    pub fn msg_show_menu(msg: &mut Message) -> Result<(), MessageError> {
        let slots = msg.read_u16()?;
        let time = msg.read_u8()?;
        let more = msg.read_u8()? != 0;
        let data = msg.read_str()?;
        let hud = hud();
        hud.items
            .get_mut::<Menu>()
            .show_menu(&hud.state, slots, time, more, data);
        Ok(())
    }
}

impl HudItem for Menu {
    fn draw(&mut self, state: &mut State) {
        // TODO: do not draw if score board is visible

        if self.time == 0.0 {
            return;
        } else if self.time <= state.time {
            self.time = 0.0;
            return;
        }

        let engine = engine();
        let screen = engine.get_screen_info();
        let font_height = cmp::max(12, screen.iCharHeight);
        let nlc = self.data.chars().filter(|&c| c == '\n').count() as c_int;
        let start_x = 20;
        let mut x = start_x;
        let mut y =
            screen.iHeight / 2 - (nlc / 2 * font_height) - (3 * font_height + font_height / 3);
        let mut color = RGB::WHITE;
        let mut ralign = false;
        let mut cur = self.data.as_str();
        while !cur.is_empty() {
            while let Some(b'\\') = cur.as_bytes().first() {
                cur = &cur[1..];

                let mut skip = true;
                match cur.as_bytes().first().unwrap_or(&0) {
                    b'w' => color = RGB::new(255, 255, 255),
                    b'd' => color = RGB::new(100, 100, 100),
                    b'y' => color = RGB::new(255, 210, 64),
                    b'r' => color = RGB::new(210, 24, 0),
                    b'R' => {
                        x = screen.iWidth / 2;
                        ralign = true;
                    }
                    b'\\' => {
                        x += engine.draw_string(x, y, c"\\", color);
                    }
                    _ => skip = false,
                }

                if skip {
                    cur = &cur[1..];
                }
            }

            while let Some(b'\n') = cur.as_bytes().first() {
                x = start_x;
                y += font_height;
                ralign = false;
                cur = &cur[1..];
            }

            let offset = cur.find(['\\', '\n']).unwrap_or(cur.len());
            let (head, tail) = cur.split_at(offset);
            if ralign {
                x -= engine.draw_string_reverse(x, y, head, color);
            } else {
                x += engine.draw_string(x, y, head, color);
            }

            cur = tail;
        }
    }
}
