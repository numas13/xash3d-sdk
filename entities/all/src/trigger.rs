pub use xash3d_server::entities::trigger::Trigger;

define_export! {
    export_trigger as export if "trigger" {
        trigger = trigger::Trigger,
    }
}
