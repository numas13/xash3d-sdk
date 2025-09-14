use core::mem;

use shared::ffi::server::entvars_s;

use crate::str::MapString;

pub use shared::entity::*;

// TODO: add safe wrapper for entvars_s and remove this trait
pub trait EntityVarsExt {
    fn classname(&self) -> Option<MapString>;

    fn globalname(&self) -> Option<MapString>;

    fn model(&self) -> Option<MapString>;

    fn viewmodel(&self) -> Option<MapString>;

    fn weaponmodel(&self) -> Option<MapString>;

    fn flags(&self) -> &EdictFlags;

    fn flags_mut(&mut self) -> &mut EdictFlags;

    fn effects(&self) -> &Effects;

    fn effects_mut(&mut self) -> &mut Effects;
}

impl EntityVarsExt for entvars_s {
    fn classname(&self) -> Option<MapString> {
        MapString::from_index(self.classname)
    }

    fn globalname(&self) -> Option<MapString> {
        MapString::from_index(self.globalname)
    }

    fn model(&self) -> Option<MapString> {
        MapString::from_index(self.model)
    }

    fn viewmodel(&self) -> Option<MapString> {
        MapString::from_index(self.viewmodel)
    }

    fn weaponmodel(&self) -> Option<MapString> {
        MapString::from_index(self.weaponmodel)
    }

    fn flags(&self) -> &EdictFlags {
        unsafe { mem::transmute(&self.flags) }
    }

    fn flags_mut(&mut self) -> &mut EdictFlags {
        unsafe { mem::transmute(&mut self.flags) }
    }

    fn effects(&self) -> &Effects {
        unsafe { mem::transmute(&self.effects) }
    }

    fn effects_mut(&mut self) -> &mut Effects {
        unsafe { mem::transmute(&mut self.effects) }
    }
}
