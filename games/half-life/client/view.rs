use core::{f32, ffi::c_int, ptr};

use xash3d_client::{
    consts::{CONTENTS_WATER, PITCH, ROLL, YAW},
    ffi::common::{cl_entity_s, ref_params_s, vec3_t},
    input::KeyState,
    macros::hook_command,
    math::{fabsf, fmaxf, fminf, sinf, sqrtf},
    prelude::*,
};

use crate::{
    export::{camera, view_mut},
    helpers::*,
    input,
};

mod cvar {
    xash3d_client::cvar::define! {
        pub static cl_bobcycle(c"0.8", NONE);
        pub static cl_bob(c"0.01", ARCHIVE);
        pub static cl_bobup(c"0.5", NONE);

        pub static v_centermove(c"0.15", NONE);
        pub static v_centerspeed(c"500", NONE);

        pub static cl_vsmoothing(c"0.05", ARCHIVE);
        pub static cl_forwardspeed(c"400", ARCHIVE);

        pub static scr_ofsx(c"0", NONE);
        pub static scr_ofsy(c"0", NONE);
        pub static scr_ofsz(c"0", NONE);

        pub static cl_waterdist(c"4", NONE);
    }
}

struct Bob {
    bob_time: f64,
    bob: f32,
    last_time: f32,
}

impl Bob {
    fn new() -> Self {
        Self {
            bob_time: 0.0,
            bob: 0.0,
            last_time: 0.0,
        }
    }

    fn calc_bob(&mut self, params: &mut ref_params_s) -> f32 {
        if params.onground == -1 || params.time == self.last_time {
            return self.bob;
        }

        self.last_time = params.time;

        self.bob_time += params.frametime as f64;
        let tmp = (self.bob_time / cvar::cl_bobcycle.value() as f64) as c_int;
        let mut cycle = (self.bob_time - (tmp as f32 * cvar::cl_bobcycle.value()) as f64) as f32;
        cycle /= cvar::cl_bobcycle.value();

        if cycle < cvar::cl_bobup.value() {
            cycle = f32::consts::PI * cycle / cvar::cl_bobup.value();
        } else {
            cycle = f32::consts::PI
                + f32::consts::PI * (cycle - cvar::cl_bobup.value())
                    / (1.0 - cvar::cl_bobup.value());
        }

        let vel = params.simvel.copy_with_z(0.0);

        self.bob = sqrtf(vel[0] * vel[0] + vel[1] * vel[1]) * cvar::cl_bob.value();
        self.bob = self.bob * 0.3 + self.bob * 0.7 * sinf(cycle);
        self.bob = self.bob.clamp(-7.0, 4.0);
        self.bob
    }
}

impl Default for Bob {
    fn default() -> Self {
        Self::new()
    }
}

struct PitchDrift {
    engine: ClientEngineRef,
    pitchvel: f32,
    drift: bool,
    driftmove: f32,
    laststop: f64,
}

impl PitchDrift {
    fn new(engine: ClientEngineRef) -> Self {
        Self {
            engine,
            pitchvel: 0.0,
            drift: false,
            driftmove: 0.0,
            laststop: 0.0,
        }
    }

    fn start(&mut self) {
        if self.laststop == self.engine.get_client_time() as f64 {
            return;
        }

        if !self.drift || self.pitchvel == 0.0 {
            self.pitchvel = cvar::v_centerspeed.value();
            self.drift = true;
            self.driftmove = 0.0;
        }
    }

    fn stop(&mut self) {
        self.laststop = self.engine.get_client_time() as f64;
        self.drift = false;
        self.pitchvel = 0.0;
    }

