use sv::cvar::define;

use sv::prelude::*;

#[allow(dead_code)]
mod flags {
    use sv::cvar::CVarFlags;

    pub const NONE: CVarFlags = CVarFlags::NONE;
    pub const ARCHIVE: CVarFlags = CVarFlags::ARCHIVE;
    pub const USERINFO: CVarFlags = CVarFlags::USERINFO;
    pub const SERVER: CVarFlags = CVarFlags::SERVER;
    pub const EXTDLL: CVarFlags = CVarFlags::EXTDLL;
    pub const CLIENTDLL: CVarFlags = CVarFlags::CLIENTDLL;
    pub const PROTECTED: CVarFlags = CVarFlags::PROTECTED;
    pub const SPONLY: CVarFlags = CVarFlags::SPONLY;
    pub const PRINTABLEONLY: CVarFlags = CVarFlags::PRINTABLEONLY;
    pub const UNLOGGED: CVarFlags = CVarFlags::UNLOGGED;
    pub const NOEXTRAWHITEPACE: CVarFlags = CVarFlags::NOEXTRAWHITEPACE;
    pub const PRIVILEGED: CVarFlags = CVarFlags::PRIVILEGED;
    pub const FILTERSTUFFTEXT: CVarFlags = CVarFlags::FILTERSTUFFTEXT;
    pub const FILTERCHARS: CVarFlags = CVarFlags::FILTERCHARS;
    pub const NOBADPATHS: CVarFlags = CVarFlags::NOBADPATHS;
}

