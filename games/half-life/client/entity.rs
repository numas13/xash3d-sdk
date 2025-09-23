use core::{cell::Cell, ffi::c_int};

use csz::CStrThin;
use xash3d_client::{
    consts::{DEAD_NO, PM_STUDIO_BOX, PM_WORLD_ONLY, YAW},
    entity::{EntityType, TempEntityFlags, TempEntityList},
    ffi::{
        api::efx::TEMPENTITY,
        common::{cl_entity_s, clientdata_s, entity_state_s, vec3_t, weapon_data_s},
    },
    math::sinf,
    prelude::*,
    render::RenderMode,
};

use crate::{helpers, hud::MAX_WEAPONS};

pub struct Entities {
    engine: ClientEngineRef,
    temp_ent_frame: Cell<c_int>,
}

impl Entities {
    pub fn new(engine: ClientEngineRef) -> Self {
        Self {
            engine,
            temp_ent_frame: Cell::new(0),
        }
    }

    pub fn txfer_local_overrides(&self, state: &mut entity_state_s, client: &clientdata_s) {
        state.origin = client.origin;

        // spectator
        state.iuser1 = client.iuser1;
        state.iuser2 = client.iuser2;

        // duck prevention
        state.iuser3 = client.iuser3;

        // fire prevention
        state.iuser4 = client.iuser4;
    }

    pub fn txfer_prediction_data(
        &mut self,
        ps: &mut entity_state_s,
        pps: &entity_state_s,
        pcd: &mut clientdata_s,
        ppcd: &clientdata_s,
        wd: &mut [weapon_data_s; MAX_WEAPONS],
        pwd: &[weapon_data_s; MAX_WEAPONS],
    ) {
        ps.flFallVelocity = pps.flFallVelocity;
        ps.iStepLeft = pps.iStepLeft;
        ps.oldbuttons = pps.oldbuttons;
        ps.playerclass = pps.playerclass;

        pcd.ammo_cells = ppcd.ammo_cells;
        pcd.ammo_nails = ppcd.ammo_nails;
        pcd.ammo_rockets = ppcd.ammo_rockets;
        pcd.ammo_shells = ppcd.ammo_shells;
        pcd.deadflag = ppcd.deadflag;
        pcd.fov = ppcd.fov;
        pcd.m_flNextAttack = ppcd.m_flNextAttack;
        pcd.m_iId = ppcd.m_iId;
        pcd.maxspeed = ppcd.maxspeed;
        pcd.tfstate = ppcd.tfstate;
        pcd.viewmodel = ppcd.viewmodel;
        pcd.weaponanim = ppcd.weaponanim;

        unsafe {
            // get control over view angles if spectating or not dead
            helpers::g_iAlive = (ppcd.iuser1 != 0 || pcd.deadflag == DEAD_NO).into();
        }

        // spectator
        pcd.iuser1 = ppcd.iuser1;
        pcd.iuser2 = ppcd.iuser2;

        // duck prevention
        pcd.iuser3 = ppcd.iuser3;

        if self.engine.is_spectator_only() {
            pcd.iuser1 = unsafe { helpers::g_iUser1 }; // observer mode
            pcd.iuser2 = unsafe { helpers::g_iUser2 }; // first target
            pcd.iuser3 = unsafe { helpers::g_iUser3 }; // second target
        }

        // fire prevention
        pcd.iuser4 = ppcd.iuser4;

        pcd.fuser2 = ppcd.fuser2;
        pcd.fuser3 = ppcd.fuser3;

        pcd.vuser1 = ppcd.vuser1;
        pcd.vuser2 = ppcd.vuser2;
        pcd.vuser3 = ppcd.vuser3;
        pcd.vuser4 = ppcd.vuser4;

        wd.copy_from_slice(pwd);
    }

    pub fn process_player_state(&mut self, dst: &mut entity_state_s, src: &entity_state_s) {
        dst.origin = src.origin;
        dst.angles = src.angles;
        dst.velocity = src.velocity;
        dst.basevelocity = src.basevelocity;

        dst.frame = src.frame;
        dst.modelindex = src.modelindex;
        dst.skin = src.skin;
        dst.effects = src.effects;
        dst.weaponmodel = src.weaponmodel;
        dst.movetype = src.movetype;
        dst.sequence = src.sequence;
        dst.animtime = src.animtime;

        dst.solid = src.solid;

        dst.rendermode = src.rendermode;
        dst.renderamt = src.renderamt;
        dst.rendercolor.r = src.rendercolor.r;
        dst.rendercolor.g = src.rendercolor.g;
        dst.rendercolor.b = src.rendercolor.b;
        dst.renderfx = src.renderfx;

        dst.framerate = src.framerate;
        dst.body = src.body;

        dst.controller = src.controller;
        dst.blending = src.blending; // FIXME: only 2 elements???

        dst.friction = src.friction;
        dst.gravity = src.gravity;
        dst.gaitsequence = src.gaitsequence;
        dst.spectator = src.spectator;
        dst.usehull = src.usehull;
        dst.playerclass = src.playerclass;
        dst.team = src.team;
        dst.colormap = src.colormap;

        let player = unsafe { &*self.engine.get_local_player() };
        if dst.number == player.index {
            unsafe {
                helpers::g_iPlayerClass = dst.playerclass;
                helpers::g_iTeamNumber = dst.team;

                helpers::g_iUser1 = src.iuser1;
                helpers::g_iUser2 = src.iuser2;
                helpers::g_iUser3 = src.iuser3;
            }
        }
    }

