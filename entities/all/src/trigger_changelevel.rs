pub use xash3d_server::entities::trigger_changelevel::ChangeLevel;

define_export! {
    export_trigger_changelevel as export if "trigger-changelevel" {
        trigger_changelevel = trigger_changelevel::ChangeLevel,
    }
}
