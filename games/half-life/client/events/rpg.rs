use core::ffi::c_int;

use res::valve::sound;
use xash3d_client::{consts::PITCH, engine::event::EventArgs};
use xash3d_hl_shared::weapons::rpg::RpgAnimation;

use crate::export::view_mut;

impl super::Events {
    pub(super) fn fire_rpg(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let origin = args.origin();

        let engine = self.engine;
        let ev = engine.event_api();

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .volume(0.9)
            .play(sound::weapons::ROCKETFIRE1);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_item()
            .volume(0.7)
            .play(sound::weapons::GLAUNCHER);

        if self.is_local(idx) {
            ev.weapon_animation(RpgAnimation::Fire2 as c_int, 1);

            view_mut().punch_axis(PITCH, -5.0);
        }
    }
}
