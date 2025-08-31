use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_STATIC},
    prelude::*,
    raw::{event_args_s, SoundFlags},
};
use res::valve::sound;

use super::Events;

impl Events {
    pub(super) fn train_pitch_adjust(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin;
        let us_params = args.iparam1 as u16;
        let stop = args.bparam1 != 0;
        let volume = (us_params & 0x3f) as f32 / 40.0;
        let noise = ((us_params >> 12) & 0x7) as c_int;
        let pitch = (10.0 * ((us_params >> 6) & 0x3f) as f32) as c_int;

        let sample = match noise {
            1 => sound::plats::TTRAIN1,
            2 => sound::plats::TTRAIN2,
            3 => sound::plats::TTRAIN3,
            4 => sound::plats::TTRAIN4,
            5 => sound::plats::TTRAIN6,
            6 => sound::plats::TTRAIN7,
            _ => c"",
        };

        let engine = engine();
        let ev = engine.event_api();

        if stop {
            ev.stop_sound(idx, CHAN_STATIC, sample);
        } else {
            ev.play_sound(
                idx,
                origin,
                CHAN_STATIC,
                sample,
                volume,
                ATTN_NORM,
                SoundFlags::CHANGE_PITCH,
                pitch,
            );
        }
    }
}
