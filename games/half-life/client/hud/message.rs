use core::{
    cmp,
    ffi::{c_int, CStr},
};

use alloc::{boxed::Box, vec, vec::Vec};
use cl::{
    color::RGB,
    engine,
    math::fabsf,
    message::hook_message,
    raw::{byte, client_textmessage_s, SCREENINFO},
    Engine,
};

use super::{hud, HudFlags, HudItem, Sprite, State};

const MAX_HUD_MESSAGES: usize = 16;

#[derive(Clone)]
struct Msg {
    start_time: f32,
    effect: c_int,
    r1: byte,
    g1: byte,
    b1: byte,
    r2: byte,
    g2: byte,
    b2: byte,
    x: f32,
    y: f32,
    fadein: f32,
    fadeout: f32,
    holdtime: f32,
    fxtime: f32,
    message: Box<str>,
}

impl Msg {
    fn end_time(&self) -> f32 {
        match self.effect {
            0 | 1 => self.start_time + self.fadein + self.fadeout + self.holdtime,
            2 => {
                self.start_time
                    + self.fadein * self.message.len() as f32
                    + self.fadeout
                    + self.holdtime
            }
            _ => todo!(),
        }
    }

    fn draw(&self, state: &State, engine: &Engine, screen: &SCREENINFO) {
        // TODO: utf8

        let mut length = 0;
        let mut total_width = 0;
        let mut total_height = screen.char_height;
        for line in self.message.lines() {
            let width = line
                .as_bytes()
                .iter()
                .map(|i| screen.char_widths[*i as usize] as c_int)
                .sum();

            length += line.len() + 1;
            total_width = cmp::max(width, total_width);
            total_height += screen.char_height;
        }

        let time = state.time - self.start_time;
        let fade_time;
        let mut fade_blend = 0.0;
        let mut char_time = 0.0;
        match self.effect {
            0 | 1 => {
                fade_time = self.fadein + self.holdtime;
                if time < self.fadein {
                    fade_blend = (self.fadein - time) * (1.0 / self.fadein * 255.0);
                } else if time > fade_time {
                    if self.fadeout > 0.0 {
                        fade_blend = (time - fade_time) / self.fadeout * 255.0;
                    } else {
                        fade_blend = 255.0;
                    }
                }

                if self.effect == 1 && (engine.rand() % 100) < 10 {
                    char_time = 1.0;
                }
            }
            2 => {
                fade_time = self.fadein * length as f32 + self.holdtime;

                if time > fade_time && self.fadeout > 0.0 {
                    fade_blend = (time - fade_time) / self.fadeout * 255.0;
                }
            }
            _ => todo!(),
        }

        let mut y = position(self.y, total_height, 0, screen.height);
        for line in self.message.lines() {
            let width = line
                .as_bytes()
                .iter()
                .map(|i| screen.char_widths[*i as usize] as c_int)
                .sum();

            let mut x = position(self.x, width, total_width, screen.width);
            for &c in line.as_bytes() {
                let x_next = x + screen.char_widths[c as usize] as c_int;
                if x_next > screen.width || x < 0 || y < 0 {
                    x = x_next;
                    continue;
                }

                let mut blend = 0.0;
                let mut src = RGB::new(self.r1, self.g1, self.b1);
                let mut dest = RGB::BLACK;
                match self.effect {
                    0 | 1 => blend = fade_blend,
                    2 => {
                        char_time += self.fadein;
                        if char_time > time {
                            src = RGB::BLACK;
                        } else if time > fade_time {
                            blend = fade_blend;
                        } else if time - char_time > self.fxtime {
                            // nop
                        } else {
                            dest = RGB::new(self.r2, self.g2, self.b2);
                        }
                    }
                    _ => todo!(),
                }

                if self.effect == 1 && char_time != 0.0 {
                    let color = RGB::new(self.r2, self.g2, self.b2);
                    engine.draw_character(x, y, c as c_int, color);
                }

                let alpha = blend.clamp(0.0, 255.0) as u8;
                let color = dest.blend_alpha(src, alpha);
                engine.draw_character(x, y, c as c_int, color);

                x = x_next;
            }
            y += screen.char_height;
        }
    }
}

pub struct HudMessage {
    active: bool,

    messages: Vec<Option<Msg>>,
    fixup_time: f32,
    end_after_message: bool,

    game_title: Option<&'static client_textmessage_s>,
    game_title_time: f32,

    title_half: Option<Sprite>,
    title_life: Option<Sprite>,
}

impl HudMessage {
    pub fn new() -> Self {
        hook_message!(HudText, |msg| {
            let s = msg.read_str()?;
            let hud = hud();
            hud.items
                .get_mut::<HudMessage>()
                .msg_hud_text(&hud.state, s);
            Ok(())
        });

        hook_message!(GameTitle, {
            let hud = hud();
            hud.items.get_mut::<HudMessage>().msg_game_title(&hud.state);
            true
        });

        Self {
            active: false,

            messages: vec![None; MAX_HUD_MESSAGES],
            fixup_time: 0.0,
            end_after_message: false,

            game_title: None,
            game_title_time: 0.0,

            title_half: None,
            title_life: None,
        }
    }

    fn msg_hud_text(&mut self, state: &State, s: &str) {
        if s == "END3" {
            self.end_after_message = true;
        }
        self.message_add(s, state.time);
        // save time to fixup level transitions
        self.fixup_time = state.time;
        self.active = true;
    }

