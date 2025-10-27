use core::ffi::c_int;

use xash3d_client::{consts::PM_NORMAL, engine::event::EventArgs, prelude::*};
use xash3d_player_move::{DUCK_HULL_MIN, HULL_MIN};

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

impl super::Events {
    pub(super) fn fire_snark(&mut self, args: &mut EventArgs) {
        let idx = args.entindex();
        if !self.utils.is_local(idx) {
            return;
        }

        let origin = args.origin();
        let angles = args.angles();
        let mut src = origin;
        if args.ducking() {
            src -= HULL_MIN.z - DUCK_HULL_MIN.z;
        }

        let engine = self.engine;
        let ev = engine.event_api();
        let pm_states = ev.push_pm_states();
        ev.set_solid_players(idx.to_i32() - 1);
        ev.set_trace_hull(2);
        let forward = angles.angle_vectors().forward();
        let end = src + forward * 64.0;
        let src = src + forward * 20.0;
        let tr = ev.player_trace(src, end, PM_NORMAL, -1);
        if tr.allsolid == 0 && tr.startsolid == 0 && tr.fraction > 0.25 {
            ev.weapon_animation(Squeak::Throw as c_int, 0);
        }
        pm_states.pop();
    }
}
