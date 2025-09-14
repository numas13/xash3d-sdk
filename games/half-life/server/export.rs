use core::{
    ffi::{c_int, c_uchar, c_void, CStr},
    ptr,
};

use csz::CStrThin;
use pm::{VEC_DUCK_HULL_MIN, VEC_HULL_MIN};
use sv::{
    entity::EdictFlags,
    export::{export_dll, impl_unsync_global, RestoreResult, ServerDll, SpawnResult},
    ffi::{
        common::{clientdata_s, entity_state_s, vec3_t},
        server::{edict_s, KeyValueData, SAVERESTOREDATA, TYPEDESCRIPTION},
    },
    prelude::*,
};

use crate::{
    gamerules::game_rules,
    global_state::{global_state, EntityState},
    player,
    private_data::{Private, PrivateDataRef},
    save, triggers,
};

struct Instance {}

impl Default for Instance {
    fn default() -> Self {
        crate::cvar::init();
        Self {}
    }
}

impl_unsync_global!(Instance);

impl ServerDll for Instance {
    fn dispatch_spawn(&self, ent: &mut edict_s) -> SpawnResult {
        let Some(ent) = ent.private_mut() else {
            return SpawnResult::Delete;
        };

        let ev = ent.vars_mut();
        ev.absmin = ev.origin - vec3_t::splat(1.0);
        ev.absmax = ev.origin + vec3_t::splat(1.0);

        if !ent.spawn() {
            return SpawnResult::Delete;
        }

        if let Some(false) = game_rules().map(|rules| rules.is_allowed_to_spawn(&**ent)) {
            return SpawnResult::Delete;
        }

        if ent.vars().flags().intersects(EdictFlags::KILLME) {
            return SpawnResult::Delete;
        }

        if let Some(globalname) = ent.vars().globalname() {
            let global_state = global_state();
            let mut entities = global_state.entities.borrow_mut();
            let map_name = globals().map_name().unwrap();
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
        if let Some(entity) = ent.private_mut() {
            if entity.vars().flags().intersects(EdictFlags::DORMANT) {
                let classname = entity.classname();
                warn!("Dormant entity {classname:?} is thinkng");
            }
            entity.think();
        }
    }

    fn dispatch_use(&self, _used: &mut edict_s, _other: &mut edict_s) {}

    fn dispatch_touch(&self, touched: &mut edict_s, other: &mut edict_s) {
        crate::todo::dispatch_touch(touched, other);
        let touched = touched.private().unwrap();
        let other = other.private().unwrap();
        trace!(
            "Touch entity {} by {}",
            touched.classname(),
            other.classname()
        );
    }

    fn dispatch_blocked(&self, _blocked: &mut edict_s, _other: &mut edict_s) {}

    fn dispatch_key_value(&self, ent: &mut edict_s, data: &mut KeyValueData) {
        save::dispatch_key_value(ent, data);
    }

    fn dispatch_save(&self, ent: &mut edict_s, save_data: &mut SAVERESTOREDATA) {
        save::dispatch_save(ent, save_data);
    }

    fn dispatch_restore(
        &self,
        ent: &mut edict_s,
        save_data: &mut SAVERESTOREDATA,
        global_entity: bool,
    ) -> RestoreResult {
        save::dispatch_restore(ent, save_data, global_entity)
    }

    fn dispatch_object_collsion_box(&self, ent: &mut edict_s) {
        crate::todo::dispatch_object_collision_box(ent);
    }

    fn save_write_fields(
        &self,
        save_data: &mut SAVERESTOREDATA,
        name: &CStrThin,
        base_data: *mut c_void,
        fields: &mut [TYPEDESCRIPTION],
    ) {
        save::write_fields(save_data, name.as_c_str(), base_data, fields);
    }

    fn save_read_fields(
        &self,
        save_data: &mut SAVERESTOREDATA,
        name: &CStrThin,
        base_data: *mut c_void,
        fields: &mut [TYPEDESCRIPTION],
    ) {
        save::read_fields(&mut *save_data, name.as_c_str(), base_data, fields);
    }

    fn save_global_state(&self, save_data: &mut SAVERESTOREDATA) {
        if let Err(e) = global_state().save(save_data) {
            error!("Failed to save global state: {e:?}");
        }
    }

    fn restore_global_state(&self, save_data: &mut SAVERESTOREDATA) {
        if let Err(e) = global_state().restore(save_data) {
            error!("Failed to restore global state: {e:?}");
        }
    }

    fn reset_global_state(&self) {
        global_state().reset();
    }

    fn client_put_in_server(&self, ent: &mut edict_s) {
        player::client_put_in_server(ent);
    }

    fn client_command(&self, ent: &mut edict_s) {
        let classname = ent.private().map(|pd| pd.classname());
        let classname = classname
            .as_ref()
            .map_or(c"unknown".into(), |s| s.as_thin());
        let engine = engine();
        let cmd = engine.cmd_argv(0);
        let args = engine.cmd_args_raw().unwrap_or_default();
        debug!("{classname}: client command \"{cmd} {args}\"");
    }

    fn parms_change_level(&self) {
        if let Some(mut save_data) = globals().save_data() {
            let save_data = unsafe { save_data.as_mut() };
            save_data.connectionCount =
                triggers::build_change_list(&mut save_data.levelList) as c_int;
        }
    }

    fn get_game_description(&self) -> &'static CStr {
        game_rules().map_or(c"Half-Life", |rules| rules.get_game_description())
    }

    fn setup_visibility(
        &self,
        view_entity: Option<&mut edict_s>,
        client: &mut edict_s,
        pvs: *mut *mut c_uchar,
        pas: *mut *mut c_uchar,
    ) {
        if client.v.flags().intersects(EdictFlags::PROXY) {
            unsafe {
                *pvs = ptr::null_mut();
                *pas = ptr::null_mut();
            }
            return;
        }

        let view = view_entity.unwrap_or(client);
        let mut org = view.v.origin + view.v.view_ofs;
        if view.v.flags().intersects(EdictFlags::DUCKING) {
            org += VEC_HULL_MIN - VEC_DUCK_HULL_MIN;
        }

        let engine = engine();
        unsafe {
            *pvs = engine.set_pvs(org);
            *pas = engine.set_pas(org);
        }
    }

    fn update_client_data(&self, ent: &edict_s, send_weapons: bool, cd: &mut clientdata_s) {
        crate::todo::update_client_data(ent, send_weapons, cd);
    }

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
        crate::todo::add_to_full_pack(state, e, ent, host, hostflags, player, set)
    }

    fn create_baseline(
        &self,
        player: bool,
        eindex: c_int,
        baseline: &mut entity_state_s,
        entity: &mut edict_s,
        player_model_index: c_int,
        player_mins: vec3_t,
        player_maxs: vec3_t,
    ) {
        crate::todo::create_baseline(
            player,
            eindex,
            baseline,
            entity,
            player_model_index,
            player_mins,
            player_maxs,
        );
    }

    unsafe fn on_free_entity_private_data(&self, ent: *mut edict_s) {
        unsafe { PrivateDataRef::free(ent) }
    }
}

export_dll!(Instance);
