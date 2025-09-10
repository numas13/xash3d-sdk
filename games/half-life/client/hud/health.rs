use core::ffi::c_int;

use alloc::collections::VecDeque;
use bitflags::bitflags;
use cl::{
    color::RGB,
    ffi::common::vec3_t,
    macros::spr_load,
    math::{fabsf, fmaxf, sinf},
    message::hook_message,
    prelude::*,
    sprite::SpriteHandle,
};

use crate::{
    export::{hud, hud_mut},
    hud::{try_spr_load, Fade, Hide, Sprite, State},
};

// seconds that image is up
const DMG_IMAGE_LIFE: f32 = 2.0;

// const DMG_IMAGE_POISON: c_int = 0;
// const DMG_IMAGE_ACID: c_int = 1;
// const DMG_IMAGE_COLD: c_int = 2;
// const DMG_IMAGE_DROWN: c_int = 3;
// const DMG_IMAGE_BURN: c_int = 4;
// const DMG_IMAGE_NERVE: c_int = 5;
// const DMG_IMAGE_RAD: c_int = 6;
// const DMG_IMAGE_SHOCK: c_int = 7;

const NUM_DMG_TYPES: usize = 8;

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    struct DamageFlags: u32 {
        const GENERIC = 0;

        const CRUSH = 1 << 0;
        const BULLET = 1 << 1;
        const SLASH = 1 << 2;
        const BURN = 1 << 3;
        const FREEZE = 1 << 4;
        const FALL = 1 << 5;
        const BLAST = 1 << 6;
        const CLUB = 1 << 7;
        const SHOCK = 1 << 8;
        const SONIC = 1 << 9;
        const ENERGYBEAM = 1 << 10;
        const NEVERGIB = 1 << 12;
        const ALWAYSGIB = 1 << 13;

        const TIMEBASED = !0xff003fff;

        const DROWN = 1 << 14;
        const FIRSTTIMEBASED = Self::DROWN.bits();

        const PARALYZE = 1 << 15;
        const NERVEGAS = 1 << 16;
        const POISON = 1 << 17;
        const RADIATION = 1 << 18;
        const DROWNRECOVER = 1 << 19;
        const ACID = 1 << 20;
        const SLOWBURN = 1 << 21;
        const SLOWFREEZE = 1 << 22;
        const MORTAR = 1 << 23;
    }
}

const DAMAGE_FLAGS: [DamageFlags; NUM_DMG_TYPES] = [
    DamageFlags::POISON,
    DamageFlags::ACID,
    DamageFlags::FREEZE.union(DamageFlags::SLOWFREEZE),
    DamageFlags::DROWN,
    DamageFlags::BURN.union(DamageFlags::SLOWBURN),
    DamageFlags::NERVEGAS,
    DamageFlags::RADIATION,
    DamageFlags::SHOCK,
];

#[derive(Copy, Clone)]
struct DamageImage {
    index: usize,
    expire: f32,
    flags: DamageFlags,
}

const ATTACK_FRONT: usize = 0;
const ATTACK_RIGHT: usize = 1;
const ATTACK_REAR: usize = 2;
const ATTACK_LEFT: usize = 3;

pub struct Health {
    current: u8,
    fade: Fade,
    cross: Option<Sprite>,
    pain_sprite: Option<SpriteHandle>,
    attack: [f32; 4],
    damages: VecDeque<DamageImage>,
    dmg_spr_index: Option<usize>,
}

