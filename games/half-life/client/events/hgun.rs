use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_WEAPON, PITCH},
    engine,
    raw::{event_args_s, SoundFlags},
};
use res::valve::sound;

use crate::view::view_mut;

use super::{is_local, Events};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Hgun {
    Idle1 = 0,
    FidgetSway,
    FidgetShake,
    Down,
    Up,
    Shoot,
}

impl Events {
    pub(super) fn fire_hornet_gun(&mut self, args: &mut event_args_s) {
        let engine = engine();
        let ev = engine.event_api();

        let idx = args.entindex;
        let origin = args.origin;
        let _fire_mode = args.iparam1;

        if is_local(idx) {
            ev.weapon_animation(Hgun::Shoot as c_int, 1);
            view_mut().punch_axis(PITCH, engine.random_int(0, 2) as f32);
        }

        let sample = match engine.random_int(0, 2) {
            0 => sound::agrunt::AG_FIRE1,
            1 => sound::agrunt::AG_FIRE2,
            _ => sound::agrunt::AG_FIRE3,
        };
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            1.0,
            ATTN_NORM,
            SoundFlags::NONE,
            100,
        );
    }
}
