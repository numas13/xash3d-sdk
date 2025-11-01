use core::{
    cell::{Cell, RefCell},
    ffi::CStr,
};

use csz::CStrThin;
use xash3d_entities::player::Player as BasePlayer;
use xash3d_server::{
    color::RGB,
    entity::{
        delegate_entity, delegate_player, BaseEntity, Buttons, Effects, EntityPlayer, EntityVars,
        UseType,
    },
    prelude::*,
    private::impl_private,
    save::{Restore, Save},
    str::MapString,
    time::MapTime,
    utils,
};

use crate::user_message;

pub const WEAPON_SUIT: u32 = 1_u32 << 31;
pub const MAX_NORMAL_BATTERY: f32 = 100.0;

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

#[derive(Default, Save, Restore)]
struct Flashlight {
    /// Time until next battery draw/Recharge.
    time: Cell<MapTime>,
    /// Flashlight battery draw.
    battery: Cell<u8>,
}

#[derive(Default)]
struct ClientState {
    health: Cell<f32>,
    battery: Cell<f32>,
}

#[derive(Save, Restore)]
pub struct TestPlayer {
    base: BasePlayer,
    init_hud: Cell<bool>,
    game_hud_initialized: Cell<bool>,

    flashlight: Flashlight,

    #[save(skip)]
    geiger: RefCell<Geiger>,

    #[save(skip)]
    client: ClientState,

    find: Cell<Option<MapString>>,
}

impl CreateEntity for TestPlayer {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BasePlayer::create(base),
            init_hud: Cell::new(true),
            game_hud_initialized: Cell::new(false),

            flashlight: Flashlight {
                time: Cell::default(),
                battery: Cell::new(100),
            },

            geiger: Default::default(),

            client: ClientState::default(),

            find: Cell::default(),
        }
    }
}

impl TestPlayer {
    fn has_suit(&self) -> bool {
        self.vars().weapons() & WEAPON_SUIT != 0
    }

    fn is_flashlight_on(&self) -> bool {
        self.vars().effects().intersects(Effects::DIMLIGHT)
    }

    fn flashlight_turn_on(&self) {
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
        let msg = user_message::Flashlight::new(true, self.flashlight.battery.get());
        engine.msg_one(self, &msg);
        self.flashlight
            .time
            .set(engine.globals.map_time() + FLASH_DRAIN_TIME);
    }

    fn flashlight_turn_off(&self) {
        let engine = self.engine();
        engine
            .build_sound()
            .channel_weapon()
            .emit_dyn(SOUND_FLASHLIGHT_OFF, self);
        self.vars()
            .with_effects(|f| f.difference(Effects::DIMLIGHT));
        let msg = user_message::Flashlight::new(false, self.flashlight.battery.get());
        engine.msg_one(self, &msg);
        self.flashlight
            .time
            .set(engine.globals.map_time() + FLASH_CHARGE_TIME);
    }

    fn impulse_commands(&self) {
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
            101 => {
                self.give_named_item(c"item_suit".into());
            }
            impulse => {
                warn!("unimplemented impulse command {impulse}");
            }
        }
    }

    fn check_suit_update(&self) {
        if !self.has_suit() {
            return;
        }

        self.geiger.borrow_mut().update(self.base.vars());

        // if self.global_state().game_rules().is_multiplayer() {
        //     return;
        // }
    }

    fn update_client_data(&self) {
        let engine = self.engine();
        let global_state = self.global_state();
        let v = self.vars();
        let time = engine.globals.map_time();

        if self.init_hud.get() {
            self.init_hud.set(false);
            global_state.set_init_hud(false);

            engine.msg_one_reliable(self, &user_message::ResetHUD::default());

            if !self.game_hud_initialized.get() {
                self.game_hud_initialized.set(true);
                engine.msg_one_reliable(self, &user_message::InitHUD::default());
            }

            utils::fire_targets(c"game_playerspawn".into(), UseType::Toggle, None, self);

            let msg = user_message::Flashlight::new(
                self.is_flashlight_on(),
                self.flashlight.battery.get(),
            );
            engine.msg_one_reliable(self, &msg);

            engine.msg_one_reliable(self, &user_message::Geiger::default());
        }

        if v.health() != self.client.health.get() {
            let health = if v.health() > 0.0 && v.health() < 1.0 {
                1
            } else {
                v.health() as u8
            };
            let msg = user_message::Health::new(health);
            engine.msg_one_reliable(v, &msg);
            self.client.health.set(v.health());
        }

        if v.armor_value() != self.client.battery.get() {
            let msg = user_message::Battery::new(v.armor_value() as i16);
            engine.msg_one_reliable(v, &msg);
            self.client.battery.set(v.armor_value());
        }

        // update flashlight
        let flashlight = &self.flashlight;
        if flashlight.time.get() != 0.0 && flashlight.time.get() <= time {
            if self.is_flashlight_on() {
                if flashlight.battery.get() != 0 {
                    flashlight.time.set(time + FLASH_DRAIN_TIME);
                    flashlight.battery.set(flashlight.battery.get() - 1);
                    if flashlight.battery.get() == 0 {
                        self.flashlight_turn_off();
                    }
                }
            } else if flashlight.battery.get() < 100 {
                flashlight.time.set(time + FLASH_CHARGE_TIME);
                flashlight.battery.set(flashlight.battery.get() + 1);
            } else {
                flashlight.time.set(MapTime::ZERO);
            }

            trace!("send flashlight battery {}%", flashlight.battery.get());
            let msg = user_message::FlashBat::new(flashlight.battery.get());
            self.engine().msg_one_reliable(self, &msg);
        }
    }

    pub fn force_update_client_data(&self) {
        self.client.health.set(-1.0);
        self.client.battery.set(1.0);
        self.init_hud.set(true);

        self.update_client_data();
    }

    pub fn find_class_name(&self, class_name: &CStrThin) {
        let v = self.vars();
        if class_name.is_empty() {
            v.stop_thinking();
            self.find.set(None);
            return;
        }
        info!("searching {class_name}");
        self.find
            .set(Some(self.engine().new_map_string(class_name)));
        v.set_next_think_time_from_now(0.1)
    }
}

