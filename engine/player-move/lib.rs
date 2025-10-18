#![no_std]

extern crate alloc;

#[macro_use]
extern crate log;

mod debug;

pub mod raw;

use core::{
    cmp,
    ffi::{c_char, c_int, CStr},
    ptr, str,
    sync::atomic::{AtomicU8, Ordering},
};

use alloc::vec::Vec;
use csz::{CStrArray, CStrThin};
use xash3d_shared::{
    cell::SyncOnceCell,
    consts::{
        CONTENTS_CURRENT_0, CONTENTS_CURRENT_DOWN, CONTENTS_EMPTY, CONTENTS_LADDER, CONTENTS_LAVA,
        CONTENTS_SLIME, CONTENTS_SOLID, CONTENTS_TRANSLUCENT, CONTENTS_WATER, DEAD_DISCARDBODY,
        IN_ATTACK, IN_BACK, IN_DUCK, IN_FORWARD, IN_JUMP, IN_MOVELEFT, IN_MOVERIGHT, IN_USE,
        MAX_CLIP_PLANES, MAX_PHYSENTS, PITCH, PM_NORMAL, ROLL, YAW,
    },
    entity::{EdictFlags, MoveType},
    ffi::{
        common::{pmtrace_s, qboolean, vec3_t},
        player_move::{physent_s, playermove_s},
    },
    math::{self, fabsf, fmaxf, fminf, pow2, sqrtf, ToAngleVectors},
    model::ModelType,
    sound::{Attenuation, Channel, Pitch, SoundFlags},
};

const TIME_TO_DUCK: f32 = 0.4;
const VEC_DUCK_VIEW: f32 = 12.0;
const PM_DEAD_VIEWHEIGHT: f32 = -8.0;
const MAX_CLIMB_SPEED: f32 = 200.0;
const STUCK_MOVEUP: c_int = 1;
// const STUCK_MOVEDOWN: c_int = -1;
pub const VEC_VIEW_Z: f32 = 28.0;
pub const VEC_VIEW: vec3_t = vec3_t::new(0.0, 0.0, VEC_VIEW_Z);

pub const VEC_DUCK_HULL_MIN: f32 = -18.0;
pub const VEC_DUCK_HULL_MAX: f32 = 18.0;
pub const VEC_HULL_MIN: f32 = -36.0;
pub const VEC_HULL_MAX: f32 = 36.0;

// const OBS_NONE: c_int = 0;
// const OBS_CHASE_LOCKED: c_int = 1;
// const OBS_CHASE_FREE: c_int = 2;
const OBS_ROAMING: c_int = 3;
// const OBS_IN_EYE: c_int = 4;
// const OBS_MAP_FREE: c_int = 5;
// const OBS_MAP_CHASE: c_int = 6;

// const PLAYER_FATAL_FALL_SPEED: f32 = 1024.0;
const PLAYER_MAX_SAFE_FALL_SPEED: f32 = 580.0;
// const DAMAGE_FOR_FALL_SPEED: f32 = 100.0 / (PLAYER_FATAL_FALL_SPEED - PLAYER_MAX_SAFE_FALL_SPEED);
const PLAYER_MIN_BOUNCE_SPEED: f32 = 200.0;
const PLAYER_FALL_PUNCH_THRESHOLD: f32 = 350.0;

const PLAYER_LONGJUMP_SPEED: f32 = 350.0;

const PLAYER_DUCKING_MULTIPLIER: f32 = 0.333;

const MAX_CLIENTS: usize = 32;

pub const CHAR_TEX_CONCRETE: c_char = b'C' as c_char;
pub const CHAR_TEX_METAL: c_char = b'M' as c_char;
pub const CHAR_TEX_DIRT: c_char = b'D' as c_char;
pub const CHAR_TEX_VENT: c_char = b'V' as c_char;
pub const CHAR_TEX_GRATE: c_char = b'G' as c_char;
pub const CHAR_TEX_TILE: c_char = b'T' as c_char;
pub const CHAR_TEX_SLOSH: c_char = b'S' as c_char;
pub const CHAR_TEX_WOOD: c_char = b'W' as c_char;
pub const CHAR_TEX_COMPUTER: c_char = b'P' as c_char;
pub const CHAR_TEX_GLASS: c_char = b'Y' as c_char;
pub const CHAR_TEX_FLESH: c_char = b'F' as c_char;

/// Max number of textures loaded.
const CTEXTURESMAX: usize = 512;
/// Only load first n chars of name.
const CBTEXTURENAMEMAX: usize = 13;

/// Default step sound.
const STEP_CONCRETE: c_int = 0;
/// Metal floor.
const STEP_METAL: c_int = 1;
/// Dirt, sand or rock.
const STEP_DIRT: c_int = 2;
/// Ventillation duct.
const STEP_VENT: c_int = 3;
/// Metal grating.
const STEP_GRATE: c_int = 4;
// Floor tiles.
const STEP_TILE: c_int = 5;
/// Shallow liquid puddle.
const STEP_SLOSH: c_int = 6;
/// Wading in liquid.
const STEP_WADE: c_int = 7;
/// Climbing ladder.
const STEP_LADDER: c_int = 8;

static TEXTURES: SyncOnceCell<Vec<(u8, CStrArray<CBTEXTURENAMEMAX>)>> =
    unsafe { SyncOnceCell::new() };

static STUCK_TABLE: SyncOnceCell<[vec3_t; 54]> = unsafe { SyncOnceCell::new() };
static mut STUCK_LAST: [[c_int; 2]; MAX_CLIENTS] = [[0; 2]; MAX_CLIENTS];

fn trim_ascii_start(s: &[u8]) -> &[u8] {
    s.iter()
        .position(|i| !i.is_ascii_whitespace())
        .map_or(s, |i| &s[i..])
}

fn map_texture_type_step_type(texture_type: c_char) -> c_int {
    match texture_type {
        CHAR_TEX_METAL => STEP_METAL,
        CHAR_TEX_DIRT => STEP_DIRT,
        CHAR_TEX_VENT => STEP_VENT,
        CHAR_TEX_GRATE => STEP_GRATE,
        CHAR_TEX_TILE => STEP_TILE,
        CHAR_TEX_SLOSH => STEP_SLOSH,
        _ => STEP_CONCRETE,
    }
}

fn clip_velocity(input: vec3_t, normal: vec3_t, overbounce: f32) -> (c_int, vec3_t) {
    const STOP_EPSILON: f32 = 0.1;

    let backoff = input.dot(normal) * overbounce;
    let mut output = input - normal * backoff;

    for i in 0..3 {
        if (-STOP_EPSILON..STOP_EPSILON).contains(&output[i]) {
            output[i] = 0.0;
        }
    }

    let angle = normal[2];
    let blocked = if angle > 0.0 {
        1 // floor
    } else if angle == 0.0 {
        2 // wall
    } else {
        0 // unblocked
    };

    (blocked, output)
}

fn spline_fraction(value: f32, scale: f32) -> f32 {
    let value = value * scale;
    let value_squared = value * value;
    3.0 * value_squared - 2.0 * value_squared * value
}

struct WishMove {
    vel: vec3_t,
    dir: vec3_t,
    speed: f32,
}

impl WishMove {
    fn new(mut vel: vec3_t, max_speed: f32) -> Self {
        let (dir, mut speed) = vel.normalize_and_length();
        if speed > max_speed {
            vel *= max_speed / speed;
            speed = max_speed;
        }
        Self { vel, dir, speed }
    }
}

struct PlayerMove<'a> {
    raw: &'a mut playermove_s,
    ladder: bool,
}

impl<'a> PlayerMove<'a> {
    fn new(raw: &'a mut playermove_s) -> Self {
        Self { raw, ladder: false }
    }