    fn msg_game_title(&mut self, state: &State) {
        self.game_title = engine().text_message_get(c"GAMETITLE");
        if self.game_title.is_some() {
            self.game_title_time = state.time;
            self.active = true;
        }
    }

    fn message_add(&mut self, mut name: &str, start_time: f32) {
        let Some(index) = self.messages.iter_mut().position(|i| i.is_none()) else {
            return;
        };

        if name.starts_with("#") {
            name = &name[1..];
        }

        let new = match engine().text_message_get(name) {
            Some(msg) => {
                let message = unsafe { CStr::from_ptr(msg.pMessage).to_string_lossy().into() };
                Msg {
                    start_time,
                    effect: msg.effect,
                    r1: msg.r1,
                    g1: msg.g1,
                    b1: msg.b1,
                    r2: msg.r2,
                    g2: msg.g2,
                    b2: msg.b2,
                    x: msg.x,
                    y: msg.y,
                    fadein: msg.fadein,
                    fadeout: msg.fadeout,
                    fxtime: msg.fxtime,
                    holdtime: msg.holdtime,
                    message,
                }
            }
            None => Msg {
                start_time,
                effect: 2,
                r1: 100,
                g1: 100,
                b1: 100,
                r2: 240,
                g2: 110,
                b2: 0,
                x: -1.0,
                y: 0.7,
                fadein: 0.01,
                fadeout: 1.5,
                fxtime: 0.25,
                holdtime: 5.0,
                message: name.into(),
            },
        };

        for i in self.messages.iter_mut() {
            if let Some(old) = i {
                if new.message == old.message {
                    return;
                }
                if fabsf(new.y - old.y) < 0.0001 && fabsf(new.x - old.y) < 0.0001 {
                    *i = None;
                }
            }
        }

        self.messages[index] = Some(new);
    }

    fn draw_game_title(&mut self, state: &mut State) -> bool {
        let Some(title) = self.game_title else {
            return false;
        };
        let Some(title_half) = self.title_half else {
            return false;
        };
        let Some(title_life) = self.title_life else {
            return false;
        };

        if self.game_title_time > state.time {
            self.game_title_time = state.time;
        }

        let local_time = state.time - self.game_title_time;
        if local_time > (title.fadein + title.holdtime + title.fadeout) {
            self.game_title = None;
            return false;
        }
        let brightness = fade_blend(title.fadein, title.fadeout, title.holdtime, local_time);
        let color = RGB::new(title.r1, title.g1, title.b1).scale_color((brightness * 255.0) as u8);

        let half_width = title_half.rect.width();
        let full_width = half_width + title_life.rect.width();
        let full_height = title_life.rect.height();
        let engine = engine();
        let screen = engine.get_screen_info();
        let x = position(title.x, full_width, full_width, screen.width);
        let y = position(title.y, full_height, 0, screen.height);

        engine.spr_set(title_half.hspr, color);
        engine.spr_draw_additive_rect(0, x, y, title_half.rect);
        engine.spr_set(title_life.hspr, color);
        engine.spr_draw_additive_rect(0, x + half_width, y, title_life.rect);

        true
    }

    fn draw_messages(&mut self, state: &mut State) -> bool {
        let mut drawn = false;

        for i in self.messages.iter_mut().filter_map(|i| i.as_mut()) {
            if i.start_time > state.time {
                i.start_time = state.time + self.fixup_time - i.start_time + 0.2;
            }
        }

        let engine = engine();
        let screen = engine.get_screen_info();
        for i in self.messages.iter_mut() {
            if let Some(msg) = i {
                if state.time <= msg.end_time() {
                    msg.draw(state, engine, &screen);
                    drawn = true;
                } else {
                    *i = None;
                    if self.end_after_message {
                        engine
                            .client_cmd(c"wait\nwait\nwait\nwait\nwait\nwait\nwait\ndisconnect\n");
                    }
                }
            }
        }

        self.fixup_time = state.time;

        drawn
    }
}

impl HudItem for HudMessage {
    fn flags(&self) -> super::HudFlags {
        if self.active {
            HudFlags::ACTIVE
        } else {
            HudFlags::empty()
        }
    }

    fn reset(&mut self) {
        self.active = false;
        self.fixup_time = 0.0;
        self.end_after_message = false;
        self.game_title = None;
        self.messages.fill(None);
    }

    fn vid_init(&mut self, state: &mut State) {
        self.title_half = state.find_sprite("title_half");
        self.title_life = state.find_sprite("title_life");
    }

    fn draw(&mut self, state: &mut State) {
        if !self.draw_game_title(state) && !self.draw_messages(state) {
            self.active = false;
        }
    }
}

fn fade_blend(fadein: f32, fadeout: f32, hold: f32, local_time: f32) -> f32 {
    if local_time < 0.0 {
        return 0.0;
    }
    let fade_time = fadein + hold;
    if local_time < fadein {
        1.0 - (fadein - local_time) / fadein
    } else if local_time > fade_time {
        if fadeout > 0.0 {
            1.0 - (local_time - fade_time) / fadeout
        } else {
            0.0
        }
    } else {
        1.0
    }
}

fn position(c: f32, size: c_int, total_size: c_int, screen_size: c_int) -> c_int {
    let pos = if c == -1.0 {
        (screen_size - size) / 2
    } else if c < 0.0 {
        ((1.0 + c) * screen_size as f32) as c_int - total_size
    } else {
        (c * screen_size as f32) as c_int
    };

    if pos + size > screen_size {
        screen_size - size
    } else if pos < 0 {
        0
    } else {
        pos
    }
}
