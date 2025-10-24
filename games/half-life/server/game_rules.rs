use core::{ffi::CStr, fmt};

use xash3d_server::{
    entity::{Entity, EntityPlayer},
    ffi::common::vec3_t,
    game_rules::GameRules,
    global_state::GlobalStateRef,
    prelude::*,
    time::MapTime,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillLevel {
    Easy,
    #[default]
    Medium,
    Hard,
}

impl SkillLevel {
    pub fn from_cvar(engine: &ServerEngine) -> Self {
        match engine.get_cvar::<i32>(c"skill").clamp(1, 3) {
            1 => Self::Easy,
            2 => Self::Medium,
            3 => Self::Hard,
            _ => unreachable!(),
        }
    }

    fn cvar_index(&self) -> u8 {
        match self {
            Self::Easy => 1,
            Self::Medium => 2,
            Self::Hard => 3,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
        }
    }
}

impl fmt::Display for SkillLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// TODO: remove me
#[allow(dead_code)]
pub struct SkillData {
    pub skill_level: SkillLevel,

    //***********************************************************************/
    // Monster health and damage.
    //***********************************************************************/

    // Agrunt
    pub agrunt_health: f32,
    pub agrunt_dmg_punch: f32,

    // Apache
    pub apache_health: f32,

    // Barney
    pub barney_health: f32,

    // Big Momma
    pub bigmomma_health_factor: f32,
    pub bigmomma_dmg_slash: f32,
    pub bigmomma_dmg_blast: f32,
    pub bigmomma_radius_blast: f32,

    // Bullsquid
    pub bullsquid_health: f32,
    pub bullsquid_dmg_bite: f32,
    pub bullsquid_dmg_whip: f32,
    pub bullsquid_dmg_spit: f32,

    // Gargantua
    pub gargantua_health: f32,
    pub gargantua_dmg_slash: f32,
    pub gargantua_dmg_fire: f32,
    pub gargantua_dmg_stomp: f32,

    // Hassassin
    pub hassassin_health: f32,

    // Headcrab
    pub headcrab_health: f32,
    pub headcrab_dmg_bite: f32,

    // Hgrunt
    pub hgrunt_health: f32,
    pub hgrunt_dmg_kick: f32,
    pub hgrunt_shotgun_pellets: f32,
    pub hgrunt_grenade_speed: f32,

    // Houndeye
    pub houndeye_health: f32,
    pub houndeye_dmg_blast: f32,

    // ISlave
    pub islave_health: f32,
    pub islave_dmg_claw: f32,
    pub islave_dmg_clawrake: f32,
    pub islave_dmg_zap: f32,

    // Icthyosaur
    pub ichthyosaur_health: f32,
    pub ichthyosaur_dmg_shake: f32,

    // Leech
    pub leech_health: f32,
    pub leech_dmg_bite: f32,

    // Controller
    pub controller_health: f32,
    pub controller_dmg_zap: f32,
    pub controller_speed_ball: f32,
    pub controller_dmg_ball: f32,

    // Nihilanth
    pub nihilanth_health: f32,
    pub nihilanth_zap: f32,

    // Scientist
    pub scientist_health: f32,

    // Snark
    pub snark_health: f32,
    pub snark_dmg_bite: f32,
    pub snark_dmg_pop: f32,

    // Zombie
    pub zombie_health: f32,
    pub zombie_dmg_one_slash: f32,
    pub zombie_dmg_both_slash: f32,

    // Turret
    pub turret_health: f32,

    // MiniTurret
    pub miniturret_health: f32,

    // Sentry Turret
    pub sentry_health: f32,

    //***********************************************************************/
    // Player weapons.
    //***********************************************************************/

    // Crowbar whack
    pub player_dmg_crowbar: f32,
    // Glock Round
    pub player_dmg_9mm: f32,
    // 357 Round
    pub player_dmg_357: f32,
    // MP5 Round
    pub player_dmg_mp5: f32,
    // M203 grenade
    pub player_dmg_m203_grenade: f32,
    // Shotgun buckshot
    pub player_dmg_buckshot: f32,
    // Crossbow
    pub player_dmg_crossbow_client: f32,
    pub player_dmg_crossbow_monster: f32,
    // RPG
    pub player_dmg_rpg: f32,
    // Gauss gun
    pub player_dmg_gauss: f32,
    // Egon Gun
    pub player_dmg_egon_narrow: f32,
    pub player_dmg_egon_wide: f32,
    // Hornet
    pub player_dmg_hornet: f32,
    // Hand Grendade
    pub player_dmg_hand_grenade: f32,
    // Satchel Charge
    pub player_dmg_satchel: f32,
    // Tripmine
    pub player_dmg_tripmine: f32,

    //***********************************************************************/
    // Weapons shared by monsters.
    //***********************************************************************/

    // Glock Round
    pub monster_dmg_9mm: f32,
    // MP5 Round
    pub monster_dmg_mp5: f32,
    // ???
    pub monster_dmg_12mm: f32,
    // Hornet
    pub monster_dmg_hornet: f32,

    //***********************************************************************/
    // Health/suit charge.
    //***********************************************************************/
    pub suitcharger_capacity: f32,
    pub battery_capacity: f32,
    pub healthcharger_capacity: f32,
    pub healthkit_capacity: f32,
    pub scientist_heal: f32,

    //***********************************************************************/
    // Monster damage adjustments.
    //***********************************************************************/
    pub monster_head: f32,
    pub monster_chest: f32,
    pub monster_stomach: f32,
    pub monster_leg: f32,
    pub monster_arm: f32,

    //***********************************************************************/
    // Player damage adjustments.
    //***********************************************************************/
    pub player_head: f32,
    pub player_chest: f32,
    pub player_stomach: f32,
    pub player_leg: f32,
    pub player_arm: f32,
}

impl SkillData {
    fn new(engine: ServerEngineRef) -> Self {
        let skill_level = SkillLevel::from_cvar(&engine);
        info!("GAME SKILL LEVEL: {skill_level}");

        let skill_cvar = |name| {
            let skill_level = skill_level.cvar_index();
            let value = engine.get_cvar::<f32>(format_args!("{name}{skill_level}"));
            if value <= 0.0 {
                error!("skill cvar {name}{skill_level} has invalid value {value}");
            }
            value
        };

        Self {
            skill_level,

            //***********************************************************************/
            // Monster health and damage.
            //***********************************************************************/
            agrunt_health: skill_cvar("sk_agrunt_health"),
            agrunt_dmg_punch: skill_cvar("sk_agrunt_dmg_punch"),

            apache_health: skill_cvar("sk_apache_health"),

            barney_health: skill_cvar("sk_barney_health"),

            bigmomma_health_factor: skill_cvar("sk_bigmomma_health_factor"),
            bigmomma_dmg_slash: skill_cvar("sk_bigmomma_dmg_slash"),
            bigmomma_dmg_blast: skill_cvar("sk_bigmomma_dmg_blast"),
            bigmomma_radius_blast: skill_cvar("sk_bigmomma_radius_blast"),

            bullsquid_health: skill_cvar("sk_bullsquid_health"),
            bullsquid_dmg_bite: skill_cvar("sk_bullsquid_dmg_bite"),
            bullsquid_dmg_whip: skill_cvar("sk_bullsquid_dmg_whip"),
            bullsquid_dmg_spit: skill_cvar("sk_bullsquid_dmg_spit"),

            gargantua_health: skill_cvar("sk_gargantua_health"),
            gargantua_dmg_slash: skill_cvar("sk_gargantua_dmg_slash"),
            gargantua_dmg_fire: skill_cvar("sk_gargantua_dmg_fire"),
            gargantua_dmg_stomp: skill_cvar("sk_gargantua_dmg_stomp"),

            hassassin_health: skill_cvar("sk_hassassin_health"),

            headcrab_health: skill_cvar("sk_headcrab_health"),
            headcrab_dmg_bite: skill_cvar("sk_headcrab_dmg_bite"),

            hgrunt_health: skill_cvar("sk_hgrunt_health"),
            hgrunt_dmg_kick: skill_cvar("sk_hgrunt_kick"),
            hgrunt_shotgun_pellets: skill_cvar("sk_hgrunt_pellets"),
            hgrunt_grenade_speed: skill_cvar("sk_hgrunt_gspeed"),

            houndeye_health: skill_cvar("sk_houndeye_health"),
            houndeye_dmg_blast: skill_cvar("sk_houndeye_dmg_blast"),

            islave_health: skill_cvar("sk_islave_health"),
            islave_dmg_claw: skill_cvar("sk_islave_dmg_claw"),
            islave_dmg_clawrake: skill_cvar("sk_islave_dmg_clawrake"),
            islave_dmg_zap: skill_cvar("sk_islave_dmg_zap"),

            ichthyosaur_health: skill_cvar("sk_ichthyosaur_health"),
            ichthyosaur_dmg_shake: skill_cvar("sk_ichthyosaur_shake"),

            leech_health: skill_cvar("sk_leech_health"),
            leech_dmg_bite: skill_cvar("sk_leech_dmg_bite"),

            controller_health: skill_cvar("sk_controller_health"),
            controller_dmg_zap: skill_cvar("sk_controller_dmgzap"),
            controller_speed_ball: skill_cvar("sk_controller_speedball"),
            controller_dmg_ball: skill_cvar("sk_controller_dmgball"),

            nihilanth_health: skill_cvar("sk_nihilanth_health"),
            nihilanth_zap: skill_cvar("sk_nihilanth_zap"),

            scientist_health: skill_cvar("sk_scientist_health"),

            snark_health: skill_cvar("sk_snark_health"),
            snark_dmg_bite: skill_cvar("sk_snark_dmg_bite"),
            snark_dmg_pop: skill_cvar("sk_snark_dmg_pop"),

            zombie_health: skill_cvar("sk_zombie_health"),
            zombie_dmg_one_slash: skill_cvar("sk_zombie_dmg_one_slash"),
            zombie_dmg_both_slash: skill_cvar("sk_zombie_dmg_both_slash"),

            turret_health: skill_cvar("sk_turret_health"),

            miniturret_health: skill_cvar("sk_miniturret_health"),

            sentry_health: skill_cvar("sk_sentry_health"),

            //***********************************************************************/
            // Player weapons.
            //***********************************************************************/
            player_dmg_crowbar: skill_cvar("sk_plr_crowbar"),
            player_dmg_9mm: skill_cvar("sk_plr_9mm_bullet"),
            player_dmg_357: skill_cvar("sk_plr_357_bullet"),
            player_dmg_mp5: skill_cvar("sk_plr_9mmAR_bullet"),
            player_dmg_m203_grenade: skill_cvar("sk_plr_9mmAR_grenade"),
            player_dmg_buckshot: skill_cvar("sk_plr_buckshot"),
            player_dmg_crossbow_client: skill_cvar("sk_plr_xbow_bolt_client"),
            player_dmg_crossbow_monster: skill_cvar("sk_plr_xbow_bolt_monster"),
            player_dmg_rpg: skill_cvar("sk_plr_rpg"),
            player_dmg_gauss: skill_cvar("sk_plr_gauss"),
            player_dmg_egon_narrow: skill_cvar("sk_plr_egon_narrow"),
            player_dmg_egon_wide: skill_cvar("sk_plr_egon_wide"),
            // not present in skill.cfg
            player_dmg_hornet: 7.0,
            player_dmg_hand_grenade: skill_cvar("sk_plr_hand_grenade"),
            player_dmg_satchel: skill_cvar("sk_plr_satchel"),
            player_dmg_tripmine: skill_cvar("sk_plr_tripmine"),

            //***********************************************************************/
            // Weapons shared by monsters.
            //***********************************************************************/
            monster_dmg_9mm: skill_cvar("sk_9mm_bullet"),
            monster_dmg_mp5: skill_cvar("sk_9mmAR_bullet"),
            monster_dmg_12mm: skill_cvar("sk_12mm_bullet"),
            monster_dmg_hornet: skill_cvar("sk_hornet_dmg"),

            //***********************************************************************/
            // Health/suit charge.
            //***********************************************************************/
            suitcharger_capacity: skill_cvar("sk_suitcharger"),
            battery_capacity: skill_cvar("sk_battery"),
            healthcharger_capacity: skill_cvar("sk_healthcharger"),
            healthkit_capacity: skill_cvar("sk_healthkit"),
            scientist_heal: skill_cvar("sk_scientist_heal"),

            //***********************************************************************/
            // Monster damage adjustments.
            //***********************************************************************/
            monster_head: skill_cvar("sk_monster_head"),
            monster_chest: skill_cvar("sk_monster_chest"),
            monster_stomach: skill_cvar("sk_monster_stomach"),
            monster_leg: skill_cvar("sk_monster_leg"),
            monster_arm: skill_cvar("sk_monster_arm"),

            //***********************************************************************/
            // Player damage adjustments.
            //***********************************************************************/
            player_head: skill_cvar("sk_player_head"),
            player_chest: skill_cvar("sk_player_chest"),
            player_stomach: skill_cvar("sk_player_stomach"),
            player_leg: skill_cvar("sk_player_leg"),
            player_arm: skill_cvar("sk_player_arm"),
        }
    }
}

pub struct HalfLifeRules {
    engine: ServerEngineRef,
}

impl HalfLifeRules {
    pub fn new(engine: ServerEngineRef) -> Self {
        engine.server_command("exec spserver.cfg\n");
        engine.global_state_ref().add(SkillData::new(engine));
        Self { engine }
    }
}

impl GameRules for HalfLifeRules {
    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn get_game_description(&self) -> &'static CStr {
        c"Half-Life"
    }

    fn allow_flashlight(&self) -> bool {
        true
    }

    fn can_have_item(&self, _: &dyn EntityPlayer, _: &dyn Entity) -> bool {
        true
    }

    fn player_got_item(&self, player: &dyn EntityPlayer, item: &dyn Entity) {
        trace!(
            "{} got an item {}",
            player.pretty_name(),
            item.pretty_name()
        );
    }

    fn item_respawn(&self, _: &dyn Entity) -> Option<(MapTime, vec3_t)> {
        None
    }
}

pub fn install_game_rules(engine: ServerEngineRef, global_state: GlobalStateRef) {
    engine.server_command(c"exec game.cfg\n");
    engine.server_execute();

    if !engine.globals.is_deathmatch() {
        // TODO: g_teamplay = 0;
        global_state.set_game_rules(HalfLifeRules::new(engine));
        return;
    } else {
        // TODO:
    }
    todo!();
}
