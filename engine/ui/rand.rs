use core::ffi::c_int;

use crate::engine;

pub trait Random {
    fn rand(start: Self, end: Self) -> Self;
}

impl Random for i8 {
    fn rand(start: Self, end: Self) -> Self {
        engine().rand_int(start as c_int, end as c_int) as Self
    }
}

impl Random for i16 {
    fn rand(start: Self, end: Self) -> Self {
        engine().rand_int(start as c_int, end as c_int) as Self
    }
}

impl Random for i32 {
    fn rand(start: Self, end: Self) -> Self {
        engine().rand_int(start as c_int, end as c_int) as Self
    }
}

impl Random for f32 {
    fn rand(start: Self, end: Self) -> Self {
        engine().rand_f32(start, end)
    }
}