    fn create_stuck_table(&self) {
        let mut table = [vec3_t::ZERO; 54];
        let mut idx = 0;

        let i = [-0.125, 0.0, 0.125];
        for z in i {
            table[idx] = vec3_t::new(0.0, 0.0, z);
            idx += 1;
        }
        for y in i {
            table[idx] = vec3_t::new(0.0, y, 0.0);
            idx += 1;
        }
        for x in i {
            table[idx] = vec3_t::new(x, 0.0, 0.0);
            idx += 1;
        }

        let i = [-0.125, 0.125];
        for x in i {
            for y in i {
                for z in i {
                    table[idx] = vec3_t::new(x, y, z);
                    idx += 1;
                }
            }
        }

        let zi = [0.0, 1.0, 6.0];
        for z in zi {
            table[idx] = vec3_t::new(0.0, 0.0, z);
            idx += 1;
        }
        let i = [-2.0, 0.0, 2.0];
        for y in i {
            table[idx] = vec3_t::new(0.0, y, 0.0);
            idx += 1;
        }
        for x in i {
            table[idx] = vec3_t::new(x, 0.0, 0.0);
            idx += 1;
        }

        for z in zi {
            for x in i {
                for y in i {
                    table[idx] = vec3_t::new(x, y, z);
                    idx += 1;
                }
            }
        }

        if STUCK_TABLE.set(table).is_err() {
            warn!("stuck table initialized twice");
        }
    }

    fn reset_stuck_offsets(&self) {
        unsafe {
            STUCK_LAST[self.raw.player_index as usize][self.is_server() as usize] = 0;
        }
    }

    fn get_random_stuck_offsets(&self) -> (c_int, vec3_t) {
        let index = self.raw.player_index as usize;
        let server = self.is_server() as usize;
        let v = unsafe { &mut STUCK_LAST[index][server] };
        let index = *v % 54;
        *v += 1;
        let offset = STUCK_TABLE.get().unwrap()[index as usize];
        (index, offset)
    }

    fn init_texture_types(&self) {
        if TEXTURES.get().is_some() {
            return;
        }

        let Some(file) = self.load_file(c"sound/materials.txt", 5) else {
            return;
        };

        let mut textures = Vec::with_capacity(CTEXTURESMAX);
        for line in file.as_slice().split(|&c| c == b'\n') {
            let (ty, name) = match trim_ascii_start(line).split_first() {
                Some((&c, tail)) if c.is_ascii_alphabetic() => {
                    let name = trim_ascii_start(tail);
                    (c, name.strip_suffix(b"\r").unwrap_or(name))
                }
                _ => continue,
            };
            if name.is_empty() || name.iter().any(|i| i.is_ascii_whitespace()) {
                continue;
            }
            if textures.len() >= CTEXTURESMAX {
                break;
            }
            let Ok(name) = CStrArray::from_bytes(name) else {
                continue;
            };
            textures.push((ty.to_ascii_uppercase(), name));
        }
        textures.sort_by(|(_, a), (_, b)| a.cmp_ignore_case(b));
        textures.shrink_to_fit();
        TEXTURES.set(textures).ok();
    }

    fn ladder(&self) -> *const physent_s {
        for pe in &self.raw.moveents[..self.raw.nummoveent as usize] {
            if pe.model.is_null() {
                continue;
            }
            if self.get_model_type(unsafe { &*pe.model }) != ModelType::Brush {
                continue;
            }
            if pe.skin != CONTENTS_LADDER {
                continue;
            }

            let (hull, mut test) = self.hull_for_bsp(pe);
            let hull = unsafe { &*hull };
            let num = hull.firstclipnode;
            test = self.raw.origin - test;
            if self.hull_point_contents(hull, num, test) != CONTENTS_EMPTY {
                return pe;
            }
        }
        ptr::null()
    }

    fn drop_punch_angle(&mut self) {
        let (punch_angle, mut len) = self.raw.punchangle.normalize_and_length();
        len -= (10.0 + len * 0.5) * self.raw.frametime;
        len = fmaxf(len, 0.0);
        self.raw.punchangle = punch_angle * len;
    }

    fn check_paramters(&mut self) {
        let spd = self.move_vector().length();
        let maxspeed = self.raw.clientmaxspeed;
        if maxspeed != 0.0 {
            self.raw.maxspeed = fminf(maxspeed, self.raw.maxspeed);
        }

        if self.raw.onground != -1 && self.is_button(IN_USE) {
            self.raw.maxspeed *= 1.0 / 3.0;
        }

        if spd != 0.0 && spd > self.raw.maxspeed {
            let ratio = self.raw.maxspeed / spd;
            self.set_move_vector(self.move_vector() * ratio);
        }

        if self
            .flags()
            .intersects(EdictFlags::FROZEN | EdictFlags::ONTRAIN)
            || self.is_dead()
        {
            self.set_move_vector(vec3_t::ZERO);
        }

        self.drop_punch_angle();

        if self.is_alive() {
            let v_angle = self.raw.cmd.viewangles + self.raw.punchangle;
            self.raw.angles[ROLL] = math::calc_roll(
                v_angle,
                self.raw.velocity,
                self.movevars().rollangle,
                self.movevars().rollspeed,
            ) * 4.0;
            self.raw.angles[PITCH] = v_angle[PITCH];
            self.raw.angles[YAW] = v_angle[YAW];
        } else {
            self.raw.angles = self.raw.oldangles;
        }

        if self.is_dead() {
            self.raw.view_ofs[2] = PM_DEAD_VIEWHEIGHT;
        }

        if self.raw.angles[YAW] > 180.0 {
            self.raw.angles[YAW] -= 360.0;
        }
    }

    fn reduce_timers(&mut self) {
        if self.raw.flTimeStepSound > 0 {
            let new = self.raw.flTimeStepSound - self.raw.cmd.msec as c_int;
            self.raw.flTimeStepSound = cmp::max(0, new);
        }

        if self.raw.flDuckTime > 0.0 {
            let new = self.raw.flDuckTime - self.raw.cmd.msec as f32;
            self.raw.flDuckTime = fmaxf(0.0, new);
        }

        if self.raw.flSwimTime > 0.0 {
            let new = self.raw.flSwimTime - self.raw.cmd.msec as f32;
            self.raw.flSwimTime = fmaxf(0.0, new);
        }
    }

    fn add_to_touched(&mut self, trace: pmtrace_s, impact_velocity: vec3_t) -> qboolean {
        if self.raw.touchindex[..self.raw.numtouch as usize]
            .iter()
            .any(|i| i.ent == trace.ent)
        {
            return false.into();
        }

        if self.raw.numtouch >= MAX_PHYSENTS as c_int {
            panic!("Too many entities were touched!");
        }

        self.raw.touchindex[self.raw.numtouch as usize] = pmtrace_s {
            deltavelocity: impact_velocity,
            ..trace
        };
        self.raw.numtouch += 1;

        true.into()
    }

    fn push_entity(&mut self, push: &vec3_t) -> pmtrace_s {
        let push = *push;
        let end = self.raw.origin + push;
        let trace = self.player_trace(self.raw.origin, end, PM_NORMAL, -1);
        self.raw.origin = trace.endpos;
        if trace.fraction < 1.0 && trace.allsolid == 0 {
            self.add_to_touched(trace, self.raw.velocity);
        }
        trace
    }

    fn check_velocity(&mut self) {
        fn fix_nan(v: &mut vec3_t, name: &str) {
            if v.is_nan() {
                debug!("PM  Got a NaN {name} {v:?}");
                for i in v.as_mut().iter_mut().filter(|i| i.is_nan()) {
                    *i = 0.0;
                }
            }
        }

        fix_nan(&mut self.raw.velocity, "velocity");
        fix_nan(&mut self.raw.origin, "origin");

        let max = self.movevars().maxvelocity;
        let min = -max;

        if self
            .raw
            .velocity
            .as_ref()
            .iter()
            .any(|&i| i < min || i > max)
        {
            debug!("PM  Got a velocity too high/low {:?}", self.raw.velocity);
            self.raw.velocity = self
                .raw
                .velocity
                .clamp(vec3_t::splat(min), vec3_t::splat(max));
        }
    }

