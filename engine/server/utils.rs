use csz::CStrThin;
use xash3d_shared::entity::EdictFlags;
pub use xash3d_shared::utils::*;

use crate::{
    engine::ServerEngine,
    entity::{Entity, GetPrivateData, ObjectCaps, UseType},
    str::MapString,
};

pub fn is_master_triggered(
    engine: &ServerEngine,
    master: MapString,
    activator: &dyn Entity,
) -> bool {
    engine
        .find_ent_by_targetname_iter(&master)
        .filter_map(|mut ent| unsafe { ent.as_mut() }.get_entity_mut())
        .find(|ent| ent.object_caps().intersects(ObjectCaps::MASTER))
        .map_or(true, |ent| ent.is_triggered(activator))
}

pub fn fire_targets(
    target_name: &CStrThin,
    use_type: UseType,
    value: f32,
    activator: &mut dyn Entity,
    mut caller: Option<&mut dyn Entity>,
) {
    let engine = activator.engine();
    trace!("Firing: ({target_name})");
    for mut target in engine.find_ent_by_targetname_iter(target_name) {
        if let Some(target) = unsafe { target.as_mut() }.get_entity_mut() {
            if !target.vars().flags().intersects(EdictFlags::KILLME) {
                trace!("Found: {}, firing ({target_name})", target.classname());
                target.used(activator, caller.as_deref_mut(), use_type, value);
            }
        }
    }
}

pub fn use_targets(
    kill_target: Option<MapString>,
    use_type: UseType,
    value: f32,
    activator: &mut dyn Entity,
    caller: Option<&mut dyn Entity>,
) {
    if let Some(kill_target) = kill_target {
        let engine = activator.engine();
        trace!("KillTarget: {kill_target}");
        for mut target in engine.find_ent_by_targetname_iter(&kill_target) {
            if let Some(target) = unsafe { target.as_mut() }.get_entity_mut() {
                trace!("killing {}", target.classname());
                target.remove_from_world();
            }
        }
    }

    if let Some(target) = activator.vars().target() {
        fire_targets(&target, use_type, value, activator, caller);
    }
}
