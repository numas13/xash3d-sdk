use xash3d_server::entities::point_entity::PointEntity;

pub type InfoTarget = PointEntity;

define_export! {
    export_info_target as export if "info-target" {
        // Lightning target, just alias landmark.
        info_target = info_target::InfoTarget,
    }
}
