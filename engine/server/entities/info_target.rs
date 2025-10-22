use crate::export::export_entity_default;

use super::point_entity::PointEntity;

// Lightning target, just alias landmark.
export_entity_default!("export-info_target", info_target, PointEntity);
