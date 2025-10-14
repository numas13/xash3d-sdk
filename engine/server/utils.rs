use core::ffi::CStr;

use csz::{CStrSlice, CStrThin};
pub use xash3d_shared::utils::*;
use xash3d_shared::{entity::EdictFlags, ffi::common::vec3_t};

use crate::{
    engine::ServerEngine,
    entity::{Entity, GetPrivateData, ObjectCaps, UseType},
    str::MapString,
};

/// Used for view cone checking.
#[derive(Copy, Clone, PartialEq)]
pub struct ViewField(f32);

impl ViewField {
    /// +-180 degrees
    pub const FULL: Self = Self(-1.0);
    /// +-135 degrees
    pub const WIDE: Self = Self(-0.7);
    /// +-85 degrees
    pub const FOV: Self = Self(0.09);
    /// +-45 degrees
    pub const NARROW: Self = Self(0.7);
    /// +-25 degrees
    pub const ULTRA_NARROW: Self = Self(0.9);

    pub fn from_degress(degrees: f32) -> Self {
        use xash3d_shared::math::cosf;
        Self(cosf(degrees.to_radians()))
    }

    pub fn to_dot(self) -> f32 {
        self.0
    }
}

pub fn is_master_triggered(
    engine: &ServerEngine,
    master: MapString,
    activator: &dyn Entity,
) -> bool {
    engine
        .find_ent_by_targetname_iter(&master)
        .filter_map(|ent| unsafe { ent.as_ref() }.get_entity())
        .find(|ent| ent.object_caps().intersects(ObjectCaps::MASTER))
        .map_or(true, |ent| ent.is_triggered(activator))
}

pub fn fire_targets(
    target_name: &CStrThin,
    use_type: UseType,
    activator: Option<&dyn Entity>,
    caller: &dyn Entity,
) {
    let engine = caller.engine();
    trace!("Firing: ({target_name})");
    for target in engine.find_ent_by_targetname_iter(target_name) {
        if let Some(target) = unsafe { target.as_ref() }.get_entity() {
            if !target.vars().flags().intersects(EdictFlags::KILLME) {
                trace!("Found: {}, firing ({target_name})", target.classname());
                target.used(use_type, activator, caller);
            }
        }
    }
}

pub fn use_targets(
    kill_target: Option<MapString>,
    use_type: UseType,
    activator: Option<&dyn Entity>,
    caller: &dyn Entity,
) {
    if let Some(kill_target) = kill_target {
        let engine = caller.engine();
        trace!("KillTarget: {kill_target}");
        for target in engine.find_ent_by_targetname_iter(&kill_target) {
            if let Some(target) = unsafe { target.as_ref() }.get_entity() {
                trace!("killing {}", target.classname());
                target.remove_from_world();
            }
        }
    }

    if let Some(target) = caller.vars().target() {
        fire_targets(&target, use_type, activator, caller);
    }
}

pub fn strip_token(key: &CStr, dest: &mut CStrSlice) -> Result<(), csz::CursorError> {
    if let Some(head) = key.to_bytes().split(|i| *i == b'#').next() {
        dest.cursor().write_bytes(head)
    } else {
        dest.clear();
        Ok(())
    }
}

pub fn clamp_vector_to_box(mut v: vec3_t, clamp_size: vec3_t) -> vec3_t {
    if v.x > clamp_size.x {
        v.x -= clamp_size.x;
    } else if v.x < -clamp_size.x {
        v.x += clamp_size.x;
    } else {
        v.x = 0.0;
    }

    if v.y > clamp_size.y {
        v.y -= clamp_size.y;
    } else if v.y < -clamp_size.y {
        v.y += clamp_size.y;
    } else {
        v.y = 0.0;
    }

    if v.z > clamp_size.z {
        v.z -= clamp_size.z;
    } else if v.z < -clamp_size.z {
        v.z += clamp_size.z;
    } else {
        v.z = 0.0;
    }

    v.normalize()
}
