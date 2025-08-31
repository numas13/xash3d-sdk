use core::ffi::c_int;

use cl::{color::RGB, message::hook_message, prelude::*};

use crate::hud::{hud, Hide, Sprite, State};

pub struct Flashlight {
    battery_f: f32,
    battery: u8,
    enabled: bool,
    flash_empty: Option<Sprite>,
    flash_full: Option<Sprite>,
    flash_beam: Option<Sprite>,
}

impl Flashlight {
    pub fn new() -> Self {
        hook_message!(FlashBat, |msg| {
            let x = msg.read_u8()?;
            hud().items.get_mut::<Flashlight>().set(x);
            Ok(())
        });

        hook_message!(Flashlight, |msg| {
            let on = msg.read_u8()?;
            let x = msg.read_u8()?;
            let hud = hud();
            let mut flash = hud.items.get_mut::<Flashlight>();
            flash.enabled(on != 0);
            flash.set(x);
            Ok(())
        });

        Self {
            battery_f: 0.0,
            battery: 0,
            enabled: false,
            flash_empty: None,
            flash_full: None,
            flash_beam: None,
        }
    }

    pub fn set(&mut self, value: u8) {
        if self.battery != value {
            self.battery = value;
            self.battery_f = value as f32 / 100.0;
        }
    }

    pub fn enabled(&mut self, value: bool) {
        self.enabled = value;
    }
}

impl super::HudItem for Flashlight {
    fn vid_init(&mut self, state: &mut State) {
        self.flash_empty = state.find_sprite("flash_empty");
        self.flash_full = state.find_sprite("flash_full");
        self.flash_beam = state.find_sprite("flash_beam");
    }

    fn reset(&mut self) {
        self.enabled = false;
    }

    fn draw(&mut self, state: &mut State) {
        if state.is_hidden(Hide::FLASHLIGHT) || !state.has_suit() {
            return;
        }

        let (Some(empty), Some(full), Some(beam)) =
            (self.flash_empty, self.flash_full, self.flash_beam)
        else {
            return;
        };

        let a = if self.enabled { 225 } else { super::MIN_ALPHA };

        let color = if self.battery < 20 {
            RGB::REDISH
        } else {
            state.color
        };
        let color = color.scale_color(a);

        let engine = engine();
        let screen = engine.get_screen_info();

        let width = empty.rect.width();
        let x = screen.width - width - width / 2;
        let y = (empty.rect.bottom - full.rect.top) / 2;

        engine.spr_set(empty.hspr, color);
        engine.spr_draw_additive_rect(0, x, y, empty.rect);

        if self.enabled {
            let x = screen.width - width / 2;
            engine.spr_set(beam.hspr, color);
            engine.spr_draw_additive_rect(0, x, y, beam.rect);
        }

        let offset = (width as f32 * (1.0 - self.battery_f)) as c_int;
        if offset < width {
            let mut rect = full.rect;
            rect.left += offset;
            engine.spr_set(full.hspr, color);
            engine.spr_draw_additive_rect(0, x + offset, y, rect);
        }
    }
}