    fn check_falling(&mut self) {
        if self.raw.onground != -1
            && !self.is_dead()
            && self.raw.flFallVelocity >= PLAYER_FALL_PUNCH_THRESHOLD
        {
            let mut vol = 0.5;

            if self.raw.waterlevel > 0 {
                // nop
            } else if self.raw.flFallVelocity > PLAYER_MAX_SAFE_FALL_SPEED {
                let s = c"player/pl_fallpain3.wav";
                self.play_sound(
                    Channel::Voice,
                    s,
                    1.0,
                    Attenuation::NORM,
                    SoundFlags::NONE,
                    Pitch::NORM,
                );
                vol = 1.0;
            } else if self.raw.flFallVelocity > PLAYER_MAX_SAFE_FALL_SPEED / 2.0 {
                if let Some(1) = self.info_value_for_key::<i32>(self.physinfo(), c"tfc") {
                    let s = c"player/pl_fallpain3.wav";
                    self.play_sound(
                        Channel::Voice,
                        s,
                        1.0,
                        Attenuation::NORM,
                        SoundFlags::NONE,
                        Pitch::NORM,
                    );
                }
                vol = 0.85;
            } else if self.raw.flFallVelocity < PLAYER_MIN_BOUNCE_SPEED {
                vol = 0.0;
            }

            if vol > 0.0 {
                self.raw.flTimeStepSound = 0;

                self.update_step_sound();
                self.play_step_sound(map_texture_type_step_type(self.raw.chtexturetype), vol);

                self.raw.punchangle[2] = self.raw.flFallVelocity * 0.013;
                if self.raw.punchangle[0] > 8.0 {
                    self.raw.punchangle[0] = 8.0;
                }
            }
        }

        if self.raw.onground != -1 {
            self.raw.flFallVelocity = 0.0;
        }
    }

    fn check_water(&mut self) -> qboolean {
        let usehull = self.raw.usehull as usize;
        let max = self.raw.player_maxs[usehull];
        let tmp = vec3_t::new(max[0] * 0.5, max[1] * 0.5, 1.0);
        let mut point = self.raw.origin + self.raw.player_mins[usehull] + tmp;

        self.raw.watertype = CONTENTS_EMPTY;
        self.raw.waterlevel = 0;

        let (cont, truecont) = self.point_contents(point);

        let is_water = |cont| cont > CONTENTS_TRANSLUCENT && cont <= CONTENTS_WATER;
        if is_water(cont) {
            self.raw.watertype = cont;
            self.raw.waterlevel = 1;

            let heightover2 = self.height() * 0.5;
            point[2] = self.raw.origin[2] + heightover2;
            let cont = self.point_contents(point).0;
            if is_water(cont) {
                self.raw.waterlevel = 2;

                point[2] = self.raw.origin[2] + self.raw.view_ofs[2];
                let cont = self.point_contents(point).0;
                if is_water(cont) {
                    self.raw.waterlevel = 3;
                }
            }

            if (CONTENTS_CURRENT_DOWN..=CONTENTS_CURRENT_0).contains(&truecont) {
                let current_table = [
                    vec3_t::X,
                    vec3_t::Y,
                    vec3_t::NEG_X,
                    vec3_t::NEG_Y,
                    vec3_t::Z,
                    vec3_t::NEG_Z,
                ];

                let index = (CONTENTS_CURRENT_0 - truecont) as usize;
                self.raw.basevelocity += current_table[index] * (self.raw.waterlevel as f32 * 50.0);
            }
        }

        (self.raw.waterlevel > 1).into()
    }

    fn check_water_jump(&mut self) {
        const WJ_HEIGHT: f32 = 8.0;

        if self.raw.waterjumptime != 0.0 {
            return;
        }

        if self.raw.velocity[2] < -180.0 {
            return;
        }

        let (flat_velocity, curspeed) = self.raw.velocity.with_z(0.0).normalize_and_length();
        let flat_forward = self.raw.forward.with_z(0.0).normalize();

        if curspeed != 0.0 && flat_velocity.dot(flat_forward) < 0.0 {
            return;
        }

        let savehull = self.usehull();
        self.raw.usehull = 2;
        let mut start = self.raw.origin.with_z(self.raw.origin[2] + WJ_HEIGHT);
        let end = start + flat_forward * 24.0;
        let trace = self.player_trace(start, end, PM_NORMAL, -1);
        if trace.fraction < 1.0 && fabsf(trace.plane.normal[2]) < 0.1 {
            self.raw.movedir = trace.plane.normal * -50.0;
            start[2] += self.raw.player_maxs[savehull][2] - WJ_HEIGHT;
            let end = start + flat_forward * 24.0;
            let trace = self.player_trace(start, end, PM_NORMAL, -1);
            if trace.fraction == 1.0 {
                self.raw.waterjumptime = 2000.0;
                self.raw.velocity[2] = 225.0;
                self.raw.oldbuttons |= IN_JUMP as c_int;
                self.flags_mut().insert(EdictFlags::WATERJUMP);
            }
        }
        self.raw.usehull = savehull as c_int;
    }

    fn check_stuck(&mut self) -> c_int {
        const PM_CHECKSTUCK_MINTIME: f32 = 0.05;

        static mut STUCK_CHECK_TIME: [[f32; 2]; MAX_CLIENTS] = [[0.0; 2]; MAX_CLIENTS];

        let (hitent, mut trace_result) = self.test_player_position(self.raw.origin);
        if hitent == -1 {
            self.reset_stuck_offsets();
            return 0;
        }

        let base = self.raw.origin;

        if (self.is_client() || self.is_singleplayer())
            && (hitent == 0 || !self.raw.physents[hitent as usize].model.is_null())
        {
            self.reset_stuck_offsets();
            for _ in 0..=54 {
                let (_, offset) = self.get_random_stuck_offsets();
                let test = base + offset;
                let (ent, tr) = self.test_player_position(test);
                trace_result = tr;
                if ent == -1 {
                    self.reset_stuck_offsets();
                    self.raw.origin = test;
                    return 0;
                }
            }
        }

        let idx = if self.is_server() { 0 } else { 1 };
        let time = self.system_time_f64();

        let player_index = self.raw.player_index as usize;
        if unsafe { STUCK_CHECK_TIME[player_index][idx] } >= time as f32 - PM_CHECKSTUCK_MINTIME {
            return 1;
        }
        unsafe {
            STUCK_CHECK_TIME[player_index][idx] = time as f32;
        }

        self.stuck_touch(hitent, &mut trace_result);

        let (index, offset) = self.get_random_stuck_offsets();
        let test = base + offset;
        let (hitent, _) = self.test_player_position(test);
        if hitent == -1 {
            self.reset_stuck_offsets();
            if index >= 27 {
                self.raw.origin = test;
            }
            return 0;
        }

        if self.is_button(IN_JUMP | IN_DUCK | IN_ATTACK)
            && self.raw.physents[hitent as usize].player != 0
        {
            let xystep = 8.0;
            let zstep = 18.0;
            let xyminmax = xystep;
            let zminmax = 4.0 * zstep;

            let mut z = 0.0;
            while z <= zminmax {
                let mut x = -xyminmax;
                while x <= xyminmax {
                    let mut y = -xyminmax;
                    while y <= xyminmax {
                        let test = base + vec3_t::new(x, y, z);
                        if self.test_player_position(test).0 == -1 {
                            self.raw.origin = test;
                            return 0;
                        }
                        y += xystep;
                    }
                    x += xystep;
                }
                z += zstep;
            }
        }

        1
    }

    fn friction(&mut self) {
        if self.raw.waterjumptime != 0.0 {
            return;
        }

        let speed = self.raw.velocity.length();
        if speed < 0.1 {
            return;
        }

        let mut drop = 0.0;

        if self.raw.onground != -1 {
            let mut start = self.raw.origin + self.raw.velocity / speed * 16.0;
            let mut stop = start;
            start[2] = self.raw.origin[2] + self.raw.player_mins[self.raw.usehull as usize][2];
            stop[2] = start[2] - 34.0;

            let trace = self.player_trace(start, stop, PM_NORMAL, -1);
            let mut friction = self.movevars().friction;
            if trace.fraction == 1.0 {
                friction *= self.movevars().edgefriction;
            }
            friction *= self.raw.friction;

            let contol = fmaxf(speed, self.movevars().stopspeed);
            drop += contol * friction * self.raw.frametime;
        }

        self.raw.velocity *= fmaxf(speed - drop, 0.0) / speed;
    }

