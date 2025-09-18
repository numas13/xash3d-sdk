use core::ffi::c_int;

use cl::{message::hook_message, prelude::*};

use crate::{
    export::hud,
    hud::{Hide, HudItem, State},
};

const MAX_HISTORY: usize = 12;

mod cvar {
    cl::cvar::define! {
        pub static hud_drawhistory_time(c"5", NONE);
    }
}

#[derive(Copy, Clone, Debug)]
enum ItemKind {
    Ammo(u8, u8),
    Weapon(u8),
    Item(Option<usize>),
}

#[derive(Copy, Clone, Debug)]
struct Item {
    time: f32,
    kind: ItemKind,
}

pub struct History {
    engine: ClientEngineRef,
    items: [Option<Item>; MAX_HISTORY],
    slot: usize,
}

impl History {
    pub fn new(engine: ClientEngineRef) -> Self {
        hook_message!(engine, AmmoPickup, |_, msg| {
            let index = msg.read_u8()?;
            let count = msg.read_u8()?;
            if count != 0 {
                let hud = hud();
                hud.items
                    .get_mut::<History>()
                    .add(&hud.state, ItemKind::Ammo(index, count));
            }
            Ok(())
        });

        hook_message!(engine, WeapPickup, |_, msg| {
            let index = msg.read_u8()?;
            let hud = hud();
            hud.items
                .get_mut::<History>()
                .add(&hud.state, ItemKind::Weapon(index));
            Ok(())
        });

        hook_message!(engine, ItemPickup, |_, msg| {
            let name = msg.read_str()?;
            let hud = hud();
            let index = hud.state.find_sprite_index(name);
            hud.items
                .get_mut::<History>()
                .add(&hud.state, ItemKind::Item(index));
            Ok(())
        });

        Self {
            engine,
            items: [None; MAX_HISTORY],
            slot: 0,
        }
    }

    fn add(&mut self, state: &State, kind: ItemKind) {
        let engine = self.engine;
        let height = self.slot as c_int * state.inv.pickup_gap() + state.inv.pickup_height();
        let height_max = engine.screen_info().height() - 100;
        if height > height_max || self.slot >= self.items.len() {
            self.slot = 0;
        }
        self.items[self.slot] = Some(Item {
            time: state.time + cvar::hud_drawhistory_time.value(),
            kind,
        });
        self.slot += 1;
    }
}

impl HudItem for History {
    fn reset(&mut self) {
        self.items.fill(None);
    }

    fn draw(&mut self, state: &mut State) {
        if !state.has_suit() || state.is_hidden(Hide::WEAPONS | Hide::ALL) {
            return;
        }

        let gap = state.inv.pickup_gap();
        let height = state.inv.pickup_height();

        let engine = self.engine;
        let screen = engine.screen_info();

        for i in 0..self.items.len() {
            let item = match self.items[i] {
                Some(item) => {
                    if item.time <= state.time {
                        self.items[i] = None;
                        if self.items.iter().all(|i| i.is_none()) {
                            self.slot = 0;
                        }
                        continue;
                    }
                    item
                }
                None => continue,
            };

            let a = ((item.time - state.time) * 80.0) as u8;
            let color = state.color.scale_color(a);
            let mut x = screen.width() - 4;
            let y = screen.height() - height - gap * i as c_int;

            match item.kind {
                ItemKind::Ammo(index, count) => {
                    let Some(icon) = state.inv.ammo_icon(index as u32) else {
                        continue;
                    };

                    x -= icon.rect.width();

                    engine.spr_set(icon.hspr, color);
                    engine.spr_draw_additive_rect(0, x, y, icon.rect);

                    state
                        .draw_number(count as i32)
                        .color(color)
                        .string(true)
                        .reverse(true)
                        .at(x - 10, y);
                }
                ItemKind::Weapon(index) => {
                    let Some(icon) = state.inv.weapon_icon(index as u32) else {
                        continue;
                    };

                    x -= icon.rect.width();

                    engine.spr_set(icon.hspr, color);
                    engine.spr_draw_additive_rect(0, x, y, icon.rect);
                }
                ItemKind::Item(index) => {
                    let Some(index) = index else { continue };
                    let Some(icon) = state.sprites.get(index) else {
                        continue;
                    };

                    x -= icon.rect.width();

                    engine.spr_set(icon.hspr, color);
                    engine.spr_draw_additive_rect(0, x, y, icon.rect);
                }
            }
        }
    }
}
