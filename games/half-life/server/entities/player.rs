use core::ffi::CStr;

use xash3d_hl_shared::user_message;
use xash3d_server::{
    entities::player::Player as BasePlayer,
    entity::{
        delegate_entity, delegate_player, impl_entity_cast, BaseEntity, CreateEntity, Effects,
        Entity, EntityPlayer, EntityVars, UseType,
    },
    prelude::*,
    save::{Restore, Save},
    time::MapTime,
    utils,
};

const WEAPON_SUIT: u32 = 1_u32 << 31;

const SOUND_FLASHLIGHT_ON: &CStr = res::valve::sound::items::FLASHLIGHT1;
const SOUND_FLASHLIGHT_OFF: &CStr = res::valve::sound::items::FLASHLIGHT1;

const FLASH_DRAIN_TIME: f32 = 1.2; // 100 units/3 minutes
const FLASH_CHARGE_TIME: f32 = 0.2; // 100 units/20 seconds (seconds per unit)

#[derive(Copy, Clone, Default)]
struct Geiger {
    range: f32,
    range_prev: u8,
    delay: MapTime,
}

impl Geiger {
    fn set_range(&mut self, range: f32) {
        if self.range >= range {
            self.range = range;
        }
    }

    fn set_delay(&mut self, delay: MapTime) {
        self.delay = delay;
    }

    fn update(&mut self, player: &EntityVars) {
        const GEIGER_DELAY: f32 = 0.25;

        let engine = player.engine();
        let now = engine.globals.map_time();
        if now < self.delay {
            return;
        }
        self.delay = now + GEIGER_DELAY;

        let range = (self.range / 4.0) as u8;
        if range != self.range_prev {
            self.range_prev = range;

            let msg = user_message::Geiger::new(range);
            engine.msg_one_reliable(player, &msg);
        }

        if engine.random_int(0, 3) == 0 {
            self.reset();
        }
    }

    fn reset(&mut self) {
        self.range = 1000.0;
        self.range_prev = 250;
    }
}

#[derive(Save, Restore)]
pub struct TestPlayer {
    base: BasePlayer,
    init_hud: bool,
    game_hud_initialized: bool,

    health: u8,
    battery: i16,

    /// Time until next battery draw/Recharge.
    flashlight_time: MapTime,
    /// Flashlight battery draw.
    flashlight_battery: u8,

    #[save(skip)]
    geiger: Geiger,
}

impl TestPlayer {
    fn has_suit(&self) -> bool {
        self.vars().weapons() & WEAPON_SUIT != 0
    }

    fn is_flashlight_on(&self) -> bool {
        self.vars().effects().intersects(Effects::DIMLIGHT)
    }

    fn flashlight_turn_on(&mut self) {
        let engine = self.engine();
        let global_state = self.global_state();
        if !global_state.game_rules().allow_flashlight() || !self.has_suit() {
            return;
        }

        engine
            .build_sound()
            .channel_weapon()
            .emit_dyn(SOUND_FLASHLIGHT_ON, self);
        self.vars().with_effects(|f| f | Effects::DIMLIGHT);
        let msg = user_message::Flashlight::new(true, self.flashlight_battery);
        engine.msg_one(self, &msg);
        self.flashlight_time = engine.globals.map_time() + FLASH_DRAIN_TIME;
    }

    fn flashlight_turn_off(&mut self) {
        let engine = self.engine();
        engine
            .build_sound()
            .channel_weapon()
            .emit_dyn(SOUND_FLASHLIGHT_OFF, self);
        self.vars()
            .with_effects(|f| f.difference(Effects::DIMLIGHT));
        let msg = user_message::Flashlight::new(false, self.flashlight_battery);
        engine.msg_one(self, &msg);
        self.flashlight_time = engine.globals.map_time() + FLASH_CHARGE_TIME;
    }

    fn impulse_commands(&mut self) {
        let v = self.vars();
        let impulse = v.impulse();
        v.set_impulse(0);
        match impulse {
            0 => {}
            100 => {
                if !self.is_flashlight_on() {
                    self.flashlight_turn_on();
                } else {
                    self.flashlight_turn_off();
                }
            }
            impulse => {
                warn!("unimplemented impulse command {impulse}");
            }
        }
    }

