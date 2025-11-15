use core::ffi::c_int;

use res::valve::{models, sound};
use xash3d_client::{
    consts::{PITCH, TE_BOUNCE_SHELL, YAW},
    engine::event::EventArgs,
    prelude::*,
};
use xash3d_hl_shared::weapons::mp5::Mp5Animation;

use crate::export::view;

use super::Bullet;

impl super::Events {
    pub(super) fn fire_mp5(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let origin = args.origin();
        let angles = args.angles();
        let velocity = args.velocity();
        let av = angles.angle_vectors().all();
        let engine = self.engine;
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHELL);

        if self.is_local(idx) {
            self.muzzle_flash();
            let rand = engine.random_int(0, 2);
            ev.weapon_animation(Mp5Animation::Fire1 as c_int + rand, 2);
            let pitch = engine.random_float(-2.0, 2.0);
            view().punch_axis(PITCH, pitch);
        }

        let si = self.get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 6.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        self.eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        let sample = match engine.random_int(0, 1) {
            0 => sound::weapons::HKS1,
            _ => sound::weapons::HKS2,
        };
        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .pitch(94 + engine.random_int(0, 0xf))
            .play(sample);

        let src = self.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::PlayerMp5;
        let mut tracer_count = self.tracer_count.borrow_mut();
        let tracer = Some((2, &mut tracer_count[idx.to_u16() as usize - 1]));
        let spread = (args.fparam1(), args.fparam2());
        self.fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, tracer, spread);
    }

    pub(super) fn fire_mp5_2(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let origin = args.origin();
        let engine = self.engine;
        let ev = engine.event_api();

        if self.is_local(idx) {
            ev.weapon_animation(Mp5Animation::Launch as c_int, 2);
            view().punch_axis(PITCH, -10.0);
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
