use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_WEAPON, PITCH_NORM},
    engine,
    raw::{event_args_s, SoundFlags},
};
use res::valve::sound;

use super::{is_local, Events};

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

impl Events {
    pub(super) fn crowbar(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let engine = engine();
        let ev = engine.event_api();

        let sample = sound::weapons::CBAR_MISS1;
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            1.0,
            ATTN_NORM,
            SoundFlags::NONE,
            PITCH_NORM,
        );

        if is_local(idx) {
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
