use core::{cmp, ffi::c_int, mem};

use xash3d_client::{color::RGB, consts, macros::hook_command, prelude::*};

use crate::{
    export::{hud, hud_mut},
    hud::{
        Hide, HudItem, Sprite, State,
        inventory::{MAX_WEAPON_POSITIONS, MAX_WEAPON_SLOTS, Weapon},
    },
};

mod cvar {
    xash3d_client::cvar::define! {
        pub static hud_fastswitch(c"0", ARCHIVE);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Select {
    None,
    Menu,
    Weapon(u32, u32, u32),
}

impl From<&'_ Weapon> for Select {
    fn from(value: &'_ Weapon) -> Self {
        Self::Weapon(value.id, value.slot, value.slot_pos)
    }
}

pub struct WeaponMenu {
    engine: ClientEngineRef,
    active: Select,
    last: Select,
    bucket0: Option<usize>,
    selection: Option<Sprite>,
    ab_width: c_int,
    ab_height: c_int,
    weapon_select: u32,
}

impl WeaponMenu {
    pub fn new(engine: ClientEngineRef) -> Self {
        hook_command!(engine, c"cancelselect", |_| {
            hud().items.get_mut::<WeaponMenu>().cmd_close();
        });
        hook_command!(engine, c"invnext", |_| {
            let hud = &mut *hud_mut();
            hud.items
                .get_mut::<WeaponMenu>()
                .cmd_next_weapon(&mut hud.state);
        });
        hook_command!(engine, c"invprev", |_| {
            let hud = &mut *hud_mut();
            hud.items
                .get_mut::<WeaponMenu>()
                .cmd_prev_weapon(&mut hud.state);
        });

        Self {
            engine,
            active: Select::None,
            last: Select::None,
            bucket0: None,
            selection: None,
            weapon_select: 0,
            ab_width: 0,
            ab_height: 0,
        }
    }

    pub fn take_weapon_select(&mut self) -> u32 {
        mem::take(&mut self.weapon_select)
    }

    pub fn select_slot(&mut self, state: &mut State, slot: u32) {
        if slot == 0 || slot > MAX_WEAPON_SLOTS as u32 {
            return;
        }

        if state.hide.intersects(Hide::WEAPONS | Hide::ALL) {
            return;
        }

        if !state.has_suit() {
            return;
        }

        let engine = self.engine;
        let fast_switch = cvar::hud_fastswitch.value() != 0.0;

        let slot = slot - 1;
        let mut selected = None;

        match self.active {
            Select::Weapon(_, s, p) if s == slot => {
                engine.play_sound_by_name(c"common/wpn_moveselect.wav", 1.0);

                if self.active != Select::None {
                    selected = state.inv.get_next_active_pos(s, p);
                }

                if selected.is_none() {
                    selected = state.inv.get_first_pos(slot);
                }
            }
            _ => {
                engine.play_sound_by_name(c"common/wpn_hudon.wav", 1.0);

                selected = state.inv.get_first_pos(slot);
                if let Some(weapon) = selected {
                    let next = state.inv.get_next_active_pos(weapon.slot, weapon.slot_pos);
                    if fast_switch && next.is_none() {
                        engine.server_cmd(&weapon.name);
                        self.weapon_select = weapon.id;
                        self.active = Select::None;
                        return;
                    }
                }
            }
        };

        self.active = match selected {
            Some(weapon) => weapon.into(),
            None if !fast_switch => Select::Menu,
            None => Select::None,
        };
    }

    pub fn close(&mut self) -> bool {
        if self.active != Select::None {
            self.last = self.active;
            self.active = Select::None;
            true
        } else {
            false
        }
    }

    fn cmd_close(&mut self) {
        if !self.close() {
            self.engine.client_cmd(c"escape");
        }
    }

    fn cmd_next_weapon(&mut self, state: &mut State) {
        if state.hide.intersects(Hide::WEAPONS | Hide::ALL) {
            return;
        }

        if matches!(self.active, Select::None | Select::Menu) {
            if let Some(weapon) = state.inv.current() {
                self.active = weapon.into();
            }
        }

        let mut slot = 0;
        let mut pos = 0;

        if let Select::Weapon(_, s, p) = self.active {
            slot = s;
            pos = p + 1;
        }

        for _ in 0..2 {
            for slot in slot..MAX_WEAPON_SLOTS as u32 {
                for pos in pos..MAX_WEAPON_POSITIONS as u32 {
                    if let Some(weapon) = state.inv.slot_weapon(slot, pos) {
                        if state.inv.has_ammo(weapon) {
                            self.active = weapon.into();
                            return;
                        }
                    }
                }
                pos = 0;
            }
            slot = 0;
        }

        self.active = Select::None;
    }

    fn cmd_prev_weapon(&mut self, state: &mut State) {
        if state.hide.intersects(Hide::WEAPONS | Hide::ALL) {
            return;
        }

        if matches!(self.active, Select::None | Select::Menu) {
            if let Some(weapon) = state.inv.current() {
                self.active = weapon.into();
            }
        }

        let mut slot = MAX_WEAPON_SLOTS as u32 - 1;
        let mut pos = MAX_WEAPON_POSITIONS as u32 - 1;

        if let Select::Weapon(_, s, p) = self.active {
            slot = s;
            pos = p;
        }

        for _ in 0..2 {
            for slot in (0..=slot).rev() {
                for pos in (0..pos).rev() {
                    if let Some(weapon) = state.inv.slot_weapon(slot, pos) {
                        if state.inv.has_ammo(weapon) {
                            self.active = weapon.into();
                            return;
                        }
                    }
                }
                pos = MAX_WEAPON_POSITIONS as u32 - 1;
            }
            slot = MAX_WEAPON_SLOTS as u32 - 1;
        }

        self.active = Select::None;
    }

