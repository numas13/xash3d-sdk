use core::{
    ffi::{c_int, c_short, c_uchar, CStr},
    ptr,
};

use sv::{
    consts::{EFLAG_SLERP, ENTITY_BEAM, ENTITY_NORMAL, SOLID_SLIDEBOX},
    prelude::*,
    raw::{
        clientdata_s, edict_s, entity_state_s, entvars_s, vec3_t, EdictFlags, Effects, MoveType,
    },
    str::MapString,
};

use crate::{
    entity::{impl_cast, Entity, EntityVars},
    private_data::Private,
};

pub fn update_client_data(ent: &edict_s, sendweapons: bool, cd: &mut clientdata_s) {
    if ent.pvPrivateData.is_null() {
        return;
    }

    let engine = engine();
    let ev = &ent.v;

    // TODO:

    cd.flags = ev.flags;
    cd.health = ev.health;

    cd.viewmodel = engine.model_index(ev.viewmodel.as_ref().map_or(c"".into(), |s| s.as_thin()));

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

    cd.physinfo
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

pub fn add_to_full_pack(
    state: &mut entity_state_s,
    e: c_int,
    ent: &edict_s,
    host: &edict_s,
    hostflags: c_int,
    player: bool,
    set: *mut c_uchar,
) -> bool {
    if ent.v.effects.intersects(Effects::NODRAW) && !ptr::eq(ent, host) {
        return false;
    }

    if ent.v.modelindex == 0 || ent.v.model.unwrap().is_empty() {
        return false;
    }

    if ent.v.flags.intersects(EdictFlags::SPECTATOR) && !ptr::eq(ent, host) {
        return false;
    }

    let engine = engine();
    if !ptr::eq(ent, host) && !engine.check_visibility(ent, set) {
        return false;
    }

    // do not send if the client say it is predicting the entity itself
    if ent.v.flags.intersects(EdictFlags::SKIPLOCALHOST)
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

    state.entityType = if ent.v.flags.intersects(EdictFlags::CUSTOMENTITY) {
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

    state.movetype = ent.v.movetype;
    state.sequence = ent.v.sequence;
    state.framerate = ent.v.framerate;
    state.body = ent.v.body;

    state.controller = ent.v.controller;
    state.blending[0] = ent.v.blending[0];
    state.blending[1] = ent.v.blending[1];

    state.rendermode = ent.v.rendermode;
    state.renderamt = ent.v.renderamt as c_int;
    state.renderfx = ent.v.renderfx;
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
        if owner >= 1 && owner <= globals().max_clients() {
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
                .weaponmodel
                .as_ref()
                .map_or(c"".into(), |s| s.as_thin()),
        );
        state.gaitsequence = ent.v.gaitsequence;
        state.spectator = ent.v.flags.intersects(EdictFlags::SPECTATOR).into();
        state.friction = ent.v.friction;

        state.gravity = ent.v.gravity;
        // state.team = env.v.team;

        state.usehull = if ent.v.flags.intersects(EdictFlags::DUCKING) {
            1
        } else {
            0
        };
        state.health = ent.v.health as c_int;
    }

    // TODO: state.eflags |= EFLAG_FLESH_SOUND

    true
}

pub fn create_baseline(
    player: bool,
    eindex: c_int,
    baseline: &mut entity_state_s,
    ent: &edict_s,
    playermodelindex: c_int,
    player_mins: vec3_t,
    player_maxs: vec3_t,
) {
    baseline.origin = ent.v.origin;
    baseline.angles = ent.v.angles;
    baseline.frame = ent.v.frame;
    baseline.skin = ent.v.skin as c_short;

    baseline.rendermode = ent.v.rendermode;
    baseline.renderamt = ent.v.renderamt as u8 as c_int;
    baseline.rendercolor.r = ent.v.rendercolor[0] as u8;
    baseline.rendercolor.g = ent.v.rendercolor[1] as u8;
    baseline.rendercolor.b = ent.v.rendercolor[2] as u8;
    baseline.renderfx = ent.v.renderfx;

    if player {
        baseline.mins = player_mins;
        baseline.maxs = player_maxs;

        baseline.colormap = eindex;
        baseline.modelindex = playermodelindex;
        baseline.friction = 1.0;
        baseline.movetype = MoveType::Walk;

        baseline.scale = ent.v.scale;
        baseline.solid = SOLID_SLIDEBOX as c_short;
        baseline.framerate = 1.0;
        baseline.gravity = 1.0;
    } else {
        baseline.mins = ent.v.mins;
        baseline.maxs = ent.v.maxs;

        baseline.colormap = 0;
        baseline.modelindex = ent.v.modelindex;
        baseline.movetype = ent.v.movetype;

        baseline.scale = ent.v.scale;
        baseline.solid = ent.v.solid as c_short;
        baseline.framerate = ent.v.framerate;
        baseline.gravity = ent.v.gravity;
    }
}

pub fn dispatch_touch(entity: &mut edict_s, other: &mut edict_s) {
    // TODO: disable touch

    let Some(entity) = entity.private_mut() else {
        return;
    };
    let Some(other) = other.private_mut() else {
        return;
    };

    if !entity
        .vars()
        .flags
        .union(other.vars().flags)
        .intersects(EdictFlags::KILLME)
    {
        entity.touch(&mut **other);
    }
}

pub fn dispatch_object_collision_box(ent: &mut edict_s) {
    match ent.private_mut() {
        Some(entity) => entity.set_object_collision_box(),
        None => crate::entity::set_object_collision_box(&mut ent.v),
    }
}

#[derive(Debug)]
pub struct Stub {
    pub vars: *mut entvars_s,
    pub name: &'static CStr,
}

impl_cast!(Stub);

impl EntityVars for Stub {
    fn vars_ptr(&self) -> *mut entvars_s {
        self.vars
    }
}

impl Entity for Stub {
    fn spawn(&mut self) -> bool {
        let classname = MapString::new(self.name);
        let ev = self.vars_mut();
        ev.classname = Some(classname);
        true
    }
}

macro_rules! link_entity_stub {
    ($($name:ident),* $(,)?) => {
        $($crate::macros::link_entity!($name, |vars| {
            $crate::todo::Stub { vars, name: sv::macros::cstringify!($name) }
        });)*
    };
}
pub(super) use link_entity_stub;

link_entity_stub! {
    ambient_generic,
    ammo_glockclip,
    env_beam,
    env_beverage,
    env_explosion,
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
    func_rotating,
    func_train,
    func_wall,
    gibshooter,
    info_landmark,
    info_node,
    info_player_start,
    info_target,
    infodecal,
    item_battery,
    light,
    light_spot,
    monster_barney,
    monster_barney_dead,
    monster_generic,
    monster_headcrab,
    monster_scientist,
    monster_scientist_dead,
    monster_zombie,
    monstermaker,
    multi_manager,
    multisource,
    path_corner,
    scripted_sentence,
    scripted_sequence,
    weapon_crowbar,
    world_items,
}
