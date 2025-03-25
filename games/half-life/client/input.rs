use core::{
    cell::{Ref, RefCell, RefMut},
    ffi::{c_int, CStr},
    mem,
    ptr::addr_of_mut,
};

use alloc::vec::Vec;
use cell::SyncOnceCell;
use cl::{
    engine,
    macros::{hook_command, hook_command_key},
    raw::{kbutton_t, KeyState},
    KeyButtonExt,
};
use csz::{CStrBox, CStrThin};
use math::{
    angle_mod,
    consts::{PITCH, ROLL, YAW},
    pow, sqrt, sqrtf, vec3_t,
};
use shared::{consts, raw::usercmd_s};

use crate::{
    helpers,
    hud::{hud, hud_mut, weapon_menu::WeaponMenu},
    input,
    view::view_mut,
};

const MOUSE_BUTTON_COUNT: c_int = 5;

#[link(name = "SDL2")]
extern "C" {
    fn SDL_GetRelativeMouseState(x: &mut c_int, y: &mut c_int) -> u32;
}

mod cvar {
    shared::cvar::define! {
        pub static lookstrafe(c"0", ARCHIVE);
        pub static lookspring(c"0", ARCHIVE);
        pub static cl_pitchup(c"89", NONE);
        pub static cl_pitchdown(c"89", NONE);
        pub static cl_pitchspeed(c"225", NONE);
        pub static cl_anglespeedkey(c"0.67", NONE);
        pub static cl_yawspeed(c"210", NONE);
        pub static cl_upspeed(c"320", NONE);
        pub static cl_forwardspeed(c"400", ARCHIVE);
        pub static cl_backspeed(c"400", ARCHIVE);
        pub static cl_sidespeed(c"400", NONE);
        pub static cl_movespeedkey(c"0.3", NONE);

        pub static m_pitch(c"0.022", ARCHIVE);
        pub static m_yaw(c"0.022", ARCHIVE);
        pub static m_forward(c"1", ARCHIVE);
        pub static m_side(c"0.8", ARCHIVE);

        pub static m_rawinput(c"1", ARCHIVE);
        pub static m_filter(c"0", ARCHIVE);
        pub static sensitivity(c"3", ARCHIVE.union(FILTERSTUFFTEXT));

        pub static m_customaccel(c"0", ARCHIVE);
        pub static m_customaccel_scale(c"0.04", ARCHIVE);
        pub static m_customaccel_max(c"0", ARCHIVE);
        pub static m_customaccel_exponent(c"1", ARCHIVE);
    }
}

#[allow(non_upper_case_globals)]
static mut in_graph: kbutton_t = kbutton_t::new();
#[allow(non_upper_case_globals)]
pub static mut in_mlook: kbutton_t = kbutton_t::new();
#[allow(non_upper_case_globals)]
static mut in_jlook: kbutton_t = kbutton_t::new();

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Point {
    x: c_int,
    y: c_int,
}

struct Mouse {}

impl Mouse {
    fn new() -> Self {
        Self {}
    }
}

struct Key {
    name: CStrBox,
    kb: *mut kbutton_t,
}

pub struct KeyList {
    list: Vec<Key>,
}

impl KeyList {
    fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn find(&self, name: &CStrThin) -> Option<*mut kbutton_t> {
        self.list.iter().find(|i| i.name == *name).map(|i| i.kb)
    }

    fn add(&mut self, name: &CStr, kb: *mut kbutton_t) {
        if self.find(name.into()).is_some() {
            return;
        }

        self.list.push(Key {
            name: name.into(),
            kb,
        })
    }

    fn clear(&mut self) {
        self.list.clear();
    }
}

pub struct Input {
    pub keys: KeyList,
    oldangles: vec3_t,
    #[allow(dead_code)]
    mouse: Mouse,

    in_impulse: u8,
    in_cancel: bool,

    mouse_initialized: bool,
    mouse_active: bool,
    mouse_visible: bool,
    mouse_in_use: bool,
    mouse_raw_used: bool,

