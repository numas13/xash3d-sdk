use core::{
    ffi::{c_int, c_uchar, CStr},
    mem::MaybeUninit,
};

use xash3d_server::{
    entity::{EdictFlags, GetPrivateData, ObjectCaps, RestoreResult},
    export::{export_dll, impl_unsync_global, ServerDll, SpawnResult, UnsyncGlobal},
    ffi::{
        common::{clientdata_s, entity_state_s, vec3_t},
        server::{edict_s, entvars_s, SAVERESTOREDATA},
    },
    game_rules::GameRulesRef,
    prelude::*,
    save::SaveReader,
    str::MapString,
};

use crate::{
    global_state::{EntityState, GlobalState},
    player, triggers,
};

pub fn global_state() -> &'static GlobalState {
    unsafe { &Dll::global_assume_init_ref().global_state }
}

struct Dll {
    engine: ServerEngineRef,
    game_rules: GameRulesRef,
    global_state: GlobalState,
}

impl_unsync_global!(Dll);

impl ServerDll for Dll {
    fn new(engine: ServerEngineRef) -> Self {
        crate::cvar::init(engine);
        Self {
            engine,
            game_rules: unsafe { GameRulesRef::new() },
            global_state: GlobalState::new(engine),
        }
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn dispatch_spawn(&self, ent: &mut edict_s) -> SpawnResult {
        let Some(ent) = ent.get_entity_mut() else {
            return SpawnResult::Delete;
        };

        let ev = ent.vars_mut().as_raw_mut();
        ev.absmin = ev.origin - vec3_t::splat(1.0);
        ev.absmax = ev.origin + vec3_t::splat(1.0);

        ent.spawn();

        if let Some(false) = self
            .game_rules
            .get()
            .map(|rules| rules.is_allowed_to_spawn(ent))
        {
            return SpawnResult::Delete;
        }

        if ent.vars().flags().intersects(EdictFlags::KILLME) {
            return SpawnResult::Delete;
        }

        if let Some(globalname) = ent.vars().globalname() {
            let mut entities = self.global_state.entities.borrow_mut();
            let map_name = self.engine.globals.map_name().unwrap();
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

    fn dispatch_restore(
        &self,
        mut ent: &mut edict_s,
        save_data: &mut SAVERESTOREDATA,
        global_entity: bool,
    ) -> RestoreResult {
        let engine = self.engine();
        let mut global_vars = MaybeUninit::<entvars_s>::uninit();

        if global_entity {
            let mut restore = SaveReader::new(engine, save_data);
            restore.precache_mode(false);
            restore
                .read_ent_vars(c"ENTVARS", global_vars.as_mut_ptr())
                .unwrap();
        }

        let mut restore = SaveReader::new(engine, save_data);
        let mut old_offset = vec3_t::ZERO;

        if global_entity {
            let tmp_vars = unsafe { global_vars.assume_init_mut() };
            // HACK: restore save pointers
            restore.data.restore_save_pointers();

            let mut entities = self.global_state.entities.borrow_mut();
            let global = entities.find(tmp_vars.globalname().unwrap()).unwrap();
            if restore.data.current_map_name() != global.map_name() {
                return RestoreResult::Ok;
            }

            old_offset = restore.data.landmark_offset();
            let classname = tmp_vars.classname().unwrap();
            let globalname = tmp_vars.globalname().unwrap();
            if let Some(new_ent) = find_global_entity(engine, classname, globalname) {
                let new_ent = unsafe { &mut *new_ent };
                restore.global_mode(true);
                let mut landmark_offset = restore.data.landmark_offset();
                landmark_offset -= new_ent.v.mins;
                landmark_offset += tmp_vars.mins;
                restore.data.set_landmark_offset(landmark_offset);
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
        entity.restore(&mut restore).unwrap();
        if entity.object_caps().intersects(ObjectCaps::MUST_SPAWN) {
            entity.spawn();
        } else {
            entity.precache();
        }

        if global_entity {
            restore.data.set_landmark_offset(old_offset);
            let origin = entity.vars().as_raw().origin;
            engine.set_origin(entity.as_edict_mut(), origin);
            entity.override_reset();
            return RestoreResult::Ok;
        } else if let Some(globalname) = entity.vars().globalname() {
            let globals = &engine.globals;
            let mut entities = self.global_state.entities.borrow_mut();
            if let Some(global) = entities.find(globalname) {
                if global.is_dead() {
                    return RestoreResult::Delete;
                }
                if globals.map_name().unwrap().as_thin() != global.map_name() {
                    entity.make_dormant();
                }
            } else {
                let globalname = entity.globalname();
                let classname = entity.classname();
                error!("Global entity \"{globalname}\" (\"{classname}\") not in table!!!");
                entities.add(globalname, globals.map_name().unwrap(), EntityState::On);
            }
        }

        RestoreResult::Ok
    }

    fn save_global_state(&self, save_data: &mut SAVERESTOREDATA) {
        if let Err(e) = self.global_state.save(save_data) {
            error!("Failed to save global state: {e:?}");
        }
    }

    fn restore_global_state(&self, save_data: &mut SAVERESTOREDATA) {
        if let Err(e) = self.global_state.restore(save_data) {
            error!("Failed to restore global state: {e:?}");
        }
    }

    fn reset_global_state(&self) {
        self.global_state.reset();
    }

    fn client_put_in_server(&self, ent: &mut edict_s) {
        player::client_put_in_server(self.engine, ent);
    }

    fn client_command(&self, ent: &mut edict_s) {
        let classname = ent.get_entity_mut().map(|pd| pd.classname());
        let classname = classname
            .as_ref()
            .map_or(c"unknown".into(), |s| s.as_thin());
        let engine = self.engine;
        let cmd = engine.cmd_argv(0);
        let args = engine.cmd_args_raw().unwrap_or_default();
        debug!("{classname}: client command \"{cmd} {args}\"");
    }

    fn parms_change_level(&self) {
        if let Some(mut save_data) = self.engine.globals.save_data() {
            let save_data = unsafe { save_data.as_mut() };
            save_data.connectionCount =
                triggers::build_change_list(self.engine, &mut save_data.levelList) as c_int;
        }
    }

    fn get_game_description(&self) -> &'static CStr {
        self.game_rules
            .get()
            .map_or(c"Half-Life", |rules| rules.get_game_description())
    }

    fn update_client_data(&self, ent: &edict_s, send_weapons: bool, cd: &mut clientdata_s) {
        crate::todo::update_client_data(self.engine, ent, send_weapons, cd);
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
        crate::todo::add_to_full_pack(self.engine, state, e, ent, host, hostflags, player, set)
    }
}

fn find_global_entity(
    engine: ServerEngineRef,
    classname: MapString,
    globalname: MapString,
) -> Option<*mut edict_s> {
    engine
        .find_ent_by_globalname_iter(&globalname)
        .find(|&ent| {
            if let Some(entity) = unsafe { &mut *ent }.get_entity_mut() {
                if entity.is_classname(&classname) {
                    return true;
                } else {
                    debug!("Global entity found \"{globalname}\", wrong class \"{classname}\"");
                }
            }
            false
        })
}

export_dll!(Dll);
