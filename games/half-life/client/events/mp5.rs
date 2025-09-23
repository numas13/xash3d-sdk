use core::ffi::c_int;

use res::valve::{models, sound};
use xash3d_client::{
    consts::{PITCH, TE_BOUNCE_SHELL, YAW},
    engine::event::event_args_s,
    prelude::*,
};

use crate::export::view_mut;

use super::Bullet;

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

impl super::Events {
    pub(super) fn fire_mp5(&mut self, args: &mut event_args_s) {
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
            let rand = engine.random_int(0, 2);
            ev.weapon_animation(Mp5::Fire1 as c_int + rand, 2);
            let pitch = engine.random_float(-2.0, 2.0);
            view_mut().punch_axis(PITCH, pitch);
        }

        let si = self
            .utils
            .get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 6.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        self.utils
            .eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        let sample = match engine.random_int(0, 1) {
            0 => sound::weapons::HKS1,
            _ => sound::weapons::HKS2,
        };
        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .pitch(94 + engine.random_int(0, 0xf))
            .play(sample);

        let src = self.utils.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::PlayerMp5;
        let tracer = Some((2, &mut self.tracer_count[idx as usize - 1]));
        let spread = (args.fparam1, args.fparam2);
        self.utils
            .fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, tracer, spread);
    }

    pub(super) fn fire_mp5_2(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin();
        let engine = self.engine;
        let ev = engine.event_api();

        if self.utils.is_local(idx) {
            ev.weapon_animation(Mp5::Launch as c_int, 2);
            view_mut().punch_axis(PITCH, -10.0);
        }

        let sample = match engine.random_int(0, 1) {
            0 => sound::weapons::GLAUNCHER,
            _ => sound::weapons::GLAUNCHER2,
        };
        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .pitch(94 + engine.random_int(0, 0xf))
            .play(sample);
    }
}