    mouse_buttons: c_int,
    mouse_oldbuttonstate: c_int,
    old_mouse_x: c_int,
    old_mouse_y: c_int,
    mx_accum: c_int,
    my_accum: c_int,
    mouse_x: f32,
    mouse_y: f32,

    // TODO: need global ptr
    // in_graph: kbutton_t,
    // in_mlook: kbutton_t,
    // in_jlook: kbutton_t,
    in_klook: kbutton_t,
    in_left: kbutton_t,
    in_right: kbutton_t,
    in_forward: kbutton_t,
    in_back: kbutton_t,
    in_lookup: kbutton_t,
    in_lookdown: kbutton_t,
    in_moveleft: kbutton_t,
    in_moveright: kbutton_t,
    in_strafe: kbutton_t,
    in_speed: kbutton_t,
    in_use: kbutton_t,
    in_jump: kbutton_t,
    in_attack: kbutton_t,
    in_attack2: kbutton_t,
    in_up: kbutton_t,
    in_down: kbutton_t,
    in_duck: kbutton_t,
    in_reload: kbutton_t,
    in_alt1: kbutton_t,
    in_score: kbutton_t,
    in_break: kbutton_t,
}

impl Input {
    fn new() -> Self {
        hook_command_key!("moveup", input_mut().in_up);
        hook_command_key!("movedown", input_mut().in_down);
        hook_command_key!("left", input_mut().in_left);
        hook_command_key!("right", input_mut().in_right);
        hook_command_key!("forward", input_mut().in_forward);
        hook_command_key!("back", input_mut().in_back);
        hook_command_key!("lookup", input_mut().in_lookup);
        hook_command_key!("lookdown", input_mut().in_lookdown);
        hook_command_key!("strafe", input_mut().in_strafe);
        hook_command_key!("moveleft", input_mut().in_moveleft);
        hook_command_key!("moveright", input_mut().in_moveright);
        hook_command_key!("speed", input_mut().in_speed);
        hook_command_key!("attack", input_mut().in_attack, up {
            input_mut().in_cancel = false;
        });
        hook_command_key!("attack2", input_mut().in_attack2);
        hook_command_key!("use", input_mut().in_use);
        hook_command_key!("jump", input_mut().in_jump);
        hook_command_key!("klook", input_mut().in_klook);
        hook_command_key!("mlook", unsafe { &mut *addr_of_mut!(in_mlook) }, up {
            let state = unsafe { in_mlook.state };
            if !state.contains(KeyState::DOWN) && cvar::lookspring.value() != 0.0 {
                view_mut().start_pitch_drift();
            }
        });
        hook_command_key!("jlook", unsafe { &mut *addr_of_mut!(in_jlook) });
        hook_command_key!("duck", input_mut().in_duck);
        hook_command_key!("reload", input_mut().in_reload);
        hook_command_key!("alt1", input_mut().in_alt1);
        hook_command_key!("score", input_mut().in_score, down {
            hud_mut().show_score_board(true);
        }, up {
            hud_mut().show_score_board(false);
        });
        hook_command_key!("showscores", input_mut().in_score, down {
            hud_mut().show_score_board(true);
        }, up {
            hud_mut().show_score_board(false);
        });
        hook_command_key!("graph", unsafe { &mut *addr_of_mut!(in_graph) });
        hook_command_key!("break", input_mut().in_break);

        // TODO: hook in_cancel???

        hook_command!(c"impulse", {
            let s = engine().cmd_argv(1);
            let n = s.to_str().ok().and_then(|s| s.parse().ok()).unwrap_or(0);
            input_mut().in_impulse = n;
        });

        hook_command!(c"force_centerview", {
            if !input().mouse_in_use {
                let engine = engine();
                let mut viewangles = engine.get_view_angles();
                viewangles[PITCH] = 0.0;
                engine.set_view_angles(viewangles);
            }
        });

        hook_command!("joyadvancedupdate", {
            // TODO: joystick
        });

        let mut keys = KeyList::new();
        #[allow(unused_unsafe)] // MSRV 1.77
        {
            keys.add(c"in_graph", unsafe { addr_of_mut!(in_graph) });
            keys.add(c"in_mlook", unsafe { addr_of_mut!(in_mlook) });
            keys.add(c"in_jlook", unsafe { addr_of_mut!(in_jlook) });
        }

        Self {
            keys,
            oldangles: vec3_t::ZERO,
            mouse: Mouse::new(),

            in_impulse: 0,
            in_cancel: false,

            mouse_initialized: engine().check_parm(c"-nomouse") == 0,
            mouse_active: false,
            mouse_visible: false,
            mouse_in_use: false,
            mouse_raw_used: true,

            mouse_buttons: MOUSE_BUTTON_COUNT,
            mouse_oldbuttonstate: 0,
            old_mouse_x: 0,
            old_mouse_y: 0,
            mx_accum: 0,
            my_accum: 0,
            mouse_x: 0.0,
            mouse_y: 0.0,

            // in_graph: kbutton_t::new(),
            // in_mlook: kbutton_t::new(),
            // in_jlook: kbutton_t::new(),
            in_klook: kbutton_t::new(),
            in_left: kbutton_t::new(),
            in_right: kbutton_t::new(),
            in_forward: kbutton_t::new(),
            in_back: kbutton_t::new(),
            in_lookup: kbutton_t::new(),
            in_lookdown: kbutton_t::new(),
            in_moveleft: kbutton_t::new(),
            in_moveright: kbutton_t::new(),
            in_strafe: kbutton_t::new(),
            in_speed: kbutton_t::new(),
            in_use: kbutton_t::new(),
            in_jump: kbutton_t::new(),
            in_attack: kbutton_t::new(),
            in_attack2: kbutton_t::new(),
            in_up: kbutton_t::new(),
            in_down: kbutton_t::new(),
            in_duck: kbutton_t::new(),
            in_reload: kbutton_t::new(),
            in_alt1: kbutton_t::new(),
            in_score: kbutton_t::new(),
            in_break: kbutton_t::new(),
        }
    }

