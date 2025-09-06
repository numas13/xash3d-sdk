use core::ffi::c_int;

use cl::{
    consts::{
        ATTN_NORM, CHAN_BODY, CHAN_ITEM, CHAN_WEAPON, CONTENTS_WATER, PITCH, PITCH_NORM, PM_NORMAL,
        SOLID_BSP, TE_BOUNCE_NULL,
    },
    engine::event::EventArgs,
    prelude::*,
    raw::{vec3_t, RenderMode, SoundFlags, TempEntFlags, TEMPENTITY},
};
use res::valve::{models, sound};

use crate::export::view_mut;

use super::{get_gun_position, is_local, Events};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Crossbow {
    Idle1 = 0,
    Idle2,
    Fidget1,
    Fidget2,
    Fire1,
    Fire2,
    Fire3,
    Reload,
    Draw1,
    Draw2,
    Holster1,
    Holster2,
}

impl Events {
    pub(super) fn fire_crossbow(&mut self, args: &mut EventArgs) {
        let idx = args.entindex;
        let origin = args.origin;

        let engine = engine();
        let ev = engine.event_api();

        let sample = sound::weapons::XBOW_FIRE1;
        let pitch = 93 + engine.random_int(0, 0xf);
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            1.0,
            ATTN_NORM,
            SoundFlags::NONE,
            pitch,
        );

        let sample = sound::weapons::XBOW_RELOAD1;
        let vol = engine.random_float(0.95, 1.0);
        let pitch = 93 + engine.random_int(0, 0xf);
        ev.play_sound(
            idx,
            origin,
            CHAN_ITEM,
            sample,
            vol,
            ATTN_NORM,
            SoundFlags::NONE,
            pitch,
        );

        if is_local(idx) {
            if args.iparam1 != 0 {
                ev.weapon_animation(Crossbow::Fire1 as c_int, 1);
            } else if args.iparam2 != 0 {
                ev.weapon_animation(Crossbow::Fire3 as c_int, 1);
            }

            view_mut().punch_axis(PITCH, -2.0);
        }
    }

    pub(super) fn fire_crossbow2(&mut self, args: &mut EventArgs) {
        let idx = args.entindex;
        let origin = args.origin;
        let forward = args.angles.angle_vectors().forward();
        let engine = engine();
        let ev = engine.event_api();
        let efx = engine.efx_api();

        let src = get_gun_position(args, origin);
        let end = src + forward * 8192.0;

        let sample = sound::weapons::XBOW_FIRE1;
        let pitch = 93 + engine.random_int(0, 0xf);
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            1.0,
            ATTN_NORM,
            SoundFlags::NONE,
            pitch,
        );

        let sample = sound::weapons::XBOW_RELOAD1;
        let vol = engine.random_float(0.95, 1.0);
        let pitch = 93 + engine.random_int(0, 0xf);
        ev.play_sound(
            idx,
            origin,
            CHAN_ITEM,
            sample,
            vol,
            ATTN_NORM,
            SoundFlags::NONE,
            pitch,
        );

        if is_local(idx) {
            if args.iparam1 != 0 {
                ev.weapon_animation(Crossbow::Fire1 as c_int, 1);
            } else if args.iparam2 != 0 {
                ev.weapon_animation(Crossbow::Fire3 as c_int, 1);
            }
        }

        let pm_states = ev.push_pm_states();
        ev.set_solid_players(idx - 1);
        ev.set_trace_hull(2);
        let tr = ev.player_trace(src, end, PM_NORMAL, -1);

        if tr.fraction < 1.0 {
            let pe = ev.get_phys_ent(tr.ent).unwrap();

            if pe.solid != SOLID_BSP {
                let sample = match engine.random_int(0, 1) {
                    0 => sound::weapons::XBOW_HITBOD1,
                    _ => sound::weapons::XBOW_HITBOD2,
                };
                ev.play_sound(
                    idx,
                    tr.endpos,
                    CHAN_BODY,
                    sample,
                    1.0,
                    ATTN_NORM,
                    SoundFlags::NONE,
                    PITCH_NORM,
                );
            } else if pe.rendermode == RenderMode::Normal {
                let sample = sound::weapons::XBOW_HIT1;
                let vol = engine.random_float(0.95, 1.0);
                ev.play_sound(
                    0,
                    tr.endpos,
                    CHAN_BODY,
                    sample,
                    vol,
                    ATTN_NORM,
                    SoundFlags::NONE,
                    PITCH_NORM,
                );

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
                    bolt.flags.insert(TempEntFlags::CLIENTCUSTOM);
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
