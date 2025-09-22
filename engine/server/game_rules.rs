use core::{any::Any, ffi::CStr, ptr};

use alloc::rc::Rc;
use shared::{
    cell::Sync,
    ffi::{common::vec3_t, server::edict_s},
};

use crate::{
    engine::ServerEngineRef,
    entity::{Entity, EntityPlayer},
};

pub trait GameRules: Any {
    fn engine(&self) -> ServerEngineRef;

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

    fn get_game_description(&self) -> &'static CStr;

    #[allow(unused_variables)]
    fn is_allowed_to_spawn(&self, entity: &dyn Entity) -> bool {
        true
    }

    fn get_player_spawn_spot(&self, player: &mut dyn EntityPlayer) -> *mut edict_s {
        let spawn_spot = player.select_spawn_point();
        let sev = unsafe { &(*spawn_spot).v };
        let pev = player.vars_mut().as_raw_mut();
        pev.origin = sev.origin + vec3_t::new(0.0, 0.0, 1.0);
        pev.v_angle = vec3_t::ZERO;
        pev.velocity = vec3_t::ZERO;
        pev.angles = sev.angles;
        pev.punchangle = vec3_t::ZERO;
        pev.fixangle = 1;
        spawn_spot
    }

    #[allow(unused_variables)]
    fn player_spawn(&self, player: &mut dyn EntityPlayer) {}
}

// TODO: do not use Rc for GAME_RULES?
static mut GAME_RULES: Sync<Option<Rc<dyn GameRules>>> = unsafe { Sync::new(None) };

#[derive(Debug)]
pub struct GameRulesRef {}

impl GameRulesRef {
    /// Creates a new `GameRulesRef`.
    ///
    /// # Safety
    ///
    /// Must be called only from the main engine thread.
    pub unsafe fn new() -> Self {
        Self {}
    }

    /// Sets the global game rules object.
    ///
    /// # Safety
    ///
    /// * Must be called only from the main engine thread.
    pub unsafe fn set(game_rules: Rc<dyn GameRules>) {
        unsafe {
            *GAME_RULES = Some(game_rules);
        }
    }

    pub fn get(&self) -> Option<Rc<dyn GameRules>> {
        let game_rules = unsafe { &*ptr::addr_of_mut!(GAME_RULES) };
        (*game_rules).clone()
    }
}
