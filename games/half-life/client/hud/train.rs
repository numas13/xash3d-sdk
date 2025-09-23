use core::ffi::c_int;

use xash3d_client::{macros::spr_load, message::hook_message, prelude::*, sprite::SpriteHandle};

use crate::{
    export::hud,
    hud::{try_spr_load, HudItem, State},
};

pub struct Train {
    engine: ClientEngineRef,
    pos: u8,
    sprite: Option<SpriteHandle>,
}

impl Train {
    pub fn new(engine: ClientEngineRef) -> Self {
        hook_message!(engine, Train, |_, msg| {
            let x = msg.read_u8()?;
            hud().items.get_mut::<Train>().set_pos(x);
            Ok(())
        });

        Self {
            engine,
            pos: 0,
            sprite: None,
        }
    }

    fn set_pos(&mut self, pos: u8) {
        self.pos = pos;
    }
}

impl HudItem for Train {
    fn draw(&mut self, state: &mut State) {
        if self.pos == 0 {
            return;
        }

        let engine = self.engine;
        if self.sprite.is_none() {
            self.sprite = try_spr_load(state.res, |res| {
                spr_load!(engine, "sprites/{res}_train.spr")
            });
        }

        let Some(sprite) = self.sprite else { return };

        engine.spr_set(sprite, state.color);

        let (w, h) = engine.spr_size(sprite, 0);
        let screen = engine.screen_info();
        let x = screen.width() / 3 + w / 4;
        let y = screen.height() - h - state.num_height;
        engine.spr_draw_additive(self.pos as c_int - 1, x, y);
    }
}
