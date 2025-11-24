use core::ffi::c_uint;

use xash3d_client::{
    cvar::Cvar,
    ffi::common::{local_state_s, usercmd_s},
    prelude::*,
};

use crate::export::hud;

pub struct Weapons {
    cl_lw: Option<Cvar<bool>>,
}

impl Weapons {
    pub fn new(engine: ClientEngineRef) -> Self {
        let cl_lw = engine.find_cvar(c"cl_lw");
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
        if cfg!(feature = "client-weapons") && self.cl_lw.as_ref().is_some_and(|i| i.get()) {
            self.weapons_post_think(from, to, cmd, time, random_seed);
        } else {
            to.client.fov = hud().last_fov() as f32;
        }

        // TODO: gauss predication

        hud().set_last_fov(to.client.fov as u8);
    }
}
