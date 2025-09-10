use core::{cmp::Ordering, ffi::c_int};

use cl::{
    consts::{PITCH, ROLL, YAW},
    ffi::common::{kbutton_t, vec3_t},
    input::KeyButtonExt,
    macros::{hook_command, hook_command_key},
    math::fabsf,
    prelude::*,
};

use crate::{
    export::{camera_mut, input_mut},
    helpers,
};

// const CAM_COMMAND_NONE: c_int = 0;
const CAM_COMMAND_TOTHIRDPERSON: c_int = 1;
const CAM_COMMAND_TOFIRSTPERSON: c_int = 2;

const CAM_DIST_DELTA: f32 = 1.0;
const CAM_ANGLE_DELTA: f32 = 2.5;
const CAM_ANGLE_SPEED: f32 = 2.5;
const CAM_MIN_DIST: f32 = 30.0;
const CAM_ANGLE_MOVE: f32 = 0.5;
// const MAX_ANGLE_DIFF: f32 = 10.0;
// const PITCH_MAX: f32 = 90.0;
// const PITCH_MIN: f32 = 0.0;
// const YAW_MAX: f32 = 135.0;
// const YAW_MIN: f32 = -135.0;

mod cvar {
    cl::cvar::define! {
        pub static cam_command(c"0", NONE);
        pub static cam_snapto(c"0", NONE);
        pub static cam_idealyaw(c"0", NONE);
        pub static cam_idealpitch(c"0", NONE);
        pub static cam_idealdist(c"64", NONE);

        pub static c_maxpitch(c"90.0", NONE);
        pub static c_minpitch(c"0.0", NONE);
        pub static c_maxyaw(c"135.0", NONE);
        pub static c_minyaw(c"-135.0", NONE);
        pub static c_maxdistance(c"200.0", NONE);
        pub static c_mindistance(c"30.0", NONE);
    }
}

pub struct Camera {
    cam_thirdperson: bool,
    cam_mousemove: bool,
    cam_distancemove: bool,
    cam_ofs: vec3_t,

    cam_pitchup: kbutton_t,
    cam_pitchdown: kbutton_t,
    cam_yawleft: kbutton_t,
    cam_yawright: kbutton_t,
    cam_in: kbutton_t,
    cam_out: kbutton_t,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        hook_command_key!("campitchup", camera_mut().cam_pitchup);
        hook_command_key!("campitchdown", camera_mut().cam_pitchdown);
        hook_command_key!("camyawleft", camera_mut().cam_yawleft);
        hook_command_key!("camyawright", camera_mut().cam_yawright);
        hook_command_key!("camin", camera_mut().cam_in);
        hook_command_key!("camout", camera_mut().cam_out);

        hook_command!(c"snapto", camera_mut().toggle_snapto());
        hook_command!(c"thirdperson", camera_mut().set_third_person());
        hook_command!(c"firstperson", camera_mut().set_first_person());
        hook_command!(c"+cammousemove", camera_mut().start_mouse_move());
        hook_command!(c"-cammousemove", camera_mut().end_mouse_move());
        hook_command!(c"+camdistance", camera_mut().start_distance());
        hook_command!(c"-camdistance", camera_mut().end_distance());

