use core::ops;

use crate::{
    consts::{PITCH, ROLL, YAW},
    math::{sqrtf, AngleVectors},
};

#[allow(non_camel_case_types)]
pub type vec2_t = Vector<2>;
#[allow(non_camel_case_types)]
pub type vec3_t = Vector<3>;
#[allow(non_camel_case_types)]
pub type vec4_t = Vector<4>;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Vector<const N: usize> {
    arr: [f32; N],
}

impl<const N: usize> Vector<N> {
    pub const ZERO: Self = Self::splat(0.0);

    pub const fn splat(value: f32) -> Self {
        Self { arr: [value; N] }
    }

    pub fn sum(&self) -> f32 {
        self.arr.iter().sum()
    }

    pub fn dot_product(self, other: Self) -> f32 {
        (self * other).sum()
    }

    pub fn distance(self, other: Self) -> f32 {
        sqrtf(self.dot_product(other))
    }

    pub fn length(self) -> f32 {
        self.distance(self)
    }

    pub fn normalize_length(self) -> (Self, f32) {
        let len = self.length();
        if len != 0.0 {
            (self * (1.0 / len), len)
        } else {
            (self, len)
        }
    }

    pub fn normalize(self) -> Self {
        self.normalize_length().0
    }

    pub fn average(mut self, other: Self) -> Self {
        for i in 0..N {
            self[i] = (self[i] + other[i]) * 0.5;
        }
        self
    }

    pub fn to_radians(self) -> Self {
        let mut arr = [0.0; N];
        let mut i = 0;
        while i < N {
            arr[i] = self.arr[i].to_radians();
            i += 1;
        }
        Self { arr }
    }
}

impl Vector<3> {
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);
    pub const NX: Self = Self::new(-1.0, 0.0, 0.0);
    pub const NY: Self = Self::new(0.0, -1.0, 0.0);
    pub const NZ: Self = Self::new(0.0, 0.0, -1.0);

    pub const fn copy_with_z(self, z: f32) -> Self {
        Self::new(self.arr[0], self.arr[1], z)
    }

    pub fn cross_product(self, other: Self) -> Self {
        let a = &self.arr;
        let b = &other.arr;
        Self::new(
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        )
    }

    pub fn pitch(&self) -> f32 {
        self.arr[PITCH]
    }

    pub fn set_pitch(&mut self, value: f32) {
        self.arr[PITCH] = value;
    }

    pub fn yaw(&self) -> f32 {
        self.arr[YAW]
    }

    pub fn set_yaw(&mut self, value: f32) {
        self.arr[YAW] = value;
    }

    pub fn roll(&self) -> f32 {
        self.arr[ROLL]
    }

    pub fn set_roll(&mut self, value: f32) {
        self.arr[ROLL] = value;
    }

    pub fn angle_vectors(&self) -> AngleVectors {
        AngleVectors::new(*self)
    }
}

macro_rules! impl_vector {
    ($n:expr, $($f:ident, $s:ident = $p:expr),+ $(,)?) => {
        impl Vector<$n> {
            pub const fn new($($f: f32),+) -> Self {
                Self { arr: [$($f),+] }
            }

            $(
                pub const fn $f(&self) -> f32 {
                    self.arr[$p]
                }

                pub fn $s(&mut self, $f: f32) {
                    self.arr[$p] = $f;
                }
            )+
        }
    };
}

impl_vector!(2, x, set_x = 0, y, set_y = 1);
impl_vector!(3, x, set_x = 0, y, set_y = 1, z, set_z = 2);
impl_vector!(4, x, set_x = 0, y, set_y = 1, z, set_z = 2, w, set_w = 3);

impl<const N: usize> Default for Vector<N> {
    fn default() -> Self {
        Self { arr: [0.0; N] }
    }
}

impl<const N: usize> From<[f32; N]> for Vector<N> {
    fn from(arr: [f32; N]) -> Self {
        Self { arr }
    }
}

impl<const N: usize> From<Vector<N>> for [f32; N] {
    fn from(vec: Vector<N>) -> Self {
        vec.arr
    }
}

impl<const N: usize> ops::Deref for Vector<N> {
    type Target = [f32; N];

    fn deref(&self) -> &Self::Target {
        &self.arr
    }
}

impl<const N: usize> ops::DerefMut for Vector<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arr
    }
}

impl<const N: usize> ops::Neg for Vector<N> {
    type Output = Self;

    fn neg(mut self) -> Self {
        for i in 0..N {
            self[i] = -self[i];
        }
        self
    }
}

macro_rules! impl_ops {
    ($op:ident::$m:ident, $op_assign:ident::$m_assign:ident) => {
        impl<const N: usize> ops::$op for Vector<N> {
            type Output = Self;

            fn $m(mut self, rhs: Self) -> Self::Output {
                ops::$op_assign::$m_assign(&mut self, rhs);
                self
            }
        }

        impl<const N: usize> ops::$op<f32> for Vector<N> {
            type Output = Self;

            fn $m(mut self, rhs: f32) -> Self::Output {
                ops::$op_assign::$m_assign(&mut self, rhs);
                self
            }
        }

        impl<const N: usize> ops::$op_assign for Vector<N> {
            fn $m_assign(&mut self, rhs: Self) {
                for i in 0..N {
                    ops::$op_assign::$m_assign(&mut self[i], rhs[i]);
                }
            }
        }

        impl<const N: usize> ops::$op_assign<f32> for Vector<N> {
            fn $m_assign(&mut self, rhs: f32) {
                for i in 0..N {
                    ops::$op_assign::$m_assign(&mut self[i], rhs);
                }
            }
        }
    };
}

impl_ops!(Add::add, AddAssign::add_assign);
impl_ops!(Sub::sub, SubAssign::sub_assign);
impl_ops!(Mul::mul, MulAssign::mul_assign);
impl_ops!(Div::div, DivAssign::div_assign);
impl_ops!(Rem::rem, RemAssign::rem_assign);
