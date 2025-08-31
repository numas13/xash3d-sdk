use csz::CStrThin;

use crate::{
    cvar::{GetCvar, SetCvar},
    str::ToEngineStr,
};

/// Engine API to read and modify console variables.
pub trait EngineCvar: Sized {
    fn get_cvar_float(&self, name: impl ToEngineStr) -> f32;

    fn set_cvar_float(&self, name: impl ToEngineStr, value: f32);

    fn get_cvar_string(&self, name: impl ToEngineStr) -> &CStrThin;

    fn set_cvar_string(&self, name: impl ToEngineStr, value: impl ToEngineStr);

    fn get_cvar<'a, T: GetCvar<'a>>(&'a self, name: impl ToEngineStr) -> T {
        T::get_cvar(self, name)
    }

    fn set_cvar<T: SetCvar>(&self, name: impl ToEngineStr, value: T) {
        T::set_cvar(self, name, value)
    }
}