    fn accelerate(&mut self, wishdir: &vec3_t, wishspeed: f32, accel: f32) {
        if self.is_dead() || self.raw.waterjumptime != 0.0 {
            return;
        }

        let current_speed = self.raw.velocity.dot(*wishdir);
        let add_speed = wishspeed - current_speed;
        if add_speed > 0.0 {
            let accel_speed = accel * self.raw.frametime * wishspeed * self.raw.friction;
            self.raw.velocity += *wishdir * fminf(accel_speed, add_speed);
        }
    }

    fn air_accelerate(&mut self, wishdir: &vec3_t, wishspeed: f32, accel: f32) {
        if self.is_dead() || self.raw.waterjumptime != 0.0 {
            return;
        }
        let wishspd = fminf(wishspeed, 30.0);
        let current_speed = self.raw.velocity.dot(*wishdir);
        let add_speed = wishspd - current_speed;
        if add_speed <= 0.0 {
            return;
        }
        let new = accel * wishspeed * self.raw.frametime * self.raw.friction;
        let accel_speed = fminf(add_speed, new);
        self.raw.velocity += *wishdir * accel_speed;
    }

    fn catagorize_position(&mut self) {
        self.check_water();

        let mut point = self.raw.origin;
        point[2] -= 2.0;

        if self.raw.velocity[2] > 180.0 {
            self.raw.onground = -1;
        } else {
            let trace = self.player_trace(self.raw.origin, point, PM_NORMAL, -1);

            if trace.plane.normal[2] < 0.7 {
                self.raw.onground = -1;
            } else {
                self.raw.onground = trace.ent;
            }

            if self.raw.onground != -1 {
                self.raw.waterjumptime = 0.0;
                if self.raw.waterlevel < 2 && trace.startsolid == 0 && trace.allsolid == 0 {
                    self.raw.origin = trace.endpos;
                }
            }

            if trace.ent > 0 {
                self.add_to_touched(trace, self.raw.velocity);
            }
        }
    }

    fn catagorize_texture_type(&mut self) {
        let start = self.raw.origin;
        let end = self.raw.origin - vec3_t::new(0.0, 0.0, 64.0);
        let name = match self.trace_texture(self.raw.onground != 0, start, end) {
            Some(s) => s.to_bytes(),
            None => {
                self.texture_name_clear();
                self.raw.chtexturetype = CHAR_TEX_CONCRETE;
                return;
            }
        };

        let name = strip_texture_prefix(name);
        self.texture_name_clear()
            .cursor()
            .write_bytes(name)
            .unwrap();
        self.raw.chtexturetype = find_texture_type(self.texture_name());
    }

    fn fix_player_crouch_stuck(&mut self, direction: c_int) {
        let mut origin = self.raw.origin;
        for _ in 0..37 {
            let (hitent, _) = self.test_player_position(origin);
            if hitent != -1 {
                self.raw.origin = origin;
                return;
            }
            origin[2] += direction as f32;
        }
    }

    fn add_gravity(&mut self) {
        let ent_gravity = if self.raw.gravity != 0.0 {
            self.raw.gravity
        } else {
            1.0
        };
        self.raw.velocity[2] -= ent_gravity * self.movevars().gravity * self.raw.frametime;
        self.raw.velocity[2] += self.raw.basevelocity[2] * self.raw.frametime;
        self.raw.basevelocity[2] = 0.0;
        self.check_velocity();
    }

    fn add_correct_gravity(&mut self) {
        if self.raw.waterjumptime != 0.0 {
            return;
        }

        let ent_gravity = if self.raw.gravity != 0.0 {
            self.raw.gravity
        } else {
            1.0
        };

        self.raw.velocity[2] -= ent_gravity * self.movevars().gravity * self.raw.frametime * 0.5;
        self.raw.velocity[2] += self.raw.basevelocity[2] * self.raw.frametime;
        self.raw.basevelocity[2] = 0.0;

        self.check_velocity();
    }

    fn fixup_gravity_velocity(&mut self) {
        if self.raw.waterjumptime != 0.0 {
            return;
        }

        let ent_gravity = if self.raw.gravity != 0.0 {
            self.raw.gravity
        } else {
            1.0
        };

        self.raw.velocity[2] -= ent_gravity * self.movevars().gravity * self.raw.frametime * 0.5;

        self.check_velocity();
    }

    fn play_step_sound(&mut self, step: c_int, vol: f32) {
        self.raw.iStepLeft = (self.raw.iStepLeft == 0) as c_int;

        if self.raw.runfuncs == 0 {
            return;
        } else if self.is_multiplayer() {
            if self.movevars().footsteps == 0 {
                return;
            }
            let vel = vec3_t::new(self.raw.velocity[0], self.raw.velocity[1], 0.0);
            if self.ladder && vel.length() <= 220.0 {
                return;
            }
        }

        let play = |i, samples: &[&CStr]| {
            self.play_sound(
                Channel::Body,
                samples[i as usize],
                vol,
                Attenuation::NORM,
                SoundFlags::NONE,
                Pitch::NORM,
            );
        };
        let rand = self.random_int(0, 1) + self.raw.iStepLeft * 2;

        match step {
            STEP_CONCRETE => {
                let samples = &[
                    c"player/pl_step1.wav",
                    c"player/pl_step3.wav",
                    c"player/pl_step2.wav",
                    c"player/pl_step4.wav",
                ];
                play(rand, samples);
            }
            STEP_METAL => {
                let samples = &[
                    c"player/pl_metal1.wav",
                    c"player/pl_metal3.wav",
                    c"player/pl_metal2.wav",
                    c"player/pl_metal4.wav",
                ];
                play(rand, samples);
            }
            STEP_DIRT => {
                let samples = &[
                    c"player/pl_dirt1.wav",
                    c"player/pl_dirt3.wav",
                    c"player/pl_dirt2.wav",
                    c"player/pl_dirt4.wav",
                ];
                play(rand, samples);
            }
            STEP_VENT => {
                let samples = &[
                    c"player/pl_duct1.wav",
                    c"player/pl_duct3.wav",
                    c"player/pl_duct2.wav",
                    c"player/pl_duct4.wav",
                ];
                play(rand, samples);
            }
            STEP_GRATE => {
                let samples = &[
                    c"player/pl_grate1.wav",
                    c"player/pl_grate3.wav",
                    c"player/pl_grate2.wav",
                    c"player/pl_grate4.wav",
                ];
                play(rand, samples);
            }
            STEP_TILE => {
                let rand = if self.random_int(0, 4) != 0 { 4 } else { rand };
                let samples = &[
                    c"player/pl_tile1.wav",
                    c"player/pl_tile3.wav",
                    c"player/pl_tile2.wav",
                    c"player/pl_tile4.wav",
                    c"player/pl_tile5.wav",
                ];
                play(rand, samples);
            }
            STEP_SLOSH => {
                let samples = &[
                    c"player/pl_slosh1.wav",
                    c"player/pl_slosh3.wav",
                    c"player/pl_slosh2.wav",
                    c"player/pl_slosh4.wav",
                ];
                play(rand, samples);
            }
            STEP_WADE => {
                // FIXME: shared global state
                static SKIP_STEP: AtomicU8 = AtomicU8::new(0);
                match SKIP_STEP.fetch_add(1, Ordering::Relaxed) {
                    0 => {}
                    n => {
                        if n == 3 {
                            SKIP_STEP.store(0, Ordering::Relaxed);
                        }
                        let samples = &[
                            c"player/pl_wade1.wav",
                            c"player/pl_wade3.wav",
                            c"player/pl_wade2.wav",
                            c"player/pl_wade4.wav",
                        ];
                        play(rand, samples);
                    }
                }
            }
            STEP_LADDER => {
                let samples = &[
                    c"player/pl_ladder1.wav",
                    c"player/pl_ladder3.wav",
                    c"player/pl_ladder2.wav",
                    c"player/pl_ladder4.wav",
                ];
                play(rand, samples);
            }
            _ => warn!("unimplemented play sound step({step})"),
        }
    }

