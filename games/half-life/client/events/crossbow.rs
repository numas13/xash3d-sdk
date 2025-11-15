use core::ffi::c_int;

use res::valve::{models, sound};
use xash3d_client::{
    consts::{CONTENTS_WATER, PITCH, PM_NORMAL, SOLID_BSP, TE_BOUNCE_NULL},
    engine::event::EventArgs,
    entity::{EntityIndex, TempEntityFlags},
    ffi::{api::efx::TEMPENTITY, common::vec3_t},
    prelude::*,
    render::RenderMode,
};
use xash3d_hl_shared::weapons::crossbow::CrossbowAnimation;

use crate::export::view_mut;

impl super::Events {
    pub(super) fn fire_crossbow(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let origin = args.origin();

        let engine = self.engine;
        let ev = engine.event_api();

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .pitch(93 + engine.random_int(0, 0xf))
            .play(sound::weapons::XBOW_FIRE1);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_item()
            .volume(engine.random_float(0.95, 1.0))
            .pitch(93 + engine.random_int(0, 0xf))
            .play(sound::weapons::XBOW_RELOAD1);

        if self.is_local(idx) {
            if args.iparam1() != 0 {
                ev.weapon_animation(CrossbowAnimation::Fire1 as c_int, 1);
            } else if args.iparam2() != 0 {
                ev.weapon_animation(CrossbowAnimation::Fire3 as c_int, 1);
            }

            view_mut().punch_axis(PITCH, -2.0);
        }
    }

    pub(super) fn fire_crossbow2(&self, args: &mut EventArgs) {
        let idx = args.entindex();
        let origin = args.origin();
        let forward = args.angles().angle_vectors().forward();
        let engine = self.engine;
        let ev = engine.event_api();
        let efx = engine.efx_api();

        let src = self.get_gun_position(args, origin);
        let end = src + forward * 8192.0;

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_weapon()
            .pitch(93 + engine.random_int(0, 0xf))
            .play(sound::weapons::XBOW_FIRE1);

        ev.build_sound_at(origin)
            .entity(idx)
            .channel_item()
            .volume(engine.random_float(0.95, 1.0))
            .pitch(93 + engine.random_int(0, 0xf))
            .play(sound::weapons::XBOW_RELOAD1);

        if self.is_local(idx) {
            if args.iparam1() != 0 {
                ev.weapon_animation(CrossbowAnimation::Fire1 as c_int, 1);
            } else if args.iparam2() != 0 {
                ev.weapon_animation(CrossbowAnimation::Fire3 as c_int, 1);
            }
        }

        let pm_states = ev.push_pm_states();
        ev.set_solid_players(idx.to_i32() - 1);
        ev.set_trace_hull(2);
        let tr = ev.player_trace(src, end, PM_NORMAL, -1);

        if tr.fraction < 1.0 {
            let pe = ev.get_phys_ent(tr.ent).unwrap();

            if pe.solid != SOLID_BSP {
                let sample = match engine.random_int(0, 1) {
                    0 => sound::weapons::XBOW_HITBOD1,
                    _ => sound::weapons::XBOW_HITBOD2,
                };
                ev.build_sound_at(tr.endpos)
                    .entity(idx)
                    .channel_body()
                    .play(sample);
            } else if pe.rendermode == RenderMode::Normal as c_int {
                ev.build_sound_at(tr.endpos)
                    .entity(EntityIndex::WORLD_SPAWN)
                    .channel_body()
                    .volume(engine.random_float(0.95, 1.0))
                    .play(sound::weapons::XBOW_HIT1);

                if engine.pm_point_contents(tr.endpos).0 != CONTENTS_WATER {
                    efx.spark_shower(tr.endpos);
                }

                let model_index = ev.find_model_index(models::CROSSBOW_BOLT);
                let bolt_angles = forward;

                let pos = tr.endpos - forward * 10.0;
                let bolt = efx.temp_model(
                    pos,
                    vec3_t::ZERO,
                    bolt_angles,
                    5.0,
                    model_index,
                    TE_BOUNCE_NULL as c_int,
                );

                if !bolt.is_null() {
                    let bolt = unsafe { &mut *bolt };
                    bolt.flags_mut().insert(TempEntityFlags::CLIENTCUSTOM);
                    bolt.entity.baseline.vuser1 = pos;
                    bolt.entity.baseline.vuser2 = bolt_angles;

                    unsafe extern "C" fn bolt_callback(
                        temp: *mut TEMPENTITY,
                        _frametime: f32,
                        _currenttime: f32,
                    ) {
                        let temp = unsafe { &mut *temp };
                        temp.entity.origin = temp.entity.baseline.vuser1;
                        temp.entity.angles = temp.entity.baseline.vuser2;
                    }

                    bolt.callback = Some(bolt_callback);
                }
            }
        }

        pm_states.pop();
    }
}
