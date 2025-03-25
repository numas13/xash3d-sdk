#![allow(non_upper_case_globals)]

use core::ffi::c_int;

pub static mut g_iAlive: c_int = 1;

pub static mut g_iPlayerClass: c_int = 0;
pub static mut g_iTeamNumber: c_int = 0;

/// Observer mode.
pub static mut g_iUser1: c_int = 0;
/// First target.
pub static mut g_iUser2: c_int = 0;
/// Second target.
pub static mut g_iUser3: c_int = 0;
