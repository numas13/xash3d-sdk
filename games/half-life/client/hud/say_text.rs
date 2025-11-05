use core::ffi::{CStr, c_int};

use alloc::collections::vec_deque::VecDeque;
use csz::CStrArray;
use xash3d_client::{color::RGB, math::fminf, prelude::*, user_message::hook_user_message};
use xash3d_hl_shared::user_message;

use crate::export::hud;

use super::{HudFlags, HudItem, State};

const MAX_LINES: usize = 5;
const MAX_CHARS_PER_LINE: usize = 256;

const SAY_MESSAGE: u8 = 2;

mod cvar {
    xash3d_client::cvar::define! {
        pub static hud_saytext(c"1", NONE);
        pub static hud_saytext_time(c"5", NONE);
    }
}

struct Line {
    name_len: usize,
    color: RGB,
    data: CStrArray<MAX_CHARS_PER_LINE>,
}

pub struct SayText {
    engine: ClientEngineRef,
    scroll_time: f32,
    line_height: c_int,
    lines: VecDeque<Line>,
}

impl SayText {
    pub fn new(engine: ClientEngineRef) -> Self {
        hook_user_message!(engine, SayText, |_, msg| {
            let msg = msg.read::<user_message::SayText>()?;
            let hud = hud();
            hud.items.get_mut::<SayText>().say_text(
                &hud.state,
                msg.text,
                msg.client_index as c_int,
            );
            Ok(())
        });

        Self {
            engine,
            scroll_time: 0.0,
            line_height: 0,
            lines: Default::default(),
        }
    }

    pub fn say_text(&mut self, state: &State, msg: &CStr, client: c_int) {
        let mut bytes = msg.to_bytes();
        if bytes.is_empty() {
            return;
        }

        let mut name_len = 0;
        let mut color = RGB::WHITE;

        let engine = self.engine;
        if bytes[0] == SAY_MESSAGE && client > 0 {
            if let Some(info) = engine.get_player_info(client) {
                let name = info.name().to_bytes();
                if bytes[1..].starts_with(name) {
                    name_len = name.len();
                    color = state.get_client_color(client);
                    bytes = &bytes[1..];
                }
            }
        }

        if self.lines.is_empty() {
            self.scroll_time = state.time + cvar::hud_saytext_time.value();
        }

        // TODO: ensure text fits in one line

        let line = Line {
            name_len,
            color,
            data: CStrArray::from_bytes(bytes).unwrap(),
        };
        while self.lines.len() >= MAX_LINES {
            self.lines.pop_back();
        }
        self.lines.push_front(line);

        engine.play_sound_by_name(c"misc/talk.wav", 1.0);
    }
}

impl HudItem for SayText {
    fn flags(&self) -> HudFlags {
        HudFlags::ACTIVE | HudFlags::INTERMISSION
    }

    fn init_hud_data(&mut self, _: &mut State) {
        self.lines.clear();
    }

    fn vid_init(&mut self, _: &mut State) {
        self.line_height = self.engine.console_string_height(c"test");
    }

    fn draw(&mut self, state: &mut State) {
        if self.lines.is_empty() {
            return;
        }

        let engine = self.engine;

        let saytext_time = cvar::hud_saytext_time.value();
        self.scroll_time = fminf(self.scroll_time, state.time + saytext_time);
        if self.scroll_time <= state.time {
            self.scroll_time += saytext_time;
            self.lines.pop_back();
            if self.lines.is_empty() {
                return;
            }
        }

        let screen = engine.screen_info();
        let mut y = screen.height() - 60 - self.line_height * (MAX_LINES + 2) as c_int;

        for line in self.lines.iter_mut().rev() {
            let mut x = 10;
            let mut msg = unsafe { &mut line.data.inner_slice_mut()[..] };

            if line.name_len != 0 {
                engine.set_text_color(line.color);

                // numas13: I hate C strings...
                let saved_c = msg[line.name_len];
                msg[line.name_len] = b'\0';

                let s = CStr::from_bytes_until_nul(msg).unwrap();
                x = engine.draw_console_string(x, y, s);

                msg[line.name_len] = saved_c;
                msg = &mut msg[line.name_len..];
            }

            let s = CStr::from_bytes_until_nul(msg).unwrap();
            engine.draw_console_string(x, y, s);

            y += self.line_height;
        }
    }
}
