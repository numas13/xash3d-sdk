use core::ffi::c_int;

use cl::{
    consts::{ATTN_NORM, CHAN_WEAPON, PITCH, PM_NORMAL, SOLID_BSP, TE_SPRITETRAIL},
    engine::{efx::BeamEntity, event::event_args_s},
    entity::TempEntityFlags,
    ffi::common::vec3_t,
    prelude::*,
    raw::{RenderFx, RenderMode, SoundFlags},
};
use res::valve::{self, sound, sprites};

use crate::export::view_mut;

use super::{decal_gunshot, get_gun_position, is_local, muzzle_flash, Bullet, Events};

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(C)]
enum Gauss {
    Idle = 0,
    Idle2,
    Fidget,
    Spinup,
    Spin,
    Fire,
    Fire2,
    Holster,
    Draw,
}

impl Events {
    fn stop_previous_gauss(&self, idx: c_int) {
        let engine = engine();
        let ev = engine.event_api();
        ev.kill_events(idx, valve::events::GAUSSSPIN);
        ev.stop_sound(idx, CHAN_WEAPON, sound::ambience::PULSEMACHINE);
    }

    pub(super) fn fire_gauss(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        if args.bparam2 != 0 {
            self.stop_previous_gauss(idx);
            return;
        }

        let primary_fire = args.bparam1 != 0;
        let origin = args.origin();
        let angles = args.angles();
        let mut forward = angles.angle_vectors().forward();

        let engine = engine();
        let ev = engine.event_api();
        let efx = engine.efx_api();

        let beam = ev.find_model_index(sprites::SMOKE);
        let glow = ev.find_model_index(sprites::HOTGLOW);
        let balls = glow;

        if is_local(idx) {
            ev.weapon_animation(Gauss::Fire2 as c_int, 2);
            view_mut().punch_axis(PITCH, -2.0);

            if !primary_fire {
                // TODO: g_flApplyVel = flDamage;
                // prediction
            }
        }

        let mut damage = args.fparam1;
        let sample = sound::weapons::GAUSS2;
        let vol = 0.5 + damage * (1.0 / 400.0);
        let pitch = 85 + engine.random_int(0, 0x1f);
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            vol,
            ATTN_NORM,
            SoundFlags::NONE,
            pitch,
        );

        // FIXME: what range is used for colors? 0..=1 or 0..=255?
        let mut color = if primary_fire {
            [255.0, 128.0, 0.0]
        } else {
            [255.0, 255.0, 255.0]
        };
        // TODO: remove me
        for i in &mut color {
            *i /= 255.0;
        }

        let width = if primary_fire { 1.0 } else { 2.5 };

