use core::{ffi::CStr, ptr};

use alloc::boxed::Box;
use sv::{cell::Sync, engine, globals, math::vec3_t, raw::edict_s};

use crate::{entity::Entity, player::Player};

#[allow(dead_code)]
pub trait GameRules {
    fn is_multiplayer(&self) -> bool;
    fn is_deathmatch(&self) -> bool;
    fn is_teamplay(&self) -> bool;
    fn is_coop(&self) -> bool;

    fn get_game_description(&self) -> &'static CStr {
        c"Half-Life"
    }

    fn is_allowed_to_spawn(&self, _entity: &dyn Entity) -> bool {
        true
    }

    fn get_player_spawn_spot(&self, player: &mut Player) -> *mut edict_s {
        let spawn_spot = player.ent_select_spawn_point();
        let sev = unsafe { &(*spawn_spot).v };
        let pev = player.vars_mut();
        pev.origin = sev.origin + vec3_t::new(0.0, 0.0, 1.0);
        pev.v_angle = vec3_t::ZERO;
        pev.velocity = vec3_t::ZERO;
        pev.angles = sev.angles;
        pev.punchangle = vec3_t::ZERO;
        pev.fixangle = 1;
        spawn_spot
    }

    fn player_spawn(&self, _player: &Player) {}
}

pub struct HalfLifeRules {}

impl HalfLifeRules {
    pub fn new() -> Self {
        engine().server_command("exec spserver.cfg\n");
        // TODO: refresh skill data

        Self {}
    }
}

impl GameRules for HalfLifeRules {
    fn is_multiplayer(&self) -> bool {
        false
    }
    fn is_deathmatch(&self) -> bool {
        false
    }
    fn is_teamplay(&self) -> bool {
        false
    }
    fn is_coop(&self) -> bool {
        false
    }
}

static mut GAME_RULES: Sync<Option<Box<dyn GameRules>>> = unsafe { Sync::new(None) };

unsafe fn game_rules_set(game_rules: Box<dyn GameRules>) {
    unsafe {
        *GAME_RULES = Some(game_rules);
    }
}

pub fn game_rules() -> Option<&'static dyn GameRules> {
    let game_rules = unsafe { &*ptr::addr_of_mut!(GAME_RULES) };
    game_rules.as_deref()
}

pub fn install_game_rules() {
    let engine = engine();

    engine.server_command(c"exec game.cfg\n");
    engine.server_execute();

    if globals().deathmatch == 0.0 {
        // TODO: g_teamplay = 0;
        unsafe {
            game_rules_set(Box::new(HalfLifeRules::new()));
        }
        return;
    } else {
        // TODO:
    }
    todo!();
}