        Self {
            cam_thirdperson: false,
            cam_mousemove: false,
            cam_distancemove: false,
            cam_ofs: vec3_t::ZERO,

            cam_pitchup: kbutton_t::new(),
            cam_pitchdown: kbutton_t::new(),
            cam_yawleft: kbutton_t::new(),
            cam_yawright: kbutton_t::new(),
            cam_in: kbutton_t::new(),
            cam_out: kbutton_t::new(),
        }
    }

    pub fn is_third_person(&self) -> bool {
        if self.cam_thirdperson || unsafe { helpers::g_iUser1 } != 0 {
            return true;
        }
        let player = engine().get_local_player();
        unsafe { helpers::g_iUser2 == (*player).index }
    }

    // pub fn is_first_person(&self) -> bool {
    //     !self.is_third_person()
    // }

    fn offset_set(&mut self, offset: vec3_t) {
        self.cam_ofs = offset;
    }

    pub fn offset(&self) -> vec3_t {
        self.cam_ofs
    }

    pub fn toggle_snapto(&mut self) {
        let v = if cvar::cam_snapto.value() != 0.0 {
            0.0
        } else {
            1.0
        };
        cvar::cam_snapto.value_set(v);
    }

    pub fn set_third_person(&mut self) {
        let engine = engine();
        if engine.is_multiplayer() {
            return;
        }

        let viewangles = engine.get_view_angles();

        if !self.cam_thirdperson {
            self.cam_thirdperson = true;

            self.cam_ofs[YAW] = viewangles[YAW];
            self.cam_ofs[PITCH] = viewangles[PITCH];
            self.cam_ofs[ROLL] = CAM_MIN_DIST;
        }

        cvar::cam_command.value_set(0.0);
    }

    pub fn set_first_person(&mut self) {
        self.cam_thirdperson = false;
        cvar::cam_command.value_set(0.0);
    }

    fn start_mouse_move(&mut self) {
        if !self.is_third_person() {
            self.end_mouse_move();
            return;
        }

        if !self.cam_mousemove {
            self.cam_mousemove = false;
            input_mut().mouse_in_use(true);
        }
    }

    fn end_mouse_move(&mut self) {
        self.cam_mousemove = false;
        input_mut().mouse_in_use(false);
    }

    fn start_distance(&mut self) {
        if !self.is_third_person() {
            self.end_distance();
            return;
        }

        if !self.cam_distancemove {
            self.cam_distancemove = true;
            self.cam_mousemove = true;
            input_mut().mouse_in_use(true);
        }
    }

    fn end_distance(&mut self) {
        self.cam_distancemove = false;
        self.cam_mousemove = false;
        input_mut().mouse_in_use(false);
    }

    pub fn think(&mut self) {
        match cvar::cam_command.value() as c_int {
            CAM_COMMAND_TOTHIRDPERSON => self.set_third_person(),
            CAM_COMMAND_TOFIRSTPERSON => self.set_first_person(),
            _ => {}
        }

        if !self.cam_thirdperson {
            return;
        }

        let engine = engine();
        let (mouse_x, mouse_y) = engine.get_mouse_position();
        let (center_x, center_y) = engine.get_window_center();

        let mut cam_angles = vec3_t::ZERO;
        cam_angles[PITCH] = cvar::cam_idealpitch.value();
        cam_angles[YAW] = cvar::cam_idealyaw.value();
        let mut dist = cvar::cam_idealdist.value();

        if self.cam_mousemove && !self.cam_distancemove {
            match mouse_x.cmp(&center_x) {
                Ordering::Greater => {
                    let c_maxyaw = cvar::c_maxyaw.value();
                    if cam_angles[YAW] < c_maxyaw {
                        let diff = mouse_x - center_x;
                        cam_angles[YAW] += CAM_ANGLE_MOVE * (diff / 2) as f32;
                    }
                    if cam_angles[YAW] > c_maxyaw {
                        cam_angles[YAW] = c_maxyaw;
                    }
                }
                Ordering::Less => {
                    let c_minyaw = cvar::c_minyaw.value();
                    if cam_angles[YAW] > c_minyaw {
                        let diff = center_x - mouse_x;
                        cam_angles[YAW] -= CAM_ANGLE_MOVE * (diff / 2) as f32;
                    }
                    if cam_angles[YAW] < c_minyaw {
                        cam_angles[YAW] = c_minyaw;
                    }
                }
                Ordering::Equal => {}
            }

            match mouse_y.cmp(&center_y) {
                Ordering::Greater => {
                    let c_maxpitch = cvar::c_maxpitch.value();
                    if cam_angles[PITCH] < c_maxpitch {
                        let diff = mouse_y - center_y;
                        cam_angles[PITCH] += CAM_ANGLE_MOVE * (diff / 2) as f32;
                    }
                    if cam_angles[PITCH] > c_maxpitch {
                        cam_angles[PITCH] = c_maxpitch;
                    }
                }
                Ordering::Less => {
                    let c_minpitch = cvar::c_minpitch.value();
                    if cam_angles[PITCH] > c_minpitch {
                        let diff = center_y - mouse_y;
                        cam_angles[PITCH] -= CAM_ANGLE_MOVE * (diff / 2) as f32;
                    }
                    if cam_angles[PITCH] < c_minpitch {
                        cam_angles[PITCH] = c_minpitch;
                    }
                }
                Ordering::Equal => {}
            }
        }

        if self.cam_pitchup.key_state() != 0.0 {
            cam_angles[PITCH] += CAM_ANGLE_DELTA;
        } else if self.cam_pitchdown.key_state() != 0.0 {
            cam_angles[PITCH] -= CAM_ANGLE_DELTA;
        }

        if self.cam_yawleft.key_state() != 0.0 {
            cam_angles[YAW] -= CAM_ANGLE_DELTA;
        } else if self.cam_yawright.key_state() != 0.0 {
            cam_angles[YAW] += CAM_ANGLE_DELTA;
        }

        if self.cam_in.key_state() != 0.0 {
            dist -= CAM_DIST_DELTA;
            if dist < CAM_MIN_DIST {
                cam_angles[PITCH] = 0.0;
                cam_angles[YAW] = 0.0;
                dist = CAM_MIN_DIST;
            }
        } else if self.cam_out.key_state() != 0.0 {
            dist += CAM_DIST_DELTA;
        }

        if self.cam_distancemove {
            match mouse_y.cmp(&center_y) {
                Ordering::Greater => {
                    let c_maxdistance = cvar::c_maxdistance.value();
                    if dist < c_maxdistance {
                        let diff = mouse_y - center_y;
                        dist += CAM_DIST_DELTA * (diff / 2) as f32;
                    }
                    if dist > c_maxdistance {
                        dist = c_maxdistance;
                    }
                }
                Ordering::Less => {
                    let c_mindistance = cvar::c_mindistance.value();
                    if dist > c_mindistance {
                        let diff = center_y - mouse_y;
                        dist -= CAM_DIST_DELTA * (diff / 2) as f32;
                    }
                    if dist < c_mindistance {
                        dist = c_mindistance;
                    }
                }
                Ordering::Equal => {}
            }
        }

        cvar::cam_idealpitch.value_set(cam_angles[PITCH]);
        cvar::cam_idealyaw.value_set(cam_angles[YAW]);
        cvar::cam_idealdist.value_set(dist);

        let viewangles = engine.get_view_angles();
        cam_angles = self.offset();
        if cvar::cam_snapto.value() != 0.0 {
            cam_angles[YAW] = cvar::cam_idealyaw.value() + viewangles[YAW];
            cam_angles[PITCH] = cvar::cam_idealpitch.value() + viewangles[PITCH];
            cam_angles[2] = cvar::cam_idealdist.value();
        } else {
            if cam_angles[YAW] - viewangles[YAW] != cvar::cam_idealyaw.value() {
                let yaw = cvar::cam_idealyaw.value() + viewangles[YAW];
                cam_angles[YAW] = move_toward(cam_angles[YAW], yaw, CAM_ANGLE_SPEED);
            }

            if cam_angles[PITCH] - viewangles[PITCH] != cvar::cam_idealpitch.value() {
                let pitch = cvar::cam_idealpitch.value() + viewangles[PITCH];
                cam_angles[PITCH] = move_toward(cam_angles[PITCH], pitch, CAM_ANGLE_SPEED);
            }

            if fabsf(cam_angles[2] - cvar::cam_idealdist.value()) < 2.0 {
                cam_angles[2] = cvar::cam_idealdist.value();
            } else {
                cam_angles[2] += (cvar::cam_idealdist.value() - cam_angles[2]) / 4.0;
            }
        }

        self.offset_set(vec3_t::new(cam_angles[0], cam_angles[1], dist));
    }
}

fn move_toward(mut cur: f32, goal: f32, _maxspeed: f32) -> f32 {
    if cur != goal {
        if fabsf(cur - goal) > 180.0 {
            if cur < goal {
                cur += 360.0;
            } else {
                cur -= 360.0;
            }
        }

        if cur < goal {
            if cur < goal - 1.0 {
                cur += (goal - cur) / 4.0;
            } else {
                cur = goal;
            }
        } else if cur > goal + 1.0 {
            cur -= (cur - goal) / 4.0;
        } else {
            cur = goal;
        }
    }

    if cur < 0.0 {
        cur += 360.0;
    } else if cur >= 360.0 {
        cur -= 360.0;
    }

    cur
}
