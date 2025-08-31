use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_WEAPON, PITCH, TE_BOUNCE_SHELL, YAW},
    math::angle_vectors,
    prelude::*,
    raw::{event_args_s, SoundFlags},
};
use res::valve::{models, sound};

use crate::view::view_mut;

use super::{
    eject_brass, fire_bullets, get_default_shell_info, get_gun_position, is_local, muzzle_flash,
    Bullet, Events,
};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Mp5 {
    Longidle = 0,
    Idle1,
    Launch,
    Reload,
    Deploy,
    Fire1,
    Fire2,
    Fire3,
}

impl Events {
    pub(super) fn fire_mp5(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let angles = args.angles;
        let velocity = args.velocity;
        let av @ (forward, _, _) = angle_vectors(angles).all();
        let engine = engine();
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHELL);

        if is_local(idx) {
            muzzle_flash();
            let rand = engine.random_int(0, 2);
            ev.weapon_animation(Mp5::Fire1 as c_int + rand, 2);
            let pitch = engine.random_float(-2.0, 2.0);
            view_mut().punch_axis(PITCH, pitch);
        }

        let si = get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 6.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        let sample = match engine.random_int(0, 1) {
            0 => sound::weapons::HKS1,
            _ => sound::weapons::HKS2,
        };
        let vol = 1.0;
        let pitch = 94 + engine.random_int(0, 0xf);
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            vol,
            ATTN_NORM,
            SoundFlags::NONE,
            pitch,
        );

        let src = get_gun_position(args, origin);
        let aiming = forward;
        let bullet = Bullet::PlayerMp5;
        let tracer = Some((2, &mut self.tracer_count[idx as usize - 1]));
        let spread = (args.fparam1, args.fparam2);
        fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, tracer, spread);
    }

    pub(super) fn fire_mp5_2(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let engine = engine();
        let ev = engine.event_api();

        if is_local(idx) {
            ev.weapon_animation(Mp5::Launch as c_int, 2);
            view_mut().punch_axis(PITCH, -10.0);
        }

        let sample = match engine.random_int(0, 1) {
            0 => sound::weapons::GLAUNCHER,
            _ => sound::weapons::GLAUNCHER2,
        };
        let pitch = 94 + engine.random_int(0, 0xf);
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            1.0,
            ATTN_NORM,
            SoundFlags::NONE,
            pitch,
        );
    }
}