    fn check_suit_update(&mut self) {
        if !self.has_suit() {
            return;
        }

        self.geiger.update(self.base.vars());

        // if self.global_state().game_rules().is_multiplayer() {
        //     return;
        // }
    }

    fn client_update_data(&mut self) {
        let engine = self.engine();
        let global_state = self.global_state();
        let time = engine.globals.map_time();

        if self.init_hud {
            self.init_hud = false;
            global_state.set_init_hud(false);

            engine.msg_one(self, &user_message::ResetHUD::default());

            if !self.game_hud_initialized {
                self.game_hud_initialized = true;
                engine.msg_one(self, &user_message::InitHUD::default());
            }

            utils::fire_targets(c"game_playerspawn".into(), UseType::Toggle, None, self);

            let msg =
                user_message::Flashlight::new(self.is_flashlight_on(), self.flashlight_battery);
            engine.msg_one(self, &msg);

            engine.msg_one(self, &user_message::Geiger::default());
        }

        // update flashlight
        if self.flashlight_time != 0.0 && self.flashlight_time <= time {
            if self.is_flashlight_on() {
                if self.flashlight_battery != 0 {
                    self.flashlight_time = time + FLASH_DRAIN_TIME;
                    self.flashlight_battery -= 1;
                    if self.flashlight_battery == 0 {
                        self.flashlight_turn_off();
                    }
                }
            } else if self.flashlight_battery < 100 {
                self.flashlight_time = time + FLASH_CHARGE_TIME;
                self.flashlight_battery += 1;
            } else {
                self.flashlight_time = MapTime::ZERO;
            }

            trace!("send flashlight battery {}%", self.flashlight_battery);
            let msg = user_message::FlashBat::new(self.flashlight_battery);
            self.engine().msg_one(self, &msg);
        }
    }
}

impl_entity_cast!(TestPlayer);

impl CreateEntity for TestPlayer {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BasePlayer::create(base),
            init_hud: true,
            game_hud_initialized: false,

            health: 100,
            battery: 0,

            flashlight_time: MapTime::ZERO,
            flashlight_battery: 100,

            geiger: Geiger::default(),
        }
    }
}

impl Entity for TestPlayer {
    delegate_entity!(base not { precache, spawn, think });

    fn precache(&mut self) {
        self.base.precache();

        self.geiger.reset();

        let engine = self.engine();
        engine.precache_sound(SOUND_FLASHLIGHT_ON);
        engine.precache_sound(SOUND_FLASHLIGHT_OFF);

        // force message after level change
        self.flashlight_time = MapTime::from_secs_f32(1.0);

        if self.global_state().init_hud() {
            self.init_hud = true;
        }
    }

    fn spawn(&mut self) {
        self.base.spawn();
        let engine = self.engine();

        // wait a few seconds until user-defined message registrations are recived by all clients
        self.geiger.set_delay(engine.globals.map_time() + 2.0);

        // enable suit
        // TODO: move Weapons type to shared crate
        self.vars().with_weapons(|f| f | WEAPON_SUIT);

        self.precache();

        self.vars().set_next_think_time_from_now(0.1);

        self.init_hud = true;
    }

    fn think(&mut self) {
        let engine = self.engine();

        self.health -= 1;
        engine.msg_one(
            self,
            &user_message::Health {
                health: self.health,
            },
        );
        if self.health == 0 {
            self.health = 100;
        }

        self.battery += 1;
        engine.msg_one(
            self,
            &user_message::Battery {
                battery: self.battery,
            },
        );
        if self.battery >= 100 {
            self.battery = 0;
        }

        self.vars().set_next_think_time_from_now(0.1);
    }
}

impl EntityPlayer for TestPlayer {
    delegate_player!(base not { pre_think, post_think, set_geiger_range });

    fn pre_think(&mut self) {
        self.base.pre_think();

        if self.base.check_player_use() {
            self.base.player_use();
        }

        self.client_update_data();

        self.check_suit_update();
    }

    fn post_think(&mut self) {
        self.impulse_commands();

        self.base.post_think();
    }

    fn set_geiger_range(&mut self, range: f32) {
        self.geiger.set_range(range);
    }
}