    pub fn clear_states(&mut self) {
        if !self.mouse_active {
            return;
        }
        self.mx_accum = 0;
        self.my_accum = 0;
        self.mouse_oldbuttonstate = 0;
    }

    pub fn shutdown(&mut self) {
        self.deactivate_mouse();
        self.keys.clear();
    }

    fn use_raw_input(&self) -> bool {
        cvar::m_rawinput.value() != 0.0
    }

    pub fn activate_mouse(&mut self) {
        if self.mouse_initialized {
            self.mouse_active = true;
        }
    }

    pub fn deactivate_mouse(&mut self) {
        if self.mouse_initialized {
            self.mouse_active = false;
        }
    }

    pub fn get_mouse_sensitivity(&self) -> f32 {
        let v = cvar::sensitivity.value();
        if !(0.01..=10000.0).contains(&v) {
            let v = v.clamp(0.01, 10000.0);
            cvar::sensitivity.value_set(v);
            v
        } else {
            v
        }
    }

    pub fn button_bits(&mut self, reset_state: bool, show_score: bool) -> u32 {
        let mut bits = 0;

        macro_rules! set {
            ($($name:expr => $bits:expr),* $(,)?) => (
                $(if $name.state.intersects(KeyState::ANY_DOWN) {
                    bits |= $bits;
                })*
            );
        }

        set! {
            self.in_attack => consts::IN_ATTACK,
            self.in_duck => consts::IN_DUCK,
            self.in_jump => consts::IN_JUMP,
            self.in_forward => consts::IN_FORWARD,
            self.in_back => consts::IN_BACK,
            self.in_use => consts::IN_USE,
            self.in_left => consts::IN_LEFT,
            self.in_right => consts::IN_RIGHT,
            self.in_moveleft => consts::IN_MOVELEFT,
            self.in_moveright => consts::IN_MOVERIGHT,
            self.in_attack2 => consts::IN_ATTACK2,
            self.in_reload => consts::IN_RELOAD,
            self.in_alt1 => consts::IN_ALT1,
            self.in_score => consts::IN_SCORE,
        }

        if self.in_cancel {
            bits |= consts::IN_CANCEL;
        }

        if show_score {
            bits |= consts::IN_SCORE;
        }

        if reset_state {
            macro_rules! reset {
                ($($name:expr),* $(,)?) => (
                    $($name.state.remove(KeyState::IMPULSE_DOWN);)*
                );
            }
            reset! {
                self.in_attack,
                self.in_duck,
                self.in_jump,
                self.in_forward,
                self.in_back,
                self.in_use,
                self.in_left,
                self.in_right,
                self.in_moveleft,
                self.in_moveright,
                self.in_attack2,
                self.in_reload,
                self.in_alt1,
                self.in_score,
            }
        }

        bits
    }

