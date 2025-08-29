use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_WEAPON, PITCH, PITCH_NORM},
    engine,
    raw::{event_args_s, SoundFlags},
};
use res::valve::sound;

use crate::view::view_mut;

use super::{fire_bullets, get_gun_position, is_local, Bullet, Events};

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

impl Events {
    pub(super) fn fire_python(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let angles = args.angles;
        let av @ (forward, _, _) = cl::math::angle_vectors(angles).all();
        let engine = engine();
        let ev = engine.event_api();

        if is_local(idx) {
            let body = if engine.is_singleplayer() { 0 } else { 1 };
            ev.weapon_animation(Python::Fire1 as c_int, body);
            view_mut().punch_axis(PITCH, -10.0);
        }

        let sample = match engine.random_int(0, 1) {
            0 => sound::weapons::_357_SHOT1,
            _ => sound::weapons::_357_SHOT2,
        };
        let vol = engine.random_float(0.8, 0.9);

        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            vol,
            ATTN_NORM,
            SoundFlags::NONE,
            PITCH_NORM,
        );

        let src = get_gun_position(args, origin);
        let aiming = forward;
        let bullet = Bullet::Player357;
        let spread = (args.fparam1, args.fparam2);

        fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }
}