    pub fn create_entities(&self) {
        // TODO:
    }

    pub fn add_entity(
        &self,
        _ty: EntityType,
        _ent: &mut cl_entity_s,
        _modelname: &CStrThin,
    ) -> bool {
        // draw this entity
        true
    }

    pub fn update_temp_entities(
        &self,
        frametime: f64,
        client_time: f64,
        cl_gravity: f64,
        list: &mut TempEntityList,
        mut add_visible_entity: impl FnMut(&mut cl_entity_s) -> c_int,
        mut temp_ent_play_sound: impl FnMut(&mut TEMPENTITY, f32),
    ) {
        use TempEntityFlags as F;

        if list.is_empty() {
            return;
        }

        let engine = self.engine;
        let event = engine.event_api();
        let efx = engine.efx_api();
        event.setup_player_predication(false, true);
        let _pm_states = event.push_pm_states();
        event.set_solid_players(-1);

        if frametime <= 0.0 {
            for temp in list.iter_mut() {
                if !temp.flags().intersects(F::NOMODEL) {
                    add_visible_entity(&mut temp.entity);
                }
            }
            return;
        }

        self.temp_ent_frame
            .set((self.temp_ent_frame.get() + 1) & 31);

        let _freq = (client_time * 0.01) as f32;
        let fast_freq = (client_time * 5.5) as f32;
        let gravity = (-frametime * cl_gravity) as f32;
        let gravity_slow = gravity * 0.5;
        let frametime = frametime as f32;
        let client_time = client_time as f32;

        list.retain_mut(|temp| {
            let life = temp.die - client_time;
            if life < 0.0 {
                if temp.flags().intersects(F::FADEOUT) {
                    let cs = &mut temp.entity.curstate;
                    if cs.rendermode == RenderMode::Normal as c_int {
                        cs.rendermode = RenderMode::TransTexture as c_int;
                    }
                    let tmp = 1.0 + life * temp.fadeSpeed;
                    cs.renderamt = (temp.entity.baseline.renderamt as f32 * tmp) as c_int;
                    if cs.renderamt <= 0 {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            temp.entity.prevstate.origin = temp.entity.origin;

            if temp.flags().intersects(F::SPARKSHOWER) {
                if client_time > temp.entity.baseline.scale {
                    efx.spark_effect(temp.entity.origin, 8, -200, 200);

                    temp.entity.baseline.framerate -= 0.1;

                    if temp.entity.baseline.framerate <= 0.0 {
                        temp.die = client_time;
                    } else {
                        temp.die = client_time + 0.5;
                        temp.entity.baseline.scale = client_time + 0.1;
                    }
                }
            } else if temp.flags().intersects(F::PLYRATTACHMENT) {
                let client = engine.get_entity_by_index(temp.clientIndex as c_int);
                if !client.is_null() {
                    let client = unsafe { &*client };
                    temp.entity.origin = client.origin + temp.tentOffset;
                }
            } else if temp.flags().intersects(F::SINEWAVE) {
                temp.x += temp.entity.baseline.origin[0] * frametime;
                temp.y += temp.entity.baseline.origin[1] * frametime;

                temp.entity.origin[0] = temp.x
                    + sinf(
                        temp.entity.baseline.origin[2] + client_time * temp.entity.prevstate.frame,
                    ) * (temp.entity.curstate.framerate * 10.0);
                temp.entity.origin[1] = temp.y
                    + sinf(temp.entity.baseline.origin[2] + fast_freq + 0.7)
                        * (temp.entity.curstate.framerate * 8.0);
                temp.entity.origin[2] += temp.entity.baseline.origin[2] * frametime;
            } else if temp.flags().intersects(F::SPIRAL) {
                temp.entity.origin += temp.entity.baseline.origin * frametime;
                let t = temp as *const _ as i32 as f32;
                temp.entity.origin[0] += 8.0 * sinf(client_time * 20.0 + t);
                temp.entity.origin[1] += 4.0 * sinf(client_time * 30.0 + t);
            } else {
                temp.entity.origin += temp.entity.baseline.origin * frametime;
            }

            if temp.flags().intersects(F::SPRANIMATE) {
                temp.entity.curstate.frame += frametime * temp.entity.curstate.framerate;
                if temp.entity.curstate.frame >= temp.frameMax {
                    temp.entity.curstate.frame -= (temp.entity.curstate.frame as i32) as f32;

                    // destroy if animation sprite is not set to loop
                    if !temp.flags().intersects(F::SPRANIMATELOOP) {
                        temp.die = client_time;
                        return true;
                    }
                }
            } else if temp.flags().intersects(F::SPRCYCLE) {
                temp.entity.curstate.frame += frametime * 10.0;
                if temp.entity.curstate.frame >= temp.frameMax {
                    temp.entity.curstate.frame -= (temp.entity.curstate.frame as i32) as f32;
                }
            }

            if temp.flags().intersects(F::ROTATE) {
                temp.entity.angles += temp.entity.baseline.angles * frametime;
                temp.entity.latched.prevangles = temp.entity.angles;
            }

            if temp.flags().intersects(F::COLLIDEALL | F::COLLIDEWORLD) {
                let mut trace_fraction = 1.0;
                let mut trace_normal = vec3_t::ZERO;

                if temp.flags().intersects(F::COLLIDEALL) {
                    event.set_trace_hull(2);
                    let mut pmtrace = event.player_trace(
                        temp.entity.prevstate.origin,
                        temp.entity.origin,
                        PM_STUDIO_BOX,
                        -1,
                    );

                    if pmtrace.fraction != 1.0 {
                        let pe = event.get_phys_ent(pmtrace.ent).unwrap();

                        if pmtrace.ent == 0 || pe.info != temp.clientIndex.into() {
                            trace_fraction = pmtrace.fraction;

                            if let Some(hitcallback) = temp.hitcallback {
                                unsafe {
                                    hitcallback(temp, &mut pmtrace);
                                }
                            }
                        }
                    }
                } else if temp.flags().intersects(F::COLLIDEWORLD) {
                    event.set_trace_hull(2);
                    let mut pmtrace = event.player_trace(
                        temp.entity.prevstate.origin,
                        temp.entity.origin,
                        PM_STUDIO_BOX | PM_WORLD_ONLY,
                        -1,
                    );

                    if pmtrace.fraction != 1.0 {
                        trace_fraction = pmtrace.fraction;
                        trace_normal = pmtrace.plane.normal;

                        if temp.flags().intersects(F::SPARKSHOWER) {
                            temp.entity.baseline.origin *= 0.6;

                            if temp.entity.baseline.origin.length() < 10.0 {
                                temp.entity.baseline.framerate = 0.0;
                            }
                        }

                        if let Some(hitcallback) = temp.hitcallback {
                            unsafe {
                                hitcallback(temp, &mut pmtrace);
                            }
                        }
                    }
                }

                if trace_fraction != 1.0 {
                    temp.entity.origin = temp.entity.prevstate.origin
                        + temp.entity.baseline.origin * (trace_fraction * frametime);

                    let mut damp = temp.bounceFactor;
                    if temp.flags().intersects(F::GRAVITY | F::SLOWGRAVITY) {
                        damp *= 0.5;

                        if trace_normal[2] > 0.9
                            && temp.entity.baseline.origin[2] <= 0.0
                            && temp.entity.baseline.origin[2] >= gravity * 3.0
                        {
                            damp = 0.0;
                            temp.flags_mut().remove(
                                F::ROTATE
                                    | F::GRAVITY
                                    | F::SLOWGRAVITY
                                    | F::COLLIDEWORLD
                                    | F::SMOKETRAIL,
                            );
                        }
                    }

                    if temp.hitSound != 0 {
                        temp_ent_play_sound(temp, damp);
                    }

                    if temp.flags().intersects(F::COLLIDEKILL) {
                        temp.flags_mut().remove(F::FADEOUT);
                        temp.die = client_time;
                    } else {
                        if damp != 0.0 {
                            let proj = temp.entity.baseline.origin.dot_product(trace_normal);
                            temp.entity.baseline.origin += trace_normal * (-proj * 2.0);
                            temp.entity.angles[YAW] = -temp.entity.angles[YAW];
                        }

                        if damp != 1.0 {
                            temp.entity.baseline.origin *= damp;
                            temp.entity.angles *= 0.9;
                        }
                    }
                }
            }

            if temp.flags().intersects(F::FLICKER)
                && self.temp_ent_frame.get() == temp.entity.curstate.effects
            {
                let dl = efx.alloc_dlight(0);
                assert!(!dl.is_null());
                let dl = unsafe { &mut *dl };
                dl.origin = temp.entity.origin;
                dl.radius = 60.0;
                dl.color.r = 255;
                dl.color.g = 120;
                dl.color.b = 0;
                dl.die = client_time + 0.01;
            }

            if temp.flags().intersects(F::SMOKETRAIL) {
                efx.rocket_trail(temp.entity.prevstate.origin, temp.entity.origin, 1);
            }

            if temp.flags().intersects(F::GRAVITY) {
                temp.entity.baseline.origin[2] += gravity;
            } else if temp.flags().intersects(F::SLOWGRAVITY) {
                temp.entity.baseline.origin[2] += gravity_slow;
            }

            if temp.flags().intersects(F::CLIENTCUSTOM) {
                if let Some(callback) = temp.callback {
                    unsafe {
                        callback(temp, frametime, client_time);
                    }
                }
            }

            if !temp.flags().intersects(F::NOMODEL)
                && add_visible_entity(&mut temp.entity) == 0
                && !temp.flags().intersects(F::PERSIST)
            {
                temp.flags_mut().remove(F::FADEOUT);
                temp.die = client_time;
            }

            true
        });
    }
}