    fn play_water_sounds(&mut self) {
        if (self.raw.oldwaterlevel == 0 && self.raw.waterlevel != 0)
            || (self.raw.oldwaterlevel != 0 && self.raw.waterlevel == 0)
        {
            let samples = [
                c"player/pl_wade1.wav",
                c"player/pl_wade2.wav",
                c"player/pl_wade3.wav",
                c"player/pl_wade4.wav",
            ];
            let s = samples[self.random_int(0, 3) as usize];
            self.play_sound(
                Channel::Body,
                s,
                1.0,
                Attenuation::NORM,
                SoundFlags::NONE,
                Pitch::NORM,
            );
        }
    }

    fn update_step_sound(&mut self) {
        if self.raw.flTimeStepSound > 0 || self.flags().contains(EdictFlags::FROZEN) {
            return;
        }

        self.catagorize_texture_type();

        let speed = self.raw.velocity.length();
        let ladder = self.raw.movetype == MoveType::Fly as c_int
            && !self.flags().contains(EdictFlags::IMMUNE_LAVA);
        let (velwalk, velrun, flduck) = if self.flags().contains(EdictFlags::DUCKING) || ladder {
            (60.0, 80.0, 100)
        } else {
            (120.0, 210.0, 0)
        };

        if !(ladder || self.raw.onground != -1)
            || speed <= 0.0
            || !(speed >= velwalk || self.raw.flTimeStepSound != 0)
        {
            return;
        }

        let walking = speed < velrun;
        let usehull = self.raw.usehull as usize;
        let height = self.raw.player_maxs[usehull][2] - self.raw.player_mins[usehull][2];
        let knee = self.raw.origin - vec3_t::new(0.0, 0.0, 0.3 * height);
        let feet = self.raw.origin - vec3_t::new(0.0, 0.0, 0.5 * height);
        let volume = |walk, run| if walking { walk } else { run };
        let time = |walk, run| if walking { walk } else { run };
        let (step, mut vol, time_step_sound) = if ladder {
            (STEP_LADDER, 0.35, 350)
        } else if let (CONTENTS_WATER, _) = self.point_contents(knee) {
            (STEP_WADE, 0.65, 600)
        } else if let (CONTENTS_WATER, _) = self.point_contents(feet) {
            (STEP_SLOSH, volume(0.2, 0.5), time(400, 300))
        } else {
            let (vol, time) = match self.raw.chtexturetype {
                CHAR_TEX_METAL => (volume(0.2, 0.5), time(400, 300)),
                CHAR_TEX_DIRT => (volume(0.25, 0.55), time(400, 300)),
                CHAR_TEX_VENT => (volume(0.4, 0.7), time(400, 300)),
                CHAR_TEX_GRATE => (volume(0.2, 0.5), time(400, 300)),
                CHAR_TEX_TILE => (volume(0.2, 0.5), time(400, 300)),
                CHAR_TEX_SLOSH => (volume(0.2, 0.5), time(400, 300)),
                _ => (volume(0.2, 0.5), time(400, 300)),
            };
            (
                map_texture_type_step_type(self.raw.chtexturetype),
                vol,
                time,
            )
        };
        self.raw.flTimeStepSound = time_step_sound + flduck;

        if self.flags().contains(EdictFlags::DUCKING) {
            vol *= 0.35;
        }

        self.play_step_sound(step, vol);
    }

    // TODO: client only
    // #[no_mangle]
    // static mut iJumpSpectator: c_int = 0;
    //
    // #[no_mangle]
    // static mut vJumpOrigin: vec3_t = vec3_t::ZERO;
    //
    // #[no_mangle]
    // static mut vJumpAngles: vec3_t = vec3_t::ZERO;

    fn normalize_angle_vectors(&mut self) {
        self.raw.forward = self.raw.forward.normalize();
        self.raw.right = self.raw.right.normalize();
    }

    fn normalize_angle_vectors_no_z(&mut self) {
        self.raw.forward[2] = 0.0;
        self.raw.right[2] = 0.0;
        self.normalize_angle_vectors();
    }

    fn wish_vel(&self) -> vec3_t {
        let mut vel = self.raw.forward * self.raw.cmd.forwardmove;
        vel += self.raw.right * self.raw.cmd.sidemove;
        vel[2] += self.raw.cmd.upmove;
        vel
    }

    fn wish_move(&self, vel: vec3_t) -> WishMove {
        WishMove::new(vel, self.raw.maxspeed)
    }

    fn spectator_move(&mut self) {
        if self.raw.iuser1 == OBS_ROAMING {
            // TODO: client only
            // unsafe {
            //     if iJumpSpectator != 0 {
            //         self.raw.origin = vJumpOrigin;
            //         self.raw.angles = vJumpAngles;
            //         self.raw.velocity = vec3_t::ZERO;
            //         iJumpSpectator = 0;
            //         return;
            //     }
            // }

            let speed = self.raw.velocity.length();
            if speed < 1.0 {
                self.raw.velocity = vec3_t::ZERO;
            } else {
                let friction = self.movevars().friction * 1.5;
                let control = fminf(speed, self.movevars().stopspeed);
                let drop = control * friction * self.raw.frametime;
                self.raw.velocity *= fmaxf(0.0, speed - drop) / speed;
            }

            self.normalize_angle_vectors();
            let wish = WishMove::new(self.wish_vel(), self.movevars().spectatormaxspeed);

            let currentspeed = self.raw.velocity.dot(wish.dir);
            let addspeed = wish.speed - currentspeed;
            if addspeed <= 0.0 {
                return;
            }
            let accelspeed = fminf(
                addspeed,
                self.movevars().accelerate * self.raw.frametime * wish.speed,
            );
            self.raw.velocity += wish.dir * accelspeed;
            self.raw.origin += self.raw.velocity * self.raw.frametime;
        } else {
            if self.raw.iuser2 <= 0 {
                return;
            }

            if let Some(i) = self
                .physents()
                .iter()
                .position(|i| i.info == self.raw.iuser2)
            {
                let target = &self.raw.physents[i];
                self.raw.origin = target.origin;
                self.raw.angles = target.angles;
                self.raw.velocity = vec3_t::ZERO;
            }
        }
    }

    fn duck(&mut self) {
        use EdictFlags as Flags;

        let buttons_changed = self.raw.oldbuttons ^ self.raw.cmd.buttons as c_int;
        let button_pressed = buttons_changed & self.raw.cmd.buttons as c_int;

        if self.is_button(IN_DUCK) {
            self.raw.oldbuttons |= IN_DUCK as c_int;
        } else {
            self.raw.oldbuttons &= !IN_DUCK as c_int;
        }

        if self.raw.iuser3 != 0 || self.is_dead() {
            if self.flags().contains(Flags::DUCKING) {
                self.un_duck();
            }
            return;
        }

        if self.flags().contains(Flags::DUCKING) {
            self.raw.cmd.forwardmove *= PLAYER_DUCKING_MULTIPLIER;
            self.raw.cmd.sidemove *= PLAYER_DUCKING_MULTIPLIER;
            self.raw.cmd.upmove *= PLAYER_DUCKING_MULTIPLIER;
        }

        if self.is_button(IN_DUCK) {
            if button_pressed & IN_DUCK as c_int != 0 && !self.flags().contains(Flags::DUCKING) {
                self.raw.flDuckTime = 1000.0;
                self.raw.bInDuck = true.into();
            }

            if self.raw.bInDuck != 0 {
                if self.raw.flDuckTime / 1000.0 <= 1.0 - TIME_TO_DUCK || self.raw.onground == -1 {
                    self.raw.usehull = 1;
                    self.raw.view_ofs[2] = VEC_DUCK_VIEW;
                    self.flags_mut().insert(Flags::DUCKING);
                    self.raw.bInDuck = false.into();

                    if self.raw.onground != -1 {
                        self.raw.origin -= self.raw.player_mins[1] - self.raw.player_mins[0];
                        self.fix_player_crouch_stuck(STUCK_MOVEUP);
                        self.catagorize_position();
                    }
                } else {
                    let more = VEC_DUCK_HULL_MIN - VEC_HULL_MIN;
                    let time = fmaxf(0.0, 1.0 - self.raw.flDuckTime / 1000.0);
                    let duck_fraction = spline_fraction(time, 1.0 / TIME_TO_DUCK);
                    self.raw.view_ofs[2] =
                        (VEC_DUCK_VIEW - more) * duck_fraction + VEC_VIEW_Z * (1.0 - duck_fraction);
                }
            }
        } else if self.raw.bInDuck != 0 || self.flags().contains(Flags::DUCKING) {
            self.un_duck();
        }
    }

