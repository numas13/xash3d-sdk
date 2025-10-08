use core::{ffi::CStr, mem};

use xash3d_hl_shared::user_message;
use xash3d_server::{
    entities::player::Player as BasePlayer,
    entity::{
        delegate_entity, delegate_player, impl_entity_cast, AsEdict, BaseEntity, CreateEntity,
        Effects, Entity, EntityPlayer, UseType::Toggle,
    },
    save::{Restore, Save},
    time::MapTime,
    utils,
};

const WEAPON_SUIT: i32 = (1_u32 << 31) as i32;

const SOUND_FLASHLIGHT_ON: &CStr = res::valve::sound::items::FLASHLIGHT1;
const SOUND_FLASHLIGHT_OFF: &CStr = res::valve::sound::items::FLASHLIGHT1;

const FLASH_DRAIN_TIME: f32 = 1.2; // 100 units/3 minutes
const FLASH_CHARGE_TIME: f32 = 0.2; // 100 units/20 seconds (seconds per unit)

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
}

impl TestPlayer {
    fn has_suit(&self) -> bool {
        self.vars().as_raw().weapons & WEAPON_SUIT != 0
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
            .emit_dyn(SOUND_FLASHLIGHT_ON, self.as_edict_mut());
        self.vars_mut().effects_mut().insert(Effects::DIMLIGHT);
        let msg = user_message::Flashlight::new(true, self.flashlight_battery);
        engine.msg_one(self, &msg);
        self.flashlight_time = engine.globals.map_time() + FLASH_DRAIN_TIME;
    }

    fn flashlight_turn_off(&mut self) {
        let engine = self.engine();
        engine
            .build_sound()
            .channel_weapon()
            .emit_dyn(SOUND_FLASHLIGHT_OFF, self.as_edict_mut());
        self.vars_mut().effects_mut().remove(Effects::DIMLIGHT);
        let msg = user_message::Flashlight::new(false, self.flashlight_battery);
        engine.msg_one(self, &msg);
        self.flashlight_time = engine.globals.map_time() + FLASH_CHARGE_TIME;
    }

    fn impulse_commands(&mut self) {
        match mem::take(&mut self.vars_mut().as_raw_mut().impulse) {
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

            utils::fire_targets(c"game_playerspawn".into(), Toggle, 0.0, self, None);

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
            self.engine().msg_one(self.as_edict_mut(), &msg);
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
        }
    }
}

impl Entity for TestPlayer {
    delegate_entity!(base not { precache, spawn, think });

    fn precache(&mut self) {
        self.base.precache();

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

        // enable suit
        // TODO: move Weapons type to shared crate
        self.vars_mut().as_raw_mut().weapons |= WEAPON_SUIT;

        self.precache();

        self.vars_mut().set_next_think_time(0.1);

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

        self.vars_mut().set_next_think_time(0.1);
    }
}

impl EntityPlayer for TestPlayer {
    delegate_player!(base not { pre_think, post_think });

    fn pre_think(&mut self) {
        self.client_update_data();
    }

    fn post_think(&mut self) {
        self.impulse_commands();
    }
}