        let mut src = get_gun_position(args, origin);
        let mut dest = src + forward * 8192.0;
        let mut first_beam = true;
        let mut has_punched = false;
        for _ in 0..10 {
            if damage <= 10.0 {
                break;
            }

            ev.setup_player_predication(false, true);
            let pm_states = ev.push_pm_states();
            ev.set_solid_players(idx - 1);
            ev.set_trace_hull(2);
            let tr = ev.player_trace(src, dest, PM_NORMAL, -1);
            pm_states.pop();

            if tr.allsolid != 0 {
                break;
            }

            if first_beam {
                first_beam = false;
                if is_local(idx) {
                    muzzle_flash();
                }

                efx.beam_ent_point(
                    BeamEntity::new(idx as u16, 1).unwrap(),
                    tr.endpos,
                    beam,
                    0.1,
                    width,
                    0.0,
                    if primary_fire { 128.0 } else { damage },
                    0.0,
                    0,
                    0.0,
                    color,
                );
            } else {
                efx.beam_points(
                    src,
                    tr.endpos,
                    beam,
                    0.1,
                    width,
                    0.0,
                    if primary_fire { 128.0 } else { damage },
                    0.0,
                    0,
                    0.0,
                    color,
                );
            }

            let Some(entity) = ev.get_phys_ent(tr.ent) else {
                break;
            };

            if entity.solid == SOLID_BSP {
                let n = -tr.plane.normal.dot_product(forward);
                if n < 0.5 {
                    forward += tr.plane.normal * (n * 2.0);

                    src = tr.endpos + forward * 8.0;
                    dest = src + forward * 8192.0;

                    efx.temp_sprite(
                        tr.endpos,
                        vec3_t::ZERO,
                        0.2,
                        glow,
                        RenderMode::Glow,
                        RenderFx::NoDissipation,
                        damage * n / 255.0,
                        damage * n * 0.5 * 0.1,
                        TempEntityFlags::FADEOUT,
                    );

                    let fwd = tr.endpos + tr.plane.normal;

                    efx.sprite_trail(
                        TE_SPRITETRAIL as c_int,
                        tr.endpos,
                        fwd,
                        balls,
                        3,
                        0.1,
                        engine.random_float(10.0, 20.0) / 100.0,
                        100.0,
                        255,
                        100.0,
                    );

                    let n = if n == 0.0 { 0.1 } else { n };
                    damage *= 1.0 - n;
                } else {
                    decal_gunshot(&tr, Bullet::Monster12mm);

                    efx.temp_sprite(
                        tr.endpos,
                        vec3_t::ZERO,
                        1.0,
                        glow,
                        RenderMode::Glow,
                        RenderFx::NoDissipation,
                        damage / 255.0,
                        6.0,
                        TempEntityFlags::FADEOUT,
                    );

                    if has_punched {
                        break;
                    }
                    has_punched = true;

                    if !primary_fire {
                        let start = tr.endpos + forward * 8.0;

                        let pm_states = ev.push_pm_states();
                        ev.set_solid_players(idx - 1);
                        ev.set_trace_hull(2);
                        let mut beam_tr = ev.player_trace(start, dest, PM_NORMAL, -1);

                        if beam_tr.allsolid == 0 {
                            beam_tr = ev.player_trace(beam_tr.endpos, tr.endpos, PM_NORMAL, -1);
                            let delta = beam_tr.endpos - tr.endpos;
                            let mut n = delta.length();
                            if n < damage {
                                if n == 0.0 {
                                    n = 1.0;
                                }
                                damage -= n;

                                let fwd = tr.endpos - forward;
                                efx.sprite_trail(
                                    TE_SPRITETRAIL as c_int,
                                    tr.endpos,
                                    fwd,
                                    balls,
                                    3,
                                    0.1,
                                    engine.random_float(10.0, 20.0) / 100.0,
                                    100.0,
                                    255,
                                    100.0,
                                );

                                decal_gunshot(&beam_tr, Bullet::Monster12mm);

                                efx.temp_sprite(
                                    beam_tr.endpos,
                                    vec3_t::ZERO,
                                    0.1,
                                    glow,
                                    RenderMode::Glow,
                                    RenderFx::NoDissipation,
                                    damage / 255.0,
                                    6.0,
                                    TempEntityFlags::FADEOUT,
                                );

                                let fwd = beam_tr.endpos - forward;

                                efx.sprite_trail(
                                    TE_SPRITETRAIL as c_int,
                                    beam_tr.endpos,
                                    fwd,
                                    balls,
                                    (damage * 0.3) as c_int,
                                    0.1,
                                    engine.random_float(10.0, 20.0) / 100.0,
                                    200.0,
                                    255,
                                    40.0,
                                );

                                src = beam_tr.endpos - forward;
                            }
                        } else {
                            damage = 0.0;
                        }

                        pm_states.pop();
                    } else {
                        efx.temp_sprite(
                            tr.endpos,
                            vec3_t::ZERO,
                            0.2,
                            glow,
                            RenderMode::Glow,
                            RenderFx::NoDissipation,
                            200.0 / 255.0,
                            0.3,
                            TempEntityFlags::FADEOUT,
                        );

                        damage = 0.0;
                    }
                }
            } else {
                src = tr.endpos - forward;
            }
        }
    }

    pub(super) fn spin_gauss(&mut self, args: &mut event_args_s) {
        let idx = args.entindex;
        let origin = args.origin();
        let sample = sound::ambience::PULSEMACHINE;
        let vol = 1.0;
        let flags = if args.bparam1 != 0 {
            SoundFlags::CHANGE_PITCH
        } else {
            SoundFlags::NONE
        };
        let pitch = args.iparam1;
        let engine = engine();
        let ev = engine.event_api();
        ev.play_sound(
            idx,
            origin,
            CHAN_WEAPON,
            sample,
            vol,
            ATTN_NORM,
            flags,
            pitch,
        );
    }
}
