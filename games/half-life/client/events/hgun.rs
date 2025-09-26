use core::ffi::c_int;

use res::valve::sound;
use xash3d_client::{consts::PITCH, engine::event::EventArgs, prelude::*};

use crate::export::view_mut;

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

impl super::Events {
    pub(super) fn fire_hornet_gun(&mut self, args: &mut EventArgs) {
        let engine = self.engine;
        let ev = engine.event_api();

        let idx = args.entindex();
        let origin = args.origin();
        let _fire_mode = args.iparam1();

        if self.utils.is_local(idx) {
            ev.weapon_animation(Hgun::Shoot as c_int, 1);
            view_mut().punch_axis(PITCH, engine.random_int(0, 2) as f32);
        }

        let sample = match engine.random_int(0, 2) {
            0 => sound::agrunt::AG_FIRE1,
            1 => sound::agrunt::AG_FIRE2,
            _ => sound::agrunt::AG_FIRE3,
        };
        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .play(sample);
    }
}