    fn un_duck(&mut self) {
        let mut new_origin = self.raw.origin;
        if self.raw.onground != -1 {
            new_origin += self.raw.player_mins[1] - self.raw.player_mins[0];
        }

        let trace = self.player_trace(new_origin, new_origin, PM_NORMAL, -1);
        if trace.startsolid == 0 {
            self.raw.usehull = 0;
            let trace = self.player_trace(new_origin, new_origin, PM_NORMAL, -1);
            if trace.startsolid != 0 {
                self.raw.usehull = 1;
                return;
            }

            self.flags_mut().remove(EdictFlags::DUCKING);
            self.raw.bInDuck = false.into();
            self.raw.view_ofs[2] = VEC_VIEW_Z;
            self.raw.flDuckTime = 0.0;
            self.raw.origin = new_origin;

            self.catagorize_position();
        }
    }

    fn ladder_move(&mut self, ladder: &physent_s) {
        use EdictFlags as Flags;

        if self.raw.movetype == MoveType::NoClip as c_int {
            return;
        }

        self.raw.movetype = MoveType::Fly as c_int;
        self.raw.gravity = 0.0;

        let (model_mins, model_maxs) = self.get_model_bounds(unsafe { &*ladder.model });
        let ladder_center = (model_mins + model_maxs) * 0.5;
        let trace = self.trace_model(ladder, self.raw.origin, ladder_center).0;
        if trace.fraction == 1.0 {
            return;
        } else if self.is_button(IN_JUMP) {
            self.raw.movetype = MoveType::Walk as c_int;
            self.raw.velocity = trace.plane.normal * 270.0;
            return;
        }

        let mut speed = fminf(self.raw.maxspeed, MAX_CLIMB_SPEED);
        if self.flags().contains(Flags::DUCKING) {
            speed *= PLAYER_DUCKING_MULTIPLIER;
        }

        let mut forward = 0.0;
        let mut side = 0.0;
        if self.is_button(IN_BACK) {
            forward -= speed;
        }
        if self.is_button(IN_FORWARD) {
            forward += speed;
        }
        if self.is_button(IN_MOVELEFT) {
            side -= speed;
        }
        if self.is_button(IN_MOVERIGHT) {
            side += speed;
        }

        if forward != 0.0 || side != 0.0 {
            let av = self.raw.angles.angle_vectors();
            let velocity = av.forward() * forward + av.right() * side;
            let perp = vec3_t::Z.cross(trace.plane.normal).normalize();
            let normal = velocity.dot(trace.plane.normal);
            let lateral = velocity - trace.plane.normal * normal;
            self.raw.velocity = lateral + trace.plane.normal.cross(perp) * -normal;

            let mut floor = self.raw.origin;
            floor[2] += self.raw.player_mins[self.usehull()][2] - 1.0;
            let on_floor = self.point_contents(floor).0 == CONTENTS_SOLID;
            if on_floor && normal > 0.0 {
                self.raw.velocity += trace.plane.normal * MAX_CLIMB_SPEED;
            }
        } else {
            self.raw.velocity = vec3_t::ZERO;
        }
    }

    fn no_clip(&mut self) {
        self.normalize_angle_vectors();
        self.raw.origin += self.wish_vel() * self.raw.frametime;
        self.raw.velocity = vec3_t::ZERO;
    }

    fn physics_toss(&mut self) {
        self.check_water();

        if self.raw.velocity[2] > 0.0 {
            self.raw.onground = -1;
        } else if self.raw.onground != -1
            && self.raw.basevelocity == vec3_t::ZERO
            && self.raw.velocity == vec3_t::ZERO
        {
            return;
        }

        self.check_velocity();

        if MoveType::from_raw(self.raw.movetype).is_some_and(|t| !t.is_flying()) {
            self.add_gravity();
        }

        self.raw.velocity += self.raw.basevelocity;
        self.check_velocity();
        let move_ = self.raw.velocity * self.raw.frametime;
        self.raw.velocity -= self.raw.basevelocity;

        let trace = self.push_entity(&move_);
        self.check_velocity();

        if trace.allsolid != 0 {
            self.raw.onground = trace.ent;
            self.raw.velocity = vec3_t::ZERO;
            return;
        }

        if trace.fraction == 1.0 {
            self.check_water();
            return;
        }

        let backoff = match MoveType::from_raw(self.raw.movetype) {
            Some(MoveType::Bounce) => 2.0 - self.raw.friction,
            Some(MoveType::BounceMissile) => 2.0,
            _ => 1.0,
        };

        self.raw.velocity = clip_velocity(self.raw.velocity, trace.plane.normal, backoff).1;

        if trace.plane.normal[2] > 0.7 {
            if self.raw.velocity[2] < self.movevars().gravity * self.raw.frametime {
                self.raw.onground = trace.ent;
                self.raw.velocity[2] = 0.0;
            }

            let vel = self.raw.velocity.dot(self.raw.velocity);
            if vel < (30.0 * 30.0)
                || (self.raw.movetype != MoveType::Bounce as c_int
                    && self.raw.movetype != MoveType::BounceMissile as c_int)
            {
                self.raw.onground = trace.ent;
                self.raw.velocity = vec3_t::ZERO;
            } else {
                let move_ = self.raw.velocity * (1.0 - trace.fraction) * self.raw.frametime * 0.9;
                self.push_entity(&move_);
            }
        }

        self.check_water();
    }

    fn fly_move(&mut self) -> c_int {
        let mut planes = [vec3_t::ZERO; MAX_CLIP_PLANES];
        let mut blocked = 0;
        let mut numplanes = 0;
        let mut original_velocity = self.raw.velocity;
        let primal_velocity = self.raw.velocity;
        let mut all_fraction = 0.0;
        let mut time_left = self.raw.frametime;

        for _ in 0..4 {
            if self.raw.velocity == vec3_t::ZERO {
                break;
            }

            let end = self.raw.origin + self.raw.velocity * time_left;
            let trace = self.player_trace(self.raw.origin, end, PM_NORMAL, -1);
            all_fraction += trace.fraction;
            if trace.allsolid != 0 {
                self.raw.velocity = vec3_t::ZERO;
                return 4;
            }

            if trace.fraction > 0.0 {
                self.raw.origin = trace.endpos;
                original_velocity = self.raw.velocity;
                numplanes = 0;
            } else if trace.fraction == 1.0 {
                break;
            }

            self.add_to_touched(trace, self.raw.velocity);

            if trace.plane.normal[2] > 0.7 {
                blocked |= 1; // floor
            } else if trace.plane.normal[2] == 0.0 {
                blocked |= 2; // wall/step
            }

            time_left -= time_left * trace.fraction;

            if numplanes >= MAX_CLIP_PLANES {
                self.raw.velocity = vec3_t::ZERO;
                break;
            }

            planes[numplanes] = trace.plane.normal;
            numplanes += 1;

            if numplanes == 1
                && self.raw.movetype == MoveType::Walk as c_int
                && (self.raw.onground == -1 || self.raw.friction != 1.0)
            {
                for plane in &planes[..numplanes] {
                    let mut overbounce = 1.0;
                    if plane[2] <= 0.7 {
                        overbounce += self.movevars().bounce * (1.0 - self.raw.friction);
                    };
                    let new_velocity = clip_velocity(original_velocity, *plane, overbounce).1;
                    self.raw.velocity = new_velocity;
                    original_velocity = new_velocity;
                }
            } else {
                let mut i = 0;
                while i < numplanes {
                    self.raw.velocity = clip_velocity(original_velocity, planes[i], 1.0).1;
                    let mut j = 0;
                    while j < numplanes {
                        if j != i && self.raw.velocity.dot(planes[j]) < 0.0 {
                            break;
                        }
                        j += 1;
                    }
                    if j == numplanes {
                        break;
                    }
                    i += 1;
                }

                if i == numplanes {
                    if numplanes != 2 {
                        self.raw.velocity = vec3_t::ZERO;
                        break;
                    }

                    let dir = planes[0].cross(planes[1]);
                    let d = dir.dot(self.raw.velocity);
                    self.raw.velocity = dir * d;
                }

                if self.raw.velocity.dot(primal_velocity) <= 0.0 {
                    self.raw.velocity = vec3_t::ZERO;
                    break;
                }
            }
        }

        if all_fraction == 0.0 {
            self.raw.velocity = vec3_t::ZERO;
        }

        blocked
    }

