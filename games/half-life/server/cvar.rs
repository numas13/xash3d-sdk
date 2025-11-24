// use core::ffi::CStr;

use xash3d_server::cvar::{CvarStorage, SERVER, UNLOGGED};

use xash3d_server::prelude::*;

// pub static SV_GRAVITY: &CStr = c"sv_gravity";
// pub static SV_AIM: &CStr = c"sv_aim";
// pub static SV_ALLOW_AUTOAIM: &CStr = c"sv_allow_autoaim";
// pub static MP_FOOTSTEPS: &CStr = c"mp_footsteps";

macro_rules! define {
    (
        $( #[$register_attr:meta] )*
        $register_vis:vis fn $register:ident;

        $(
            $( #[$attr:meta] )*
            $vis:vis static $var:ident($name:expr, $value:expr $(, $flags:expr)? $(,)?);
        )*
    ) => {
        $( #[$register_attr] )*
        $register_vis fn $register(engine: &ServerEngine) {
            $( engine.register_cvar(&$var); )*
        }

        $(
            $( #[$attr] )*
            $vis static $var: CvarStorage = define!(@create $name, $value $(, $flags)?);
        )*
    };
    (
        $(
            $( #[$attr:meta] )*
            $vis:vis static $var:ident($name:expr, $value:expr $(, $flags:expr)? $(,)?);
        )*
    ) => {
        $(
            $( #[$attr] )*
            $vis static $var: CvarStorage = define!(@create $name, $value $(, $flags)?);
        )*
    };
    (@create $name:expr, $value:expr $(,)?) => {
        CvarStorage::new($name, $value)
    };
    (@create $name:expr, $value:expr, $flags:expr $(,)?) => {
        CvarStorage::with_flags($name, $value, $flags)
    };
}

define! {
    fn register;

    pub static DISPLAYSOUNDLIST(c"displaysoundlist", c"0");

    pub static MP_FRAGSLEFT(c"mp_fragsleft", c"0", SERVER.union(UNLOGGED));
    pub static MP_TIMELEFT(c"mp_timeleft", c"0" , SERVER.union(UNLOGGED));
    pub static MP_TEAMPLAY(c"mp_teamplay", c"0", SERVER);
    pub static MP_FRAGLIMIT(c"mp_fraglimit", c"0", SERVER);
    pub static MP_TIMELIMIT(c"mp_timelimit", c"0", SERVER);
    pub static MP_FRIENDLYFIRE(c"mp_friendlyfire", c"0", SERVER);
    pub static MP_FALLDAMAGE(c"mp_falldamage", c"0", SERVER);
    pub static MP_WEAPONSTAY(c"mp_weaponstay", c"0", SERVER);
    pub static MP_FORCERESPAWN(c"mp_forcerespawn", c"1", SERVER);
    pub static MP_FLASHLIGHT(c"mp_flashlight", c"0", SERVER);
    pub static MP_AUTOCROSSHAIR(c"mp_autocrosshair", c"1", SERVER);
    pub static DECALFREQUENCY(c"decalfrequency", c"30", SERVER);
    pub static MP_TEAMLIST(c"mp_teamlist", c"hgrunt;scientist", SERVER);
    pub static MP_TEAMOVERRIDE(c"mp_teamoverride", c"1");
    pub static MP_DEFAULTTEAM(c"mp_defaultteam", c"0");
    pub static MP_ALLOWMONSTERS(c"mp_allowmonsters", c"0", SERVER);
    pub static ALLOW_SPECTATORS(c"allow_spectators", c"0.0", SERVER);
    pub static MP_CHATTIME(c"mp_chattime", c"10", SERVER);

    pub static SK_AGRUNT_HEALTH1(c"sk_agrunt_health1", c"0");
    pub static SK_AGRUNT_HEALTH2(c"sk_agrunt_health2", c"0");
    pub static SK_AGRUNT_HEALTH3(c"sk_agrunt_health3", c"0");
    pub static SK_AGRUNT_DMG_PUNCH1(c"sk_agrunt_dmg_punch1", c"0");
    pub static SK_AGRUNT_DMG_PUNCH2(c"sk_agrunt_dmg_punch2", c"0");
    pub static SK_AGRUNT_DMG_PUNCH3(c"sk_agrunt_dmg_punch3", c"0");

    pub static SK_APACHE_HEALTH1(c"sk_apache_health1", c"0");
    pub static SK_APACHE_HEALTH2(c"sk_apache_health2", c"0");
    pub static SK_APACHE_HEALTH3(c"sk_apache_health3", c"0");

    pub static SK_BARNEY_HEALTH1(c"sk_barney_health1", c"0");
    pub static SK_BARNEY_HEALTH2(c"sk_barney_health2", c"0");
    pub static SK_BARNEY_HEALTH3(c"sk_barney_health3", c"0");

    pub static SK_BULLSQUID_HEALTH1(c"sk_bullsquid_health1", c"0");
    pub static SK_BULLSQUID_HEALTH2(c"sk_bullsquid_health2", c"0");
    pub static SK_BULLSQUID_HEALTH3(c"sk_bullsquid_health3", c"0");
    pub static SK_BULLSQUID_DMG_BITE1(c"sk_bullsquid_dmg_bite1", c"0");
    pub static SK_BULLSQUID_DMG_BITE2(c"sk_bullsquid_dmg_bite2", c"0");
    pub static SK_BULLSQUID_DMG_BITE3(c"sk_bullsquid_dmg_bite3", c"0");
    pub static SK_BULLSQUID_DMG_WHIP1(c"sk_bullsquid_dmg_whip1", c"0");
    pub static SK_BULLSQUID_DMG_WHIP2(c"sk_bullsquid_dmg_whip2", c"0");
    pub static SK_BULLSQUID_DMG_WHIP3(c"sk_bullsquid_dmg_whip3", c"0");
    pub static SK_BULLSQUID_DMG_SPIT1(c"sk_bullsquid_dmg_spit1", c"0");
    pub static SK_BULLSQUID_DMG_SPIT2(c"sk_bullsquid_dmg_spit2", c"0");
    pub static SK_BULLSQUID_DMG_SPIT3(c"sk_bullsquid_dmg_spit3", c"0");

    pub static SK_BIGMOMMA_HEALTH_FACTOR1(c"sk_bigmomma_health_factor1", c"1.0");
    pub static SK_BIGMOMMA_HEALTH_FACTOR2(c"sk_bigmomma_health_factor2", c"1.0");
    pub static SK_BIGMOMMA_HEALTH_FACTOR3(c"sk_bigmomma_health_factor3", c"1.0");
    pub static SK_BIGMOMMA_DMG_SLASH1(c"sk_bigmomma_dmg_slash1", c"50");
    pub static SK_BIGMOMMA_DMG_SLASH2(c"sk_bigmomma_dmg_slash2", c"50");
    pub static SK_BIGMOMMA_DMG_SLASH3(c"sk_bigmomma_dmg_slash3", c"50");
    pub static SK_BIGMOMMA_DMG_BLAST1(c"sk_bigmomma_dmg_blast1", c"100");
    pub static SK_BIGMOMMA_DMG_BLAST2(c"sk_bigmomma_dmg_blast2", c"100");
    pub static SK_BIGMOMMA_DMG_BLAST3(c"sk_bigmomma_dmg_blast3", c"100");
    pub static SK_BIGMOMMA_RADIUS_BLAST1(c"sk_bigmomma_radius_blast1", c"250");
    pub static SK_BIGMOMMA_RADIUS_BLAST2(c"sk_bigmomma_radius_blast2", c"250");
    pub static SK_BIGMOMMA_RADIUS_BLAST3(c"sk_bigmomma_radius_blast3", c"250");

    pub static SK_GARGANTUA_HEALTH1(c"sk_gargantua_health1", c"0");
    pub static SK_GARGANTUA_HEALTH2(c"sk_gargantua_health2", c"0");
    pub static SK_GARGANTUA_HEALTH3(c"sk_gargantua_health3", c"0");
    pub static SK_GARGANTUA_DMG_SLASH1(c"sk_gargantua_dmg_slash1", c"0");
    pub static SK_GARGANTUA_DMG_SLASH2(c"sk_gargantua_dmg_slash2", c"0");
    pub static SK_GARGANTUA_DMG_SLASH3(c"sk_gargantua_dmg_slash3", c"0");
    pub static SK_GARGANTUA_DMG_FIRE1(c"sk_gargantua_dmg_fire1", c"0");
    pub static SK_GARGANTUA_DMG_FIRE2(c"sk_gargantua_dmg_fire2", c"0");
    pub static SK_GARGANTUA_DMG_FIRE3(c"sk_gargantua_dmg_fire3", c"0");
    pub static SK_GARGANTUA_DMG_STOMP1(c"sk_gargantua_dmg_stomp1", c"0");
    pub static SK_GARGANTUA_DMG_STOMP2(c"sk_gargantua_dmg_stomp2", c"0");
    pub static SK_GARGANTUA_DMG_STOMP3(c"sk_gargantua_dmg_stomp3", c"0");

    pub static SK_HASSASSIN_HEALTH1(c"sk_hassassin_health1", c"0");
    pub static SK_HASSASSIN_HEALTH2(c"sk_hassassin_health2", c"0");
    pub static SK_HASSASSIN_HEALTH3(c"sk_hassassin_health3", c"0");

    pub static SK_HEADCRAB_HEALTH1(c"sk_headcrab_health1", c"0");
    pub static SK_HEADCRAB_HEALTH2(c"sk_headcrab_health2", c"0");
    pub static SK_HEADCRAB_HEALTH3(c"sk_headcrab_health3", c"0");
    pub static SK_HEADCRAB_DMG_BITE1(c"sk_headcrab_dmg_bite1", c"0");
    pub static SK_HEADCRAB_DMG_BITE2(c"sk_headcrab_dmg_bite2", c"0");
    pub static SK_HEADCRAB_DMG_BITE3(c"sk_headcrab_dmg_bite3", c"0");

    pub static SK_HGRUNT_HEALTH1(c"sk_hgrunt_health1", c"0");
    pub static SK_HGRUNT_HEALTH2(c"sk_hgrunt_health2", c"0");
    pub static SK_HGRUNT_HEALTH3(c"sk_hgrunt_health3", c"0");
    pub static SK_HGRUNT_KICK1(c"sk_hgrunt_kick1", c"0");
    pub static SK_HGRUNT_KICK2(c"sk_hgrunt_kick2", c"0");
    pub static SK_HGRUNT_KICK3(c"sk_hgrunt_kick3", c"0");
    pub static SK_HGRUNT_PELLETS1(c"sk_hgrunt_pellets1", c"0");
    pub static SK_HGRUNT_PELLETS2(c"sk_hgrunt_pellets2", c"0");
    pub static SK_HGRUNT_PELLETS3(c"sk_hgrunt_pellets3", c"0");
    pub static SK_HGRUNT_GSPEED1(c"sk_hgrunt_gspeed1", c"0");
    pub static SK_HGRUNT_GSPEED2(c"sk_hgrunt_gspeed2", c"0");
    pub static SK_HGRUNT_GSPEED3(c"sk_hgrunt_gspeed3", c"0");

    pub static SK_HOUNDEYE_HEALTH1(c"sk_houndeye_health1", c"0");
    pub static SK_HOUNDEYE_HEALTH2(c"sk_houndeye_health2", c"0");
    pub static SK_HOUNDEYE_HEALTH3(c"sk_houndeye_health3", c"0");
    pub static SK_HOUNDEYE_DMG_BLAST1(c"sk_houndeye_dmg_blast1", c"0");
    pub static SK_HOUNDEYE_DMG_BLAST2(c"sk_houndeye_dmg_blast2", c"0");
    pub static SK_HOUNDEYE_DMG_BLAST3(c"sk_houndeye_dmg_blast3", c"0");

    pub static SK_ISLAVE_HEALTH1(c"sk_islave_health1", c"0");
    pub static SK_ISLAVE_HEALTH2(c"sk_islave_health2", c"0");
    pub static SK_ISLAVE_HEALTH3(c"sk_islave_health3", c"0");
    pub static SK_ISLAVE_DMG_CLAW1(c"sk_islave_dmg_claw1", c"0");
    pub static SK_ISLAVE_DMG_CLAW2(c"sk_islave_dmg_claw2", c"0");
    pub static SK_ISLAVE_DMG_CLAW3(c"sk_islave_dmg_claw3", c"0");

    pub static SK_ISLAVE_DMG_CLAWRAKE1(c"sk_islave_dmg_clawrake1", c"0");
    pub static SK_ISLAVE_DMG_CLAWRAKE2(c"sk_islave_dmg_clawrake2", c"0");
    pub static SK_ISLAVE_DMG_CLAWRAKE3(c"sk_islave_dmg_clawrake3", c"0");
    pub static SK_ISLAVE_DMG_ZAP1(c"sk_islave_dmg_zap1", c"0");
    pub static SK_ISLAVE_DMG_ZAP2(c"sk_islave_dmg_zap2", c"0");
    pub static SK_ISLAVE_DMG_ZAP3(c"sk_islave_dmg_zap3", c"0");

    pub static SK_ICHTHYOSAUR_HEALTH1(c"sk_ichthyosaur_health1", c"0");
    pub static SK_ICHTHYOSAUR_HEALTH2(c"sk_ichthyosaur_health2", c"0");
    pub static SK_ICHTHYOSAUR_HEALTH3(c"sk_ichthyosaur_health3", c"0");
    pub static SK_ICHTHYOSAUR_SHAKE1(c"sk_ichthyosaur_shake1", c"0");
    pub static SK_ICHTHYOSAUR_SHAKE2(c"sk_ichthyosaur_shake2", c"0");
    pub static SK_ICHTHYOSAUR_SHAKE3(c"sk_ichthyosaur_shake3", c"0");

    pub static SK_LEECH_HEALTH1(c"sk_leech_health1", c"0");
    pub static SK_LEECH_HEALTH2(c"sk_leech_health2", c"0");
    pub static SK_LEECH_HEALTH3(c"sk_leech_health3", c"0");
    pub static SK_LEECH_DMG_BITE1(c"sk_leech_dmg_bite1", c"0");
    pub static SK_LEECH_DMG_BITE2(c"sk_leech_dmg_bite2", c"0");
    pub static SK_LEECH_DMG_BITE3(c"sk_leech_dmg_bite3", c"0");

    pub static SK_CONTROLLER_HEALTH1(c"sk_controller_health1", c"0");
    pub static SK_CONTROLLER_HEALTH2(c"sk_controller_health2", c"0");
    pub static SK_CONTROLLER_HEALTH3(c"sk_controller_health3", c"0");

    pub static SK_CONTROLLER_DMGZAP1(c"sk_controller_dmgzap1", c"0");
    pub static SK_CONTROLLER_DMGZAP2(c"sk_controller_dmgzap2", c"0");
    pub static SK_CONTROLLER_DMGZAP3(c"sk_controller_dmgzap3", c"0");

    pub static SK_CONTROLLER_SPEEDBALL1(c"sk_controller_speedball1", c"0");
    pub static SK_CONTROLLER_SPEEDBALL2(c"sk_controller_speedball2", c"0");
    pub static SK_CONTROLLER_SPEEDBALL3(c"sk_controller_speedball3", c"0");

    pub static SK_CONTROLLER_DMGBALL1(c"sk_controller_dmgball1", c"0");
    pub static SK_CONTROLLER_DMGBALL2(c"sk_controller_dmgball2", c"0");
    pub static SK_CONTROLLER_DMGBALL3(c"sk_controller_dmgball3", c"0");

    pub static SK_NIHILANTH_HEALTH1(c"sk_nihilanth_health1", c"0");
    pub static SK_NIHILANTH_HEALTH2(c"sk_nihilanth_health2", c"0");
    pub static SK_NIHILANTH_HEALTH3(c"sk_nihilanth_health3", c"0");
    pub static SK_NIHILANTH_ZAP1(c"sk_nihilanth_zap1", c"0");
    pub static SK_NIHILANTH_ZAP2(c"sk_nihilanth_zap2", c"0");
    pub static SK_NIHILANTH_ZAP3(c"sk_nihilanth_zap3", c"0");

    pub static SK_SCIENTIST_HEALTH1(c"sk_scientist_health1", c"0");
    pub static SK_SCIENTIST_HEALTH2(c"sk_scientist_health2", c"0");
    pub static SK_SCIENTIST_HEALTH3(c"sk_scientist_health3", c"0");

    pub static SK_SNARK_HEALTH1(c"sk_snark_health1", c"0");
    pub static SK_SNARK_HEALTH2(c"sk_snark_health2", c"0");
    pub static SK_SNARK_HEALTH3(c"sk_snark_health3", c"0");
    pub static SK_SNARK_DMG_BITE1(c"sk_snark_dmg_bite1", c"0");
    pub static SK_SNARK_DMG_BITE2(c"sk_snark_dmg_bite2", c"0");
    pub static SK_SNARK_DMG_BITE3(c"sk_snark_dmg_bite3", c"0");
    pub static SK_SNARK_DMG_POP1(c"sk_snark_dmg_pop1", c"0");
    pub static SK_SNARK_DMG_POP2(c"sk_snark_dmg_pop2", c"0");
    pub static SK_SNARK_DMG_POP3(c"sk_snark_dmg_pop3", c"0");

    pub static SK_ZOMBIE_HEALTH1(c"sk_zombie_health1", c"0");
    pub static SK_ZOMBIE_HEALTH2(c"sk_zombie_health2", c"0");
    pub static SK_ZOMBIE_HEALTH3(c"sk_zombie_health3", c"0");
    pub static SK_ZOMBIE_DMG_ONE_SLASH1(c"sk_zombie_dmg_one_slash1", c"0");
    pub static SK_ZOMBIE_DMG_ONE_SLASH2(c"sk_zombie_dmg_one_slash2", c"0");
    pub static SK_ZOMBIE_DMG_ONE_SLASH3(c"sk_zombie_dmg_one_slash3", c"0");
    pub static SK_ZOMBIE_DMG_BOTH_SLASH1(c"sk_zombie_dmg_both_slash1", c"0");
    pub static SK_ZOMBIE_DMG_BOTH_SLASH2(c"sk_zombie_dmg_both_slash2", c"0");
    pub static SK_ZOMBIE_DMG_BOTH_SLASH3(c"sk_zombie_dmg_both_slash3", c"0");

    pub static SK_TURRET_HEALTH1(c"sk_turret_health1", c"0");
    pub static SK_TURRET_HEALTH2(c"sk_turret_health2", c"0");
    pub static SK_TURRET_HEALTH3(c"sk_turret_health3", c"0");

    pub static SK_MINITURRET_HEALTH1(c"sk_miniturret_health1", c"0");
    pub static SK_MINITURRET_HEALTH2(c"sk_miniturret_health2", c"0");
    pub static SK_MINITURRET_HEALTH3(c"sk_miniturret_health3", c"0");

    pub static SK_SENTRY_HEALTH1(c"sk_sentry_health1", c"0");
    pub static SK_SENTRY_HEALTH2(c"sk_sentry_health2", c"0");
    pub static SK_SENTRY_HEALTH3(c"sk_sentry_health3", c"0");

    pub static SK_PLR_CROWBAR1(c"sk_plr_crowbar1", c"0");
    pub static SK_PLR_CROWBAR2(c"sk_plr_crowbar2", c"0");
    pub static SK_PLR_CROWBAR3(c"sk_plr_crowbar3", c"0");

    pub static SK_PLR_9MM_BULLET1(c"sk_plr_9mm_bullet1", c"0");
    pub static SK_PLR_9MM_BULLET2(c"sk_plr_9mm_bullet2", c"0");
    pub static SK_PLR_9MM_BULLET3(c"sk_plr_9mm_bullet3", c"0");

    pub static SK_PLR_357_BULLET1(c"sk_plr_357_bullet1", c"0");
    pub static SK_PLR_357_BULLET2(c"sk_plr_357_bullet2", c"0");
    pub static SK_PLR_357_BULLET3(c"sk_plr_357_bullet3", c"0");

    pub static SK_PLR_9MMAR_BULLET1(c"sk_plr_9mmAR_bullet1", c"0");
    pub static SK_PLR_9MMAR_BULLET2(c"sk_plr_9mmAR_bullet2", c"0");
    pub static SK_PLR_9MMAR_BULLET3(c"sk_plr_9mmAR_bullet3", c"0");

    pub static SK_PLR_9MMAR_GRENADE1(c"sk_plr_9mmAR_grenade1", c"0");
    pub static SK_PLR_9MMAR_GRENADE2(c"sk_plr_9mmAR_grenade2", c"0");
    pub static SK_PLR_9MMAR_GRENADE3(c"sk_plr_9mmAR_grenade3", c"0");

    pub static SK_PLR_BUCKSHOT1(c"sk_plr_buckshot1", c"0");
    pub static SK_PLR_BUCKSHOT2(c"sk_plr_buckshot2", c"0");
    pub static SK_PLR_BUCKSHOT3(c"sk_plr_buckshot3", c"0");

    pub static SK_PLR_XBOW_BOLT_CLIENT1(c"sk_plr_xbow_bolt_client1", c"0");
    pub static SK_PLR_XBOW_BOLT_CLIENT2(c"sk_plr_xbow_bolt_client2", c"0");
    pub static SK_PLR_XBOW_BOLT_CLIENT3(c"sk_plr_xbow_bolt_client3", c"0");

    pub static SK_PLR_XBOW_BOLT_MONSTER1(c"sk_plr_xbow_bolt_monster1", c"0");
    pub static SK_PLR_XBOW_BOLT_MONSTER2(c"sk_plr_xbow_bolt_monster2", c"0");
    pub static SK_PLR_XBOW_BOLT_MONSTER3(c"sk_plr_xbow_bolt_monster3", c"0");

    pub static SK_PLR_RPG1(c"sk_plr_rpg1", c"0");
    pub static SK_PLR_RPG2(c"sk_plr_rpg2", c"0");
    pub static SK_PLR_RPG3(c"sk_plr_rpg3", c"0");

    pub static SK_PLR_GAUSS1(c"sk_plr_gauss1", c"0");
    pub static SK_PLR_GAUSS2(c"sk_plr_gauss2", c"0");
    pub static SK_PLR_GAUSS3(c"sk_plr_gauss3", c"0");

    pub static SK_PLR_EGON_NARROW1(c"sk_plr_egon_narrow1", c"0");
    pub static SK_PLR_EGON_NARROW2(c"sk_plr_egon_narrow2", c"0");
    pub static SK_PLR_EGON_NARROW3(c"sk_plr_egon_narrow3", c"0");
    pub static SK_PLR_EGON_WIDE1(c"sk_plr_egon_wide1", c"0");
    pub static SK_PLR_EGON_WIDE2(c"sk_plr_egon_wide2", c"0");
    pub static SK_PLR_EGON_WIDE3(c"sk_plr_egon_wide3", c"0");

    pub static SK_PLR_HAND_GRENADE1(c"sk_plr_hand_grenade1", c"0");
    pub static SK_PLR_HAND_GRENADE2(c"sk_plr_hand_grenade2", c"0");
    pub static SK_PLR_HAND_GRENADE3(c"sk_plr_hand_grenade3", c"0");

    pub static SK_PLR_SATCHEL1(c"sk_plr_satchel1", c"0");
    pub static SK_PLR_SATCHEL2(c"sk_plr_satchel2", c"0");
    pub static SK_PLR_SATCHEL3(c"sk_plr_satchel3", c"0");

    pub static SK_PLR_TRIPMINE1(c"sk_plr_tripmine1", c"0");
    pub static SK_PLR_TRIPMINE2(c"sk_plr_tripmine2", c"0");
    pub static SK_PLR_TRIPMINE3(c"sk_plr_tripmine3", c"0");

    pub static SK_12MM_BULLET1(c"sk_12mm_bullet1", c"0");
    pub static SK_12MM_BULLET2(c"sk_12mm_bullet2", c"0");
    pub static SK_12MM_BULLET3(c"sk_12mm_bullet3", c"0");

    pub static SK_9MMAR_BULLET1(c"sk_9mmAR_bullet1", c"0");
    pub static SK_9MMAR_BULLET2(c"sk_9mmAR_bullet2", c"0");
    pub static SK_9MMAR_BULLET3(c"sk_9mmAR_bullet3", c"0");

    pub static SK_9MM_BULLET1(c"sk_9mm_bullet1", c"0");
    pub static SK_9MM_BULLET2(c"sk_9mm_bullet2", c"0");
    pub static SK_9MM_BULLET3(c"sk_9mm_bullet3", c"0");

    pub static SK_HORNET_DMG1(c"sk_hornet_dmg1", c"0");
    pub static SK_HORNET_DMG2(c"sk_hornet_dmg2", c"0");
    pub static SK_HORNET_DMG3(c"sk_hornet_dmg3", c"0");

    pub static SK_SUITCHARGER1(c"sk_suitcharger1", c"0");
    pub static SK_SUITCHARGER2(c"sk_suitcharger2", c"0");
    pub static SK_SUITCHARGER3(c"sk_suitcharger3", c"0");

    pub static SK_BATTERY1(c"sk_battery1", c"0");
    pub static SK_BATTERY2(c"sk_battery2", c"0");
    pub static SK_BATTERY3(c"sk_battery3", c"0");

    pub static SK_HEALTHCHARGER1(c"sk_healthcharger1", c"0");
    pub static SK_HEALTHCHARGER2(c"sk_healthcharger2", c"0");
    pub static SK_HEALTHCHARGER3(c"sk_healthcharger3", c"0");

    pub static SK_HEALTHKIT1(c"sk_healthkit1", c"0");
    pub static SK_HEALTHKIT2(c"sk_healthkit2", c"0");
    pub static SK_HEALTHKIT3(c"sk_healthkit3", c"0");

    pub static SK_SCIENTIST_HEAL1(c"sk_scientist_heal1", c"0");
    pub static SK_SCIENTIST_HEAL2(c"sk_scientist_heal2", c"0");
    pub static SK_SCIENTIST_HEAL3(c"sk_scientist_heal3", c"0");

    pub static SK_MONSTER_HEAD1(c"sk_monster_head1", c"2");
    pub static SK_MONSTER_HEAD2(c"sk_monster_head2", c"2");
    pub static SK_MONSTER_HEAD3(c"sk_monster_head3", c"2");

    pub static SK_MONSTER_CHEST1(c"sk_monster_chest1", c"1");
    pub static SK_MONSTER_CHEST2(c"sk_monster_chest2", c"1");
    pub static SK_MONSTER_CHEST3(c"sk_monster_chest3", c"1");

    pub static SK_MONSTER_STOMACH1(c"sk_monster_stomach1", c"1");
    pub static SK_MONSTER_STOMACH2(c"sk_monster_stomach2", c"1");
    pub static SK_MONSTER_STOMACH3(c"sk_monster_stomach3", c"1");

    pub static SK_MONSTER_ARM1(c"sk_monster_arm1", c"1");
    pub static SK_MONSTER_ARM2(c"sk_monster_arm2", c"1");
    pub static SK_MONSTER_ARM3(c"sk_monster_arm3", c"1");

    pub static SK_MONSTER_LEG1(c"sk_monster_leg1", c"1");
    pub static SK_MONSTER_LEG2(c"sk_monster_leg2", c"1");
    pub static SK_MONSTER_LEG3(c"sk_monster_leg3", c"1");

    pub static SK_PLAYER_HEAD1(c"sk_player_head1", c"2");
    pub static SK_PLAYER_HEAD2(c"sk_player_head2", c"2");
    pub static SK_PLAYER_HEAD3(c"sk_player_head3", c"2");

    pub static SK_PLAYER_CHEST1(c"sk_player_chest1", c"1");
    pub static SK_PLAYER_CHEST2(c"sk_player_chest2", c"1");
    pub static SK_PLAYER_CHEST3(c"sk_player_chest3", c"1");

    pub static SK_PLAYER_STOMACH1(c"sk_player_stomach1", c"1");
    pub static SK_PLAYER_STOMACH2(c"sk_player_stomach2", c"1");
    pub static SK_PLAYER_STOMACH3(c"sk_player_stomach3", c"1");

    pub static SK_PLAYER_ARM1(c"sk_player_arm1", c"1");
    pub static SK_PLAYER_ARM2(c"sk_player_arm2", c"1");
    pub static SK_PLAYER_ARM3(c"sk_player_arm3", c"1");

    pub static SK_PLAYER_LEG1(c"sk_player_leg1", c"1");
    pub static SK_PLAYER_LEG2(c"sk_player_leg2", c"1");
    pub static SK_PLAYER_LEG3(c"sk_player_leg3", c"1");

    pub static SV_PUSHABLE_FIXED_TICK_FUDGE(c"sv_pushable_fixed_tick_fudge", c"15");

    pub static SV_BUSTERS(c"sv_busters", c"0");
}

pub fn init(engine: &ServerEngine) {
    register(engine);
    engine.server_command(c"exec skill.cfg\n");
}
