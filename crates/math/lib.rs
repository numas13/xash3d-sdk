#![no_std]

mod vector;

pub mod consts {
    pub const PITCH: usize = 0;
    pub const YAW: usize = 1;
    pub const ROLL: usize = 2;
}

pub use vector::*;

use consts::*;

macro_rules! define {
    ($(fn $name:ident($($a:ident: $t:ty),* $(,)?) $(-> $r:ty)?;)*) => (
        $(
            #[inline(always)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                #[link(name = "m")]
                extern "C" {
                    fn $name($($a: $t),*) $(-> $r)?;
                }
                unsafe {
                    $name($($a),*)
                }
            }
        )*
    );
}

define! {
    fn cosf(x: f32) -> f32;
    fn sinf(x: f32) -> f32;
    fn sqrtf(x: f32) -> f32;
    fn fmaxf(x: f32, y: f32) -> f32;
    fn fminf(x: f32, y: f32) -> f32;
    fn powf(x: f32, y: f32) -> f32;

    fn cos(x: f64) -> f64;
    fn sin(x: f64) -> f64;
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
        let r = angles.to_radians();
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

    pub fn all(&self) -> (vec3_t, vec3_t, vec3_t) {
        (self.forward(), self.right(), self.up())
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

    pub fn transpose_all(&self) -> (vec3_t, vec3_t, vec3_t) {
        (
            self.transpose_forward(),
            self.transpose_right(),
            self.transpose_up(),
        )
    }
}

pub fn angle_vectors(angles: vec3_t) -> AngleVectors {
    AngleVectors::new(angles)
}

pub fn calc_roll(angles: vec3_t, velocity: vec3_t, roll_angle: f32, roll_speed: f32) -> f32 {
    let right = angle_vectors(angles).right();
    let side = velocity.dot_product(right);
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
