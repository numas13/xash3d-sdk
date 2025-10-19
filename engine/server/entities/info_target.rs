use crate::export::export_entity_default;

// Lightning target, just alias landmark.
export_entity_default!(
    "export-info_target",
    info_target,
    super::point_entity::PointEntity
);
