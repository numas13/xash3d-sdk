use xash3d_ffi::common::vec3_t;

use crate::consts::{PITCH, ROLL, YAW};

// TODO: float trait and libstd support

macro_rules! define {
    ($(fn $name:ident($($a:ident: $t:ty),* $(,)?) $(-> $r:ty)?;)*) => (
        $(
            #[cfg(not(feature = "libm"))]
            #[inline(always)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                #[cfg_attr(unix, link(name = "m"))]
                unsafe extern "C" {
                    fn $name($($a: $t),*) $(-> $r)?;
                }
                unsafe {
                    $name($($a),*)
                }
            }

            #[cfg(feature = "libm")]
            pub use libm::$name;
        )*
    );
}

define! {
    fn cosf(x: f32) -> f32;
    fn sinf(x: f32) -> f32;
    fn tanf(x: f32) -> f32;
    fn atanf(x: f32) -> f32;
    fn sqrtf(x: f32) -> f32;
    fn fmaxf(x: f32, y: f32) -> f32;
    fn fminf(x: f32, y: f32) -> f32;
    fn powf(x: f32, y: f32) -> f32;

    fn cos(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn tan(x: f64) -> f64;
    fn atan(x: f64) -> f64;
    fn sqrt(x: f64) -> f64;
    fn fmax(x: f64, y: f64) -> f64;
    fn fmin(x: f64, y: f64) -> f64;
    fn pow(x: f64, y: f64) -> f64;
}

#[inline(always)]
fn sign() -> u64 {
    1 << (core::mem::size_of::<f64>() * 8 - 1)
}

#[inline(always)]
fn signf() -> u32 {
    1 << (core::mem::size_of::<f32>() * 8 - 1)
}

#[inline(always)]
pub fn fabs(x: f64) -> f64 {
    f64::from_bits(x.to_bits() & !sign())
}

#[inline(always)]
pub fn fabsf(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & !signf())
}

#[inline(always)]
pub fn copysign(x: f64, y: f64) -> f64 {
    let sign = sign();
    f64::from_bits((x.to_bits() & !sign) | (y.to_bits() & sign))
}

#[inline(always)]
pub fn copysignf(x: f32, y: f32) -> f32 {
    let sign = signf();
    f32::from_bits((x.to_bits() & !sign) | (y.to_bits() & sign))
}

#[inline(always)]
pub fn pow2(x: f32) -> f32 {
    x * x
}

/// All angle vectors.
#[derive(Copy, Clone, Debug)]
pub struct AngleVectorsAll {
    pub forward: vec3_t,
    pub right: vec3_t,
    pub up: vec3_t,
}

pub struct AngleVectors {
    sp: f32,
    cp: f32,
    sy: f32,
    cy: f32,
    sr: f32,
    cr: f32,
}

impl AngleVectors {
    pub fn new(angles: vec3_t) -> Self {
        let r = [
            angles.x.to_radians(),
            angles.y.to_radians(),
            angles.z.to_radians(),
        ];
        Self {
            sp: sinf(r[PITCH]),
            cp: cosf(r[PITCH]),
            sy: sinf(r[YAW]),
            cy: cosf(r[YAW]),
            sr: sinf(r[ROLL]),
            cr: cosf(r[ROLL]),
        }
    }

    pub fn forward(&self) -> vec3_t {
        vec3_t::new(self.cp * self.cy, self.cp * self.sy, -self.sp)
    }

    pub fn right(&self) -> vec3_t {
        vec3_t::new(
            -self.sr * self.sp * self.cy + -self.cr * -self.sy,
            -self.sr * self.sp * self.sy + -self.cr * self.cy,
            -self.sr * self.sp,
        )
    }

    pub fn up(&self) -> vec3_t {
        vec3_t::new(
            self.cr * self.sp * self.cy + -self.sr * -self.sy,
            self.cr * self.sp * self.sy + -self.sr * self.cy,
            self.cr * self.cp,
        )
    }

    /// Returns all computed angle vectors.
    pub fn all(&self) -> AngleVectorsAll {
        AngleVectorsAll {
            forward: self.forward(),
            right: self.right(),
            up: self.up(),
        }
    }

    pub fn transpose_forward(&self) -> vec3_t {
        vec3_t::new(
            self.cp * self.cy,
            self.sr * self.sp * self.cy + self.cr * -self.sy,
            self.cr * self.sp * self.cy + -self.sr * -self.sy,
        )
    }

    pub fn transpose_right(&self) -> vec3_t {
        vec3_t::new(
            self.cp * self.sy,
            self.sr * self.sp * self.sy + self.cr * self.cy,
            self.cr * self.sp * self.sy + -self.sr * self.cy,
        )
    }

    pub fn transpose_up(&self) -> vec3_t {
        vec3_t::new(-self.sp, self.sr * self.cp, self.cr * self.cp)
    }

    pub fn transpose_all(&self) -> AngleVectorsAll {
        AngleVectorsAll {
            forward: self.transpose_forward(),
            right: self.transpose_right(),
            up: self.transpose_up(),
        }
    }
}

pub trait ToAngleVectors {
    fn angle_vectors(&self) -> AngleVectors;
}

impl ToAngleVectors for vec3_t {
    fn angle_vectors(&self) -> AngleVectors {
        AngleVectors::new(*self)
    }
}

pub fn calc_roll(angles: vec3_t, velocity: vec3_t, roll_angle: f32, roll_speed: f32) -> f32 {
    let right = angles.angle_vectors().right();
    let side = velocity.dot(right);
    let sign = copysignf(1.0, side);
    let side = fabsf(side);
    let side = if side < roll_speed {
        side * roll_angle / roll_speed
    } else {
        roll_angle
    };
    side * sign
}

pub fn angle_mod(a: f32) -> f32 {
    (360.0 / 65536.0) * ((a * (65536.0 / 360.0)) as i32 & 65535) as f32
}

pub fn angle_distance(next: f32, cur: f32) -> f32 {
    let delta = next - cur;
    if delta < -180.0 {
        delta + 360.0
    } else if delta > 180.0 {
        delta - 360.0
    } else {
        delta
    }
}

pub fn approach(target: f32, value: f32, speed: f32) -> f32 {
    let delta = target - value;
    if delta > speed {
        value + speed
    } else if delta < -speed {
        value - speed
    } else {
        target
    }
}

pub fn approach_angle(target: f32, value: f32, mut speed: f32) -> f32 {
    let target = angle_mod(target);
    let value = angle_mod(value);

    let mut delta = target - value;
    if delta < -180.0 {
        delta += 360.0;
    } else if delta > 180.0 {
        delta -= 360.0;
    }

    if speed < 0.0 {
        speed = -speed;
    }

    if delta > speed {
        value + speed
    } else if delta < -speed {
        value - speed
    } else {
        target
    }
}
