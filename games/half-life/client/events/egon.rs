use core::ffi::{c_int, CStr};

use cl::{
    consts::{ATTN_NORM, CHAN_STATIC, CHAN_WEAPON},
    engine::event::EventArgs,
    prelude::*,
    raw::SoundFlags,
};
use res::valve::sound;

use super::{is_local, Events};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Egon {
    Idle1 = 0,
    FIDGET1,
    AltFireOn,
    AltFireCycle,
    AltFireOff,
    Fire1,
    Fire2,
    Fire3,
    Fire4,
    Draw,
    Holster,
}

const FIRE_WIDE: c_int = 1;
// const EGON_BEAM_SPRITE: &CStr = sprites::XBEAM1;
const EGON_SOUND_OFF: &CStr = sound::weapons::EGON_OFF1;
const EGON_SOUND_RUN: &CStr = sound::weapons::EGON_RUN3;
const EGON_SOUND_STARTUP: &CStr = sound::weapons::EGON_WINDUP2;

impl Events {
    pub(super) fn fire_egon(&mut self, args: &mut EventArgs) {
        let engine = engine();
        let ev = engine.event_api();

        let idx = args.entindex;
        let origin = args.origin;
        let _fire_state = args.iparam1;
        let fire_mode = args.iparam2;
        let startup = args.bparam1 != 0;

        if startup {
            if fire_mode == FIRE_WIDE {
                ev.play_sound(
                    idx,
                    origin,
                    CHAN_WEAPON,
                    EGON_SOUND_STARTUP,
                    0.98,
                    ATTN_NORM,
                    SoundFlags::NONE,
                    125,
                );
            } else {
                ev.play_sound(
                    idx,
                    origin,
                    CHAN_WEAPON,
                    EGON_SOUND_STARTUP,
                    0.9,
                    ATTN_NORM,
                    SoundFlags::NONE,
                    100,
                );
            }
        } else {
            //
            if fire_mode == FIRE_WIDE {
                ev.play_sound(
                    idx,
                    origin,
                    CHAN_STATIC,
                    EGON_SOUND_RUN,
                    0.98,
                    ATTN_NORM,
                    SoundFlags::NONE,
                    125,
                );
            } else {
                ev.play_sound(
                    idx,
                    origin,
                    CHAN_STATIC,
                    EGON_SOUND_RUN,
                    0.9,
                    ATTN_NORM,
                    SoundFlags::NONE,
                    100,
                );
            }
        }

        if is_local(idx) {
            let seq = match engine.random_int(0, 3) {
                0 => Egon::Fire1,
                1 => Egon::Fire2,
                2 => Egon::Fire3,
                _ => Egon::Fire4,
            };
            ev.weapon_animation(seq as c_int, 1);
        }
    }

    pub(super) fn stop_egon(&mut self, args: &mut EventArgs) {
        let engine = engine();
        let ev = engine.event_api();

        let idx = args.entindex;
        let origin = args.origin;

        ev.stop_sound(idx, CHAN_STATIC, EGON_SOUND_RUN);

        if args.iparam1 != 0 {
            ev.play_sound(
                idx,
                origin,
                CHAN_WEAPON,
                EGON_SOUND_OFF,
                0.98,
                ATTN_NORM,
                SoundFlags::NONE,
                100,
            );
        }
    }
}
