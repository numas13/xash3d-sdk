use core::ffi::c_int;

use cl::{consts::PITCH, engine::event::event_args_s, prelude::*};
use res::valve::sound;

use crate::export::view_mut;

use super::Bullet;

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Python {
    Idle1 = 0,
    Fidget,
    Fire1,
    Reload,
    Holster,
    Draw,
    Idle2,
    Idle3,
}

impl super::Events {
    pub(super) fn fire_python(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin();
        let angles = args.angles();
        let av = angles.angle_vectors().all();
        let engine = self.engine;
        let ev = engine.event_api();

        if self.utils.is_local(idx) {
            let body = if engine.is_singleplayer() { 0 } else { 1 };
            ev.weapon_animation(Python::Fire1 as c_int, body);
            view_mut().punch_axis(PITCH, -10.0);
        }

        let sample = match engine.random_int(0, 1) {
            0 => sound::weapons::_357_SHOT1,
            _ => sound::weapons::_357_SHOT2,
        };

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .volume(engine.random_float(0.8, 0.9))
            .play(sample);

        let src = self.utils.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::Player357;
        let spread = (args.fparam1, args.fparam2);

        self.utils
            .fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }
}
