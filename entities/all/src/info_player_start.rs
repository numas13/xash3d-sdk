use xash3d_server::entities::point_entity::PointEntity;

pub type InfoPlayerStart = PointEntity;

define_export! {
    export_info_player_start as export if "info-player-start" {
        info_player_start = info_player_start::InfoPlayerStart,
    }
}
