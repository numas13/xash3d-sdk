use core::{
    ffi::{c_char, c_int, c_short, c_uchar, c_uint, c_void, CStr},
    fmt::Write,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
    sync::atomic::{AtomicBool, Ordering},
};

use csz::{CStrArray, CStrSlice, CStrThin};
use xash3d_player_move::{VEC_DUCK_HULL_MIN, VEC_HULL_MIN};
use xash3d_shared::{
    consts::{EFLAG_SLERP, ENTITY_BEAM, ENTITY_NORMAL},
    engine::net::netadr_s,
    entity::{EdictFlags, Effects},
    ffi::{
        common::{
            clientdata_s, customization_s, entity_state_s, qboolean, usercmd_s, vec3_t,
            weapon_data_s,
        },
        player_move::playermove_s,
        server::{
            edict_s, KeyValueData, DLL_FUNCTIONS, NEW_DLL_FUNCTIONS, SAVERESTOREDATA,
            TYPEDESCRIPTION,
        },
    },
    utils::cstr_or_none,
};

use crate::{
    entities::triggers,
    entity::{
        BaseEntity, CreateEntity, Entity, EntityPlayer, EntityVars, KeyValue, PrivateData,
        PrivateEntity, RestoreResult, UseType,
    },
    global_state::{EntityState, GlobalState, GlobalStateRef},
    prelude::*,
    save::{SaveReader, SaveRestoreData, SaveWriter},
    utils::slice_from_raw_parts_or_empty_mut,
};

pub use xash3d_shared::export::{impl_unsync_global, UnsyncGlobal};

#[cfg(feature = "save")]
const ENTITY_SAVE_NAME: &CStr = c"ENTITY";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpawnResult {
    Delete,
    Ok,
}

impl From<SpawnResult> for c_int {
    fn from(val: SpawnResult) -> Self {
        match val {
            SpawnResult::Delete => -1,
            SpawnResult::Ok => 0,
        }
    }
}

#[allow(unused_variables)]
#[allow(clippy::missing_safety_doc)]
pub trait ServerDll: UnsyncGlobal {
    /// A private world entity.
    type World: PrivateEntity;

    /// A private player entity used to spawn players.
    type Player: PrivateEntity<Entity: CreateEntity>;

    fn new(engine: ServerEngineRef, global_state: GlobalStateRef) -> Self;

    fn create_world(base: BaseEntity) -> <Self::World as PrivateEntity>::Entity;

    fn engine(&self) -> ServerEngineRef;

    fn global_state(&self) -> GlobalStateRef;

    fn is_touch_enabled(&self) -> bool {
        true
    }

    fn dispatch_spawn(&self, ent: &mut edict_s) -> SpawnResult {
        let engine = self.engine();
        let global_state = self.global_state();

        let Some(ent) = ent.get_entity() else {
            return SpawnResult::Delete;
        };

        let v = ent.vars();
        v.set_abs_min(v.origin() - vec3_t::splat(1.0));
        v.set_abs_max(v.origin() + vec3_t::splat(1.0));

        ent.spawn();

        if !global_state.game_rules().is_allowed_to_spawn(ent) {
            return SpawnResult::Delete;
        }

        if ent.vars().flags().intersects(EdictFlags::KILLME) {
            return SpawnResult::Delete;
        }

        if let Some(globalname) = ent.globalname() {
            let mut entities = global_state.entities_mut();
            let map_name = engine.globals.map_name().unwrap();
            if let Some(global) = entities.find(globalname) {
                if global.is_dead() {
                    return SpawnResult::Delete;
                }
                if map_name.as_thin() != global.map_name() {
                    ent.make_dormant();
                }
            } else {
                entities.add(globalname, map_name, EntityState::On);
            }
        }

        SpawnResult::Ok
    }

    fn dispatch_think(&self, ent: &mut edict_s) {
        if let Some(entity) = ent.get_entity() {
            if entity.vars().flags().intersects(EdictFlags::DORMANT) {
                let classname = entity.classname();
                warn!("Dormant entity {classname:?} is thinkng");
            }
            entity.think();
        }
    }

    fn dispatch_use(&self, used: &mut edict_s, other: &mut edict_s) {
        let Some(used) = used.get_entity() else {
            return;
        };
        let Some(other) = other.get_entity() else {
            error!("dispatch_use: other private data is null");
            return;
        };
        if !used.vars().flags().intersects(EdictFlags::KILLME) {
            used.used(UseType::Toggle, None, other);
        }
    }

    fn dispatch_touch(&self, touched: &mut edict_s, other: &mut edict_s) {
        if !self.is_touch_enabled() {
            return;
        }
        let Some(touched) = touched.get_entity() else {
            return;
        };
        let Some(other) = other.get_entity() else {
            error!("dispatch_touch: other private data is null");
            return;
        };
        if touched.vars().flags().intersects(EdictFlags::KILLME) {
            return;
        }
        if other.vars().flags().intersects(EdictFlags::KILLME) {
            return;
        }
        touched.touched(other);
    }

    fn dispatch_blocked(&self, blocked: &mut edict_s, other: &mut edict_s) {
        let Some(blocked) = blocked.get_entity() else {
            return;
        };
        let Some(other) = other.get_entity() else {
            error!("dispatch_blocked: other private data is null");
            return;
        };
        blocked.blocked(other);
    }

    fn dispatch_key_value(&self, ent: &mut edict_s, data: &mut KeyValue) {
        let ev = unsafe { EntityVars::from_raw(self.engine(), self.global_state(), &mut ent.v) };
        ev.key_value(data);

        if data.handled() || data.class_name().is_none() {
            return;
        }

        if let Some(ent) = ent.get_entity() {
            ent.key_value(data);
        }
    }

    #[cfg(not(feature = "save"))]
    fn dispatch_save(&self, _ent: &mut edict_s, _save_data: &mut SaveRestoreData) {
        error!("dispatch_save: feature \"save\" is not enabled");
    }

    #[cfg(not(feature = "save"))]
    fn dispatch_restore(
        &self,
        _ent: &mut edict_s,
        _save_data: &mut SaveRestoreData,
        _global_entity: bool,
    ) -> RestoreResult {
        error!("dispatch_restore: feature \"save\" is not enabled");
        RestoreResult::Ok
    }

