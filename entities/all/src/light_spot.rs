pub type LightSpot = crate::light::Light;

define_export! {
    export_light_spot as export if "light-spot" {
        light_spot = light_spot::LightSpot,
    }
}
