use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_ITEM, CHAN_WEAPON, PITCH, PITCH_NORM},
    engine,
    raw::{event_args_s, SoundFlags},
};
use res::valve::sound;

use crate::view::view_mut;

use super::{is_local, Events};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Rpg {
    Idle = 0,
    Fidget,
    Reload,
    Fire2,
    Holster1,
    Draw1,
    Holster2,
    DrawUl,
    IdleUl,
    FidgetUl,
}

impl Events {
    pub(super) fn fire_rpg(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;

        let engine = engine();
        let ev = engine.event_api();

        let sample = sound::weapons::ROCKETFIRE1;
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            0.9,
            ATTN_NORM,
            SoundFlags::NONE,
            PITCH_NORM,
        );

        let sample = sound::weapons::GLAUNCHER;
        ev.play_sound(
            idx,
            origin,
            CHAN_ITEM,
            sample,
            0.7,
            ATTN_NORM,
            SoundFlags::NONE,
            PITCH_NORM,
        );

        if is_local(idx) {
            ev.weapon_animation(Rpg::Fire2 as c_int, 1);

            view_mut().punch_axis(PITCH, -5.0);
        }
    }
}