    fn water_jump(&mut self) {
        if self.raw.waterjumptime == 0.0 {
            return;
        }

        self.raw.waterjumptime = fminf(self.raw.waterjumptime, 10000.0);
        self.raw.waterjumptime -= self.raw.cmd.msec as f32;
        if self.raw.waterjumptime < 0.0 || self.raw.waterlevel == 0 {
            self.raw.waterjumptime = 0.0;
            self.flags_mut().remove(EdictFlags::WATERJUMP);
        }

        self.raw.velocity[0] = self.raw.movedir[0];
        self.raw.velocity[1] = self.raw.movedir[1];
    }

    fn prevent_mega_bunny_jumping(&mut self) {
        const BUNNYJUMP_MAX_SPEED_FACTOR: f32 = 1.7;

        let max_scaled_speed = BUNNYJUMP_MAX_SPEED_FACTOR * self.raw.maxspeed;
        if max_scaled_speed > 0.0 {
            let speed = self.raw.velocity.length();
            if speed > max_scaled_speed {
                self.raw.velocity *= max_scaled_speed / speed * 0.65;
            }
        }
    }

    fn jump(&mut self) {
        if self.is_dead() {
            self.raw.oldbuttons |= IN_JUMP as c_int;
            return;
        }

        let tfc = self.info_value_for_key(self.physinfo(), c"tfc");
        let tfc = tfc == Some(1_i32);
        if tfc && self.raw.deadflag == DEAD_DISCARDBODY + 1 {
            return;
        }

        if self.raw.waterjumptime != 0.0 {
            self.raw.waterjumptime = fmaxf(0.0, self.raw.waterjumptime - self.raw.cmd.msec as f32);
            return;
        }

        if self.raw.waterlevel >= 2 {
            self.raw.onground = -1;

            self.raw.velocity[2] = match self.raw.watertype {
                CONTENTS_WATER => 100.0,
                CONTENTS_SLIME => 80.0,
                CONTENTS_LAVA => 50.0,
                _ => unreachable!(),
            };

            if self.raw.flSwimTime <= 0.0 {
                self.raw.flSwimTime = 1000.0;
                let samples = [
                    c"player/pl_wade1.wav",
                    c"player/pl_wade2.wav",
                    c"player/pl_wade3.wav",
                    c"player/pl_wade4.wav",
                ];
                let sample = samples[self.random_int(0, 3) as usize];
                self.play_sound(
                    Channel::Body,
                    sample,
                    1.0,
                    Attenuation::NORM,
                    SoundFlags::NONE,
                    Pitch::NORM,
                );
            }

            return;
        }

        if self.raw.onground == -1 {
            self.raw.oldbuttons |= IN_JUMP as c_int;
            return;
        }

        if self.raw.oldbuttons & IN_JUMP as c_int != 0 {
            return;
        }

        self.raw.onground = -1;

        self.prevent_mega_bunny_jumping();

        if tfc {
            self.play_sound(
                Channel::Body,
                c"player/plyrjmp8.wav",
                0.5,
                Attenuation::NORM,
                SoundFlags::NONE,
                Pitch::NORM,
            );
        } else {
            self.play_step_sound(map_texture_type_step_type(self.raw.chtexturetype), 1.0);
        }

        let cansuperjump = self.info_value_for_key::<i32>(self.physinfo(), c"slj");
        if self.raw.bInDuck != 0 || self.flags().contains(EdictFlags::DUCKING) {
            if cansuperjump == Some(1)
                && self.is_button(IN_DUCK)
                && self.raw.flDuckTime > 0.0
                && self.raw.velocity.length() > 50.0
            {
                self.raw.punchangle[0] = -5.0;
                self.raw.velocity = self.raw.forward * PLAYER_LONGJUMP_SPEED * 1.6;
                self.raw.velocity[2] = sqrtf(2.0 * 800.0 * 56.0);
            } else {
                self.raw.velocity[2] = sqrtf(2.0 * 800.0 * 45.0);
            }
        } else {
            self.raw.velocity[2] = sqrtf(2.0 * 800.0 * 45.0);
        }

        self.fixup_gravity_velocity();

        self.raw.oldbuttons |= IN_JUMP as c_int;
    }

    fn walk_move(&mut self) {
        self.normalize_angle_vectors_no_z();
        let wish = self.wish_move(self.wish_vel().with_z(0.0));

        self.raw.velocity[2] = 0.0;
        self.accelerate(&wish.dir, wish.speed, self.movevars().accelerate);
        self.raw.velocity[2] = 0.0;
        self.raw.velocity += self.raw.basevelocity;

        let speed = self.raw.velocity.length();
        if speed < 1.0 {
            self.raw.velocity = vec3_t::ZERO;
            return;
        }

        let oldonground = self.raw.onground;
        let dest = self.raw.origin
            + vec3_t::new(
                self.raw.velocity[0] * self.raw.frametime,
                self.raw.velocity[1] * self.raw.frametime,
                0.0,
            );
        let trace = self.player_trace(self.raw.origin, dest, PM_NORMAL, -1);
        if trace.fraction == 1.0 {
            self.raw.origin = trace.endpos;
            return;
        } else if (oldonground == -1 && self.raw.waterlevel == 0) || self.raw.waterjumptime != 0.0 {
            return;
        }

        let original = self.raw.origin;
        let originalvel = self.raw.velocity;

        self.fly_move();

        let down = self.raw.origin;
        let downvel = self.raw.velocity;

        self.raw.origin = original;
        self.raw.velocity = originalvel;

        let mut dest = self.raw.origin;
        dest[2] += self.movevars().stepsize;

        let trace = self.player_trace(self.raw.origin, dest, PM_NORMAL, -1);
        if trace.startsolid == 0 && trace.allsolid == 0 {
            self.raw.origin = trace.endpos;
        }

        self.fly_move();

        let mut dest = self.raw.origin;
        dest[2] -= self.movevars().stepsize;

        let trace = self.player_trace(self.raw.origin, dest, PM_NORMAL, -1);
        if trace.plane.normal[2] < 0.7 {
            self.raw.origin = down;
            self.raw.velocity = downvel;
            return;
        } else if trace.startsolid == 0 && trace.allsolid == 0 {
            self.raw.origin = trace.endpos;
        }
        self.raw.up = self.raw.origin;

        let downdist = pow2(down[0] - original[0]) + pow2(down[1] - original[1]);
        let updist = pow2(self.raw.up[0] - original[0]) + pow2(self.raw.up[1] - original[1]);

        if downdist > updist {
            self.raw.origin = down;
            self.raw.velocity = downvel;
        } else {
            self.raw.velocity[2] = downvel[2];
        }
    }

    fn air_move(&mut self) {
        self.normalize_angle_vectors_no_z();
        let wish = self.wish_move(self.wish_vel().with_z(0.0));
        self.air_accelerate(&wish.dir, wish.speed, self.movevars().airaccelerate);
        self.raw.velocity += self.raw.basevelocity;
        self.fly_move();
    }