    pub fn reset_button_bits(&mut self, bits: u32, show_score: bool) {
        let bits_new = self.button_bits(false, show_score) ^ bits;

        if bits_new & consts::IN_ATTACK != 0 {
            if bits & consts::IN_ATTACK != 0 {
                self.in_attack.key_down();
            } else {
                self.in_attack.state = KeyState::empty();
            }
        }
    }

    pub fn in_commands(&mut self) {
        // TODO: joystick
    }

    pub fn mouse_in_use(&mut self, value: bool) {
        self.mouse_in_use = value;
    }

    fn get_relative_mouse_pos(&mut self) -> (c_int, c_int) {
        if !self.use_raw_input() {
            let engine = engine();
            let screen = engine.get_screen_info();
            let (cx, cy) = (screen.width / 2, screen.height / 2);
            let (mx, my) = if !self.mouse_raw_used {
                engine.get_mouse_position()
            } else {
                // If raw input used we need to reset cursor position before we can
                // get valid relative position.
                self.mouse_raw_used = false;
                (cx, cy)
            };
            engine.set_mouse_position(cx, cy);
            (mx - cx, my - cy)
        } else {
            let mut dx = 0;
            let mut dy = 0;
            unsafe {
                SDL_GetRelativeMouseState(&mut dx, &mut dy);
            }
            if !self.mouse_raw_used {
                self.mouse_raw_used = true;
            }
            (dx, dy)
        }
    }

    pub fn accumulate(&mut self) {
        if !self.mouse_in_use && !self.mouse_visible && self.mouse_active {
            let (dx, dy) = self.get_relative_mouse_pos();
            self.mx_accum += dx;
            self.my_accum += dy;
        }
    }

    fn scale_mouse(&mut self) {
        let mx = self.mouse_x;
        let my = self.mouse_y;

        let hud = hud();
        let mouse_senstivity = if hud.get_sensitivity() != 0.0 {
            hud.get_sensitivity()
        } else {
            self.get_mouse_sensitivity()
        };

        if cvar::m_customaccel.value() != 0.0 {
            let raw_mouse_movement_distance = sqrt((mx * mx + my * my) as f64);
            let acceleration_scale = cvar::m_customaccel_scale.value();
            let accelerated_sensitivity_max = cvar::m_customaccel_max.value();
            let accelerated_sensitivity_exponent = cvar::m_customaccel_exponent.value();
            let mut accelerated_sensitivity = (pow(
                raw_mouse_movement_distance,
                accelerated_sensitivity_exponent as f64,
            ) * acceleration_scale as f64
                + mouse_senstivity as f64) as f32;

            if accelerated_sensitivity_max > 0.0001
                && accelerated_sensitivity > accelerated_sensitivity_max
            {
                accelerated_sensitivity = accelerated_sensitivity_max;
            }

            self.mouse_x *= accelerated_sensitivity;
            self.mouse_y *= accelerated_sensitivity;

            if cvar::m_customaccel.value() == 2.0 {
                self.mouse_x *= cvar::m_yaw.value();
                self.mouse_y *= cvar::m_pitch.value();
            }
        } else {
            self.mouse_x *= mouse_senstivity;
            self.mouse_y *= mouse_senstivity;
        }
    }

