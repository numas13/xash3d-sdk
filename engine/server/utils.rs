pub use xash3d_shared::utils::*;

use crate::{
    engine::ServerEngine,
    entity::{Entity, GetPrivateData, ObjectCaps},
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