    #[cfg(feature = "save")]
    fn dispatch_save(&self, ent: &mut edict_s, save_data: &mut SaveRestoreData) {
        use crate::{
            entity::{MoveType, ObjectCaps},
            save,
        };

        let engine = self.engine();
        let current_index = save_data.current_index();
        if save_data.table()[current_index].pent != ent {
            error!("Entity table or index is wrong");
        }
        let Some(entity) = ent.get_entity_mut() else {
            return;
        };
        if entity.object_caps().intersects(ObjectCaps::DONT_SAVE) {
            return;
        }

        let v = entity.vars();
        if v.move_type() == MoveType::Push {
            let delta = v.next_think_time() - v.last_think_time();
            v.set_last_think_time_from_now(0.0);
            v.set_next_think_time_from_last(delta);
        }

        let location = save_data.offset();
        let (buffer, data) = save_data.split_mut();
        let mut state = save::SaveState::new(engine, data);
        let mut cur = save::CursorMut::new(buffer.as_slice_mut());
        let start_offset = cur.offset();
        let res = cur.write_field(&mut state, ENTITY_SAVE_NAME, entity);
        let size = cur.offset() - start_offset;
        if let Err(err) = res {
            let classname = entity.classname();
            error!("dispatch_save: failed to save an entity {classname}, {err}");
        } else if let Err(err) = buffer.advance(size) {
            error!("dispatch_save: failed to advance the save buffer by {size} bytes, {err}");
        }

        let table = &mut save_data.table_mut()[current_index];
        table.classname = entity.vars().classname().map_or(0, |s| s.index());
        table.location = location as i32;
        table.size = size as i32;
    }

    #[cfg(feature = "save")]
    fn dispatch_restore(
        &self,
        mut ent: &mut edict_s,
        save_data: &mut SaveRestoreData,
        global_entity: bool,
    ) -> RestoreResult {
        use core::mem::MaybeUninit;

        use xash3d_shared::ffi::server::entvars_s;

        use crate::{entity::ObjectCaps, save};

        let engine = self.engine();
        let global_state = self.global_state();

        let mut global_vars = MaybeUninit::<entvars_s>::uninit();
        if global_entity {
            let mut reader = SaveReader::new(engine);
            reader.precache_mode(false);
            reader
                .read_fields(save_data, unsafe { global_vars.assume_init_mut() })
                .unwrap();
        }

        let mut global_mode = false;
        let mut old_offset = vec3_t::ZERO;
        if global_entity {
            let tmp_vars = unsafe { global_vars.assume_init_mut() };
            // HACK: restore save pointers
            save_data.restore_save_pointers();

            let mut entities = global_state.entities_mut();
            let global = entities.find(tmp_vars.globalname().unwrap()).unwrap();
            if save_data.current_map_name() != global.map_name() {
                return RestoreResult::Ok;
            }

            old_offset = save_data.landmark_offset();
            let classname = tmp_vars.classname().unwrap();
            let globalname = tmp_vars.globalname().unwrap();
            if let Some(mut new_ent) = engine.find_global_entity(classname, globalname) {
                let new_ent = unsafe { new_ent.as_mut() };
                global_mode = true;
                let mut landmark_offset = save_data.landmark_offset();
                landmark_offset -= new_ent.v.mins;
                landmark_offset += tmp_vars.mins;
                save_data.set_landmark_offset(landmark_offset);
                ent = new_ent;
                entities.update(
                    ent.v.globalname().unwrap(),
                    engine.globals.map_name().unwrap(),
                );
            } else {
                return RestoreResult::Ok;
            }
        }

        let Some(entity) = ent.get_entity_mut() else {
            return RestoreResult::Ok;
        };

        let (buffer, data) = save_data.split_mut();
        let mut state = save::RestoreState::new(engine, data);
        state.set_global(global_mode);
        let mut cur = save::Cursor::new(buffer.as_slice());
        let start_offset = cur.offset();
        let result = cur.read_field().and_then(|field| {
            let name = state.token_str(field.token());
            assert_eq!(name, Some(ENTITY_SAVE_NAME.into()));
            entity.restore(&state, &mut field.cursor())
        });
        let size = cur.offset() - start_offset;
        if let Err(err) = result {
            error!("dispatch_restore: failed to restore an entity, {err}");
        }
        if let Err(err) = buffer.advance(size) {
            error!("dispatch_restore: failed to advance restore buffer by {size} bytes, {err}");
        }

        if entity.object_caps().intersects(ObjectCaps::MUST_SPAWN) {
            entity.spawn();
        } else {
            entity.precache();
        }

        if global_entity {
            save_data.set_landmark_offset(old_offset);
            entity.vars().link();
            entity.override_reset();
            return RestoreResult::Ok;
        } else if let Some(globalname) = entity.globalname() {
            let globals = &engine.globals;
            let mut entities = global_state.entities_mut();
            if let Some(global) = entities.find(globalname) {
                if global.is_dead() {
                    return RestoreResult::Delete;
                }
                if globals.map_name().unwrap().as_thin() != global.map_name() {
                    entity.make_dormant();
                }
            } else {
                let classname = entity.classname();
                error!("Global entity \"{globalname}\" (\"{classname}\") not in table!!!");
                entities.add(globalname, globals.map_name().unwrap(), EntityState::On);
            }
        }

        RestoreResult::Ok
    }

    fn dispatch_object_collsion_box(&self, ent: &mut edict_s) {
        match ent.get_entity() {
            Some(entity) => entity.set_object_collision_box(),
            None => crate::entity::set_object_collision_box(&mut ent.v),
        }
    }

    unsafe fn save_write_fields(
        &self,
        save_data: &mut SaveRestoreData,
        name: &'static CStrThin,
        base_data: *mut c_void,
        fields: &mut [TYPEDESCRIPTION],
    ) {
        let writer = &mut SaveWriter::new(self.engine());
        let result =
            unsafe { writer.write_fields_raw(save_data, name.into(), base_data.cast(), fields) };
        if let Err(err) = result {
            error!("save::write_fields({name:?}): {err}");
        }
    }

    unsafe fn save_read_fields(
        &self,
        save_data: &mut SaveRestoreData,
        name: &CStrThin,
        base_data: *mut c_void,
        fields: &mut [TYPEDESCRIPTION],
    ) {
        let reader = &mut SaveReader::new(self.engine());
        let result =
            unsafe { reader.read_fields_raw(save_data, name.into(), base_data.cast(), fields) };
        if let Err(err) = result {
            error!("save::read_fields({name:?}): {err}");
        }
    }

    fn save_global_state(&self, save_data: &mut SaveRestoreData) {
        if let Err(e) = self.global_state().save(save_data) {
            error!("Failed to save global state: {e:?}");
        }
    }

    fn restore_global_state(&self, save_data: &mut SaveRestoreData) {
        if let Err(e) = self.global_state().restore(save_data) {
            error!("Failed to restore global state: {e:?}");
        }
    }