    fn draw_bar(&self, x: c_int, y: c_int, color: RGB, f: f32) -> c_int {
        let mut x = x;
        let mut width = self.ab_width;
        let height = self.ab_height;
        let engine = self.engine;
        let f = f.clamp(0.0, 1.0);
        if f != 0.0 {
            let w = cmp::max(1, (f * width as f32) as c_int);
            engine.fill_rgba(x, y, w, height, RGB::GREENISH.rgba(255));
            x += w;
            width -= w;
        }
        engine.fill_rgba(x, y, width, height, color.rgba(128));
        x + width
    }

    fn draw_ammo_bar(&self, state: &State, weapon: &Weapon, x: c_int, y: c_int) {
        if let Some(ammo) = weapon.ammo[0] {
            let count = state.inv.ammo_count(ammo.ty);
            if count == 0 {
                return;
            }
            let f = count as f32 / ammo.max as f32;
            let x = self.draw_bar(x, y, state.color, f);

            if let Some(ammo) = weapon.ammo[1] {
                let count = state.inv.ammo_count(ammo.ty);
                let f = count as f32 / ammo.max as f32;
                self.draw_bar(x + 5, y, state.color, f);
            }
        }
    }
}

impl HudItem for WeaponMenu {
    fn vid_init(&mut self, state: &mut State) {
        self.bucket0 = state.find_sprite_index("bucket1");
        self.selection = state.find_sprite("selection");

        let engine = self.engine;
        let screen = engine.screen_info();
        let scale = screen.scale();

        self.ab_width = 10 * scale as c_int;
        self.ab_height = 2 * scale as c_int;
    }

    fn reset(&mut self) {
        self.active = Select::None;
        self.last = Select::None;

        self.engine.unset_crosshair();
    }

    fn think(&mut self, state: &mut State) {
        if self.active != Select::None && state.key_bits & consts::IN_ATTACK != 0 {
            let engine = self.engine;

            if let Select::Weapon(id, _, _) = self.active {
                if let Some(weapon) = state.inv.weapon(id) {
                    engine.server_cmd(&weapon.name);
                    self.weapon_select = weapon.id;
                }
            }

            self.last = self.active;
            self.active = Select::None;

            state.key_bits &= !consts::IN_ATTACK;

            engine.play_sound_by_name(c"common/wpn_select.wav", 1.0);
        }
    }

    fn draw(&mut self, state: &mut State) {
        if !state.has_suit() || state.is_hidden(Hide::WEAPONS | Hide::ALL) {
            return;
        }

        let active_slot = match self.active {
            Select::Weapon(_, slot, _) => Some(slot),
            Select::Menu => None,
            Select::None => return,
        };

        let engine = self.engine;

        let mut x = 10;
        let y = 10;

        for slot in 0..MAX_WEAPON_SLOTS as u32 {
            let a = match active_slot {
                Some(s) if s == slot => 255,
                _ => 192,
            };

            let color = state.color.scale_color(a);

            if let Some(index) = self.bucket0 {
                let sprite = state.sprites[index + slot as usize];
                engine.spr_set(sprite.hspr, color);

                let width = match active_slot {
                    Some(s) if s == slot => state
                        .inv
                        .get_first_pos(s)
                        .and_then(|weapon| weapon.active)
                        .map_or(sprite.rect.width(), |s| s.rect.width()),
                    _ => sprite.rect.width(),
                };

                engine.spr_draw_additive_rect(0, x, y, sprite.rect);

                x += width + 5;
            }
        }

        let mut x = 10;

        let (bucket_width, bucket_height) = match self.bucket0 {
            Some(index) => state.sprites[index].rect.size(),
            None => (0, 0),
        };

        for slot in 0..MAX_WEAPON_SLOTS as u32 {
            let mut y = bucket_height + 10;

            if matches!(active_slot, Some(s) if s == slot) {
                let width = match state.inv.get_first_pos(slot).and_then(|i| i.active) {
                    Some(sprite) => sprite.rect.width(),
                    None => bucket_width,
                };

                for pos in 0..MAX_WEAPON_POSITIONS as u32 {
                    let Some(weapon) = state.inv.slot_weapon(slot, pos) else {
                        continue;
                    };

                    let color = state.color;

                    if matches!(self.active, Select::Weapon(id, ..) if id == weapon.id) {
                        if let Some(sprite) = weapon.active {
                            engine.spr_set(sprite.hspr, color);
                            engine.spr_draw_additive_rect(0, x, y, sprite.rect);
                        }

                        if let Some(sprite) = self.selection {
                            engine.spr_set(sprite.hspr, color);
                            engine.spr_draw_additive_rect(0, x, y, sprite.rect);
                        }
                    } else if let Some(sprite) = weapon.inactive {
                        let color = if state.inv.has_ammo(weapon) {
                            color.scale_color(128)
                        } else {
                            RGB::REDISH.scale_color(96)
                        };

                        engine.spr_set(sprite.hspr, color);
                        engine.spr_draw_additive_rect(0, x, y, sprite.rect);
                    }

                    self.draw_ammo_bar(state, weapon, x + self.ab_width / 2, y);

                    y += weapon.active.unwrap().rect.height() + 5;
                }

                x += width + 5;
            } else {
                for pos in 0..MAX_WEAPON_POSITIONS as u32 {
                    let Some(weapon) = state.inv.slot_weapon(slot, pos) else {
                        continue;
                    };

                    let color = if state.inv.has_ammo(weapon) {
                        state.color.rgba(128)
                    } else {
                        RGB::REDISH.rgba(96)
                    };
                    engine.fill_rgba(x, y, bucket_width, bucket_height, color);
                    y += bucket_height + 5;
                }

                x += bucket_width + 5;
            }
        }
    }
}