macro_rules! define_server {
    ($($(#[$meta:meta])* $vis:vis static $name:ident($value:expr $(, $flags:expr)?);)*) => {
        mod register {
            $(
                #[allow(non_upper_case_globals)]
                $(#[$meta])*
                static mut $name: sv::cvar::cvar_s = {
                    use $crate::cvar::flags::*;

                    #[allow(unused_variables)]
                    let flags = NONE;
                    $(let flags = $flags;)?

                    sv::cvar::cvar_s {
                        name: sv::macros::cstringify!($name).as_ptr(),
                        string: $value.as_ptr() as *mut core::ffi::c_char,
                        value: 0.0,
                        flags,
                        next: core::ptr::null_mut(),
                    }
                };
            )*

            pub(super) fn init() {
                let engine = sv::instance::engine();
                unsafe {
                    $(engine.cvar_register(&mut *core::ptr::addr_of_mut!($name));)*
                }
            }
        }

        sv::cvar::define! {
            $($(#[$meta])* $vis static $name;)*
        }

        fn get_ptrs() {
            $($name.get_ptr();)*
        }
    };
}

define! {
    pub static sv_gravity;
    pub static sv_aim;
    pub static sv_allow_autoaim;
    pub static mp_footsteps;
}

define_server! {
    pub static displaysoundlist(c"0");

    pub static mp_fragsleft(c"0", SERVER.union(UNLOGGED));
    pub static mp_timeleft(c"0" , SERVER.union(UNLOGGED));
    pub static mp_teamplay(c"0", SERVER);
    pub static mp_fraglimit(c"0", SERVER);
    pub static mp_timelimit(c"0", SERVER);
    pub static mp_friendlyfire(c"0", SERVER);
    pub static mp_falldamage(c"0", SERVER);
    pub static mp_weaponstay(c"0", SERVER);
    pub static mp_forcerespawn(c"1", SERVER);
    pub static mp_flashlight(c"0", SERVER);
    pub static mp_autocrosshair(c"1", SERVER);
    pub static decalfrequency(c"30", SERVER);
    pub static mp_teamlist(c"hgrunt;scientist", SERVER);
    pub static mp_teamoverride(c"1");
    pub static mp_defaultteam(c"0");
    pub static mp_allowmonsters(c"0", SERVER);
    pub static allow_spectators(c"0.0", SERVER);
    pub static mp_chattime(c"10", SERVER);

    pub static sk_agrunt_health1(c"0");
    pub static sk_agrunt_health2(c"0");
    pub static sk_agrunt_health3(c"0");
    pub static sk_agrunt_dmg_punch1(c"0");
    pub static sk_agrunt_dmg_punch2(c"0");
    pub static sk_agrunt_dmg_punch3(c"0");

    pub static sk_apache_health1(c"0");
    pub static sk_apache_health2(c"0");
    pub static sk_apache_health3(c"0");

    pub static sk_barney_health1(c"0");
    pub static sk_barney_health2(c"0");
    pub static sk_barney_health3(c"0");

    pub static sk_bullsquid_health1(c"0");
    pub static sk_bullsquid_health2(c"0");
    pub static sk_bullsquid_health3(c"0");
    pub static sk_bullsquid_dmg_bite1(c"0");
    pub static sk_bullsquid_dmg_bite2(c"0");
    pub static sk_bullsquid_dmg_bite3(c"0");
    pub static sk_bullsquid_dmg_whip1(c"0");
    pub static sk_bullsquid_dmg_whip2(c"0");
    pub static sk_bullsquid_dmg_whip3(c"0");
    pub static sk_bullsquid_dmg_spit1(c"0");
    pub static sk_bullsquid_dmg_spit2(c"0");
    pub static sk_bullsquid_dmg_spit3(c"0");

    pub static sk_bigmomma_health_factor1(c"1.0");
    pub static sk_bigmomma_health_factor2(c"1.0");
    pub static sk_bigmomma_health_factor3(c"1.0");
    pub static sk_bigmomma_dmg_slash1(c"50");
    pub static sk_bigmomma_dmg_slash2(c"50");
    pub static sk_bigmomma_dmg_slash3(c"50");
    pub static sk_bigmomma_dmg_blast1(c"100");
    pub static sk_bigmomma_dmg_blast2(c"100");
    pub static sk_bigmomma_dmg_blast3(c"100");
    pub static sk_bigmomma_radius_blast1(c"250");
    pub static sk_bigmomma_radius_blast2(c"250");
    pub static sk_bigmomma_radius_blast3(c"250");

    pub static sk_gargantua_health1(c"0");
    pub static sk_gargantua_health2(c"0");
    pub static sk_gargantua_health3(c"0");
    pub static sk_gargantua_dmg_slash1(c"0");
    pub static sk_gargantua_dmg_slash2(c"0");
    pub static sk_gargantua_dmg_slash3(c"0");
    pub static sk_gargantua_dmg_fire1(c"0");
    pub static sk_gargantua_dmg_fire2(c"0");
    pub static sk_gargantua_dmg_fire3(c"0");
    pub static sk_gargantua_dmg_stomp1(c"0");
    pub static sk_gargantua_dmg_stomp2(c"0");
    pub static sk_gargantua_dmg_stomp3(c"0");

    pub static sk_hassassin_health1(c"0");
    pub static sk_hassassin_health2(c"0");
    pub static sk_hassassin_health3(c"0");

    pub static sk_headcrab_health1(c"0");
    pub static sk_headcrab_health2(c"0");
    pub static sk_headcrab_health3(c"0");
    pub static sk_headcrab_dmg_bite1(c"0");
    pub static sk_headcrab_dmg_bite2(c"0");
    pub static sk_headcrab_dmg_bite3(c"0");

    pub static sk_hgrunt_health1(c"0");
    pub static sk_hgrunt_health2(c"0");
    pub static sk_hgrunt_health3(c"0");
    pub static sk_hgrunt_kick1(c"0");
    pub static sk_hgrunt_kick2(c"0");
    pub static sk_hgrunt_kick3(c"0");
    pub static sk_hgrunt_pellets1(c"0");
    pub static sk_hgrunt_pellets2(c"0");
    pub static sk_hgrunt_pellets3(c"0");
    pub static sk_hgrunt_gspeed1(c"0");
    pub static sk_hgrunt_gspeed2(c"0");
    pub static sk_hgrunt_gspeed3(c"0");

    pub static sk_houndeye_health1(c"0");
    pub static sk_houndeye_health2(c"0");
    pub static sk_houndeye_health3(c"0");
    pub static sk_houndeye_dmg_blast1(c"0");
    pub static sk_houndeye_dmg_blast2(c"0");
    pub static sk_houndeye_dmg_blast3(c"0");

    pub static sk_islave_health1(c"0");
    pub static sk_islave_health2(c"0");
    pub static sk_islave_health3(c"0");
    pub static sk_islave_dmg_claw1(c"0");
    pub static sk_islave_dmg_claw2(c"0");
    pub static sk_islave_dmg_claw3(c"0");

    pub static sk_islave_dmg_clawrake1(c"0");
    pub static sk_islave_dmg_clawrake2(c"0");
    pub static sk_islave_dmg_clawrake3(c"0");
    pub static sk_islave_dmg_zap1(c"0");
    pub static sk_islave_dmg_zap2(c"0");
    pub static sk_islave_dmg_zap3(c"0");

    pub static sk_ichthyosaur_health1(c"0");
    pub static sk_ichthyosaur_health2(c"0");
    pub static sk_ichthyosaur_health3(c"0");
    pub static sk_ichthyosaur_shake1(c"0");
    pub static sk_ichthyosaur_shake2(c"0");
    pub static sk_ichthyosaur_shake3(c"0");

    pub static sk_leech_health1(c"0");
    pub static sk_leech_health2(c"0");
    pub static sk_leech_health3(c"0");
    pub static sk_leech_dmg_bite1(c"0");
    pub static sk_leech_dmg_bite2(c"0");
    pub static sk_leech_dmg_bite3(c"0");

    pub static sk_controller_health1(c"0");
    pub static sk_controller_health2(c"0");
    pub static sk_controller_health3(c"0");

    pub static sk_controller_dmgzap1(c"0");
    pub static sk_controller_dmgzap2(c"0");
    pub static sk_controller_dmgzap3(c"0");

    pub static sk_controller_speedball1(c"0");
    pub static sk_controller_speedball2(c"0");
    pub static sk_controller_speedball3(c"0");

    pub static sk_controller_dmgball1(c"0");
    pub static sk_controller_dmgball2(c"0");
    pub static sk_controller_dmgball3(c"0");

    pub static sk_nihilanth_health1(c"0");
    pub static sk_nihilanth_health2(c"0");
    pub static sk_nihilanth_health3(c"0");
    pub static sk_nihilanth_zap1(c"0");
    pub static sk_nihilanth_zap2(c"0");
    pub static sk_nihilanth_zap3(c"0");

    pub static sk_scientist_health1(c"0");
    pub static sk_scientist_health2(c"0");
    pub static sk_scientist_health3(c"0");

    pub static sk_snark_health1(c"0");
    pub static sk_snark_health2(c"0");
    pub static sk_snark_health3(c"0");
    pub static sk_snark_dmg_bite1(c"0");
    pub static sk_snark_dmg_bite2(c"0");
    pub static sk_snark_dmg_bite3(c"0");
    pub static sk_snark_dmg_pop1(c"0");
    pub static sk_snark_dmg_pop2(c"0");
    pub static sk_snark_dmg_pop3(c"0");

    pub static sk_zombie_health1(c"0");
    pub static sk_zombie_health2(c"0");
    pub static sk_zombie_health3(c"0");
    pub static sk_zombie_dmg_one_slash1(c"0");
    pub static sk_zombie_dmg_one_slash2(c"0");
    pub static sk_zombie_dmg_one_slash3(c"0");
    pub static sk_zombie_dmg_both_slash1(c"0");
    pub static sk_zombie_dmg_both_slash2(c"0");
    pub static sk_zombie_dmg_both_slash3(c"0");

    pub static sk_turret_health1(c"0");
    pub static sk_turret_health2(c"0");
    pub static sk_turret_health3(c"0");

    pub static sk_miniturret_health1(c"0");
    pub static sk_miniturret_health2(c"0");
    pub static sk_miniturret_health3(c"0");

    pub static sk_sentry_health1(c"0");
    pub static sk_sentry_health2(c"0");
    pub static sk_sentry_health3(c"0");

    pub static sk_plr_crowbar1(c"0");
    pub static sk_plr_crowbar2(c"0");
    pub static sk_plr_crowbar3(c"0");

    pub static sk_plr_9mm_bullet1(c"0");
    pub static sk_plr_9mm_bullet2(c"0");
    pub static sk_plr_9mm_bullet3(c"0");

    pub static sk_plr_357_bullet1(c"0");
    pub static sk_plr_357_bullet2(c"0");
    pub static sk_plr_357_bullet3(c"0");

    pub static sk_plr_9mmAR_bullet1(c"0");
    pub static sk_plr_9mmAR_bullet2(c"0");
    pub static sk_plr_9mmAR_bullet3(c"0");

    pub static sk_plr_9mmAR_grenade1(c"0");
    pub static sk_plr_9mmAR_grenade2(c"0");
    pub static sk_plr_9mmAR_grenade3(c"0");

    pub static sk_plr_buckshot1(c"0");
    pub static sk_plr_buckshot2(c"0");
    pub static sk_plr_buckshot3(c"0");

    pub static sk_plr_xbow_bolt_client1(c"0");
    pub static sk_plr_xbow_bolt_client2(c"0");
    pub static sk_plr_xbow_bolt_client3(c"0");

    pub static sk_plr_xbow_bolt_monster1(c"0");
    pub static sk_plr_xbow_bolt_monster2(c"0");
    pub static sk_plr_xbow_bolt_monster3(c"0");

    pub static sk_plr_rpg1(c"0");
    pub static sk_plr_rpg2(c"0");
    pub static sk_plr_rpg3(c"0");

    pub static sk_plr_gauss1(c"0");
    pub static sk_plr_gauss2(c"0");
    pub static sk_plr_gauss3(c"0");

    pub static sk_plr_egon_narrow1(c"0");
    pub static sk_plr_egon_narrow2(c"0");
    pub static sk_plr_egon_narrow3(c"0");
    pub static sk_plr_egon_wide1(c"0");
    pub static sk_plr_egon_wide2(c"0");
    pub static sk_plr_egon_wide3(c"0");

    pub static sk_plr_hand_grenade1(c"0");
    pub static sk_plr_hand_grenade2(c"0");
    pub static sk_plr_hand_grenade3(c"0");

    pub static sk_plr_satchel1(c"0");
    pub static sk_plr_satchel2(c"0");
    pub static sk_plr_satchel3(c"0");

    pub static sk_plr_tripmine1(c"0");
    pub static sk_plr_tripmine2(c"0");
    pub static sk_plr_tripmine3(c"0");

    pub static sk_12mm_bullet1(c"0");
    pub static sk_12mm_bullet2(c"0");
    pub static sk_12mm_bullet3(c"0");

    pub static sk_9mmAR_bullet1(c"0");
    pub static sk_9mmAR_bullet2(c"0");
    pub static sk_9mmAR_bullet3(c"0");

    pub static sk_9mm_bullet1(c"0");
    pub static sk_9mm_bullet2(c"0");
    pub static sk_9mm_bullet3(c"0");

    pub static sk_hornet_dmg1(c"0");
    pub static sk_hornet_dmg2(c"0");
    pub static sk_hornet_dmg3(c"0");

    pub static sk_suitcharger1(c"0");
    pub static sk_suitcharger2(c"0");
    pub static sk_suitcharger3(c"0");

    pub static sk_battery1(c"0");
    pub static sk_battery2(c"0");
    pub static sk_battery3(c"0");

    pub static sk_healthcharger1(c"0");
    pub static sk_healthcharger2(c"0");
    pub static sk_healthcharger3(c"0");

    pub static sk_healthkit1(c"0");
    pub static sk_healthkit2(c"0");
    pub static sk_healthkit3(c"0");

    pub static sk_scientist_heal1(c"0");
    pub static sk_scientist_heal2(c"0");
    pub static sk_scientist_heal3(c"0");

    pub static sk_monster_head1(c"2");
    pub static sk_monster_head2(c"2");
    pub static sk_monster_head3(c"2");

    pub static sk_monster_chest1(c"1");
    pub static sk_monster_chest2(c"1");
    pub static sk_monster_chest3(c"1");

    pub static sk_monster_stomach1(c"1");
    pub static sk_monster_stomach2(c"1");
    pub static sk_monster_stomach3(c"1");

    pub static sk_monster_arm1(c"1");
    pub static sk_monster_arm2(c"1");
    pub static sk_monster_arm3(c"1");

    pub static sk_monster_leg1(c"1");
    pub static sk_monster_leg2(c"1");
    pub static sk_monster_leg3(c"1");

    pub static sk_player_head1(c"2");
    pub static sk_player_head2(c"2");
    pub static sk_player_head3(c"2");

    pub static sk_player_chest1(c"1");
    pub static sk_player_chest2(c"1");
    pub static sk_player_chest3(c"1");

    pub static sk_player_stomach1(c"1");
    pub static sk_player_stomach2(c"1");
    pub static sk_player_stomach3(c"1");

    pub static sk_player_arm1(c"1");
    pub static sk_player_arm2(c"1");
    pub static sk_player_arm3(c"1");

    pub static sk_player_leg1(c"1");
    pub static sk_player_leg2(c"1");
    pub static sk_player_leg3(c"1");

    pub static sv_pushable_fixed_tick_fudge(c"15");

    pub static sv_busters(c"0");
}

pub fn init() {
    sv::cvar::init(|name, _, _| engine().get_cvar(name));

    sv_gravity.get_ptr();
    sv_aim.get_ptr();
    sv_allow_autoaim.get_ptr();
    mp_footsteps.get_ptr();

    // XXX: server dll use different API for cvar registration
    self::register::init();
    self::get_ptrs();

    engine().server_command(c"exec skill.cfg\n");
}
