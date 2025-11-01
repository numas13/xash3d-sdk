pub type InfoNodeAir = crate::info_node::InfoNode;

define_export! {
    export_info_node_air as export if "info-node-air" {
        info_node_air = info_node_air::InfoNodeAir,
    }
}