    fn reset_global_state(&self) {
        self.global_state().reset();
    }

    fn client_connect(
        &self,
        ent: &mut edict_s,
        name: &CStrThin,
        address: &CStrThin,
        reject_reason: &mut CStrArray<128>,
    ) -> bool {
        true
    }

    fn client_disconnect(&self, ent: &mut edict_s) {}

    fn client_kill(&self, ent: &mut edict_s) {}

    fn client_put_in_server(&self, ent: &mut edict_s) {
        let engine = self.engine();
        let global_state = self.global_state();

        let player =
            unsafe { PrivateData::create::<Self::Player>(engine, global_state, &mut ent.v) };

        player.spawn();

        ent.v.effects_mut().insert(Effects::NOINTERP);
        ent.v.iuser1 = 0;
        ent.v.iuser2 = 0;
    }

    fn client_command(&self, ent: &mut edict_s) {}

    fn client_user_info_changed(&self, ent: &mut edict_s, info_buffer: &CStrThin) {}

    fn server_activate(&self, list: &mut [edict_s], client_max: c_int) {}

    fn server_deactivate(&self) {}

    fn player_pre_think(&self, ent: &mut edict_s) {
        if let Some(player) = ent.downcast_ref::<dyn EntityPlayer>() {
            player.pre_think();
        }
    }

    fn player_post_think(&self, ent: &mut edict_s) {
        if let Some(player) = ent.downcast_ref::<dyn EntityPlayer>() {
            player.post_think();
        }
    }

    fn start_frame(&self) {}

    fn parms_new_level(&self) {}

    fn parms_change_level(&self) {
        let engine = self.engine();
        if let Some(mut save_data) = engine.globals.save_data() {
            let save_data = unsafe { save_data.as_mut() };
            save_data.connectionCount =
                triggers::build_change_list(engine, &mut save_data.levelList) as c_int;
        }
    }

    /// Called before initialization.
    fn get_game_description_static() -> &'static CStr;

    /// Called after initialization.
    fn get_game_description(&self) -> &'static CStr {
        self.global_state().game_rules().get_game_description()
    }

    fn player_customization(&self, ent: &mut edict_s, custom: &mut customization_s) {}

    fn spectator_connect(&self, ent: &mut edict_s) {}

    fn spectator_disconnect(&self, ent: &mut edict_s) {}

    fn spectator_think(&self, ent: &mut edict_s) {}

    /// Called when the engine has encountered an error.
    fn system_error(&self, error_string: &CStrThin) {}

    fn player_move_init(&self, pm: NonNull<playermove_s>) {
        let pm = unsafe { pm.cast().as_mut() };
        xash3d_player_move::player_move_init(pm);
    }

    fn player_move(&self, pm: NonNull<playermove_s>, is_server: bool) {
        let pm = unsafe { pm.cast().as_mut() };
        xash3d_player_move::player_move(pm, is_server);
    }

    fn player_move_find_texture_type(&self, name: &CStrThin) -> c_char {
        xash3d_player_move::find_texture_type(name)
    }

    fn setup_visibility(
        &self,
        view_entity: Option<&mut edict_s>,
        client: &mut edict_s,
        pvs: &mut *mut c_uchar,
        pas: &mut *mut c_uchar,
    ) {
        if client.v.flags().intersects(EdictFlags::PROXY) {
            *pvs = ptr::null_mut();
            *pas = ptr::null_mut();
            return;
        }

        let view = view_entity.unwrap_or(client);
        let mut org = view.v.origin + view.v.view_ofs;
        if view.v.flags().intersects(EdictFlags::DUCKING) {
            org += VEC_HULL_MIN - VEC_DUCK_HULL_MIN;
        }

        let engine = self.engine();
        *pvs = engine.set_pvs(org);
        *pas = engine.set_pas(org);
    }

    fn update_client_data(&self, ent: &edict_s, send_weapons: bool, cd: &mut clientdata_s) {
        if ent.pvPrivateData.is_null() {
            return;
        }

        let engine = self.engine();
        let ev = &ent.v;

        // TODO:

        cd.flags = ev.flags;
        cd.health = ev.health;

        cd.viewmodel =
            engine.model_index(ev.viewmodel().as_ref().map_or(c"".into(), |s| s.as_thin()));

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

        // TODO: sendweapons
        // #[cfg(feature = "client-weapons")]
        // if sendweapons {
        //
        // }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_to_full_pack(
        &self,
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

        let engine = self.engine();
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
            engine.ent_index(unsafe { &*ent.v.aiment }).to_i32()
        } else {
            0
        };

        state.owner = 0;
        if !ent.v.owner.is_null() {
            let owner = engine.ent_index(unsafe { &*ent.v.owner }).to_i32();
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

    #[allow(clippy::too_many_arguments)]
    fn create_baseline(
        &self,
        player: bool,
        eindex: c_int,
        baseline: &mut entity_state_s,
        ent: &mut edict_s,
        player_model_index: c_int,
        player_mins: vec3_t,
        player_maxs: vec3_t,
    ) {
        crate::entity::create_baseline(
            player,
            eindex,
            baseline,
            ent,
            player_model_index,
            player_mins,
            player_maxs,
        );
    }

    fn register_encoders(&self) {}

    fn get_weapon_data(&self, player: &mut edict_s) -> Option<weapon_data_s> {
        None
    }

    fn command_start(&self, player: &mut edict_s, cmd: &usercmd_s, random_seed: c_uint) {}

    fn command_end(&self, player: *const edict_s) {}

    fn connectionless_packet(
        &self,
        from: &netadr_s,
        args: &CStrThin,
        buffer: &mut [u8],
    ) -> Result<usize, ()> {
        // no response
        Ok(0)
    }

    fn get_hull_bounds(&self, hullnumber: c_int, mins: &mut vec3_t, maxs: &mut vec3_t) -> c_int {
        xash3d_player_move::get_hull_bounds_ffi(hullnumber, mins, maxs)
    }

    fn create_instanced_baselines(&self) {}

    fn inconsistent_file(
        &self,
        player: &edict_s,
        filename: &CStrThin,
        disconnect_message: &mut CStrArray<256>,
    ) -> bool {
        if !self.engine().get_cvar::<bool>(c"mp_consistency") {
            // server does not care
            return false;
        }
        let mut cur = disconnect_message.cursor();
        writeln!(cur, "Server is enforcing file consistency for {filename}").ok();
        true
    }

    fn allow_lag_compensation(&self) -> bool {
        true
    }

    unsafe fn on_free_entity_private_data(&self, ent: *mut edict_s) {
        unsafe { PrivateData::drop_in_place(ent) }
    }

    fn chould_collide(&self, touched: &mut edict_s, other: &mut edict_s) -> bool {
        false
    }

    fn cvar_value(&self, ent: &edict_s, value: &CStrThin) {}

    fn cvar_value2(
        &self,
        ent: &edict_s,
        request_id: c_int,
        cvar_name: &CStrThin,
        value: &CStrThin,
    ) {
    }
}