    fn water_move(&mut self) {
        let mut wish = {
            let wishvel = if self.move_vector() != vec3_t::ZERO {
                self.wish_vel()
            } else {
                vec3_t::new(0.0, 0.0, -60.0)
            };
            self.wish_move(wishvel)
        };
        wish.speed *= 0.8;

        self.raw.velocity += self.raw.basevelocity;
        let mut newspeed = 0.0;
        let speed = self.raw.velocity.length();
        if speed != 0.0 {
            newspeed =
                speed - self.raw.frametime * speed * self.movevars().friction * self.raw.friction;
            if newspeed < 0.0 {
                newspeed = 0.0;
            }
            self.raw.velocity *= newspeed / speed;
        }

        if wish.speed < 0.1 {
            return;
        }

        let addspeed = wish.speed - newspeed;
        if addspeed > 0.0 {
            wish.vel = wish.vel.normalize();
            let accelspeed =
                self.movevars().accelerate * wish.speed * self.raw.frametime * self.raw.friction;
            self.raw.velocity += wish.vel * fminf(accelspeed, addspeed);
        }

        let end = self.raw.origin + self.raw.velocity * self.raw.frametime;
        let mut start = end;
        start[2] += self.movevars().stepsize + 1.0;
        let trace = self.player_trace(start, end, PM_NORMAL, -1);
        if trace.startsolid == 0 && trace.allsolid == 0 {
            self.raw.origin = trace.endpos;
            return;
        }

        self.fly_move();
    }

    fn player_move(&mut self, is_server: bool) {
        self.raw.server = is_server.into();
        self.check_paramters();
        self.raw.numtouch = 0;
        self.raw.frametime = self.raw.cmd.msec as f32 * 0.001;

        self.reduce_timers();

        let av = self.raw.angles.angle_vectors().all();
        self.raw.forward = av.forward;
        self.raw.right = av.right;
        self.raw.up = av.up;

        if cfg!(feature = "debug") && self.is_client() {
            self.show_clip_box();
        }

        if self.is_spectator() || self.raw.iuser1 > 0 {
            self.spectator_move();
            self.catagorize_position();
            return;
        }

        if self.raw.movetype != MoveType::NoClip as c_int
            && self.raw.movetype != MoveType::None as c_int
            && self.check_stuck() != 0
        {
            self.duck();
            if self.check_stuck() != 0 {
                return;
            }
        }

        self.catagorize_position();

        self.raw.oldwaterlevel = self.raw.waterlevel;

        if self.raw.onground == -1 {
            self.raw.flFallVelocity = -self.raw.velocity[2];
        }

        let ladder;
        if self.is_alive() && !self.flags().contains(EdictFlags::ONTRAIN) {
            ladder = self.ladder();
            if !ladder.is_null() {
                self.ladder = true;
            }
        } else {
            ladder = ptr::null();
        }

        self.update_step_sound();

        self.duck();

        if self.is_alive() && !self.flags().contains(EdictFlags::ONTRAIN) {
            if !ladder.is_null() {
                self.ladder_move(unsafe { &*ladder });
            } else if self.raw.movetype != MoveType::Walk as c_int
                && self.raw.movetype != MoveType::NoClip as c_int
            {
                self.raw.movetype = MoveType::Walk as c_int;
            }
        }

        match self.movetype() {
            MoveType::None => {}
            MoveType::NoClip => self.no_clip(),
            MoveType::Toss => self.physics_toss(),
            MoveType::Bounce => self.physics_toss(),
            MoveType::Fly => {
                self.check_water();

                if self.is_button(IN_JUMP) {
                    if ladder.is_null() {
                        self.jump();
                    }
                } else {
                    self.raw.oldbuttons &= !IN_JUMP as c_int;
                }

                self.raw.velocity += self.raw.basevelocity;
                self.fly_move();
                self.raw.velocity -= self.raw.basevelocity;
            }
            MoveType::Walk => {
                if !self.in_water() {
                    self.add_correct_gravity();
                }

                if self.raw.waterjumptime != 0.0 {
                    self.water_jump();
                    self.fly_move();

                    self.check_water();
                    return;
                }

                if self.raw.waterlevel >= 2 {
                    if self.raw.waterlevel == 2 {
                        self.check_water_jump();
                    }

                    if self.raw.velocity[2] < 0.0 && self.raw.waterjumptime != 0.0 {
                        self.raw.waterjumptime = 0.0;
                    }

                    if self.is_button(IN_JUMP) {
                        self.jump();
                    } else {
                        self.raw.oldbuttons &= !IN_JUMP as c_int;
                    }

                    self.water_move();
                    self.raw.velocity -= self.raw.basevelocity;

                    self.catagorize_position();
                } else {
                    if self.is_button(IN_JUMP) {
                        if ladder.is_null() {
                            self.jump();
                        }
                    } else {
                        self.raw.oldbuttons &= !IN_JUMP as c_int;
                    }

                    if self.raw.onground != -1 {
                        self.raw.velocity[2] = 0.0;
                        self.friction();
                    }

                    self.check_velocity();

                    if self.raw.onground != -1 {
                        self.walk_move();
                    } else {
                        self.air_move();
                    }

                    self.catagorize_position();

                    self.raw.velocity -= self.raw.basevelocity;

                    self.check_velocity();

                    if !self.in_water() {
                        self.fixup_gravity_velocity();
                    }

                    if self.raw.onground != -1 {
                        self.raw.velocity[2] = 0.0;
                    }

                    self.check_falling();
                }

                self.play_water_sounds();
            }
            _ => {
                let s = ["client", "server"][is_server as usize];
                error!("invalid player move type {:?} on {s}", self.raw.movetype);
            }
        }
    }
}

pub fn get_hull_bounds(hullnumber: c_int) -> Option<(vec3_t, vec3_t)> {
    const NORMAL_PLAYER_HULL: c_int = 0;
    const CROUCHED_PLAYER_HULL: c_int = 1;
    const POINT_BASED_HULL: c_int = 2;

    match hullnumber {
        NORMAL_PLAYER_HULL => Some((
            vec3_t::new(-16.0, -16.0, VEC_HULL_MIN),
            vec3_t::new(16.0, 16.0, VEC_HULL_MAX),
        )),
        CROUCHED_PLAYER_HULL => Some((
            vec3_t::new(-16.0, -16.0, VEC_DUCK_HULL_MIN),
            vec3_t::new(16.0, 16.0, VEC_DUCK_HULL_MAX),
        )),
        POINT_BASED_HULL => Some((vec3_t::ZERO, vec3_t::ZERO)),
        _ => None,
    }
}

pub fn get_hull_bounds_ffi(hullnumber: c_int, mins: &mut vec3_t, maxs: &mut vec3_t) -> c_int {
    get_hull_bounds(hullnumber).map_or(0, |(min, max)| {
        *mins = min;
        *maxs = max;
        1
    })
}

pub fn strip_texture_prefix(mut name: &[u8]) -> &[u8] {
    if matches!(name.first(), Some(b'-' | b'+') if name.len() >= 2) {
        name = &name[2..];
    }
    if matches!(name.first(), Some(b'{' | b'!' | b'~' | b' ')) {
        name = &name[1..];
    }
    if name.len() >= CBTEXTURENAMEMAX {
        name = &name[..CBTEXTURENAMEMAX - 1];
    }
    name
}

pub fn find_texture_type(name: &CStrThin) -> c_char {
    let textures = TEXTURES.get().unwrap();
    textures
        .binary_search_by(|(_, s)| s.cmp_ignore_case(name))
        .map(|i| textures[i].0 as c_char)
        .unwrap_or(CHAR_TEX_CONCRETE)
}

// pub fn get_vis_ent_info(ent: c_int) -> c_int {
//     let pm = unsafe { &mut *pmove_rs };
//     if ent >= 0 && ent <= pm.numvisent {
//         pm.visents[ent as usize].info
//     } else {
//         -1
//     }
// }

// pub fn get_phys_ent_info(ent: c_int) -> c_int {
//     let pm = unsafe { &mut *pmove_rs };
//     if ent >= 0 && ent <= pm.numphysent {
//         pm.physents[ent as usize].info
//     } else {
//         -1
//     }
// }

pub fn player_move(pm: &mut playermove_s, is_server: bool) {
    let mut pm = PlayerMove::new(pm);
    pm.player_move(is_server);
    let onground = pm.raw.onground != -1;
    pm.flags_mut().set(EdictFlags::ONGROUND, onground);

    if pm.is_singleplayer() && pm.raw.movetype == MoveType::Walk as c_int {
        pm.raw.friction = 1.0;
    }
}

pub fn player_move_init(pm: &mut playermove_s) {
    let pm = PlayerMove::new(pm);
    pm.create_stuck_table();
    pm.init_texture_types();
}
