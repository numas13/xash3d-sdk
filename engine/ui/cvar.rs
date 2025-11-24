use xash3d_shared::{ffi::common::cvar_s, macros::const_assert_size_eq};

use crate::prelude::*;

pub use xash3d_shared::cvar::*;

pub type Cvar<T = f32> = xash3d_shared::cvar::Cvar<UiEngine, T>;

const_assert_size_eq!(*mut cvar_s, Cvar);
const_assert_size_eq!(*mut cvar_s, Option<Cvar>);
