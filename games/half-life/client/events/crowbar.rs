use core::ffi::c_int;

use res::valve::sound;
use xash3d_client::engine::event::EventArgs;
use xash3d_hl_shared::weapons::crowbar::CrowbarAnimation;

impl super::Events {
    pub(super) fn crowbar(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let engine = self.engine;
        let ev = engine.event_api();
        ev.build_sound_at(args.origin())
            .entity(idx)
            .channel_weapon()
            .play(sound::weapons::CBAR_MISS1);

        if self.is_local(idx) {
            let swing = self.swing.get().wrapping_add(1);
            self.swing.set(swing);
            let seq = match swing % 3 {
                0 => CrowbarAnimation::Attack1Miss,
                1 => CrowbarAnimation::Attack2Miss,
                _ => CrowbarAnimation::Attack3Miss,
            };
            ev.weapon_animation(seq as c_int, 1);
        }
    }
}
