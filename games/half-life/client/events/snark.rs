use core::ffi::c_int;

use cl::{consts::PM_NORMAL, prelude::*, raw::event_args_s};
use pm::{VEC_DUCK_HULL_MIN, VEC_HULL_MIN};

use super::{is_local, Events};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Squeak {
    Idle1 = 0,
    FidgetFit,
    FidgetNip,
    Down,
    Up,
    Throw,
}

impl Events {
    pub(super) fn fire_snark(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        if !is_local(idx) {
            return;
        }

        let origin = args.origin;
        let angles = args.angles;
        let mut src = origin;
        if args.ducking != 0 {
            src -= VEC_HULL_MIN - VEC_DUCK_HULL_MIN;
        }

        let engine = engine();
        let ev = engine.event_api();
        let pm_states = ev.push_pm_states();
        ev.set_solid_players(idx - 1);
        ev.set_trace_hull(2);
        let forward = cl::math::angle_vectors(angles).forward();
        let end = src + forward * 64.0;
        let src = src + forward * 20.0;
        let tr = ev.player_trace(src, end, PM_NORMAL, -1);
        if !tr.allsolid.to_bool() && !tr.startsolid.to_bool() && tr.fraction > 0.25 {
            ev.weapon_animation(Squeak::Throw as c_int, 0);
        }
        pm_states.pop();
    }
}
