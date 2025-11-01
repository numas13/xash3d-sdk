use xash3d_server::entities::point_entity::PointEntity;

pub type InfoTeleportDestination = PointEntity;

define_export! {
    export_info_teleport_destination as export if "info-teleport-destination" {
        info_teleport_destination = info_teleport_destination::InfoTeleportDestination,
    }
}