    fn drift_pitch(&mut self, params: &mut ref_params_s) {
        let engine = self.engine;
        if engine.is_no_clipping()
            || params.onground == 0
            || params.demoplayback != 0
            || params.spectator != 0
        {
            self.driftmove = 0.0;
            self.pitchvel = 0.0;
            return;
        }

        if !self.drift {
            let state = KeyState::from_bits_retain(unsafe { input::in_mlook.state });
            if cvar::v_centermove.value() > 0.0 && !state.contains(KeyState::DOWN) {
                if fabsf(params.cmd().forwardmove) < cvar::cl_forwardspeed.value() {
                    self.driftmove = 0.0;
                } else {
                    self.driftmove += params.frametime;
                }

                if self.driftmove > cvar::v_centermove.value() {
                    self.start();
                } else {
                    return;
                }
            }

            return;
        }

        let delta = params.idealpitch - params.cl_viewangles[PITCH];

        if delta == 0.0 {
            self.pitchvel = 0.0;
            return;
        }

        let mut mov = params.frametime * self.pitchvel;

        self.pitchvel *= 1.0 + params.frametime * 0.25;

        if delta > 0.0 {
            if mov > delta {
                self.pitchvel = 0.0;
                mov = delta;
            }
            params.cl_viewangles[PITCH] += mov;
        } else if delta < 0.0 {
            if mov > -delta {
                self.pitchvel = 0.0;
                mov = -delta;
            }
            params.cl_viewangles[PITCH] -= mov;
        }
    }
}

const ORIGIN_BACKUP: usize = 64;
const ORIGIN_MASK: usize = ORIGIN_BACKUP - 1;

struct ViewInterp {
    origins: [vec3_t; ORIGIN_BACKUP],
    origin_time: [f32; ORIGIN_BACKUP],
    current_origin: usize,
    last_origin: vec3_t,
}

impl ViewInterp {
    fn new() -> Self {
        Self {
            origins: [vec3_t::ZERO; ORIGIN_BACKUP],
            origin_time: [0.0; ORIGIN_BACKUP],
            current_origin: 0,
            last_origin: vec3_t::ZERO,
        }
    }

    fn calc(&mut self, params: &mut ref_params_s, view: &mut cl_entity_s) {
        let delta = params.simorg - self.last_origin;
        if delta.length() != 0.0 {
            self.origins[self.current_origin & ORIGIN_MASK] = params.simorg;
            self.origin_time[self.current_origin & ORIGIN_MASK] = params.time;
            self.current_origin += 1;

            self.last_origin = params.simorg;
        }

        if cvar::cl_vsmoothing.value() != 0.0 && params.smoothing != 0 && params.maxclients > 1 {
            if cvar::cl_vsmoothing.value() < 0.0 {
                cvar::cl_vsmoothing.value_set(0.0);
            }

            let t = params.time - cvar::cl_vsmoothing.value();

            let mut foundidx = 0;
            let mut i = 1;
            while i < ORIGIN_MASK {
                foundidx = self.current_origin.wrapping_sub(1).wrapping_sub(i);
                if self.origin_time[foundidx & ORIGIN_MASK] <= t {
                    break;
                }
                i += 1;
            }

            if i < ORIGIN_MASK && self.origin_time[foundidx & ORIGIN_MASK] != 0.0 {
                let dt = self.origin_time[(foundidx + 1) & ORIGIN_MASK]
                    - self.origin_time[foundidx & ORIGIN_MASK];
                if dt > 0.0 {
                    let mut frac = (t - self.origin_time[foundidx & ORIGIN_MASK]) / dt;
                    frac = fminf(1.0, frac);
                    let delta = self.origins[(foundidx + 1) & ORIGIN_MASK]
                        - self.origins[foundidx & ORIGIN_MASK];
                    let neworg = self.origins[foundidx & ORIGIN_MASK] + delta * frac;

                    if delta.length() < 64.0 {
                        let delta = neworg - params.simorg;

                        params.simorg += delta;
                        params.vieworg += delta;
                        view.origin += delta;
                    }
                }
            }
        }
    }
}

pub struct View {
    engine: ClientEngineRef,
    punchangle: vec3_t,
    bob: Bob,
    pitch_drift: PitchDrift,
    view_interp: ViewInterp,
    old_z: f32,
    last_time: f32,
}

impl View {
    pub fn new(engine: ClientEngineRef) -> Self {
        hook_command!(engine, c"centerview", |_| view_mut().pitch_drift.start());

        Self {
            engine,
            punchangle: vec3_t::ZERO,
            bob: Bob::default(),
            pitch_drift: PitchDrift::new(engine),
            view_interp: ViewInterp::new(),
            old_z: 0.0,
            last_time: 0.0,
        }
    }

