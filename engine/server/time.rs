use core::{cmp, ops, time::Duration};

#[cfg(feature = "save")]
use crate::save::{self, Restore, Save};

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct MapTime(f32);

impl MapTime {
    pub const ZERO: Self = Self(0.0);

    pub const fn from_secs_f32(time: f32) -> Self {
        Self(time)
    }

    pub const fn as_secs_f32(self) -> f32 {
        self.0
    }

    pub fn duration_since(&self, earlier: MapTime) -> Duration {
        Duration::from_secs_f32(self.as_secs_f32() - earlier.as_secs_f32())
    }
}

impl From<f32> for MapTime {
    fn from(value: f32) -> Self {
        MapTime(value)
    }
}

impl From<MapTime> for f32 {
    fn from(value: MapTime) -> Self {
        value.0
    }
}

macro_rules! impl_arith_ops {
    ($trait:ident::$meth:ident, $trait_assign:ident::$meth_assign:ident) => {
        impl ops::$trait for MapTime {
            type Output = f32;

            fn $meth(self, rhs: Self) -> Self::Output {
                self.0.$meth(rhs.0)
            }
        }

        impl ops::$trait<f32> for MapTime {
            type Output = Self;

            fn $meth(self, rhs: f32) -> Self::Output {
                Self(self.0.$meth(rhs))
            }
        }

        impl ops::$trait_assign for MapTime {
            fn $meth_assign(&mut self, rhs: Self) {
                self.0.$meth_assign(rhs.0);
            }
        }

        impl ops::$trait_assign<f32> for MapTime {
            fn $meth_assign(&mut self, rhs: f32) {
                self.0.$meth_assign(rhs);
            }
        }
    };
}

impl_arith_ops!(Add::add, AddAssign::add_assign);
impl_arith_ops!(Sub::sub, SubAssign::sub_assign);
impl_arith_ops!(Mul::mul, MulAssign::mul_assign);
impl_arith_ops!(Div::div, DivAssign::div_assign);
impl_arith_ops!(Rem::rem, RemAssign::rem_assign);

impl PartialEq<f32> for MapTime {
    fn eq(&self, other: &f32) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<f32> for MapTime {
    fn partial_cmp(&self, other: &f32) -> Option<cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<MapTime> for f32 {
    fn eq(&self, other: &MapTime) -> bool {
        self.eq(&other.0)
    }
}

impl PartialOrd<MapTime> for f32 {
    fn partial_cmp(&self, other: &MapTime) -> Option<cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[cfg(feature = "save")]
impl Save for MapTime {
    fn save(&self, state: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        cur.write_f32(self.as_secs_f32() - state.time())?;
        Ok(())
    }
}

#[cfg(feature = "save")]
impl Restore for MapTime {
    fn restore(
        &mut self,
        state: &save::RestoreState,
        cur: &mut save::Cursor,
    ) -> save::SaveResult<()> {
        *self = MapTime::from_secs_f32(cur.read_f32()? + state.time());
        Ok(())
    }
}
