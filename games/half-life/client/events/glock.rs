use core::ffi::c_int;

use res::valve::{models, sound};
use xash3d_client::{
    consts::{PITCH, TE_BOUNCE_SHELL, YAW},
    engine::event::EventArgs,
    prelude::*,
};
use xash3d_hl_shared::weapons::glock::GlockAnimation;

use crate::export::view_mut;

use super::Bullet;

impl super::Events {
    pub(super) fn fire_glock1(&self, args: &mut EventArgs) {
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
            let seq = if args.bparam1() {
                GlockAnimation::ShootEmpty
            } else {
                GlockAnimation::Shoot
            };
            ev.weapon_animation(seq as c_int, 2);
            view_mut().punch_axis(PITCH, -2.0);
        }

        let si = self.get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 4.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        self.eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .volume(engine.random_float(0.92, 1.0))
            .pitch(98 + engine.random_int(0, 3))
            .play(sound::weapons::PL_GUN3);

        let src = self.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::Player9mm;
        let spread = (args.fparam1(), args.fparam2());
        self.fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }

    pub(super) fn fire_glock2(&self, args: &mut EventArgs) {
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
            ev.weapon_animation(GlockAnimation::Shoot as c_int, 2);
            view_mut().punch_axis(PITCH, -2.0);
        }

        let si = self.get_default_shell_info(args, origin, velocity, av, 20.0, -12.0, 4.0);
        let soundtype = TE_BOUNCE_SHELL as c_int;
        self.eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .volume(engine.random_float(0.92, 1.0))
            .pitch(98 + engine.random_int(0, 3))
            .play(sound::weapons::PL_GUN3);

        let src = self.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::Player9mm;
        let spread = (args.fparam1(), args.fparam2());
        self.fire_bullets(idx, av, 1, src, aiming, 8192.0, bullet, None, spread);
    }
}
