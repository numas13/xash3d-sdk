use xash3d_ffi::common::vec3_t;

use crate::consts::{PITCH, ROLL, YAW};

// TODO: float trait

// Rust libstd
#[cfg(feature = "std")]
mod imp {
    macro_rules! define {
        ($(fn $name:ident($($a:ident: $t:ty),* $(,)?) $(-> $r:ty)? = $func:path;)*) => (
            $(
                #[inline(always)]
                pub fn $name($($a: $t),*) $(-> $r)? {
                    $func($( $a ),*)
                }
            )*
        );
    }

    define! {
        fn cosf(x: f32) -> f32 = f32::cos;
        fn sinf(x: f32) -> f32 = f32::sin;
        fn tanf(x: f32) -> f32 = f32::tan;
        fn atanf(x: f32) -> f32 = f32::atan;
        fn sqrtf(x: f32) -> f32 = f32::sqrt;
        fn fmaxf(x: f32, y: f32) -> f32 = f32::max;
        fn fminf(x: f32, y: f32) -> f32 = f32::min;
        fn powf(x: f32, y: f32) -> f32 = f32::powf;
        fn fabsf(x: f32) -> f32 = f32::abs;
        fn copysignf(x: f32, y: f32) -> f32 = f32::copysign;

        fn cos(x: f64) -> f64 = f64::cos;
        fn sin(x: f64) -> f64 = f64::sin;
        fn tan(x: f64) -> f64 = f64::tan;
        fn atan(x: f64) -> f64 = f64::atan;
        fn sqrt(x: f64) -> f64 = f64::sqrt;
        fn fmax(x: f64, y: f64) -> f64 = f64::max;
        fn fmin(x: f64, y: f64) -> f64 = f64::min;
        fn pow(x: f64, y: f64) -> f64 = f64::powf;
        fn fabs(x: f64) -> f64 = f64::abs;
        fn copysign(x: f64, y: f64) -> f64 = f64::copysign;
    }
}

// Rust libm
#[cfg(all(feature = "libm", not(feature = "std")))]
mod imp {
    pub use libm::atanf;
    pub use libm::copysignf;
    pub use libm::cosf;
    pub use libm::fabsf;
    pub use libm::fmaxf;
    pub use libm::fminf;
    pub use libm::powf;
    pub use libm::sinf;
    pub use libm::sqrtf;
    pub use libm::tanf;

    pub use libm::atan;
    pub use libm::copysign;
    pub use libm::cos;
    pub use libm::fabs;
    pub use libm::fmax;
    pub use libm::fmin;
    pub use libm::pow;
    pub use libm::sin;
    pub use libm::sqrt;
    pub use libm::tan;
}

pub use self::imp::atanf;
pub use self::imp::copysignf;
pub use self::imp::cosf;
pub use self::imp::fabsf;
pub use self::imp::fmaxf;
pub use self::imp::fminf;
pub use self::imp::powf;
pub use self::imp::sinf;
pub use self::imp::sqrtf;
pub use self::imp::tanf;

pub use self::imp::atan;
pub use self::imp::copysign;
pub use self::imp::cos;
pub use self::imp::fabs;
pub use self::imp::fmax;
pub use self::imp::fmin;
pub use self::imp::pow;
pub use self::imp::sin;
pub use self::imp::sqrt;
pub use self::imp::tan;

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
