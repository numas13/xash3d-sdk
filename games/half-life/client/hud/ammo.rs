use core::ffi::c_int;

use xash3d_client::prelude::*;

use crate::hud::{Fade, Hide, HudItem, State};

const FALLBACK_WIDTH: c_int = 24;

pub struct Ammo {
    engine: ClientEngineRef,
    fade: Fade,
}

impl Ammo {
    pub fn new(engine: ClientEngineRef) -> Self {
        Self {
            engine,
            fade: Fade::new(super::FADE_TIME_AMMO),
        }
    }

    pub fn fade_start(&mut self) {
        self.fade.start()
    }
}

impl HudItem for Ammo {
    fn reset(&mut self) {
        self.fade.stop();
    }

    fn draw(&mut self, state: &mut State) {
        if !state.has_suit() || state.is_hidden(Hide::WEAPONS) {
            return;
        }

        let Some(weapon) = state.inv.current() else {
            return;
        };

        if !weapon.ammo.iter().any(|i| i.is_some()) {
            return;
        }

        let a = self.fade.alpha(state.time_delta);
        let color = state.color.scale_color(a);

        let engine = self.engine;
        let screen = engine.screen_info();
        let ammo_width = state.num_width;

        let mut y = screen.height() - state.num_height - state.num_height / 2;
        y += (state.num_height as f32 * 0.2) as c_int;

        if let Some(ammo) = weapon.ammo[0] {
            let ammo_count = state.inv.ammo_count(ammo.ty) as c_int;
            let icon_width = ammo.icon.map_or(FALLBACK_WIDTH, |s| s.width());

            let mut x = screen.width() - icon_width;
            if weapon.clip >= 0 {
                x -= 8 * ammo_width;
                x = state
                    .draw_number(weapon.clip)
                    .width(3)
                    .color(color)
                    .at(x, y);

                let bar_width = ammo_width / 10;
                x += ammo_width / 2;
                engine.fill_rgba(x, y, bar_width, state.num_height, state.color.rgba(a));

                x += ammo_width / 2 + bar_width;
                x = state.draw_number(ammo_count).width(3).color(color).at(x, y);
            } else {
                x -= 4 * ammo_width;
                x = state.draw_number(ammo_count).width(3).color(color).at(x, y);
            }

            if let Some(icon) = ammo.icon {
                let offset = icon.height() / 8;
                icon.draw_additive(0, x, y - offset, color);
            }
        }

        if let Some(ammo) = weapon.ammo[1] {
            let ammo_count = state.inv.ammo_count(ammo.ty) as c_int;
            if ammo_count > 0 {
                let icon_width = ammo.icon.map_or(FALLBACK_WIDTH, |s| s.width());

                let mut x = screen.width() - icon_width;
                y -= state.num_height + state.num_height / 4;

                x -= 4 * ammo_width;
                x = state.draw_number(ammo_count).width(3).color(color).at(x, y);

                if let Some(icon) = ammo.icon {
                    let offset = icon.height() / 8;
                    icon.draw_additive(0, x, y - offset, color);
                }
            }
        }
    }
}