pub fn dll_functions<T: ServerDll>() -> DLL_FUNCTIONS {
    Export::<T>::dll_functions()
}

pub fn new_dll_functions<T: ServerDll>() -> NEW_DLL_FUNCTIONS {
    Export::<T>::new_dll_functions()
}

trait ServerDllExport {
    fn dll_functions() -> DLL_FUNCTIONS {
        DLL_FUNCTIONS {
            pfnGameInit: Some(Self::init),
            pfnSpawn: Some(Self::dispatch_spawn),
            pfnThink: Some(Self::dispatch_think),
            pfnUse: Some(Self::dispatch_use),
            pfnTouch: Some(Self::dispatch_touch),
            pfnBlocked: Some(Self::dispatch_blocked),
            pfnKeyValue: Some(Self::dispatch_key_value),
            pfnSave: Some(Self::dispatch_save),
            pfnRestore: Some(Self::dispatch_restore),
            pfnSetAbsBox: Some(Self::dispatch_object_collsion_box),
            pfnSaveWriteFields: Some(Self::save_write_fields),
            pfnSaveReadFields: Some(Self::save_read_fields),
            pfnSaveGlobalState: Some(Self::save_global_state),
            pfnRestoreGlobalState: Some(Self::restore_global_state),
            pfnResetGlobalState: Some(Self::reset_global_state),
            pfnClientConnect: Some(Self::client_connect),
            pfnClientDisconnect: Some(Self::client_disconnect),
            pfnClientKill: Some(Self::client_kill),
            pfnClientPutInServer: Some(Self::client_put_in_server),
            pfnClientCommand: Some(Self::client_command),
            pfnClientUserInfoChanged: Some(Self::client_user_info_changed),
            pfnServerActivate: Some(Self::server_activate),
            pfnServerDeactivate: Some(Self::server_deactivate),
            pfnPlayerPreThink: Some(Self::player_pre_think),
            pfnPlayerPostThink: Some(Self::player_post_think),
            pfnStartFrame: Some(Self::start_frame),
            pfnParmsNewLevel: Some(Self::parms_new_level),
            pfnParmsChangeLevel: Some(Self::parms_change_level),
            pfnGetGameDescription: Some(Self::get_game_description),
            pfnPlayerCustomization: Some(Self::player_customization),
            pfnSpectatorConnect: Some(Self::spectator_connect),
            pfnSpectatorDisconnect: Some(Self::spectator_disconnect),
            pfnSpectatorThink: Some(Self::spectator_think),
            pfnSys_Error: Some(Self::system_error),
            pfnPM_Move: Some(Self::player_move),
            pfnPM_Init: Some(Self::player_move_init),
            pfnPM_FindTextureType: Some(Self::player_move_find_texture_type),
            pfnSetupVisibility: Some(Self::setup_visibility),
            pfnUpdateClientData: Some(Self::update_client_data),
            pfnAddToFullPack: Some(Self::add_to_full_pack),
            pfnCreateBaseline: Some(Self::create_baseline),
            pfnRegisterEncoders: Some(Self::register_encoders),
            pfnGetWeaponData: Some(Self::get_weapon_data),
            pfnCmdStart: Some(Self::command_start),
            pfnCmdEnd: Some(Self::command_end),
            pfnConnectionlessPacket: Some(Self::connectionless_packet),
            pfnGetHullBounds: Some(Self::get_hull_bounds),
            pfnCreateInstancedBaselines: Some(Self::create_instanced_baselines),
            pfnInconsistentFile: Some(Self::inconsistent_file),
            pfnAllowLagCompensation: Some(Self::allow_lag_compensation),
        }
    }

    fn new_dll_functions() -> NEW_DLL_FUNCTIONS {
        NEW_DLL_FUNCTIONS {
            pfnOnFreeEntPrivateData: Some(Self::on_free_entity_private_data),
            pfnGameShutdown: Some(Self::shutdown),
            pfnShouldCollide: Some(Self::should_collide),
            pfnCvarValue: Some(Self::cvar_value),
            pfnCvarValue2: Some(Self::cvar_value2),
        }
    }

    unsafe extern "C" fn init();

    unsafe extern "C" fn shutdown();

    unsafe extern "C" fn dispatch_spawn(ent: *mut edict_s) -> c_int;

    unsafe extern "C" fn dispatch_think(ent: *mut edict_s);

    unsafe extern "C" fn dispatch_use(used: *mut edict_s, other: *mut edict_s);

    unsafe extern "C" fn dispatch_touch(touched: *mut edict_s, other: *mut edict_s);

    unsafe extern "C" fn dispatch_blocked(blocked: *mut edict_s, other: *mut edict_s);

    unsafe extern "C" fn dispatch_key_value(ent: *mut edict_s, data: *mut KeyValueData);

    unsafe extern "C" fn dispatch_save(ent: *mut edict_s, save_data: *mut SAVERESTOREDATA);

    unsafe extern "C" fn dispatch_restore(
        ent: *mut edict_s,
        save_data: *mut SAVERESTOREDATA,
        global_entity: c_int,
    ) -> c_int;

    unsafe extern "C" fn dispatch_object_collsion_box(ent: *mut edict_s);

    unsafe extern "C" fn save_write_fields(
        save_data: *mut SAVERESTOREDATA,
        name: *const c_char,
        base_data: *mut c_void,
        fields: *mut TYPEDESCRIPTION,
        fields_count: c_int,
    );

    unsafe extern "C" fn save_read_fields(
        save_data: *mut SAVERESTOREDATA,
        name: *const c_char,
        base_data: *mut c_void,
        fields: *mut TYPEDESCRIPTION,
        fields_count: c_int,
    );

    unsafe extern "C" fn save_global_state(save_data: *mut SAVERESTOREDATA);

    unsafe extern "C" fn restore_global_state(save_data: *mut SAVERESTOREDATA);

    unsafe extern "C" fn reset_global_state();

    unsafe extern "C" fn client_connect(
        ent: *mut edict_s,
        name: *const c_char,
        address: *const c_char,
        reject_reason: *mut [c_char; 128usize],
    ) -> qboolean;

    unsafe extern "C" fn client_disconnect(ent: *mut edict_s);

