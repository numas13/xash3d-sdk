use core::ffi::c_int;

use res::valve::{models, sound};
use xash3d_client::{
    consts::{PITCH, TE_BOUNCE_SHOTSHELL, YAW},
    engine::event::EventArgs,
    prelude::*,
};
use xash3d_hl_shared::weapons::shotgun::ShotgunAnimation;

use crate::export::view_mut;

use super::Bullet;

impl super::Events {
    pub(super) fn fire_shotgun_single(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let origin = args.origin();
        let angles = args.angles();
        let velocity = args.velocity();
        let av = angles.angle_vectors().all();
        let engine = self.engine;
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHOTGUNSHELL);

        if self.is_local(idx) {
            self.muzzle_flash();
            ev.weapon_animation(ShotgunAnimation::Fire as c_int, 2);
            view_mut().punch_axis(PITCH, -5.0);
        }

        let si = self.get_default_shell_info(args, origin, velocity, av, 32.0, -12.0, 6.0);
        let soundtype = TE_BOUNCE_SHOTSHELL as c_int;
        self.eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .volume(engine.random_float(0.95, 1.0))
            .pitch(93 + engine.random_int(0, 0x1f))
            .play(sound::weapons::SBARREL1);

        let src = self.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::PlayerBuckshot;

        if engine.is_multiplayer() {
            let spread = (0.08716, 0.04362);
            self.fire_bullets(idx, av, 4, src, aiming, 2048.0, bullet, None, spread);
        } else {
            let spread = (0.08716, 0.08716);
            self.fire_bullets(idx, av, 6, src, aiming, 2048.0, bullet, None, spread);
        }
    }

    pub(super) fn fire_shotgun_double(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let origin = args.origin();
        let angles = args.angles();
        let velocity = args.velocity();
        let av = angles.angle_vectors().all();
        let engine = self.engine;
        let ev = engine.event_api();
        let shell = ev.find_model_index(models::SHOTGUNSHELL);

        if self.is_local(idx) {
            self.muzzle_flash();
            ev.weapon_animation(ShotgunAnimation::Fire2 as c_int, 2);
            view_mut().punch_axis(PITCH, -10.0);
        }

        for _ in 0..2 {
            let si = self.get_default_shell_info(args, origin, velocity, av, 32.0, -12.0, 6.0);
            let soundtype = TE_BOUNCE_SHOTSHELL as c_int;
            self.eject_brass(si.origin, si.velocity, angles[YAW], shell, soundtype);
        }

        ev.build_sound_at(origin)
            .entity(idx)
            .volume(engine.random_float(0.99, 1.0))
            .pitch(85 + engine.random_int(0, 0x1f))
            .channel_weapon()
            .play(sound::weapons::DBARREL1);

        let src = self.get_gun_position(args, origin);
        let aiming = av.forward;
        let bullet = Bullet::PlayerBuckshot;

        if engine.is_multiplayer() {
            let spread = (0.17365, 0.04362);
            self.fire_bullets(idx, av, 8, src, aiming, 2048.0, bullet, None, spread);
        } else {
            let spread = (0.08716, 0.08716);
            self.fire_bullets(idx, av, 12, src, aiming, 2048.0, bullet, None, spread);
        }
    }
}