    fn mouse_move(&mut self, _frametime: f32, cmd: &mut usercmd_s) {
        let engine = engine();
        let mut viewangles = engine.get_view_angles();

        let state = unsafe { input::in_mlook.state };
        if state.contains(KeyState::DOWN) {
            view_mut().stop_pitch_drift();
        }

        if !self.mouse_in_use && !hud().state.intermission && !self.mouse_visible {
            let (dx, dy) = self.get_relative_mouse_pos();

            let mx = dx + self.mx_accum;
            let my = dy + self.my_accum;

            self.mx_accum = 0;
            self.my_accum = 0;

            if cvar::m_filter.value() != 0.0 {
                self.mouse_x = (mx + self.old_mouse_x) as f32 * 0.5;
                self.mouse_y = (my + self.old_mouse_y) as f32 * 0.5;
            } else {
                self.mouse_x = mx as f32;
                self.mouse_y = my as f32;
            }

            self.old_mouse_x = mx;
            self.old_mouse_y = my;

            self.scale_mouse();

            let in_mlook_state = unsafe { in_mlook.state };
            if self.in_strafe.is_down()
                || cvar::lookstrafe.value() != 0.0 && in_mlook_state.contains(KeyState::DOWN)
            {
                cmd.sidemove += cvar::m_side.value() * self.mouse_x;
            } else {
                viewangles[YAW] -= cvar::m_yaw.value() * self.mouse_x;
            }

            if in_mlook_state.contains(KeyState::DOWN) && !self.in_strafe.is_down() {
                viewangles[PITCH] += cvar::m_pitch.value() * self.mouse_y;
                if viewangles[PITCH] > cvar::cl_pitchdown.value() {
                    viewangles[PITCH] = cvar::cl_pitchdown.value();
                }
                if viewangles[PITCH] < -cvar::cl_pitchup.value() {
                    viewangles[PITCH] = -cvar::cl_pitchup.value();
                }
            } else if self.in_strafe.is_down() && engine.is_no_clipping() {
                cmd.upmove -= cvar::m_forward.value() * self.mouse_y;
            } else {
                cmd.forwardmove -= cvar::m_forward.value() * self.mouse_y;
            }
        }

        engine.set_view_angles(viewangles);
    }

    fn adjust_angles(&mut self, frametime: f32, viewangles: &mut vec3_t) {
        let speed = if self.in_speed.is_down() {
            frametime * cvar::cl_anglespeedkey.value()
        } else {
            frametime
        };

        if !self.in_strafe.is_down() {
            let yawspeed = cvar::cl_yawspeed.value();
            viewangles[YAW] -= speed * yawspeed * self.in_right.key_state();
            viewangles[YAW] += speed * yawspeed * self.in_left.key_state();
            viewangles[YAW] = angle_mod(viewangles[YAW]);
        }

        let pitchspeed = cvar::cl_pitchspeed.value();
        if self.in_klook.is_down() {
            view_mut().stop_pitch_drift();
            viewangles[PITCH] -= speed * pitchspeed * self.in_forward.key_state();
            viewangles[PITCH] += speed * pitchspeed * self.in_back.key_state();
        }

        let up = self.in_lookup.key_state();
        let down = self.in_lookdown.key_state();

        viewangles[PITCH] -= speed * pitchspeed * up;
        viewangles[PITCH] += speed * pitchspeed * down;

        if up != 0.0 || down != 0.0 {
            view_mut().stop_pitch_drift();
        }

        let pitchdown = cvar::cl_pitchdown.value();
        let pitchup = cvar::cl_pitchup.value();
        viewangles[PITCH] = viewangles[PITCH].clamp(-pitchup, pitchdown);

        viewangles[ROLL] = viewangles[ROLL].clamp(-50.0, 50.0);
    }

