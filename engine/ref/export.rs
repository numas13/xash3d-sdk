use core::ffi::{c_int, CStr};

use crate::{
    globals::RefGlobalsRaw,
    engine::RefEngineFunctions,
    raw::ref_interface_t,
};

pub const GET_REF_API: &CStr = c"GetRefAPI";

pub type REFAPI = Option<
    unsafe extern "C" fn(
        version: c_int,
        exported_funcs: &mut ref_interface_t,
        engine_funcs: &RefEngineFunctions,
        globals: *mut RefGlobalsRaw
    ) -> c_int,
>;
