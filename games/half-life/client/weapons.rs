use core::ffi::c_uint;

use cl::{
    cvar::CVarPtr,
    prelude::*,
    raw::{local_state_s, usercmd_s},
};

use crate::export::{hud, hud_mut};

pub struct Weapons {
    cl_lw: CVarPtr,
}

impl Default for Weapons {
    fn default() -> Self {
        Self::new()
    }
}

impl Weapons {
    pub fn new() -> Self {
        let engine = engine();
        let cl_lw = engine.get_cvar(c"cl_lw");

        Self { cl_lw }
    }

    fn weapons_post_think(
        &mut self,
        _from: &mut local_state_s,
        _to: &mut local_state_s,
        _cmd: &mut usercmd_s,
        _time: f64,
        _random_seed: c_uint,
    ) {
        // TODO:
    }

    pub fn post_run_cmd(
        &mut self,
        from: &mut local_state_s,
        to: &mut local_state_s,
        cmd: &mut usercmd_s,
        _runfuncs: bool,
        time: f64,
        random_seed: c_uint,
    ) {
        if cfg!(feature = "client-weapons") && !self.cl_lw.is_null() && self.cl_lw.value() != 0.0 {
            self.weapons_post_think(from, to, cmd, time, random_seed);
        } else {
            to.client.fov = hud().get_last_fov() as f32;
        }

        // TODO: gauss predication

        hud_mut().set_last_fov(to.client.fov as u8);
    }
}
