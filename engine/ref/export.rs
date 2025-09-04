use core::ffi::{c_int, CStr};

use crate::{
    globals::ref_globals_s,
    raw::{ref_api_t, ref_interface_t},
};

pub const GET_REF_API: &CStr = c"GetRefAPI";

pub type REFAPI = Option<
    unsafe extern "C" fn(
        version: c_int,
        exported_funcs: &mut ref_interface_t,
        engine_funcs: &ref_api_t,
        globals: *mut ref_globals_s,
    ) -> c_int,
>;
