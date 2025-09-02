use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_WEAPON, PITCH, TE_BOUNCE_SHOTSHELL, YAW},
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
enum Shotgun {
    Idle = 0,
    Fire,
    Fire2,
    Reload,
    Pump,
    StartReload,
    Draw,
    Holster,
    Idle4,
    IdleDeep,
}

impl Events {
    pub(super) fn fire_shotgun_single(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let angles = args.angles;
        let velocity = args.velocity;
        let av = angles.angle_vectors().all();
        let engine = engine();
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHOTGUNSHELL);

        if is_local(idx) {
            muzzle_flash();
            ev.weapon_animation(Shotgun::Fire as c_int, 2);
            view_mut().punch_axis(PITCH, -5.0);
        }

        let si = get_default_shell_info(args, origin, velocity, av, 32.0, -12.0, 6.0);
        let soundtype = TE_BOUNCE_SHOTSHELL as c_int;
        eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        let sample = sound::weapons::SBARREL1;
        let vol = engine.random_float(0.95, 1.0);
        let pitch = 93 + engine.random_int(0, 0x1f);

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
        let aiming = av.forward;
        let bullet = Bullet::PlayerBuckshot;

        if engine.is_multiplayer() {
            let spread = (0.08716, 0.04362);
            fire_bullets(idx, av, 4, src, aiming, 2048.0, bullet, None, spread);
        } else {
            let spread = (0.08716, 0.08716);
            fire_bullets(idx, av, 6, src, aiming, 2048.0, bullet, None, spread);
        }
    }

    pub(super) fn fire_shotgun_double(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let angles = args.angles;
        let velocity = args.velocity;
        let av = angles.angle_vectors().all();
        let engine = engine();
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHOTGUNSHELL);

        if is_local(idx) {
            muzzle_flash();
            ev.weapon_animation(Shotgun::Fire2 as c_int, 2);
            view_mut().punch_axis(PITCH, -10.0);
        }

        for _ in 0..2 {
            let si = get_default_shell_info(args, origin, velocity, av, 32.0, -12.0, 6.0);
            let soundtype = TE_BOUNCE_SHOTSHELL as c_int;
            eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);
        }

        let sample = sound::weapons::DBARREL1;
        let vol = engine.random_float(0.99, 1.0);
        let pitch = 85 + engine.random_int(0, 0x1f);
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
        let aiming = av.forward;
        let bullet = Bullet::PlayerBuckshot;

        if engine.is_multiplayer() {
            let spread = (0.17365, 0.04362);
            fire_bullets(idx, av, 8, src, aiming, 2048.0, bullet, None, spread);
        } else {
            let spread = (0.08716, 0.08716);
            fire_bullets(idx, av, 12, src, aiming, 2048.0, bullet, None, spread);
        }
    }
}
