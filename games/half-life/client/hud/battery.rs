use core::{cmp, ffi::c_int};

use cl::{message::hook_message, prelude::*};

use crate::{
    export::hud,
    hud::{Fade, Hide, Sprite, State},
};

pub struct Battery {
    fade: Fade,
    current: i16,
    suit_empty: Option<Sprite>,
    suit_full: Option<Sprite>,
}

impl Battery {
    pub fn new() -> Self {
        hook_message!(Battery, |msg| {
            let x = msg.read_i16()?;
            hud().items.get_mut::<Battery>().set(x);
            Ok(())
        });

        Self {
            current: 0,
            fade: Fade::default(),
            suit_empty: None,
            suit_full: None,
        }
    }

    pub fn set(&mut self, value: i16) {
        // TODO: set active???

        if self.current != value {
            self.current = value;
            self.fade.start();
        }
    }
}

impl super::HudItem for Battery {
    fn vid_init(&mut self, state: &mut State) {
        self.suit_empty = state.find_sprite("suit_empty");
        self.suit_full = state.find_sprite("suit_full");
    }

    fn draw(&mut self, state: &mut State) {
        let engine = engine();

        if state.is_hidden(Hide::HEALTH) || engine.is_spectator_only() || !state.has_suit() {
            return;
        }

        let (Some(empty), Some(full)) = (self.suit_empty, self.suit_full) else {
            warn!("suit sprites was not loaded");
            return;
        };

        let color = state.color.scale_color(self.fade.alpha(state.time_delta));
        let screen_info = engine.get_screen_info();
        let width = empty.rect.width();
        let mut x = width * 3;
        let mut y = screen_info.height - state.num_height - state.num_height / 2;
        let offset = empty.rect.height() / 6;

        engine.spr_set(empty.hspr, color);
        engine.spr_draw_additive_rect(0, x, y - offset, empty.rect);

        let height = (full.rect.bottom - empty.rect.top) as f32;
        let mut rc = full.rect;
        rc.top += (height * ((100 - cmp::min(100, self.current)) as f32 * 0.01)) as c_int;
        if rc.bottom > rc.top {
            let y = y + (rc.top - full.rect.top);
            engine.spr_set(full.hspr, color);
            engine.spr_draw_additive_rect(0, x, y - offset, rc);
        }

        x += width;
        y += (state.num_height as f32 * 0.2) as c_int;
        state
            .draw_number(self.current as c_int)
            .width(3)
            .color(color)
            .at(x, y);
    }
}
