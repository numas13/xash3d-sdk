use core::ffi::c_int;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EntOffset(pub c_int);
