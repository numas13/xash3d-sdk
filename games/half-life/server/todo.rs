use core::{
    ffi::{c_int, c_short, c_uchar, CStr},
    ptr,
};

use csz::CStrSlice;
use xash3d_server::{
    consts::{EFLAG_SLERP, ENTITY_BEAM, ENTITY_NORMAL},
    entity::{BaseEntity, EdictFlags, Effects, Entity},
    ffi::{
        common::{clientdata_s, entity_state_s, vec3_t},
        server::edict_s,
    },
    prelude::*,
};

use crate::entity::{impl_cast, Private};

pub fn update_client_data(
    engine: ServerEngineRef,
    ent: &edict_s,
    sendweapons: bool,
    cd: &mut clientdata_s,
) {
    if ent.pvPrivateData.is_null() {
        return;
    }

    let ev = &ent.v;

    // TODO:

    cd.flags = ev.flags;
    cd.health = ev.health;

    cd.viewmodel = engine.model_index(ev.viewmodel().as_ref().map_or(c"".into(), |s| s.as_thin()));

    cd.waterlevel = ev.waterlevel;
    cd.watertype = ev.watertype;
    cd.weapons = ev.weapons;

    cd.origin = ev.origin;
    cd.velocity = ev.velocity;
    cd.view_ofs = ev.view_ofs;
    cd.punchangle = ev.punchangle;

    cd.bInDuck = ev.bInDuck;
    cd.flTimeStepSound = ev.flTimeStepSound;
    cd.flDuckTime = ev.flDuckTime;
    cd.flSwimTime = ev.flSwimTime;
    cd.waterjumptime = ev.teleport_time as c_int;

    CStrSlice::new_in_slice(&mut cd.physinfo)
        .cursor()
        .write_c_str(engine.get_physics_info_string(ent).into())
        .unwrap();

    cd.maxspeed = ev.maxspeed;
    cd.fov = ev.fov;
    cd.weaponanim = ev.weaponanim;

    cd.pushmsec = ev.pushmsec;

    // TODO: spectator mode

    cd.iuser1 = ev.iuser1;
    cd.iuser2 = ev.iuser2;

    #[cfg(feature = "client-weapons")]
    if sendweapons {
        // TODO: sendweapons
    }
}

#[allow(clippy::too_many_arguments)]
pub fn add_to_full_pack(
    engine: ServerEngineRef,
    state: &mut entity_state_s,
    e: c_int,
    ent: &edict_s,
    host: &edict_s,
    hostflags: c_int,
    player: bool,
    set: *mut c_uchar,
) -> bool {
    if ent.v.effects().intersects(Effects::NODRAW) && !ptr::eq(ent, host) {
        return false;
    }

    if ent.v.modelindex == 0 || ent.v.model().unwrap().is_empty() {
        return false;
    }

    if ent.v.flags().intersects(EdictFlags::SPECTATOR) && !ptr::eq(ent, host) {
        return false;
    }

    if !ptr::eq(ent, host) && !engine.check_visibility(ent, set) {
        return false;
    }

    // do not send if the client say it is predicting the entity itself
    if ent.v.flags().intersects(EdictFlags::SKIPLOCALHOST)
        && hostflags & 1 != 0
        && ptr::eq(ent.v.owner, host)
    {
        return false;
    }

    if host.v.groupinfo != 0 {
        debug!("TODO: add_to_full_pack groupinfo");
    }

    unsafe {
        ptr::write_bytes(state, 0, 1);
    }

    state.number = e;

    state.entityType = if ent.v.flags().intersects(EdictFlags::CUSTOMENTITY) {
        ENTITY_BEAM
    } else {
        ENTITY_NORMAL
    };

    state.animtime = ((1000.0 * ent.v.animtime) as i32) as f32 / 1000.0;

    state.origin = ent.v.origin;
    state.angles = ent.v.angles;
    state.mins = ent.v.mins;
    state.maxs = ent.v.maxs;

    state.startpos = ent.v.startpos;
    state.endpos = ent.v.endpos;

    state.modelindex = ent.v.modelindex;

    state.frame = ent.v.frame;

    state.skin = ent.v.skin as c_short;
    state.effects = ent.v.effects;

    if !player && ent.v.animtime != 0.0 && ent.v.velocity == vec3_t::ZERO {
        state.eflags |= EFLAG_SLERP as u8;
    }

    state.scale = ent.v.scale;
    state.solid = ent.v.solid as c_short;
    state.colormap = ent.v.colormap;

    state.movetype = ent.v.movetype as c_int;
    state.sequence = ent.v.sequence;
    state.framerate = ent.v.framerate;
    state.body = ent.v.body;

    state.controller = ent.v.controller;
    state.blending[0] = ent.v.blending[0];
    state.blending[1] = ent.v.blending[1];

    state.rendermode = ent.v.rendermode as c_int;
    state.renderamt = ent.v.renderamt as c_int;
    state.renderfx = ent.v.renderfx as c_int;
    state.rendercolor.r = ent.v.rendercolor[0] as u8;
    state.rendercolor.g = ent.v.rendercolor[1] as u8;
    state.rendercolor.b = ent.v.rendercolor[2] as u8;

    state.aiment = if !ent.v.aiment.is_null() {
        engine.ent_index(unsafe { &*ent.v.aiment })
    } else {
        0
    };

    state.owner = 0;
    if !ent.v.owner.is_null() {
        let owner = engine.ent_index(unsafe { &*ent.v.owner });
        if owner >= 1 && owner <= engine.globals.max_clients() {
            state.owner = owner;
        }
    }

    if !player {
        state.playerclass = ent.v.playerclass;
    }

    if player {
        state.basevelocity = ent.v.basevelocity;

        state.weaponmodel = engine.model_index(
            ent.v
                .weaponmodel()
                .as_ref()
                .map_or(c"".into(), |s| s.as_thin()),
        );
        state.gaitsequence = ent.v.gaitsequence;
        state.spectator = ent.v.flags().intersects(EdictFlags::SPECTATOR).into();
        state.friction = ent.v.friction;

        state.gravity = ent.v.gravity;
        // state.team = env.v.team;

        state.usehull = if ent.v.flags().intersects(EdictFlags::DUCKING) {
            1
        } else {
            0
        };
        state.health = ent.v.health as c_int;
    }

    // TODO: state.eflags |= EFLAG_FLESH_SOUND

    true
}