    pub fn mouse_event(&mut self, mstate: c_int) {
        if self.mouse_in_use || self.mouse_visible {
            return;
        }

        let engine = engine();
        for i in 0..self.mouse_buttons {
            if mstate & (1 << i) != 0 && self.mouse_oldbuttonstate & (1 << i) == 0 {
                engine.key_event(consts::K_MOUSE1 + i as u32, true);
            }

            if mstate & (1 << i) == 0 && self.mouse_oldbuttonstate & (1 << i) != 0 {
                engine.key_event(consts::K_MOUSE1 + i as u32, false);
            }
        }

        self.mouse_oldbuttonstate = mstate;
    }

    fn in_move(&mut self, frametime: f32, cmd: &mut usercmd_s) {
        if !self.mouse_in_use && self.mouse_active {
            self.mouse_move(frametime, cmd);
        }

        // TODO: joystick
    }

    pub fn create_move(&mut self, frametime: f32, active: bool) -> usercmd_s {
        let engine = engine();
        let mut cmd = usercmd_s::default();

        if active {
            let mut viewangles = engine.get_view_angles();
            self.adjust_angles(frametime, &mut viewangles);
            engine.set_view_angles(viewangles);

            if self.in_strafe.is_down() {
                let sidespeed = cvar::cl_sidespeed.value();
                cmd.sidemove += sidespeed * self.in_right.key_state();
                cmd.sidemove -= sidespeed * self.in_left.key_state();
            }

            let sidespeed = cvar::cl_sidespeed.value();
            cmd.sidemove += sidespeed * self.in_moveright.key_state();
            cmd.sidemove -= sidespeed * self.in_moveleft.key_state();

            let upspeed = cvar::cl_upspeed.value();
            cmd.upmove += upspeed * self.in_up.key_state();
            cmd.upmove -= upspeed * self.in_down.key_state();

            if !self.in_klook.is_down() {
                let forwardspeed = cvar::cl_forwardspeed.value();
                let backspeed = cvar::cl_backspeed.value();
                cmd.forwardmove += forwardspeed * self.in_forward.key_state();
                cmd.forwardmove -= backspeed * self.in_back.key_state();
            }

            if self.in_speed.is_down() {
                let cl_movespeedkey = cvar::cl_movespeedkey.value();
                cmd.forwardmove *= cl_movespeedkey;
                cmd.sidemove *= cl_movespeedkey;
                cmd.upmove *= cl_movespeedkey;
            }

            let spd = engine.get_client_max_speed();
            if spd != 0.0 {
                let fmov = sqrtf(
                    (cmd.forwardmove * cmd.forwardmove)
                        + (cmd.sidemove * cmd.sidemove)
                        + (cmd.upmove * cmd.upmove),
                );

                if fmov > spd {
                    cmd.move_vector_set(cmd.move_vector() * (spd / fmov));
                }
            }

            self.in_move(frametime, &mut cmd);
        }

        cmd.impulse = mem::take(&mut self.in_impulse);

        cmd.weaponselect = hud().items.get_mut::<WeaponMenu>().take_weapon_select() as u8;

        let show_score = hud().show_score();
        cmd.buttons = self.button_bits(true, show_score) as u16;

        let viewangles = engine.get_view_angles();
        if unsafe { helpers::g_iAlive != 0 } {
            cmd.viewangles = viewangles;
            self.oldangles = viewangles;
        } else {
            cmd.viewangles = self.oldangles;
        }

        cmd
    }
}

static INPUT: SyncOnceCell<RefCell<Input>> = unsafe { SyncOnceCell::new() };

pub fn input_global() -> &'static RefCell<Input> {
    INPUT.get_or_init(|| RefCell::new(Input::new()))
}

pub fn input<'a>() -> Ref<'a, Input> {
    input_global().borrow()
}

pub fn input_mut<'a>() -> RefMut<'a, Input> {
    input_global().borrow_mut()
}

pub fn init() {
    input_global();
}
