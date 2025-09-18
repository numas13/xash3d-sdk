use core::ffi::c_int;

use cl::{consts::PM_NORMAL, engine::event::event_args_s, prelude::*};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Tripmine {
    Idle1 = 0,
    Idle2,
    Arm1,
    Arm2,
    Fidget,
    Holster,
    Draw,
    World,
    Ground,
}

impl super::Events {
    pub(super) fn fire_tripmine(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        if !self.utils.is_local(idx) {
            return;
        }

        let engine = engine();
        let ev = engine.event_api();

        let origin = args.origin();
        let angles = args.angles();
        let forward = angles.angle_vectors().forward();

        let view_ofs = ev.local_player_view_height();
        let src = origin + view_ofs;
        let end = src + forward * 128.0;

        let pm_states = ev.push_pm_states();
        ev.set_solid_players(idx - 1);
        ev.set_trace_hull(2);
        let tr = ev.player_trace(src, end, PM_NORMAL, -1);

        if tr.fraction < 1.0 {
            ev.weapon_animation(Tripmine::Draw as c_int, 0);
        }

        pm_states.pop();
    }
}