    unsafe extern "C" fn client_kill(ent: *mut edict_s);

    unsafe extern "C" fn client_put_in_server(ent: *mut edict_s);

    unsafe extern "C" fn client_command(ent: *mut edict_s);

    unsafe extern "C" fn client_user_info_changed(ent: *mut edict_s, info_buffer: *mut c_char);

    unsafe extern "C" fn server_activate(
        edict_list: *mut edict_s,
        edict_count: c_int,
        client_max: c_int,
    );

    unsafe extern "C" fn server_deactivate();

    unsafe extern "C" fn player_pre_think(ent: *mut edict_s);

    unsafe extern "C" fn player_post_think(ent: *mut edict_s);

    unsafe extern "C" fn start_frame();

    unsafe extern "C" fn parms_new_level();

    unsafe extern "C" fn parms_change_level();

    unsafe extern "C" fn get_game_description() -> *const c_char;

    unsafe extern "C" fn player_customization(ent: *mut edict_s, custom: *mut customization_s);

    unsafe extern "C" fn spectator_connect(ent: *mut edict_s);

    unsafe extern "C" fn spectator_disconnect(ent: *mut edict_s);

    unsafe extern "C" fn spectator_think(ent: *mut edict_s);

    unsafe extern "C" fn system_error(error_string: *const c_char);

    unsafe extern "C" fn player_move_init(pm: *mut playermove_s);

    unsafe extern "C" fn player_move(pm: *mut playermove_s, is_server: qboolean);

    unsafe extern "C" fn player_move_find_texture_type(name: *mut c_char) -> c_char;

    unsafe extern "C" fn setup_visibility(
        view_entity: *mut edict_s,
        client: *mut edict_s,
        pvs: *mut *mut c_uchar,
        pas: *mut *mut c_uchar,
    );

    unsafe extern "C" fn update_client_data(
        ent: *const edict_s,
        send_weapons: c_int,
        cd: *mut clientdata_s,
    );

    unsafe extern "C" fn add_to_full_pack(
        state: *mut entity_state_s,
        e: c_int,
        ent: *mut edict_s,
        host: *mut edict_s,
        host_flags: c_int,
        player: c_int,
        set: *mut c_uchar,
    ) -> c_int;

    unsafe extern "C" fn create_baseline(
        player: c_int,
        eindex: c_int,
        baseline: *mut entity_state_s,
        entity: *mut edict_s,
        player_model_index: c_int,
        player_mins: *mut vec3_t,
        player_maxs: *mut vec3_t,
    );

    unsafe extern "C" fn register_encoders();

    unsafe extern "C" fn get_weapon_data(player: *mut edict_s, info: *mut weapon_data_s) -> c_int;

    unsafe extern "C" fn command_start(
        player: *const edict_s,
        cmd: *const usercmd_s,
        random_seed: c_uint,
    );

    unsafe extern "C" fn command_end(player: *const edict_s);

    unsafe extern "C" fn connectionless_packet(
        from: *const netadr_s,
        args: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: *mut c_int,
    ) -> c_int;

    extern "C" fn get_hull_bounds(hullnumber: c_int, mins: *mut f32, maxs: *mut f32) -> c_int;

    unsafe extern "C" fn create_instanced_baselines();

    unsafe extern "C" fn inconsistent_file(
        player: *const edict_s,
        filename: *const c_char,
        disconnect_message: *mut c_char,
    ) -> c_int;

    unsafe extern "C" fn allow_lag_compensation() -> c_int;

    unsafe extern "C" fn on_free_entity_private_data(ent: *mut edict_s);

    unsafe extern "C" fn should_collide(touched: *mut edict_s, other: *mut edict_s) -> c_int;

    unsafe extern "C" fn cvar_value(ent: *const edict_s, value: *const c_char);

    unsafe extern "C" fn cvar_value2(
        ent: *const edict_s,
        request_id: c_int,
        cvar_name: *const c_char,
        value: *const c_char,
    );
}

struct Export<T> {
    dll: PhantomData<T>,
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

impl<T: ServerDll> ServerDllExport for Export<T> {
    unsafe extern "C" fn init() {
        unsafe {
            let engine = ServerEngineRef::new();
            (*GlobalState::global_as_mut_ptr()).write(GlobalState::new(engine));
            let global_state = GlobalStateRef::new();
            (*T::global_as_mut_ptr()).write(T::new(engine, global_state));
        }
        INITIALIZED.store(true, Ordering::Relaxed);
    }

    unsafe extern "C" fn shutdown() {
        INITIALIZED.store(false, Ordering::Relaxed);
        unsafe {
            (*T::global_as_mut_ptr()).assume_init_drop();
            (*GlobalState::global_as_mut_ptr()).assume_init_drop();
        }
    }

    unsafe extern "C" fn dispatch_spawn(ent: *mut edict_s) -> c_int {
        if let Some(ent) = unsafe { ent.as_mut() } {
            return unsafe { T::global_assume_init_ref() }
                .dispatch_spawn(ent)
                .into();
        }
        SpawnResult::Delete.into()
    }

    unsafe extern "C" fn dispatch_think(ent: *mut edict_s) {
        if let Some(ent) = unsafe { ent.as_mut() } {
            unsafe { T::global_assume_init_ref() }.dispatch_think(ent);
        }
    }

    unsafe extern "C" fn dispatch_use(used: *mut edict_s, other: *mut edict_s) {
        let used = unsafe { used.as_mut() };
        let other = unsafe { other.as_mut() };
        if let (Some(used), Some(other)) = (used, other) {
            unsafe { T::global_assume_init_ref() }.dispatch_use(used, other)
        }
    }

    unsafe extern "C" fn dispatch_touch(touched: *mut edict_s, other: *mut edict_s) {
        let touched = unsafe { touched.as_mut() };
        let other = unsafe { other.as_mut() };
        if let (Some(touched), Some(other)) = (touched, other) {
            unsafe { T::global_assume_init_ref() }.dispatch_touch(touched, other);
        }
    }

    unsafe extern "C" fn dispatch_blocked(blocked: *mut edict_s, other: *mut edict_s) {
        let blocked = unsafe { blocked.as_mut() };
        let other = unsafe { other.as_mut() };
        if let (Some(blocked), Some(other)) = (blocked, other) {
            unsafe { T::global_assume_init_ref() }.dispatch_blocked(blocked, other);
        }
    }

    unsafe extern "C" fn dispatch_key_value(ent: *mut edict_s, data: *mut KeyValueData) {
        let ent = unsafe { ent.as_mut() };
        let data = unsafe { data.as_mut() };
        if let (Some(ent), Some(data)) = (ent, data) {
            let data = KeyValue::new(data);
            unsafe { T::global_assume_init_ref() }.dispatch_key_value(ent, data);
        }
    }