    pub fn start_pitch_drift(&mut self) {
        self.pitch_drift.start();
    }

    pub fn stop_pitch_drift(&mut self) {
        self.pitch_drift.stop();
    }

    pub fn punch_axis(&mut self, axis: usize, punch: f32) {
        self.punchangle[axis] = punch;
    }

    fn calc_gun_angle(&self, params: &ref_params_s) {
        let ent = unsafe { &mut *self.engine.get_view_entity() };
        ent.angles[YAW] = params.viewangles[YAW] + params.crosshairangle[YAW];
        ent.angles[PITCH] = -params.viewangles[PITCH] + params.crosshairangle[PITCH] * 0.25;
    }

    fn calc_intermission_refdef(&mut self, params: &mut ref_params_s) {
        let engine = self.engine;
        let view = unsafe { &mut *engine.get_view_entity() };

        params.vieworg = params.simorg;
        params.viewangles = params.cl_viewangles;

        view.model = ptr::null_mut();

        if engine.is_spectator_only() {
            todo!();
        }
    }

    fn calc_view_roll(&self, params: &mut ref_params_s) {
        let viewentity = self.engine.get_entity_by_index(params.viewentity);
        if viewentity.is_null() {
            return;
        }
        let viewentity = unsafe { &*viewentity };

        let side = xash3d_client::math::calc_roll(
            viewentity.angles,
            params.simvel,
            params.movevars().rollangle,
            params.movevars().rollspeed,
        );
        params.viewangles[ROLL] += side;

        if params.health <= 0 && params.viewheight[2] != 0.0 {
            params.viewangles[ROLL] = 80.0;
        }
    }

    fn calc_normal_refdef(&mut self, params: &mut ref_params_s) {
        let engine = self.engine;

        self.pitch_drift.drift_pitch(params);

        let ent = if engine.is_spectator_only() {
            engine.get_entity_by_index(unsafe { g_iUser2 })
        } else {
            engine.get_local_player()
        };
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };

        let view = unsafe { &mut *engine.get_view_entity() };

        let bob = self.bob.calc_bob(params);

        params.vieworg = params.simorg + params.viewheight;
        params.vieworg[2] += bob;

        params.viewangles = params.cl_viewangles;

        engine.calc_shake();
        engine.apply_shake(&mut params.vieworg, &mut params.viewangles, 1.0);

        params.vieworg += 1.0 / 32.0;

        let mut water_offset = 0.0;
        if params.waterlevel >= 2 {
            let mut water_dist = cvar::cl_waterdist.value() as c_int;

            if params.hardware != 0 {
                let water_ent = engine.pm_water_entity(params.simorg);
                if water_ent >= 0 && water_ent < params.max_entities {
                    let pwater = engine.get_entity_by_index(water_ent);
                    if !pwater.is_null() {
                        let pwater = unsafe { &*pwater };
                        if !pwater.model.is_null() {
                            water_dist += (pwater.curstate.scale * 16.0) as c_int;
                        }
                    }
                }
            }

            let mut point = params.vieworg;

            if params.waterlevel == 2 {
                point[2] -= water_dist as f32;
                for _ in 0..water_dist {
                    let contents = engine.pm_point_contents(point).0;
                    if contents > CONTENTS_WATER {
                        break;
                    }
                    point[2] += 1.0;
                }
                water_offset = point[2] + water_dist as f32 - params.vieworg[2];
            } else {
                point[2] += water_dist as f32;
                for _ in 0..water_dist {
                    let contents = engine.pm_point_contents(point).0;
                    if contents <= CONTENTS_WATER {
                        break;
                    }
                    point[2] -= 1.0;
                }
                water_offset = point[2] - water_dist as f32 - params.vieworg[2];
            }
        }

        params.vieworg[2] += water_offset;

        self.calc_view_roll(params);

        let av = params.cl_viewangles.angle_vectors().all();
        params.forward = av.forward;
        params.right = av.right;
        params.up = av.up;

