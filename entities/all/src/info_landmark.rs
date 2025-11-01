pub type InfoLandmark = xash3d_server::entities::point_entity::PointEntity;

define_export! {
    export_info_landmark as export if "info-landmark" {
        info_landmark = info_landmark::InfoLandmark,
    }
}