    unsafe extern "C" fn dispatch_save(ent: *mut edict_s, save_data: *mut SAVERESTOREDATA) {
        let ent = unsafe { ent.as_mut() };
        let save_data = unsafe { save_data.as_mut() };
        if let (Some(ent), Some(save_data)) = (ent, save_data) {
            let save_data = SaveRestoreData::new(save_data);
            unsafe { T::global_assume_init_ref() }.dispatch_save(ent, save_data);
        }
    }

    unsafe extern "C" fn dispatch_restore(
        ent: *mut edict_s,
        save_data: *mut SAVERESTOREDATA,
        global_entity: c_int,
    ) -> c_int {
        let ent = unsafe { ent.as_mut() };
        let save_data = unsafe { save_data.as_mut() };
        if let (Some(ent), Some(save_data)) = (ent, save_data) {
            let save_data = SaveRestoreData::new(save_data);
            let global_entity = global_entity != 0;
            return unsafe { T::global_assume_init_ref() }
                .dispatch_restore(ent, save_data, global_entity)
                .into();
        }
        RestoreResult::Delete.into()
    }

    unsafe extern "C" fn dispatch_object_collsion_box(ent: *mut edict_s) {
        if let Some(ent) = unsafe { ent.as_mut() } {
            unsafe { T::global_assume_init_ref() }.dispatch_object_collsion_box(ent);
        }
    }

    unsafe extern "C" fn save_write_fields(
        save_data: *mut SAVERESTOREDATA,
        name: *const c_char,
        base_data: *mut c_void,
        fields: *mut TYPEDESCRIPTION,
        fields_count: c_int,
    ) {
        unsafe {
            let save_data = save_data.as_mut().unwrap();
            let save_data = SaveRestoreData::new(save_data);
            let name = cstr_or_none(name).unwrap();
            let fields = slice_from_raw_parts_or_empty_mut(fields, fields_count as usize);
            let dll = T::global_assume_init_ref();
            dll.save_write_fields(save_data, name, base_data, fields);
        }
    }

    unsafe extern "C" fn save_read_fields(
        save_data: *mut SAVERESTOREDATA,
        name: *const c_char,
        base_data: *mut c_void,
        fields: *mut TYPEDESCRIPTION,
        fields_count: c_int,
    ) {
        unsafe {
            let save_data = save_data.as_mut().unwrap();
            let save_data = SaveRestoreData::new(save_data);
            let name = cstr_or_none(name).unwrap();
            let fields = slice_from_raw_parts_or_empty_mut(fields, fields_count as usize);
            let dll = T::global_assume_init_ref();
            dll.save_read_fields(save_data, name, base_data, fields);
        }
    }

    unsafe extern "C" fn save_global_state(save_data: *mut SAVERESTOREDATA) {
        let save_data = unsafe { save_data.as_mut() }.unwrap();
        let save_data = SaveRestoreData::new(save_data);
        unsafe { T::global_assume_init_ref() }.save_global_state(save_data);
    }

    unsafe extern "C" fn restore_global_state(save_data: *mut SAVERESTOREDATA) {
        let save_data = unsafe { save_data.as_mut() }.unwrap();
        let save_data = SaveRestoreData::new(save_data);
        unsafe { T::global_assume_init_ref() }.restore_global_state(save_data);
    }

    unsafe extern "C" fn reset_global_state() {
        unsafe { T::global_assume_init_ref() }.reset_global_state();
    }

    unsafe extern "C" fn client_connect(
        ent: *mut edict_s,
        name: *const c_char,
        address: *const c_char,
        reject_reason: *mut [c_char; 128],
    ) -> qboolean {
        let ent = unsafe { ent.as_mut() }.unwrap();
        let name = unsafe { cstr_or_none(name) }.unwrap();
        let address = unsafe { cstr_or_none(address) }.unwrap();
        let reject_reason = unsafe { reject_reason.cast::<CStrArray<128>>().as_mut() }.unwrap();
        reject_reason.clear();
        unsafe { T::global_assume_init_ref() }
            .client_connect(ent, name, address, reject_reason)
            .into()
    }

    unsafe extern "C" fn client_disconnect(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.client_disconnect(ent);
    }

    unsafe extern "C" fn client_kill(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.client_kill(ent);
    }

    unsafe extern "C" fn client_put_in_server(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.client_put_in_server(ent);
    }

    unsafe extern "C" fn client_command(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.client_command(ent);
    }

    unsafe extern "C" fn client_user_info_changed(ent: *mut edict_s, info_buffer: *mut c_char) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        let info_buffer = unsafe { cstr_or_none(info_buffer) }.unwrap();
        unsafe { T::global_assume_init_ref() }.client_user_info_changed(ent, info_buffer);
    }

    unsafe extern "C" fn server_activate(
        edict_list: *mut edict_s,
        edict_count: c_int,
        client_max: c_int,
    ) {
        let list = unsafe { slice_from_raw_parts_or_empty_mut(edict_list, edict_count as usize) };
        unsafe { T::global_assume_init_ref() }.server_activate(list, client_max)
    }

    unsafe extern "C" fn server_deactivate() {
        unsafe { T::global_assume_init_ref() }.server_deactivate();
    }

    unsafe extern "C" fn player_pre_think(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.player_pre_think(ent);
    }

    unsafe extern "C" fn player_post_think(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.player_post_think(ent);
    }

    unsafe extern "C" fn start_frame() {
        unsafe { T::global_assume_init_ref() }.start_frame();
    }

    unsafe extern "C" fn parms_new_level() {
        unsafe { T::global_assume_init_ref() }.parms_new_level();
    }

    unsafe extern "C" fn parms_change_level() {
        unsafe { T::global_assume_init_ref() }.parms_change_level();
    }

    unsafe extern "C" fn get_game_description() -> *const c_char {
        if INITIALIZED.load(Ordering::Relaxed) {
            unsafe { T::global_assume_init_ref() }
                .get_game_description()
                .as_ptr()
        } else {
            T::get_game_description_static().as_ptr()
        }
    }

