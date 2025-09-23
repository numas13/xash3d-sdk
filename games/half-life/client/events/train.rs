use core::ffi::c_int;

use res::valve::sound;
use xash3d_client::{engine::event::event_args_s, prelude::*, sound::Channel};

impl super::Events {
    pub(super) fn train_pitch_adjust(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin();
        let us_params = args.iparam1 as u16;
        let stop = args.bparam1 != 0;
        let noise = ((us_params >> 12) & 0x7) as c_int;

        let sample = match noise {
            1 => sound::plats::TTRAIN1,
            2 => sound::plats::TTRAIN2,
            3 => sound::plats::TTRAIN3,
            4 => sound::plats::TTRAIN4,
            5 => sound::plats::TTRAIN6,
            6 => sound::plats::TTRAIN7,
            _ => c"",
        };

        let engine = self.engine;
        let ev = engine.event_api();

        if stop {
            ev.stop_sound(idx, Channel::Static, sample);
        } else {
            ev.build_sound_at(origin)
                .entity(idx)
                .channel_static()
                .volume((us_params & 0x3f) as f32 / 40.0)
                .change_pitch()
                .pitch((10.0 * ((us_params >> 6) & 0x3f) as f32) as c_int)
                .play(sample);
        }
    }
}