        if params.maxclients <= 1 {
            params.vieworg += params.forward * cvar::scr_ofsx.value();
            params.vieworg += params.right * cvar::scr_ofsy.value();
            params.vieworg += params.up * cvar::scr_ofsz.value();
        }

        let mut cam_angles = vec3_t::ZERO;

        let camera = camera();
        if camera.is_third_person() {
            let ofs = camera.offset();
            cam_angles = ofs;
            cam_angles[2] = 0.0;

            let cam_forward = cam_angles.angle_vectors().forward();
            params.vieworg += cam_forward * -ofs[2];
        }

        view.angles = params.cl_viewangles;

        self.calc_gun_angle(params);

        view.origin = params.simorg + params.viewheight;
        view.origin[2] += water_offset;

        engine.apply_shake(&mut view.origin, &mut view.angles, 0.9);

        view.origin += params.forward * bob * 0.4;
        view.origin[2] += bob;

        view.angles[YAW] -= bob * 0.5;
        view.angles[ROLL] -= bob * 1.0;
        view.angles[PITCH] -= bob * 0.3;

        view.origin[2] -= 1.0;

        if params.viewsize == 110.0 {
            view.origin[2] += 1.0;
        } else if params.viewsize == 100.0 {
            view.origin[2] += 2.0;
        } else if params.viewsize == 90.0 {
            view.origin[2] += 1.0;
        } else if params.viewsize == 80.0 {
            view.origin[2] += 0.5;
        }

        params.viewangles += params.punchangle;
        params.viewangles += self.punchangle;
        self.punchangle = drop_punch_angle(params.frametime, self.punchangle);

        if params.smoothing == 0 && params.onground != 0 && params.simorg[2] - self.old_z > 0.0 {
            let mut steptime = params.time - self.last_time;
            if steptime < 0.0 {
                warn!("V_CalcNormalRefdef: steptime < 0.0");
                steptime = 0.0;
            }

            self.old_z += steptime * 150.0;
            if self.old_z > params.simorg[2] {
                self.old_z = params.simorg[2];
            }
            if params.simorg[2] - self.old_z > 18.0 {
                self.old_z = params.simorg[2] - 18.0;
            }
            params.vieworg[2] += self.old_z - params.simorg[2];
            view.origin[2] += self.old_z - params.simorg[2];
        } else {
            self.old_z = params.simorg[2];
        }

        self.view_interp.calc(params, view);

        if camera.is_third_person() {
            params.viewangles = cam_angles;
            let mut pitch = cam_angles[PITCH];

            if pitch > 180.0 {
                pitch -= 360.0;
            } else if pitch < -180.0 {
                pitch += 360.0;
            }

            pitch /= -3.0;

            ent.angles[PITCH] = pitch;
            ent.curstate.angles[PITCH] = pitch;
            ent.prevstate.angles[PITCH] = pitch;
            ent.latched.prevangles[PITCH] = pitch;
        }

        if params.viewentity > params.maxclients {
            let viewentity = engine.get_entity_by_index(params.viewentity);
            if !viewentity.is_null() {
                let viewentity = unsafe { &*viewentity };
                params.vieworg = viewentity.origin;
                params.viewangles = viewentity.angles;
            }
        }

        view.curstate.origin = view.origin;
        view.latched.prevorigin = view.origin;
        view.curstate.angles = view.angles;
        view.latched.prevangles = view.angles;

        self.last_time = params.time;
    }

    pub fn calc_ref_def(&mut self, params: &mut ref_params_s) {
        if params.intermission != 0 {
            self.calc_intermission_refdef(params);
        } else if params.spectator != 0 || unsafe { g_iUser1 } != 0 {
            todo!("V_CalcSpectatorRefdef");
        } else if params.paused == 0 {
            self.calc_normal_refdef(params);
        }
    }
}

fn drop_punch_angle(frametime: f32, punchangle: vec3_t) -> vec3_t {
    let (punchangle, mut len) = punchangle.normalize_length();
    len -= (10.0 + len * 0.5) * frametime;
    len = fmaxf(len, 0.0);
    punchangle * len
}
