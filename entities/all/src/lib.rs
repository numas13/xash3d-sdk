// #![feature(doc_cfg)]
#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

#[doc(hidden)]
pub use xash3d_server::export::export_entity;

#[macro_use]
mod macros;

import_with_export! {
    export_imported;

    use xash3d_entity_ambient::ambient_generic as ambient_generic if "ambient-generic";
    use xash3d_entity_button::func_button as func_button if "func-button";
    use xash3d_entity_button::func_rot_button as func_rot_button if "func-rot-button";
    use xash3d_entity_door::func_door as func_door if "func-door";
    use xash3d_entity_door::func_door_rotating as func_door_rotating if "func-door-rotating";
    use xash3d_entity_platform::func_plat as func_plat if "func-plat";
    use xash3d_entity_platform::func_platrot as func_platrot if "func-platrot";
    use xash3d_entity_sprite::env_sprite as env_sprite if "env-sprite";
    use xash3d_entity_tracktrain::func_tracktrain as func_tracktrain if "func-tracktrain";
    use xash3d_entity_tracktrain::path_track as path_track if "path-track";
    use xash3d_entity_train::func_train as func_train if "func-train";
    use xash3d_entity_train::path_corner as path_corner if "path-corner";
}

define_with_export! {
    export_defined;

    mod env_bubbles if "env-bubbles";
    mod env_debris if "env-debris";
    mod env_explosion if "env-explosion";
    mod env_fade if "env-fade";
    mod env_glow if "env-glow";
    mod env_message if "env-message" or "world";
    mod env_render if "env-render";
    mod env_shake if "env-shake";
    mod env_sound if "env-sound";
    mod env_spark if "env-spark" or "env-debris";
    mod func_friction if "func-friction";
    mod func_illusionary if "func-illusionary";
    mod func_ladder if "func-ladder";
    mod func_pendulum if "func-pendulum";
    mod func_rotating if "func-rotating";
    mod func_wall if "func-wall" or "func-wall-toggle";
    mod func_wall_toggle if "func-wall-toggle";
    mod func_water if "func-water";
    mod info_landmark if "info-landmark";
    mod info_node if "info-node" or "info-node-air";
    mod info_node_air if "info-node-air";
    mod info_player_deathmatch if "info-player-deathmatch";
    mod info_player_start if "info-player-start";
    mod info_target if "info-target";
    mod info_teleport_destination if "info-teleport-destination";
    mod infodecal if "infodecal";
    mod light if "light" or "light-spot" or "light-environment";
    mod light_spot if "light-spot";
    mod light_environment if "light-environment";
    mod multi_manager if "multi-manager" or "multisource";
    mod multisource if "multisource";
    mod spark_shower if "spark-shower";
    mod speaker if "speaker";
    mod target_cdaudio if "target-cdaudio";
    mod trigger if "trigger";
    mod trigger_auto if "trigger-auto";
    mod trigger_autosave if "trigger-autosave";
    mod trigger_cdaudio if "trigger-cdaudio";
    mod trigger_changelevel if "trigger-changelevel";
    mod trigger_endsection if "trigger-endsection";
    mod trigger_gravity if "trigger-gravity";
    mod trigger_hurt if "trigger-hurt";
    mod trigger_multiple if "trigger-multiple" or "trigger-once";
    mod trigger_once if "trigger-once";
    mod trigger_push if "trigger-push";
    mod trigger_relay if "trigger-relay";
    mod trigger_teleport if "trigger-teleport";
    mod trigger_transition if "trigger-transition";
}

define! {
    mod item if "item";
    mod player if "player";
    mod stub if "stub";
    mod world if "world";
    mod world_items if "world-items";
}

/// Export all enabled entities to the engine.
#[macro_export]
macro_rules! export_enabled {
    () => {
        $crate::export_imported!();
        $crate::export_defined!();
    };
}
