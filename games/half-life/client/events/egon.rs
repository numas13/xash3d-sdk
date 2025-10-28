use core::ffi::{c_int, CStr};

use res::valve::sound;
use xash3d_client::{engine::event::EventArgs, prelude::*, sound::Channel};
use xash3d_hl_shared::weapons::egon::EgonAnimation;

const FIRE_WIDE: c_int = 1;
// const EGON_BEAM_SPRITE: &CStr = sprites::XBEAM1;
const EGON_SOUND_OFF: &CStr = sound::weapons::EGON_OFF1;
const EGON_SOUND_RUN: &CStr = sound::weapons::EGON_RUN3;
const EGON_SOUND_STARTUP: &CStr = sound::weapons::EGON_WINDUP2;

impl super::Events {
    pub(super) fn fire_egon(&mut self, args: &mut EventArgs) {
        let engine = self.engine;
        let ev = engine.event_api();

        let idx = args.entindex();
        let origin = args.origin();
        let _fire_state = args.iparam1();
        let fire_mode = args.iparam2();
        let startup = args.bparam1();

        if startup {
            if fire_mode == FIRE_WIDE {
                ev.build_sound_at(origin)
                    .entity(idx)
                    .channel_weapon()
                    .volume(0.98)
                    .pitch(125)
                    .play(EGON_SOUND_STARTUP);
            } else {
                ev.build_sound_at(origin)
                    .entity(idx)
                    .channel_weapon()
                    .volume(0.9)
                    .play(EGON_SOUND_STARTUP);
            }
        } else {
            // silence clippy
            if fire_mode == FIRE_WIDE {
                ev.build_sound_at(origin)
                    .entity(idx)
                    .channel_static()
                    .volume(0.98)
                    .pitch(125)
                    .play(EGON_SOUND_RUN);
            } else {
                ev.build_sound_at(origin)
                    .entity(idx)
                    .channel_static()
                    .volume(0.9)
                    .play(EGON_SOUND_RUN);
            }
        }

        if self.utils.is_local(idx) {
            let seq = match engine.random_int(0, 3) {
                0 => EgonAnimation::Fire1,
                1 => EgonAnimation::Fire2,
                2 => EgonAnimation::Fire3,
                _ => EgonAnimation::Fire4,
            };
            ev.weapon_animation(seq as c_int, 1);
        }
    }

    pub(super) fn stop_egon(&mut self, args: &mut EventArgs) {
        let engine = self.engine;
        let ev = engine.event_api();

        let idx = args.entindex();
        let origin = args.origin();

        ev.stop_sound(idx, Channel::Static, EGON_SOUND_RUN);

        if args.iparam1() != 0 {
            ev.build_sound_at(origin)
                .entity(idx)
                .channel_weapon()
                .volume(0.98)
                .play(EGON_SOUND_OFF);
        }
    }
}