    unsafe extern "C" fn player_customization(ent: *mut edict_s, custom: *mut customization_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        let custom = unsafe { custom.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.player_customization(ent, custom);
    }

    unsafe extern "C" fn spectator_connect(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.spectator_connect(ent);
    }

    unsafe extern "C" fn spectator_disconnect(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.spectator_disconnect(ent);
    }

    unsafe extern "C" fn spectator_think(ent: *mut edict_s) {
        let ent = unsafe { ent.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.spectator_think(ent);
    }

    unsafe extern "C" fn system_error(error_string: *const c_char) {
        let error_string = unsafe { cstr_or_none(error_string) }.unwrap();
        unsafe { T::global_assume_init_ref() }.system_error(error_string);
    }

    unsafe extern "C" fn player_move_init(pm: *mut playermove_s) {
        let pm = NonNull::new(pm).unwrap();
        unsafe { T::global_assume_init_ref() }.player_move_init(pm);
    }

    unsafe extern "C" fn player_move(pm: *mut playermove_s, is_server: qboolean) {
        let pm = NonNull::new(pm).unwrap();
        unsafe { T::global_assume_init_ref() }.player_move(pm, is_server != 0);
    }

    unsafe extern "C" fn player_move_find_texture_type(name: *mut c_char) -> c_char {
        let name = unsafe { cstr_or_none(name) }.unwrap();
        unsafe { T::global_assume_init_ref() }.player_move_find_texture_type(name)
    }

    unsafe extern "C" fn setup_visibility(
        view_entity: *mut edict_s,
        client: *mut edict_s,
        pvs: *mut *mut c_uchar,
        pas: *mut *mut c_uchar,
    ) {
        let view_entity = unsafe { view_entity.as_mut() };
        let client = unsafe { client.as_mut() }.unwrap();
        let pvs = unsafe { pvs.as_mut().unwrap() };
        let pas = unsafe { pas.as_mut().unwrap() };
        unsafe { T::global_assume_init_ref() }.setup_visibility(view_entity, client, pvs, pas);
    }

    unsafe extern "C" fn update_client_data(
        ent: *const edict_s,
        send_weapons: c_int,
        cd: *mut clientdata_s,
    ) {
        let ent = unsafe { ent.as_ref() }.unwrap();
        let send_weapons = send_weapons != 0;
        let cd = unsafe { cd.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.update_client_data(ent, send_weapons, cd);
    }

    unsafe extern "C" fn add_to_full_pack(
        state: *mut entity_state_s,
        e: c_int,
        ent: *mut edict_s,
        host: *mut edict_s,
        host_flags: c_int,
        player: c_int,
        set: *mut c_uchar,
    ) -> c_int {
        let state = unsafe { state.as_mut() }.unwrap();
        let ent = unsafe { ent.as_ref() }.unwrap();
        let host = unsafe { host.as_ref() }.unwrap();
        let player = player != 0;
        unsafe { T::global_assume_init_ref() }
            .add_to_full_pack(state, e, ent, host, host_flags, player, set) as c_int
    }

    unsafe extern "C" fn create_baseline(
        player: c_int,
        eindex: c_int,
        baseline: *mut entity_state_s,
        entity: *mut edict_s,
        player_model_index: c_int,
        player_mins: *mut vec3_t,
        player_maxs: *mut vec3_t,
    ) {
        let baseline = unsafe { baseline.as_mut() }.unwrap();
        let entity = unsafe { entity.as_mut() }.unwrap();
        let player_mins = *unsafe { player_mins.as_ref() }.unwrap();
        let player_maxs = *unsafe { player_maxs.as_ref() }.unwrap();
        unsafe { T::global_assume_init_ref() }.create_baseline(
            player != 0,
            eindex,
            baseline,
            entity,
            player_model_index,
            player_mins,
            player_maxs,
        );
    }

    unsafe extern "C" fn register_encoders() {
        unsafe { T::global_assume_init_ref() }.register_encoders();
    }

    unsafe extern "C" fn get_weapon_data(player: *mut edict_s, info: *mut weapon_data_s) -> c_int {
        assert!(!info.is_null());
        let player = unsafe { player.as_mut() }.unwrap();
        match unsafe { T::global_assume_init_ref() }.get_weapon_data(player) {
            Some(x) => {
                unsafe {
                    info.write(x);
                }
                1
            }
            None => {
                unsafe {
                    info.write_bytes(0, 1);
                }
                0
            }
        }
    }

    unsafe extern "C" fn command_start(
        player: *const edict_s,
        cmd: *const usercmd_s,
        random_seed: c_uint,
    ) {
        // FIXME: ffi: player must be mut
        let player = unsafe { player.cast_mut().as_mut() }.unwrap();
        let cmd = unsafe { cmd.as_ref() }.unwrap();
        unsafe { T::global_assume_init_ref() }.command_start(player, cmd, random_seed);
    }

    unsafe extern "C" fn command_end(player: *const edict_s) {
        // FIXME: ffi: player must be mut
        let player = unsafe { player.cast_mut().as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.command_end(player);
    }

    unsafe extern "C" fn connectionless_packet(
        from: *const netadr_s,
        args: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: *mut c_int,
    ) -> c_int {
        assert!(!response_buffer.is_null());
        let from = unsafe { from.as_ref() }.unwrap();
        let args = unsafe { cstr_or_none(args) }.unwrap();
        let response_buffer_size = unsafe { response_buffer_size.as_mut() }.unwrap();
        let max_buffer_size = *response_buffer_size as usize;
        let buffer = unsafe { slice::from_raw_parts_mut(response_buffer.cast(), max_buffer_size) };
        match unsafe { T::global_assume_init_ref() }.connectionless_packet(from, args, buffer) {
            Ok(len) => {
                *response_buffer_size = len as c_int;
                (len > 0) as c_int
            }
            Err(_) => 0,
        }
    }

    extern "C" fn get_hull_bounds(hullnumber: c_int, mins: *mut f32, maxs: *mut f32) -> c_int {
        let mins = unsafe { mins.cast::<vec3_t>().as_mut() }.unwrap();
        let maxs = unsafe { maxs.cast::<vec3_t>().as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.get_hull_bounds(hullnumber, mins, maxs)
    }

    unsafe extern "C" fn create_instanced_baselines() {
        unsafe { T::global_assume_init_ref() }.create_instanced_baselines();
    }

    unsafe extern "C" fn inconsistent_file(
        player: *const edict_s,
        filename: *const c_char,
        disconnect_message: *mut c_char,
    ) -> c_int {
        assert!(!disconnect_message.is_null());
        let player = unsafe { player.as_ref() }.unwrap();
        let filename = unsafe { cstr_or_none(filename) }.unwrap();
        let disconnect_message = unsafe { &mut *disconnect_message.cast() };
        unsafe { T::global_assume_init_ref() }.inconsistent_file(
            player,
            filename,
            disconnect_message,
        ) as c_int
    }

    unsafe extern "C" fn allow_lag_compensation() -> c_int {
        unsafe { T::global_assume_init_ref() }.allow_lag_compensation() as c_int
    }

    unsafe extern "C" fn on_free_entity_private_data(ent: *mut edict_s) {
        if !ent.is_null() {
            unsafe {
                T::global_assume_init_ref().on_free_entity_private_data(ent);
            }
        }
    }

    unsafe extern "C" fn should_collide(touched: *mut edict_s, other: *mut edict_s) -> c_int {
        let touched = unsafe { touched.as_mut() }.unwrap();
        let other = unsafe { other.as_mut() }.unwrap();
        unsafe { T::global_assume_init_ref() }.chould_collide(touched, other) as c_int
    }

    unsafe extern "C" fn cvar_value(ent: *const edict_s, value: *const c_char) {
        let ent = unsafe { ent.as_ref() }.unwrap();
        let value = unsafe { cstr_or_none(value) }.unwrap();
        unsafe { T::global_assume_init_ref() }.cvar_value(ent, value);
    }

    unsafe extern "C" fn cvar_value2(
        ent: *const edict_s,
        request_id: c_int,
        cvar_name: *const c_char,
        value: *const c_char,
    ) {
        let ent = unsafe { ent.as_ref() }.unwrap();
        let cvar_name = unsafe { cstr_or_none(cvar_name) }.unwrap();
        let value = unsafe { cstr_or_none(value) }.unwrap();
        unsafe { T::global_assume_init_ref() }.cvar_value2(ent, request_id, cvar_name, value);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! export_dll {
    ($server_dll:ty $($init:block)?) => {
        $crate::export::export_entity!(
            worldspawn,
            <$server_dll as $crate::export::ServerDll>::World,
            <$server_dll as $crate::export::ServerDll>::create_world
        );

        $crate::export::export_entity!(
            player,
            <$server_dll as $crate::export::ServerDll>::Player,
        );

        #[no_mangle]
        unsafe extern "system" fn GiveFnptrsToDll(
            eng_funcs: *const $crate::ffi::server::enginefuncs_s,
            globals: *mut $crate::ffi::server::globalvars_t,
        ) {
            unsafe {
                let eng_funcs = eng_funcs.as_ref().unwrap();
                $crate::instance::init_engine(eng_funcs, globals);
            }
        }

        #[no_mangle]
        unsafe extern "C" fn GetEntityAPI(
            dll_funcs: *mut $crate::ffi::server::DLL_FUNCTIONS,
            mut version: core::ffi::c_int,
        ) -> core::ffi::c_int {
            unsafe { GetEntityAPI2(dll_funcs, &mut version) }
        }

        #[no_mangle]
        unsafe extern "C" fn GetEntityAPI2(
            dll_funcs: *mut $crate::ffi::server::DLL_FUNCTIONS,
            version: *mut core::ffi::c_int,
        ) -> core::ffi::c_int {
            let expected = $crate::ffi::server::INTERFACE_VERSION as c_int;
            unsafe {
                if dll_funcs.is_null() || *version != expected {
                    *version = expected;
                    return 0;
                }
            }
            unsafe {
                *dll_funcs = $crate::export::dll_functions::<$server_dll>();
            }
            $($init)?
            1
        }

        #[no_mangle]
        unsafe extern "C" fn GetNewDLLFunctions(
            dll_funcs: *mut $crate::ffi::server::NEW_DLL_FUNCTIONS,
            version: *mut core::ffi::c_int,
        ) -> core::ffi::c_int {
            let expected = $crate::ffi::server::NEW_DLL_FUNCTIONS_VERSION as c_int;
            unsafe {
                if dll_funcs.is_null() || *version != expected {
                    *version = expected;
                    return 0;
                }
                *dll_funcs = $crate::export::new_dll_functions::<$server_dll>();
                1
            }
        }
    };
}
#[doc(inline)]
pub use export_dll;

/// Export an entity with the given name to the engine.
///
/// # Examples
///
/// ```
/// extern crate alloc;
///
/// use core::marker::PhantomData;
/// use xash3d_server::{
///     entity::{
///         Entity, BaseEntity, CreateEntity, PrivateEntity, impl_entity_cast,
///         delegate_entity,
///     },
///     export::export_entity,
///     save::{Save, Restore},
/// };
///
/// // define a private wrapper for our entities
/// struct Private<T>(PhantomData<T>);
///
/// impl<T: Entity> PrivateEntity for Private<T> {
///     type Entity = T;
/// }
///
/// // define a player entity
/// #[derive(Save, Restore)]
/// struct Player {
///     base: BaseEntity,
/// }
///
/// impl_entity_cast!(Player);
///
/// impl CreateEntity for Player {
///     fn create(base: BaseEntity) -> Self {
///         Self { base }
///     }
/// }
///
/// impl Entity for Player {
///     delegate_entity!(base);
/// }
///
/// // export the player entity to the engine
/// export_entity!(test_player, Private<Player>);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! export_entity {
    ($name:ident, $private:ty $(,)?) => {
        $crate::export::export_entity!(
            $name,
            $private,
            <$private as $crate::entity::PrivateEntity>::Entity::create,
        );
    };
    ($name:ident, $private:ty, $init:expr $(,)?) => {
        #[no_mangle]
        unsafe extern "C" fn $name(ev: *mut $crate::ffi::server::entvars_s) {
            #[allow(unused_imports)]
            use $crate::{
                engine::ServerEngineRef,
                entity::{CreateEntity, PrivateData, PrivateEntity},
                global_state::GlobalStateRef,
            };
            unsafe {
                let engine = ServerEngineRef::new();
                let global_state = GlobalStateRef::new();
                PrivateData::create_with::<$private>(engine, global_state, ev, $init);
            }
        }
    };
}
#[doc(inline)]
pub use export_entity;

#[doc(hidden)]
#[macro_export]
macro_rules! export_entity_default {
    ($feature:literal, $name:ident, $entity:ty $(, $init:expr)? $(,)?) => {
        #[cfg(any(feature = "export-default-entities", feature = $feature))]
        $crate::export::export_entity!($name, $crate::entity::Private<$entity> $(, $init)?);
    };
}
pub use export_entity_default;

#[doc(hidden)]
#[macro_export]
macro_rules! export_entity_stub {
    ($name:ident, $entity:ty $(,)?) => {
        #[cfg(any(feature = "export-stubs"))]
        $crate::export::export_entity!($name, $crate::entity::Private<$entity>);
    };
    ($name:ident $(,)?) => {
        $crate::export::export_entity_stub!($name, $crate::entity::StubEntity);
    };
}
pub use export_entity_stub;