#[derive(Debug)]
pub struct Stub {
    pub base: BaseEntity,
    pub name: &'static CStr,
}

impl_cast!(Stub);

impl Stub {
    pub fn new(base: BaseEntity, name: &'static CStr) -> Self {
        Self { base, name }
    }
}

impl Entity for Stub {
    fn spawn(&mut self) {
        let classname = self.base.engine.new_map_string(self.name);
        let ev = self.vars_mut().as_raw_mut();
        ev.classname = classname.index();
    }
}

macro_rules! export_entity_stub {
    ($($name:ident),* $(,)?) => {
        $(xash3d_server::export::export_entity!($name, Private<$crate::todo::Stub>, |base| {
            let name = xash3d_server::macros::cstringify!($name);
            $crate::todo::Stub::new(base, name)
        });)*
    };
}
pub(super) use export_entity_stub;

export_entity_stub! {
    ambient_generic,
    ammo_glockclip,
    env_beam,
    env_beverage,
    env_explosion,
    env_glow,
    env_laser,
    env_render,
    env_shake,
    env_shooter,
    env_sound,
    env_spark,
    env_sprite,
    func_breakable,
    func_button,
    func_door,
    func_door_rotating,
    func_friction,
    func_healthcharger,
    func_illusionary,
    func_ladder,
    func_pendulum,
    func_pushable,
    func_rot_button,
    func_rotating,
    func_train,
    func_wall,
    func_water,
    gibshooter,
    info_landmark,
    info_node,
    info_player_start,
    info_target,
    infodecal,
    item_battery,
    light,
    light_spot,
    monster_barnacle,
    monster_barney,
    monster_barney_dead,
    monster_bullchicken,
    monster_generic,
    monster_headcrab,
    monster_scientist,
    monster_scientist_dead,
    monster_zombie,
    monstermaker,
    multi_manager,
    multisource,
    path_corner,
    scripted_sequence,
    trigger_cdaudio,
    trigger_push,
    trigger_teleport,
    trigger_transition,
    weapon_357,
    weapon_9mmAR,
    weapon_9mmhandgun,
    weapon_crossbow,
    weapon_crowbar,
    weapon_egon,
    weapon_gauss,
    weapon_handgrenade,
    weapon_hornetgun,
    weapon_rpg,
    weapon_satchel,
    weapon_shotgun,
    weapon_snark,
    weapon_tripmine,
    world_items,
}
