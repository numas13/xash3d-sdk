use core::ffi::c_int;

use cl::{engine, raw::event_args_s};
use math::consts::{PITCH, YAW};
use res::valve::{models, sound};
use shared::{
    consts::{ATTN_NORM, CHAN_WEAPON, TE_BOUNCE_SHELL},
    raw::SoundFlags,
};

use crate::view::view_mut;

use super::{
    eject_brass, fire_bullets, get_default_shell_info, get_gun_position, is_local, muzzle_flash,
    Bullet, Events,
};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Glock {
    Idle1 = 0,
    Idle2,
    Idle3,
    Shoot,
    ShootEmpty,
    Reload,
    ReloadNotEmpty,
    Draw,
    Holster,
    AddSilencer,
}

impl Events {
    pub(super) fn fire_glock1(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let angles = args.angles;
        let velocity = args.velocity;
        let av @ (forward, _, _) = math::angle_vectors(angles).all();
        let engine = engine();
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHELL);

        if is_local(idx) {
            muzzle_flash();
            let seq = if args.bparam1 != 0 {
                Glock::ShootEmpty
            } else {
                Glock::Shoot
            };
            ev.weapon_animation(seq as c_int, 2);
            view_mut().punch_axis(PITCH, -2.0);
        }

        let si = get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 4.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        let sample = sound::weapons::PL_GUN3;
        let vol = engine.random_float(0.92, 1.0);
        let pitch = 98 + engine.random_int(0, 3);

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
        let bullet = Bullet::Player9mm;
        let spread = (args.fparam1, args.fparam2);
        fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }

    pub(super) fn fire_glock2(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let angles = args.angles;
        let velocity = args.velocity;
        let av @ (forward, _, _) = math::angle_vectors(angles).all();
        let engine = engine();
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHELL);

        if is_local(idx) {
            muzzle_flash();
            ev.weapon_animation(Glock::Shoot as c_int, 2);
            view_mut().punch_axis(PITCH, -2.0);
        }

        let si = get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 4.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        let sample = sound::weapons::PL_GUN3;
        let vol = engine.random_float(0.92, 1.0);
        let pitch = 98 + engine.random_int(0, 3);

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
        let bullet = Bullet::Player9mm;
        let spread = (args.fparam1, args.fparam2);
        fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }
}
