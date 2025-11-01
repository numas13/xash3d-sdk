pub type Water = xash3d_entity_door::func_door::Door;

define_export! {
    export_func_water as export if "func-water" {
        func_water = func_water::Water,
    }
}
