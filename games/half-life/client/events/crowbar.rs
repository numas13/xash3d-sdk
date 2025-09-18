use core::ffi::c_int;

use cl::{engine::event::event_args_s, prelude::*};
use res::valve::sound;

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Crowbar {
    Idle = 0,
    Draw,
    Holster,
    Attack1Hit,
    Attack1Miss,
    Attack2Miss,
    Attack2Hit,
    Attack3Miss,
    Attack3Hit,
}

impl super::Events {
    pub(super) fn crowbar(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let engine = engine();
        let ev = engine.event_api();
        ev.build_sound_at(args.origin())
            .entity(idx)
            .channel_weapon()
            .play(sound::weapons::CBAR_MISS1);

        if self.utils.is_local(idx) {
            self.swing = self.swing.wrapping_add(1);
            let seq = match self.swing % 3 {
                0 => Crowbar::Attack1Miss,
                1 => Crowbar::Attack2Miss,
                _ => Crowbar::Attack3Miss,
            };
            ev.weapon_animation(seq as c_int, 1);
        }
    }
}