impl Health {
    pub fn new() -> Self {
        hook_message!(Health, |msg| {
            let x = msg.read_u8()?;
            hud().items.get_mut::<Health>().set(x);
            Ok(())
        });

        hook_message!(Damage, |msg| {
            let armor = msg.read_u8()?;
            let damage_taken = msg.read_u8()?;
            let damage_bits = msg.read_u32()?;
            let from = vec3_t::new(msg.read_coord()?, msg.read_coord()?, msg.read_coord()?);

            let hud = &mut *hud_mut();
            let mut health = hud.items.get_mut::<Health>();

            if damage_bits != 0 {
                let damage_flags = DamageFlags::from_bits(damage_bits).unwrap_or_else(|| {
                    warn!("Damage: unexpected damage flags {damage_bits:08x}");
                    DamageFlags::from_bits_retain(damage_bits)
                });
                health.update_tiles(&hud.state, damage_flags);
            }

            if damage_taken > 0 || armor > 0 {
                health.calc_damage_direction(&mut hud.state, from);
            }

            Ok(())
        });

        Self {
            current: 100,
            fade: Fade::default(),
            cross: None,
            pain_sprite: None,
            attack: [0.0; 4],
            damages: Default::default(),
            dmg_spr_index: None,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current == 0
    }

    pub fn set(&mut self, new: u8) {
        // TODO: set active

        if self.current != new {
            self.current = new;
            self.fade.start();
        }
    }

    fn get_pain_color(&self) -> Option<RGB> {
        if self.current <= 25 {
            Some(RGB::new(250, 0, 0))
        } else {
            None
        }
    }

    fn draw_health(&mut self, engine: &ClientEngine, state: &mut State) {
        let Some(cross) = self.cross else { return };

        let a = if self.current > 15 {
            self.fade.alpha(state.time_delta)
        } else {
            255
        };

        let color = self.get_pain_color().unwrap_or(state.color).scale_color(a);

        let screen_info = engine.get_screen_info();
        let cross_width = cross.rect.width();
        let mut x = cross_width / 2;
        let mut y = screen_info.iHeight - state.num_height - state.num_height / 2;

        engine.spr_set(cross.hspr, color);
        engine.spr_draw_additive_rect(0, x, y, cross.rect);

        x = cross_width + state.num_width / 2;
        y += (state.num_height as f32 * 0.2) as c_int;

        x = state
            .draw_number(self.current.into())
            .width(3)
            .color(color)
            .at(x, y);

        x += state.num_width / 2;

        let height = state.num_height;
        let width = state.num_width / 10;
        engine.fill_rgba(x, y, width, height, state.color.rgba(a));
    }

    fn update_tiles(&mut self, state: &State, mut damage_flags: DamageFlags) {
        for i in &mut self.damages {
            if i.flags.intersects(damage_flags) {
                i.expire = state.time + DMG_IMAGE_LIFE;
                damage_flags.remove(i.flags);
            }
        }

        for (index, flags) in DAMAGE_FLAGS.into_iter().enumerate() {
            if flags.intersects(damage_flags) {
                let image = DamageImage {
                    index,
                    expire: state.time + DMG_IMAGE_LIFE,
                    flags,
                };
                while self.damages.len() >= NUM_DMG_TYPES {
                    self.damages.pop_back();
                }
                self.damages.push_front(image);
            }
        }
    }

    fn draw_damage(&mut self, engine: &ClientEngine, state: &mut State) {
        if self.damages.is_empty() {
            return;
        }

        let Some(index) = self.dmg_spr_index else {
            return;
        };
        let sprites = &state.sprites[index..];

        let a = (fabsf(sinf(state.time * 2.0)) * 256.0) as u8;
        let color = state.color.scale_color(a);

        let width = sprites[0].rect.width();
        let height = sprites[0].rect.height();

        let screen = engine.get_screen_info();
        let x = width / 8;
        let mut y = screen.iHeight - height * 2;

        for i in &self.damages {
            engine.spr_set(sprites[i.index].hspr, color);
            engine.spr_draw_additive_rect(0, x, y, sprites[i.index].rect);
            y -= height;
        }

        if a < 40 {
            self.damages.retain(|i| i.expire > state.time);
        }
    }

    fn calc_damage_direction(&mut self, state: &mut State, from: vec3_t) {
        if from == vec3_t::ZERO {
            self.attack = [0.0; 4];
            return;
        }

        let from = from - state.origin;
        let dist_to_target = from.length();

        if dist_to_target <= 50.0 {
            self.attack = [1.0; 4];
        } else {
            let av = state.angles.angle_vectors();
            let from = from.normalize();
            let front = from.dot_product(av.right());
            let side = from.dot_product(av.forward());

            let mut attack = |i, f| {
                if f > 0.3 && self.attack[i] < f {
                    self.attack[i] = f;
                }
            };

            if side > 0.0 {
                attack(ATTACK_FRONT, side);
            } else {
                attack(ATTACK_REAR, fabsf(side));
            }

            if front > 0.0 {
                attack(ATTACK_RIGHT, front);
            } else {
                attack(ATTACK_LEFT, fabsf(front));
            }
        }
    }

    fn draw_pain(&mut self, engine: &ClientEngine, state: &mut State) {
        if self.attack == [0.0; 4] {
            return;
        }

        let Some(hspr) = self.pain_sprite else { return };

        let a = 255;
        let fade = (state.time_delta * 2.0) as f32;
        let color = self.get_pain_color().unwrap_or(state.color);
        let screen = engine.get_screen_info();
        let x = screen.iWidth / 2;
        let y = screen.iHeight / 2;

        for i in 0..4 {
            if self.attack[i] > 0.4 {
                let color = color.scale_color((a as f32 * fmaxf(self.attack[i], 0.5)) as u8);
                engine.spr_set(hspr, color);

                let frame = i as c_int;
                let (w, h) = engine.spr_size(hspr, frame);
                let (x, y) = match i {
                    ATTACK_FRONT => (x - w / 2, y - h * 3),
                    ATTACK_RIGHT => (x + w * 2, y - h / 2),
                    ATTACK_REAR => (x - w / 2, y + h * 2),
                    ATTACK_LEFT => (x - w * 3, y - h / 2),
                    _ => unreachable!(),
                };
                engine.spr_draw_additive(frame, x, y);
                self.attack[i] = fmaxf(0.0, self.attack[i] - fade);
            } else {
                self.attack[i] = 0.0;
            };
        }
    }
}

impl super::HudItem for Health {
    fn vid_init(&mut self, state: &mut State) {
        self.cross = state.find_sprite("cross");
        self.pain_sprite = try_spr_load(state.res, |res| spr_load!("sprites/{res}_pain.spr"));
        self.dmg_spr_index = state.find_sprite_index("dmg_bio").map(|i| i + 1);
    }

    fn reset(&mut self) {
        self.attack = [0.0; 4];
        self.damages.clear();
    }

    fn draw(&mut self, state: &mut State) {
        let engine = engine();

        if state.is_hidden(Hide::HEALTH) || engine.is_spectator_only() || !state.has_suit() {
            return;
        }

        self.draw_health(engine, state);
        self.draw_damage(engine, state);
        self.draw_pain(engine, state);
    }
}
