use core::ffi::CStr;

use bitflags::bitflags;

use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, Entity, KeyValue},
    global_state::GlobalStateRef,
    prelude::*,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    struct WorldSpawnFlags: u32 {
        /// Fade from black at startup.
        const DARK = 1 << 0;
        /// Display game title at startup.
        const TITLE = 1 << 1;
        /// Force teams.
        const FORCE_TEAM = 1 << 2;
    }
}

pub type InstallGameRulesFn = fn(ServerEngineRef, GlobalStateRef);

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct World {
    base: BaseEntity,
    #[cfg_attr(feature = "save", save(skip))]
    install_game_rules: InstallGameRulesFn,
}

impl World {
    /// Fade from black at startup.
    pub const SF_DARK: i32 = 1 << 0;
    /// Display game title at startup.
    pub const SF_TITLE: i32 = 1 << 1;
    /// Force teams.
    pub const SF_FORCE_TEAM: i32 = 1 << 2;

    pub fn create(base: BaseEntity, install_game_rules: InstallGameRulesFn) -> Self {
        Self {
            base,
            install_game_rules,
        }
    }
}

impl_entity_cast!(World);

impl Entity for World {
    delegate_entity!(base not { key_value, precache, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        let class_name = data.class_name();
        let key_name = data.key_name();
        let value = data.value();
        let handled = data.handled();
        debug!("World::key_value({class_name:?}, {key_name}, {value}, {handled})");
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.engine();
        let global_state = self.global_state();

        global_state.set_last_spawn(None);

        engine.set_cvar(c"sv_gravity", c"800");
        engine.set_cvar(c"sv_stepsize", c"18");
        engine.set_cvar(c"room_type", c"0");

        (self.install_game_rules)(engine, self.global_state());

        // TODO: spawn sound entity
        // TODO: init bodyque

        self.global_state().sentence_init();

        // TODO: precache weapons

        client_precache(engine);

        // sounds used from C physics code
        const PRECACHE_SOUNDS: &[&CStr] = &[
            // clears sound channels
            res::valve::sound::common::NULL,
            // temporary sound for respawning weapons.
            res::valve::sound::items::SUITCHARGEOK1,
            // player picks up a gun.
            // res::valve::sound::items::GUNPICKUP1,
            res::valve::sound::items::GUNPICKUP2,
            // res::valve::sound::items::GUNPICKUP3,
            // res::valve::sound::items::GUNPICKUP4,

            // dead bodies hitting the ground (animation events)
            // res::valve::sound::common::BODYDROP1,
            // res::valve::sound::common::BODYDROP2,
            res::valve::sound::common::BODYDROP3,
            res::valve::sound::common::BODYDROP4,
            res::valve::sound::weapons::RIC1,
            res::valve::sound::weapons::RIC2,
            res::valve::sound::weapons::RIC3,
            res::valve::sound::weapons::RIC4,
            res::valve::sound::weapons::RIC5,
        ];

        for i in PRECACHE_SOUNDS {
            engine.precache_sound(*i);
        }

        engine.precache_model(res::valve::models::HGIBS);
        engine.precache_model(res::valve::models::AGIBS);

        // Setup light animation tables. 'a' is total darkness, 'z' is maxbright.
        const LIGHT_STYLES: &[(i32, &CStr)] = &[
            // 0 normal
            (0, c"m"),
            // 1 FLICKER (first variety)
            (1, c"mmnmmommommnonmmonqnmmo"),
            // 2 SLOW STRONG PULSE
            (2, c"abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcba"),
            // 3 CANDLE (first variety)
            (3, c"mmmmmaaaaammmmmaaaaaabcdefgabcdefg"),
            // 4 FAST STROBE
            (4, c"mamamamamama"),
            // 5 GENTLE PULSE 1
            (5, c"jklmnopqrstuvwxyzyxwvutsrqponmlkj"),
            // 6 FLICKER (second variety)
            (6, c"nmonqnmomnmomomno"),
            // 7 CANDLE (second variety)
            (7, c"mmmaaaabcdefgmmmmaaaammmaamm"),
            // 8 CANDLE (third variety)
            (8, c"mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa"),
            // 9 SLOW STROBE (fourth variety)
            (9, c"aaaaaaaazzzzzzzz"),
            // 10 FLUORESCENT FLICKER
            (10, c"mmamammmmammamamaaamammma"),
            // 11 SLOW PULSE NOT FADE TO BLACK
            (11, c"abcdefghijklmnopqrrqponmlkjihgfedcba"),
            // 12 UNDERWATER LIGHT MUTATION
            // this light only distorts the lightmap - no contribution
            // is made to the brightness of affected surfaces
            (12, c"mmnnmmnnnmmnn"),
            // styles 32-62 are assigned by the light program for switchable lights
            // 63 testing
            (63, c"a"),
        ];
        for (style, value) in LIGHT_STYLES {
            engine.light_style(*style, *value);
        }

        // TODO: init decals
        // TODO: init world graph

        let v = self.vars();
        let zmax = if v.speed() > 0.0 { v.speed() } else { 4096.0 };
        engine.set_cvar(c"sv_zmax", zmax);
        engine.set_cvar(c"sv_wateramp", v.scale());

        // TODO: if ev.netname

        let mut spawn_flags = WorldSpawnFlags::from_bits_retain(v.spawn_flags());
        engine.set_cvar(c"v_dark", spawn_flags.intersects(WorldSpawnFlags::DARK));
        engine.set_cvar(
            c"mp_defaultteam",
            spawn_flags.intersects(WorldSpawnFlags::FORCE_TEAM),
        );

        // TODO: display world title

        // do not apply fade after save/restore
        spawn_flags.remove(WorldSpawnFlags::DARK | WorldSpawnFlags::TITLE);
        v.set_spawn_flags(spawn_flags.bits());
    }

    fn spawn(&mut self) {
        // TODO: global_game_over = false;
        self.precache();
    }
}

pub fn client_precache(engine: ServerEngineRef) {
    // setup precaches always needed
    const PRECACHE_SOUNDS: &[&CStr] = &[
        // spray paint sound for PreAlpha
        res::valve::sound::player::SPRAYER,
        // fall pain
        // res::valve::sound::player::PL_FALLPAIN1,
        res::valve::sound::player::PL_FALLPAIN2,
        res::valve::sound::player::PL_FALLPAIN3,
        // walk on concrete
        res::valve::sound::player::PL_STEP1,
        res::valve::sound::player::PL_STEP2,
        res::valve::sound::player::PL_STEP3,
        res::valve::sound::player::PL_STEP4,
        // NPC walk on concrete
        res::valve::sound::common::NPC_STEP1,
        res::valve::sound::common::NPC_STEP2,
        res::valve::sound::common::NPC_STEP3,
        res::valve::sound::common::NPC_STEP4,
        // walk on metal
        res::valve::sound::player::PL_METAL1,
        res::valve::sound::player::PL_METAL2,
        res::valve::sound::player::PL_METAL3,
        res::valve::sound::player::PL_METAL4,
        // walk on dirt
        res::valve::sound::player::PL_DIRT1,
        res::valve::sound::player::PL_DIRT2,
        res::valve::sound::player::PL_DIRT3,
        res::valve::sound::player::PL_DIRT4,
        // walk in duct
        res::valve::sound::player::PL_DUCT1,
        res::valve::sound::player::PL_DUCT2,
        res::valve::sound::player::PL_DUCT3,
        res::valve::sound::player::PL_DUCT4,
        // walk on grate
        res::valve::sound::player::PL_GRATE1,
        res::valve::sound::player::PL_GRATE2,
        res::valve::sound::player::PL_GRATE3,
        res::valve::sound::player::PL_GRATE4,
        // walk in shallow water
        res::valve::sound::player::PL_SLOSH1,
        res::valve::sound::player::PL_SLOSH2,
        res::valve::sound::player::PL_SLOSH3,
        res::valve::sound::player::PL_SLOSH4,
        // walk on tile
        res::valve::sound::player::PL_TILE1,
        res::valve::sound::player::PL_TILE2,
        res::valve::sound::player::PL_TILE3,
        res::valve::sound::player::PL_TILE4,
        res::valve::sound::player::PL_TILE5,
        // breathe bubbles
        res::valve::sound::player::PL_SWIM1,
        res::valve::sound::player::PL_SWIM2,
        res::valve::sound::player::PL_SWIM3,
        res::valve::sound::player::PL_SWIM4,
        // climb ladder rung
        res::valve::sound::player::PL_LADDER1,
        res::valve::sound::player::PL_LADDER2,
        res::valve::sound::player::PL_LADDER3,
        res::valve::sound::player::PL_LADDER4,
        // wade in water
        res::valve::sound::player::PL_WADE1,
        res::valve::sound::player::PL_WADE2,
        res::valve::sound::player::PL_WADE3,
        res::valve::sound::player::PL_WADE4,
        // hit wood texture
        res::valve::sound::debris::WOOD1,
        res::valve::sound::debris::WOOD2,
        res::valve::sound::debris::WOOD3,
        // use a train
        res::valve::sound::plats::TRAIN_USE1,
        // hit computer texture
        res::valve::sound::buttons::SPARK5,
        res::valve::sound::buttons::SPARK6,
        res::valve::sound::debris::GLASS1,
        res::valve::sound::debris::GLASS2,
        res::valve::sound::debris::GLASS3,
        // player gib sounds
        res::valve::sound::common::BODYSPLAT,
        // player pain sounds
        res::valve::sound::player::PL_PAIN2,
        res::valve::sound::player::PL_PAIN4,
        res::valve::sound::player::PL_PAIN5,
        res::valve::sound::player::PL_PAIN6,
        res::valve::sound::player::PL_PAIN7,
        // hud sounds
        res::valve::sound::common::WPN_HUDOFF,
        res::valve::sound::common::WPN_HUDON,
        res::valve::sound::common::WPN_MOVESELECT,
        res::valve::sound::common::WPN_SELECT,
        res::valve::sound::common::WPN_DENYSELECT,
        // geiger sounds
        res::valve::sound::player::GEIGER1,
        res::valve::sound::player::GEIGER2,
        res::valve::sound::player::GEIGER3,
        res::valve::sound::player::GEIGER4,
        res::valve::sound::player::GEIGER5,
        res::valve::sound::player::GEIGER6,
        // other
        res::valve::sound::plats::VEHICLE_IGNITION,
    ];

    for i in PRECACHE_SOUNDS {
        engine.precache_sound(*i);
    }

    engine.precache_model(res::valve::models::PLAYER);
}
