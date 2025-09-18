use core::ffi::c_int;

use cl::{
    consts::{PITCH, TE_BOUNCE_SHELL, YAW},
    engine::event::event_args_s,
    prelude::*,
};
use res::valve::{models, sound};

use crate::export::view_mut;

use super::Bullet;

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

impl super::Events {
    pub(super) fn fire_glock1(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin();
        let angles = args.angles();
        let velocity = args.velocity();
        let av = angles.angle_vectors().all();
        let engine = self.engine;
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHELL);

        if self.utils.is_local(idx) {
            self.utils.muzzle_flash();
            let seq = if args.bparam1 != 0 {
                Glock::ShootEmpty
            } else {
                Glock::Shoot
            };
            ev.weapon_animation(seq as c_int, 2);
            view_mut().punch_axis(PITCH, -2.0);
        }

        let si = self
            .utils
            .get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 4.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        self.utils
            .eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .volume(engine.random_float(0.92, 1.0))
            .pitch(98 + engine.random_int(0, 3))
            .play(sound::weapons::PL_GUN3);

        let src = self.utils.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::Player9mm;
        let spread = (args.fparam1, args.fparam2);
        self.utils
            .fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }

    pub(super) fn fire_glock2(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin();
        let angles = args.angles();
        let velocity = args.velocity();
        let av = angles.angle_vectors().all();
        let engine = self.engine;
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHELL);

        if self.utils.is_local(idx) {
            self.utils.muzzle_flash();
            ev.weapon_animation(Glock::Shoot as c_int, 2);
            view_mut().punch_axis(PITCH, -2.0);
        }

        let si = self
            .utils
            .get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 4.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        self.utils
            .eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .volume(engine.random_float(0.92, 1.0))
            .pitch(98 + engine.random_int(0, 3))
            .play(sound::weapons::PL_GUN3);

        let src = self.utils.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::Player9mm;
        let spread = (args.fparam1, args.fparam2);
        self.utils
            .fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }
}