impl Entity for TestPlayer {
    delegate_entity!(base not { precache, spawn, think });

    fn precache(&mut self) {
        self.base.precache();

        self.geiger.borrow_mut().reset();

        let engine = self.engine();
        engine.precache_sound(SOUND_FLASHLIGHT_ON);
        engine.precache_sound(SOUND_FLASHLIGHT_OFF);

        // force message after level change
        self.flashlight.time.set(MapTime::from_secs_f32(1.0));

        if self.global_state().init_hud() {
            self.init_hud.set(true);
        }
    }

    fn spawn(&mut self) {
        self.base.spawn();
        let engine = self.engine();

        // wait a few seconds until user-defined message registrations are recived by all clients
        self.geiger
            .borrow_mut()
            .set_delay(engine.globals.map_time() + 2.0);

        // enable suit
        // TODO: move Weapons type to shared crate
        //self.vars().with_weapons(|f| f | WEAPON_SUIT);

        self.precache();

        self.init_hud.set(true);

        if self.find.get().is_some() {
            self.vars().set_next_think_time_from_now(0.1);
        }
    }

    fn think(&self) {
        if let Some(classname) = self.find.get() {
            let engine = self.engine();
            let v = self.vars();
            let start = v.origin() + v.view_ofs() * 0.5;
            for i in engine.entities().by_class_name(classname) {
                let end = i.vars().bmodel_origin();
                if (start - end).length() > 1000.0 {
                    continue;
                }
                let msg = user_message::Line {
                    start: start.into(),
                    end: end.into(),
                    duration: 0.2.into(),
                    color: RGB::GREEN,
                };
                engine.msg_one_reliable(v, &msg)
            }
            v.set_next_think_time_from_now(0.2);
        }
    }
}

impl EntityPlayer for TestPlayer {
    delegate_player!(base not { pre_think, post_think, set_geiger_range });

    fn pre_think(&self) {
        self.base.pre_think();

        if self.base.check_player_use() {
            self.base.player_use_custom(|target, use_type| {
                target.used(use_type, Some(self), self);
            });
        }

        self.update_client_data();

        self.check_suit_update();

        let pressed = self.base.input.pressed();
        if pressed.intersects(Buttons::ATTACK | Buttons::ATTACK2) {
            use xash3d_server::color::RGB;
            use xash3d_server::engine::TraceIgnore;
            use xash3d_server::user_message;
            use xash3d_server::utils;

            let engine = self.engine();
            let global_state = engine.global_state_ref();
            let v = self.vars();
            let start = v.origin() + v.view_ofs() * 0.5;
            let forward = v.view_angle().angle_vectors().forward();
            let end = start + forward * 1000.0;
            let trace = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(v));

            if true {
                let decals = global_state.decals();
                let decal_index = if pressed.intersects(Buttons::ATTACK) {
                    decals.get_random_blood()
                } else {
                    use xash3d_server::global_state::decals::DefaultDecals;
                    let decals: &DefaultDecals = decals.as_any().downcast_ref().unwrap();
                    decals.get_random_yellow_blood()
                };
                utils::decal_trace(&engine, &trace, decal_index);
            }

            if true {
                let blood = utils::Blood::Red;
                let end = trace.end_position();
                if pressed.intersects(Buttons::ATTACK) {
                    blood.emit_blood_drips(&engine, end, 10);
                } else {
                    let amount = engine.random_int(50, 150) as u8;
                    blood.emit_blood_stream(&engine, end, -forward, amount);
                }
            }

            for _ in 0..engine.random_int(1, 3) {
                use xash3d_server::entity::create_entity;
                let pos = trace.end_position();
                let angles = trace.plane_normal();
                create_entity(&engine, c"spark_shower", pos, angles, None).ok();
            }

            let msg = user_message::Line {
                start: start.into(),
                end: trace.end_position().into(),
                duration: 3.0.into(),
                color: RGB::GREEN,
            };
            engine.msg_one(v, &msg);
        }
    }

    fn post_think(&self) {
        self.impulse_commands();

        self.base.post_think();
    }

    fn set_geiger_range(&self, range: f32) {
        self.geiger.borrow_mut().set_range(range);
    }
}

impl_private!(TestPlayer { EntityPlayer });
