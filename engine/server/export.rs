use core::{
    ffi::{c_char, c_int, c_uchar, c_uint, c_void, CStr},
    fmt::Write,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
    sync::atomic::{AtomicBool, Ordering},
};

use csz::{CStrArray, CStrSlice, CStrThin};
use xash3d_player_move::{DUCK_HULL_MIN, HULL_MIN};
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
    changelevel::build_change_list,
    engine::ClientInfoBuffer,
    entity::{BaseEntity, EntityHandle, EntityPlayer, KeyValue, RestoreResult, UseType},
    global_state::{EntityState, GlobalState, GlobalStateRef},
    prelude::*,
    private::PrivateData,
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

pub fn dispatch_spawn(entity: &mut dyn Entity) -> SpawnResult {
    let engine = entity.engine();
    let global_state = engine.global_state_ref();
    let v = entity.vars();
    v.set_abs_min(v.origin() - vec3_t::splat(1.0));
    v.set_abs_max(v.origin() + vec3_t::splat(1.0));

    entity.spawn();

    if !global_state.game_rules().is_allowed_to_spawn(entity) {
        return SpawnResult::Delete;
    }

    if entity.vars().flags().intersects(EdictFlags::KILLME) {
        return SpawnResult::Delete;
    }

    if let Some(globalname) = entity.globalname() {
        let mut entities = global_state.entities_mut();
        let map_name = engine.globals.map_name().unwrap();
        if let Some(global) = entities.find(globalname) {
            if global.is_dead() {
                return SpawnResult::Delete;
            }
            if map_name.as_thin() != global.map_name() {
                entity.make_dormant();
            }
        } else {
            entities.add(globalname, map_name, EntityState::On);
        }
    }

    SpawnResult::Ok
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

    /// Dispatch spawn event to the entity.
    ///
    /// # Safety
    ///
    /// The behaviour is undefined if the entity is mutable borrowed somewhere else.
    unsafe fn dispatch_spawn(&self, mut entity: EntityHandle) -> SpawnResult {
        match unsafe { entity.get_entity_mut() } {
            Some(entity) => dispatch_spawn(entity),
            None => SpawnResult::Delete,
        }
    }

    fn dispatch_think(&self, ent: EntityHandle) {
        if let Some(entity) = ent.get_entity() {
            if entity.vars().flags().intersects(EdictFlags::DORMANT) {
                let classname = entity.classname();
                warn!("Dormant entity {classname:?} is thinkng");
            }
            entity.think();
        }
    }

    fn dispatch_use(&self, used: EntityHandle, other: EntityHandle) {
        let Some(used) = used.get_entity() else {
            return;
        };
        let Some(other) = other.get_entity() else {
            error!("dispatch_use: other private data is null");
            return;
        };
        if !used.vars().flags().intersects(EdictFlags::KILLME) {
            used.used(UseType::Toggle, Some(other), other);
        }
    }

    fn dispatch_touch(&self, touched: EntityHandle, other: EntityHandle) {
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

    fn dispatch_blocked(&self, blocked: EntityHandle, other: EntityHandle) {
        let Some(blocked) = blocked.get_entity() else {
            return;
        };
        let Some(other) = other.get_entity() else {
            error!("dispatch_blocked: other private data is null");
            return;
        };
        blocked.blocked(other);
    }

    fn dispatch_key_value(&self, mut ent: EntityHandle, data: &mut KeyValue) {
        if data.handled() || data.class_name().is_none() {
            return;
        }

        if let Some(ent) = unsafe { ent.get_entity_mut() } {
            ent.key_value(data);
        }
    }

    #[cfg(not(feature = "save"))]
    fn dispatch_save(&self, _ent: EntityHandle, _save_data: &mut SaveRestoreData) {
        error!("dispatch_save: feature \"save\" is not enabled");
    }

    #[cfg(not(feature = "save"))]
    fn dispatch_restore(
        &self,
        _ent: EntityHandle,
        _save_data: &mut SaveRestoreData,
        _global_entity: bool,
    ) -> RestoreResult {
        error!("dispatch_restore: feature \"save\" is not enabled");
        RestoreResult::Ok
    }

    #[cfg(feature = "save")]
    fn dispatch_save(&self, mut ent: EntityHandle, save_data: &mut SaveRestoreData) {
        use crate::{
            entity::{MoveType, ObjectCaps},
            save,
        };

        let engine = self.engine();
        let current_index = save_data.current_index();
        if save_data.table()[current_index].pent != ent.as_ptr() {
            error!("Entity table or index is wrong");
        }
        let Some(entity) = (unsafe { ent.get_entity_mut() }) else {
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
        if log_enabled!(target: "dispatch_save", log::Level::Trace) {
            let index = entity.entity_index();
            let name = entity.pretty_name();
            trace!(target: "dispatch_save", "save {index}:{name} to {location:#x}");
        }

        let (buffer, data) = save_data.split_mut();
        let mut state = save::SaveState::new(engine, data);
        let mut cur = save::CursorMut::new(buffer.as_slice_mut());
        let start_offset = cur.offset();

        // NOTE: Entity vars must be written at known location because we and the engine want
        // to read it for global entities.
        let result = save::write_fields(&mut state, &mut cur, unsafe { &*entity.vars().as_ptr() });

        // save other data
        let result = result.and_then(|_| cur.write_field(&mut state, ENTITY_SAVE_NAME, entity));

        let size = cur.offset() - start_offset;
        if let Err(err) = result {
            let name = entity.pretty_name();
            error!("dispatch_save: failed to save {name}, {err}");
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
        mut ent: EntityHandle,
        save_data: &mut SaveRestoreData,
        global_entity: bool,
    ) -> RestoreResult {
        use core::mem::MaybeUninit;

        use xash3d_shared::ffi::server::entvars_s;

        use crate::{
            entity::{EntityVars, ObjectCaps},
            save,
        };

        let engine = self.engine();
        let global_state = self.global_state();

        let mut global_mode = false;
        let mut old_offset = vec3_t::ZERO;

        if global_entity {
            let mut global_vars = MaybeUninit::<entvars_s>::zeroed();
            let mut reader = SaveReader::new(engine);
            reader.precache_mode(false);
            reader
                .read_fields(save_data, unsafe { global_vars.assume_init_mut() })
                .unwrap();

            let tmp_vars =
                unsafe { EntityVars::from_raw(engine, global_state, global_vars.as_mut_ptr()) };

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
            if let Some(new_ent) = engine.find_global_entity(classname, globalname) {
                global_mode = true;
                let mut landmark_offset = save_data.landmark_offset();
                landmark_offset -= new_ent.vars().min_size();
                landmark_offset += tmp_vars.min_size();
                save_data.set_landmark_offset(landmark_offset);
                ent = new_ent;
                entities.update(
                    ent.vars().globalname().unwrap(),
                    engine.globals.map_name().unwrap(),
                );
            } else {
                return RestoreResult::Ok;
            }
        }

        let Some(entity) = (unsafe { ent.get_entity_mut() }) else {
            return RestoreResult::Ok;
        };

        if log_enabled!(target: "dispatch_restore", log::Level::Trace) {
            let index = entity.entity_index();
            let name = entity.pretty_name();
            let location = save_data.offset();
            trace!(target: "dispatch_restore", "restore {index}:{name} from {location:#x}");
        }

        let (buffer, data) = save_data.split_mut();
        let mut state = save::RestoreState::new(engine, data);
        state.set_global(global_mode);
        let mut cur = save::Cursor::new(buffer.as_slice());
        let start_offset = cur.offset();

        // restore entity variables from known location
        let result = save::read_fields(&state, &mut cur, unsafe {
            &mut *entity.vars().as_mut_ptr()
        });

        // restore other data
        let result = result.and_then(|_| {
            cur.read_field().and_then(|field| {
                let name = state.token_str(field.token());
                assert_eq!(name, Some(ENTITY_SAVE_NAME.into()));
                entity.restore(&state, &mut field.cursor())
            })
        });

        let size = cur.offset() - start_offset;
        if let Err(err) = result {
            let name = entity.pretty_name();
            error!("dispatch_restore: failed to restore {name}, {err}",);
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

    fn dispatch_object_collsion_box(&self, ent: EntityHandle) {
        match ent.get_entity() {
            Some(entity) => entity.set_object_collision_box(),
            None => crate::entity::set_object_collision_box(&ent.vars()),
        }
    }

    unsafe fn save_write_fields(
        &self,
        save_data: &mut SaveRestoreData,
        name: &CStrThin,
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
        if let Err(e) = self.global_state().save_state(save_data) {
            error!("Failed to save global state: {e:?}");
        }
    }

    fn restore_global_state(&self, save_data: &mut SaveRestoreData) {
        if let Err(e) = self.global_state().restore_state(save_data) {
            error!("Failed to restore global state: {e:?}");
        }
    }

    fn reset_global_state(&self) {
        self.global_state().reset();
    }

    fn client_connect(
        &self,
        ent: EntityHandle,
        name: &CStrThin,
        address: &CStrThin,
        reject_reason: &mut CStrArray<128>,
    ) -> bool {
        true
    }

    fn client_disconnect(&self, ent: EntityHandle) {}

    fn client_kill(&self, ent: EntityHandle) {}

    fn client_put_in_server(&self, ent: EntityHandle) {
        let engine = self.engine();
        let global_state = self.global_state();
        let vars = ent.vars().as_mut_ptr();
        let player = unsafe { PrivateData::create::<Self::Player>(engine, global_state, vars) };

        player.spawn();

        let v = player.vars();
        v.with_effects(|f| f | Effects::NOINTERP);
        v.set_iuser1(0);
        v.set_iuser2(0);
    }

    fn client_command(&self, ent: EntityHandle) {}

    fn client_user_info_changed(&self, info_buffer: ClientInfoBuffer) {}

    fn server_activate(&self, list: impl Iterator<Item = EntityHandle>, client_max: c_int) {
        for (i, entity) in list.enumerate() {
            if entity.is_free() {
                continue;
            }

            if (1..=client_max as usize).contains(&i) {
                continue;
            }

            if let Some(entity) = entity.get_entity() {
                if !entity.is_dormant() {
                    entity.activate();
                } else {
                    error!("{}: failed to activate", entity.pretty_name());
                }
            }
        }
    }

    fn server_deactivate(&self) {}

    fn player_pre_think(&self, ent: EntityHandle) {
        if let Some(player) = ent.downcast_ref::<dyn EntityPlayer>() {
            player.pre_think();
        }
    }

    fn player_post_think(&self, ent: EntityHandle) {
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
            let count = build_change_list(&engine, &mut save_data.levelList);
            save_data.connectionCount = count as c_int;
            trace!("parms_change_level: connections {count}");
        }
    }

    /// Called before initialization.
    fn get_game_description_static() -> &'static CStr;

    /// Called after initialization.
    fn get_game_description(&self) -> &'static CStr {
        self.global_state().game_rules().get_game_description()
    }

    fn player_customization(&self, ent: EntityHandle, custom: &mut customization_s) {}

    fn spectator_connect(&self, ent: EntityHandle) {}

    fn spectator_disconnect(&self, ent: EntityHandle) {}

    fn spectator_think(&self, ent: EntityHandle) {}

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
        view_entity: Option<EntityHandle>,
        client: EntityHandle,
        pvs: &mut *mut c_uchar,
        pas: &mut *mut c_uchar,
    ) {
        if client.vars().flags().intersects(EdictFlags::PROXY) {
            *pvs = ptr::null_mut();
            *pas = ptr::null_mut();
            return;
        }

        let view = view_entity.unwrap_or(client);
        let mut org = view.vars().origin() + view.vars().view_ofs();
        if view.vars().flags().intersects(EdictFlags::DUCKING) {
            org += HULL_MIN.z - DUCK_HULL_MIN.z;
        }

        let engine = self.engine();
        *pvs = engine.set_pvs(org);
        *pas = engine.set_pas(org);
    }

    fn update_client_data(&self, ent: EntityHandle, send_weapons: bool, cd: &mut clientdata_s) {
        if ent.get_private().is_none() {
            return;
        }

        let engine = self.engine();
        let ev = ent.vars();

        // TODO:

        cd.flags = ev.flags().bits();
        cd.health = ev.health();

        cd.viewmodel = engine.model_index(
            ev.view_model_name()
                .as_ref()
                .map_or(c"".into(), |s| s.as_thin()),
        );

        cd.waterlevel = ev.water_level().into_raw();
        cd.watertype = ev.water_type();
        cd.weapons = ev.weapons() as i32;

        cd.origin = ev.origin();
        cd.velocity = ev.velocity();
        cd.view_ofs = ev.view_ofs();
        cd.punchangle = ev.punch_angle();

        cd.bInDuck = ev.in_duck().into();
        cd.flTimeStepSound = ev.time_step_sound();
        cd.flDuckTime = ev.duck_time();
        cd.flSwimTime = ev.swim_time();
        cd.waterjumptime = ev.teleport_time().as_secs_f32() as c_int;

        let cd_phys_info = CStrSlice::new_in_slice(&mut cd.physinfo);
        let phys_info = engine.get_physics_info_string(&ent).into();
        if cd_phys_info.cursor().write_c_str(phys_info).is_err() {
            error!("failed to write client data phys_info");
        }

        cd.maxspeed = ev.max_speed();
        cd.fov = ev.fov();
        cd.weaponanim = ev.weapon_animation();

        cd.pushmsec = ev.push_msec();

        // TODO: spectator mode

        cd.iuser1 = ev.iuser1();
        cd.iuser2 = ev.iuser2();

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
        ent: EntityHandle,
        host: EntityHandle,
        hostflags: c_int,
        player: bool,
        set: *mut c_uchar,
    ) -> bool {
        let ev = ent.vars();
        let hv = host.vars();

        if ent != host && ev.effects().intersects(Effects::NODRAW) {
            return false;
        }

        if ev.model_index().is_none() || ev.model_name().map_or(true, |s| s.is_empty()) {
            return false;
        }

        if ent != host && ev.flags().intersects(EdictFlags::SPECTATOR) {
            return false;
        }

        let engine = self.engine();
        if ent != host && !engine.check_visibility(&ent, set) {
            return false;
        }

        // do not send if the client say it is predicting the entity itself
        if ev.flags().intersects(EdictFlags::SKIPLOCALHOST)
            && hostflags & 1 != 0
            && ev.owner().map(|i| i.as_ptr()) == Some(host.as_ptr())
        {
            return false;
        }

        if hv.group_info() != 0 {
            warn!("add_to_full_pack: groupinfo is not implemented yet");
        }

        unsafe {
            ptr::write_bytes(state, 0, 1);
        }

        state.number = e;

        state.entityType = if ev.flags().intersects(EdictFlags::CUSTOMENTITY) {
            ENTITY_BEAM
        } else {
            ENTITY_NORMAL
        };

        state.animtime = ((1000.0 * ev.animation_time()) as i32) as f32 / 1000.0;

        state.origin = ev.origin();
        state.angles = ev.angles();
        state.mins = ev.min_size();
        state.maxs = ev.max_size();

        state.startpos = ev.start_pos();
        state.endpos = ev.end_pos();

        state.modelindex = ev.model_index_raw();

        state.frame = ev.frame();

        state.skin = ev.skin() as i16;
        state.effects = ev.effects().bits();

        if !player && ev.animation_time() != 0.0 && ev.velocity() == vec3_t::ZERO {
            state.eflags |= EFLAG_SLERP as u8;
        }

        state.scale = ev.scale();
        state.solid = ev.solid_raw() as i16;
        state.colormap = ev.color_map();

        state.movetype = ev.move_type_raw();
        state.sequence = ev.sequence();
        state.framerate = ev.framerate();
        state.body = ev.body();

        state.controller = ev.controller();
        state.blending[0] = ev.blending()[0];
        state.blending[1] = ev.blending()[1];

        state.rendermode = ev.render_mode_raw();
        state.renderamt = ev.render_amount() as i32;
        state.renderfx = ev.render_fx_raw();
        state.rendercolor.r = ev.render_color()[0] as u8;
        state.rendercolor.g = ev.render_color()[1] as u8;
        state.rendercolor.b = ev.render_color()[2] as u8;

        state.aiment = ev.aim_entity().map_or(0, |i| i.entity_index().to_i32());

        state.owner = 0;
        if let Some(owner) = ev.owner().map(|i| i.entity_index().to_i32()) {
            if owner >= 1 && owner <= engine.globals.max_clients() {
                state.owner = owner;
            }
        }

        if !player {
            state.playerclass = ev.player_class();
        }

        if player {
            state.basevelocity = ev.base_velocity();

            state.weaponmodel = ev
                .weapon_model_name()
                .map_or(0, |model_name| engine.model_index(model_name));

            state.gaitsequence = ev.gaitsequence();
            state.spectator = ev.flags().intersects(EdictFlags::SPECTATOR).into();
            state.friction = ev.friction();

            state.gravity = ev.gravity();
            state.team = ev.team();

            state.usehull = ev.flags().intersects(EdictFlags::DUCKING) as i32;
            state.health = ev.health() as i32;
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
        ent: EntityHandle,
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

    fn get_weapon_data(&self, player: EntityHandle) -> Option<weapon_data_s> {
        None
    }

    fn command_start(&self, player: EntityHandle, cmd: &usercmd_s, random_seed: c_uint) {}

    fn command_end(&self, player: EntityHandle) {}

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
        player: EntityHandle,
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

    fn chould_collide(&self, touched: EntityHandle, other: EntityHandle) -> bool {
        false
    }

    fn cvar_value(&self, ent: EntityHandle, value: &CStrThin) {}

    fn cvar_value2(
        &self,
        ent: EntityHandle,
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
            crate::logger::init_console_logger(&engine);
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
        unsafe {
            let engine = ServerEngineRef::new();
            if let Some(ent) = EntityHandle::new(engine, ent) {
                let dll = T::global_assume_init_ref();
                return dll.dispatch_spawn(ent).into();
            }
            SpawnResult::Delete.into()
        }
    }

    unsafe extern "C" fn dispatch_think(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            if let Some(ent) = EntityHandle::new(engine, ent) {
                let dll = T::global_assume_init_ref();
                dll.dispatch_think(ent);
            }
        }
    }

    unsafe extern "C" fn dispatch_use(used: *mut edict_s, other: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let used = EntityHandle::new(engine, used);
            let other = EntityHandle::new(engine, other);
            if let (Some(used), Some(other)) = (used, other) {
                let dll = T::global_assume_init_ref();
                dll.dispatch_use(used, other);
            }
        }
    }

    unsafe extern "C" fn dispatch_touch(touched: *mut edict_s, other: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let touched = EntityHandle::new(engine, touched);
            let other = EntityHandle::new(engine, other);
            if let (Some(touched), Some(other)) = (touched, other) {
                let dll = T::global_assume_init_ref();
                dll.dispatch_touch(touched, other);
            }
        }
    }

    unsafe extern "C" fn dispatch_blocked(blocked: *mut edict_s, other: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let blocked = EntityHandle::new(engine, blocked);
            let other = EntityHandle::new(engine, other);
            if let (Some(blocked), Some(other)) = (blocked, other) {
                let dll = T::global_assume_init_ref();
                dll.dispatch_blocked(blocked, other);
            }
        }
    }

    unsafe extern "C" fn dispatch_key_value(ent: *mut edict_s, data: *mut KeyValueData) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent);
            let data = data.as_mut();
            if let (Some(ent), Some(data)) = (ent, data) {
                let data = KeyValue::new(data);
                ent.vars().key_value(data);
                let dll = T::global_assume_init_ref();
                dll.dispatch_key_value(ent, data);
            }
        }
    }

    unsafe extern "C" fn dispatch_save(ent: *mut edict_s, save_data: *mut SAVERESTOREDATA) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent);
            let save_data = save_data.as_mut();
            if let (Some(ent), Some(save_data)) = (ent, save_data) {
                let save_data = SaveRestoreData::new(save_data);
                let dll = T::global_assume_init_ref();
                dll.dispatch_save(ent, save_data);
            }
        }
    }

    unsafe extern "C" fn dispatch_restore(
        ent: *mut edict_s,
        save_data: *mut SAVERESTOREDATA,
        global_entity: c_int,
    ) -> c_int {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent);
            let save_data = save_data.as_mut();
            if let (Some(ent), Some(save_data)) = (ent, save_data) {
                let save_data = SaveRestoreData::new(save_data);
                let global_entity = global_entity != 0;
                let dll = T::global_assume_init_ref();
                dll.dispatch_restore(ent, save_data, global_entity).into()
            } else {
                RestoreResult::Delete.into()
            }
        }
    }

    unsafe extern "C" fn dispatch_object_collsion_box(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            if let Some(ent) = EntityHandle::new(engine, ent) {
                let dll = T::global_assume_init_ref();
                dll.dispatch_object_collsion_box(ent);
            }
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
            let save_data = save_data.as_mut().expect("save_data must be non-null");
            let save_data = SaveRestoreData::new(save_data);
            let name = cstr_or_none(name).expect("name must be non-null");
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
            let save_data = save_data.as_mut().expect("save_data must be non-null");
            let save_data = SaveRestoreData::new(save_data);
            let name = cstr_or_none(name).expect("name must be non-null");
            let fields = slice_from_raw_parts_or_empty_mut(fields, fields_count as usize);
            let dll = T::global_assume_init_ref();
            dll.save_read_fields(save_data, name, base_data, fields);
        }
    }

    unsafe extern "C" fn save_global_state(save_data: *mut SAVERESTOREDATA) {
        unsafe {
            let save_data = save_data.as_mut().expect("save_data must be non-null");
            let save_data = SaveRestoreData::new(save_data);
            let dll = T::global_assume_init_ref();
            dll.save_global_state(save_data);
        }
    }

    unsafe extern "C" fn restore_global_state(save_data: *mut SAVERESTOREDATA) {
        unsafe {
            let save_data = save_data.as_mut().expect("save_data must be non-null");
            let save_data = SaveRestoreData::new(save_data);
            let dll = T::global_assume_init_ref();
            dll.restore_global_state(save_data);
        }
    }

    unsafe extern "C" fn reset_global_state() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.reset_global_state();
    }

    unsafe extern "C" fn client_connect(
        ent: *mut edict_s,
        name: *const c_char,
        address: *const c_char,
        reject_reason: *mut [c_char; 128],
    ) -> qboolean {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let name = cstr_or_none(name).expect("name must be non-null");
            let address = cstr_or_none(address).expect("address must be non-null");
            let reject_reason = reject_reason
                .cast::<CStrArray<128>>()
                .as_mut()
                .expect("reject_reason must be non-null");
            reject_reason.clear();
            let dll = T::global_assume_init_ref();
            dll.client_connect(ent, name, address, reject_reason).into()
        }
    }

    unsafe extern "C" fn client_disconnect(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.client_disconnect(ent);
        }
    }

    unsafe extern "C" fn client_kill(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.client_kill(ent);
        }
    }

    unsafe extern "C" fn client_put_in_server(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.client_put_in_server(ent);
        }
    }

    unsafe extern "C" fn client_command(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.client_command(ent);
        }
    }

    unsafe extern "C" fn client_user_info_changed(ent: *mut edict_s, info_buffer: *mut c_char) {
        unsafe {
            assert!(!info_buffer.is_null(), "info_buffer must be non-null");
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let info_buffer = ClientInfoBuffer::new(engine, ent, info_buffer);
            let dll = T::global_assume_init_ref();
            dll.client_user_info_changed(info_buffer);
        }
    }

    unsafe extern "C" fn server_activate(
        edict_list: *mut edict_s,
        edict_count: c_int,
        client_max: c_int,
    ) {
        unsafe {
            let engine = ServerEngineRef::new();
            let list = (0..edict_count).map(|i| {
                let raw = edict_list.wrapping_add(i as usize);
                EntityHandle::new_unchecked(engine, raw)
            });
            let dll = T::global_assume_init_ref();
            dll.server_activate(list, client_max);
        }
    }

    unsafe extern "C" fn server_deactivate() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.server_deactivate();
    }

    unsafe extern "C" fn player_pre_think(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.player_pre_think(ent);
        }
    }

    unsafe extern "C" fn player_post_think(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.player_post_think(ent);
        }
    }

    unsafe extern "C" fn start_frame() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.start_frame();
    }

    unsafe extern "C" fn parms_new_level() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.parms_new_level();
    }

    unsafe extern "C" fn parms_change_level() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.parms_change_level();
    }

    unsafe extern "C" fn get_game_description() -> *const c_char {
        if INITIALIZED.load(Ordering::Relaxed) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.get_game_description().as_ptr()
        } else {
            T::get_game_description_static().as_ptr()
        }
    }

    unsafe extern "C" fn player_customization(ent: *mut edict_s, custom: *mut customization_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let custom = custom.as_mut().expect("custom must be non-null");
            let dll = T::global_assume_init_ref();
            dll.player_customization(ent, custom);
        }
    }

    unsafe extern "C" fn spectator_connect(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.spectator_connect(ent);
        }
    }

    unsafe extern "C" fn spectator_disconnect(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.spectator_disconnect(ent);
        }
    }

    unsafe extern "C" fn spectator_think(ent: *mut edict_s) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let dll = T::global_assume_init_ref();
            dll.spectator_think(ent);
        }
    }

    unsafe extern "C" fn system_error(error_string: *const c_char) {
        let error_string =
            unsafe { cstr_or_none(error_string) }.expect("error_string must be non-null");
        let dll = unsafe { T::global_assume_init_ref() };
        dll.system_error(error_string);
    }

    unsafe extern "C" fn player_move_init(pm: *mut playermove_s) {
        let pm = NonNull::new(pm).expect("pm must be non-null");
        let dll = unsafe { T::global_assume_init_ref() };
        dll.player_move_init(pm);
    }

    unsafe extern "C" fn player_move(pm: *mut playermove_s, is_server: qboolean) {
        let pm = NonNull::new(pm).expect("pm must be non-null");
        let dll = unsafe { T::global_assume_init_ref() };
        dll.player_move(pm, is_server != 0);
    }

    unsafe extern "C" fn player_move_find_texture_type(name: *mut c_char) -> c_char {
        let name = unsafe { cstr_or_none(name) }.expect("name must be non-null");
        let dll = unsafe { T::global_assume_init_ref() };
        dll.player_move_find_texture_type(name)
    }

    unsafe extern "C" fn setup_visibility(
        view_entity: *mut edict_s,
        client: *mut edict_s,
        pvs: *mut *mut c_uchar,
        pas: *mut *mut c_uchar,
    ) {
        unsafe {
            let engine = ServerEngineRef::new();
            let view_entity = EntityHandle::new(engine, view_entity);
            let client = EntityHandle::new(engine, client).expect("client must be non-null");
            let pvs = pvs.as_mut().expect("pvs must be non-null");
            let pas = pas.as_mut().expect("pas must be non-null");
            let dll = T::global_assume_init_ref();
            dll.setup_visibility(view_entity, client, pvs, pas);
        }
    }

    unsafe extern "C" fn update_client_data(
        ent: *const edict_s,
        send_weapons: c_int,
        cd: *mut clientdata_s,
    ) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent.cast_mut()).expect("ent must be non-null");
            let send_weapons = send_weapons != 0;
            let cd = cd.as_mut().expect("client data must be non-null");
            let dll = T::global_assume_init_ref();
            dll.update_client_data(ent, send_weapons, cd);
        }
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
        unsafe {
            let engine = ServerEngineRef::new();
            let state = state.as_mut().expect("state must be non-null");
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let host = EntityHandle::new(engine, host).expect("ent must be non-null");
            let player = player != 0;
            let dll = T::global_assume_init_ref();
            dll.add_to_full_pack(state, e, ent, host, host_flags, player, set) as c_int
        }
    }

    unsafe extern "C" fn create_baseline(
        player: c_int,
        eindex: c_int,
        baseline: *mut entity_state_s,
        ent: *mut edict_s,
        player_model_index: c_int,
        player_mins: *mut vec3_t,
        player_maxs: *mut vec3_t,
    ) {
        unsafe {
            let engine = ServerEngineRef::new();
            let baseline = baseline.as_mut().expect("baseline must be non-null");
            let ent = EntityHandle::new(engine, ent).expect("ent must be non-null");
            let player_mins = *player_mins.as_ref().expect("player_mins must be non-null");
            let player_maxs = *player_maxs.as_ref().expect("player_maxs must be non-null");
            let dll = T::global_assume_init_ref();
            dll.create_baseline(
                player != 0,
                eindex,
                baseline,
                ent,
                player_model_index,
                player_mins,
                player_maxs,
            );
        }
    }

    unsafe extern "C" fn register_encoders() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.register_encoders();
    }

    unsafe extern "C" fn get_weapon_data(player: *mut edict_s, info: *mut weapon_data_s) -> c_int {
        unsafe {
            assert!(!info.is_null(), "info must be non-null");
            let engine = ServerEngineRef::new();
            let player = EntityHandle::new(engine, player).expect("player must be non-null");
            let dll = T::global_assume_init_ref();
            match dll.get_weapon_data(player) {
                Some(x) => {
                    info.write(x);
                    1
                }
                None => {
                    info.write_bytes(0, 1);
                    0
                }
            }
        }
    }

    unsafe extern "C" fn command_start(
        // FIXME: ffi: player must be mut
        player: *const edict_s,
        cmd: *const usercmd_s,
        random_seed: c_uint,
    ) {
        unsafe {
            let engine = ServerEngineRef::new();
            let player =
                EntityHandle::new(engine, player.cast_mut()).expect("player must be non-null");
            let cmd = cmd.as_ref().expect("cmd must be non-null");
            let dll = T::global_assume_init_ref();
            dll.command_start(player, cmd, random_seed);
        }
    }

    unsafe extern "C" fn command_end(player: *const edict_s) {
        // FIXME: ffi: player must be mut
        unsafe {
            let engine = ServerEngineRef::new();
            let player =
                EntityHandle::new(engine, player.cast_mut()).expect("player must be non-null");
            let dll = T::global_assume_init_ref();
            dll.command_end(player);
        }
    }

    unsafe extern "C" fn connectionless_packet(
        from: *const netadr_s,
        args: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: *mut c_int,
    ) -> c_int {
        assert!(
            !response_buffer.is_null(),
            "response_buffer must be non-null"
        );
        unsafe {
            let from = from.as_ref().expect("from must be non-null");
            let args = cstr_or_none(args).expect("args must be non-null");
            let response_buffer_size = response_buffer_size
                .as_mut()
                .expect("response_buffer_size must be non-null");
            let max_buffer_size = *response_buffer_size as usize;
            let buffer = slice::from_raw_parts_mut(response_buffer.cast(), max_buffer_size);
            let dll = T::global_assume_init_ref();
            match dll.connectionless_packet(from, args, buffer) {
                Ok(len) => {
                    *response_buffer_size = len as c_int;
                    (len > 0) as c_int
                }
                Err(_) => 0,
            }
        }
    }

    extern "C" fn get_hull_bounds(hullnumber: c_int, mins: *mut f32, maxs: *mut f32) -> c_int {
        unsafe {
            let mins = mins
                .cast::<vec3_t>()
                .as_mut()
                .expect("mins must be non-null");
            let maxs = maxs
                .cast::<vec3_t>()
                .as_mut()
                .expect("maxs must be non-null");
            let dll = T::global_assume_init_ref();
            dll.get_hull_bounds(hullnumber, mins, maxs)
        }
    }

    unsafe extern "C" fn create_instanced_baselines() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.create_instanced_baselines();
    }

    unsafe extern "C" fn inconsistent_file(
        player: *const edict_s,
        filename: *const c_char,
        disconnect_message: *mut c_char,
    ) -> c_int {
        unsafe {
            assert!(
                !disconnect_message.is_null(),
                "disconnect_message must be non-null"
            );
            let engine = ServerEngineRef::new();
            let player =
                EntityHandle::new(engine, player.cast_mut()).expect("player must be non-null");
            let filename = cstr_or_none(filename).expect("filename must be non-null");
            let disconnect_message = &mut *disconnect_message.cast();
            let dll = T::global_assume_init_ref();
            dll.inconsistent_file(player, filename, disconnect_message) as c_int
        }
    }

    unsafe extern "C" fn allow_lag_compensation() -> c_int {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.allow_lag_compensation() as c_int
    }

    unsafe extern "C" fn on_free_entity_private_data(ent: *mut edict_s) {
        if !ent.is_null() {
            unsafe {
                let dll = T::global_assume_init_ref();
                dll.on_free_entity_private_data(ent);
            }
        }
    }

    unsafe extern "C" fn should_collide(touched: *mut edict_s, other: *mut edict_s) -> c_int {
        unsafe {
            let engine = ServerEngineRef::new();
            let touched = EntityHandle::new(engine, touched).expect("touched must be non-null");
            let other = EntityHandle::new(engine, other).expect("other must be non-null");
            let dll = T::global_assume_init_ref();
            dll.chould_collide(touched, other) as c_int
        }
    }

    unsafe extern "C" fn cvar_value(ent: *const edict_s, value: *const c_char) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent.cast_mut()).expect("ent must be non-null");
            let value = cstr_or_none(value).expect("value must be non-null");
            let dll = T::global_assume_init_ref();
            dll.cvar_value(ent, value);
        }
    }

    unsafe extern "C" fn cvar_value2(
        ent: *const edict_s,
        request_id: c_int,
        cvar_name: *const c_char,
        value: *const c_char,
    ) {
        unsafe {
            let engine = ServerEngineRef::new();
            let ent = EntityHandle::new(engine, ent.cast_mut()).expect("ent must be non-null");
            let cvar_name = cstr_or_none(cvar_name).expect("cvar_name must be non-null");
            let value = cstr_or_none(value).expect("value must be non-null");
            let dll = T::global_assume_init_ref();
            dll.cvar_value2(ent, request_id, cvar_name, value);
        }
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
/// use xash3d_server::{
///     prelude::*,
///     entity::{delegate_entity, delegate_player, BaseEntity, EntityPlayer},
///     export::export_entity,
///     save::{Save, Restore},
/// };
/// use xash3d_entities::player::Player as BasePlayer;
///
/// // define a player entity
/// #[derive(Save, Restore)]
/// struct Player {
///     base: BasePlayer,
/// }
///
/// impl CreateEntity for Player {
///     fn create(base: BaseEntity) -> Self {
///         Self { base: BasePlayer::create(base) }
///     }
/// }
///
/// impl Entity for Player {
///     delegate_entity!(base);
/// }
///
/// impl EntityPlayer for Player {
///     delegate_player!(base);
/// }
///
/// // export the player entity to the engine
/// export_entity!(test_player, Player {
///     // downcast to EntityPlayer if EntityPlayer is implemented
///     ?EntityPlayer,
///
///     // downcast to EntityPlayer or compile error if not implemented
///     EntityPlayer,
/// });
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! export_entity {
    (
        $name:ident,
        $entity:ty { $( ?$opt:path ),+ $(, $trait:path )* $(,)? }
        $(, $init:expr)?
        $(,)?
    ) => {
        $crate::private::impl_private!($entity { $( ?$opt ),+ $(, $trait )* });
        $crate::export::export_entity!($name, $entity $(, $init )?);
    };
    (
        $name:ident,
        $entity:ty { $( $trait:path ),* $(,)? }
        $(, $init:expr)?
        $(,)?
    ) => {
        $crate::private::impl_private!($entity { $($trait ),* });
        $crate::export::export_entity!($name, $entity $(, $init )?);
    };
    ($name:ident, $entity:ty $(,)?) => {
        $crate::export::export_entity!(
            $name,
            $entity,
            <$entity as $crate::private::PrivateEntity>::Entity::create,
        );
    };
    ($name:ident, $entity:ty, $init:expr $(,)?) => {
        #[no_mangle]
        unsafe extern "C" fn $name(ev: *mut $crate::ffi::server::entvars_s) {
            #[allow(unused_imports)]
            use $crate::{
                engine::ServerEngineRef,
                entity::CreateEntity,
                global_state::GlobalStateRef,
                private::{PrivateData, PrivateEntity},
            };
            unsafe {
                let engine = ServerEngineRef::new();
                let global_state = GlobalStateRef::new();
                PrivateData::create_with::<$entity>(engine, global_state, ev, $init);
            }
        }
    };
}
#[doc(inline)]
pub use export_entity;
