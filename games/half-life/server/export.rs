use core::ffi::{c_int, c_uchar};

use xash3d_server::{
    export::{export_dll, impl_unsync_global, ServerDll},
    ffi::{
        common::{clientdata_s, entity_state_s},
        server::edict_s,
    },
    global_state::GlobalStateRef,
    prelude::*,
};

use crate::{player, triggers};

struct Dll {
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
}

impl_unsync_global!(Dll);

impl ServerDll for Dll {
    fn new(engine: ServerEngineRef, global_state: GlobalStateRef) -> Self {
        crate::cvar::init(engine);
        Self {
            engine,
            global_state,
        }
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }

    fn client_put_in_server(&self, ent: &mut edict_s) {
        player::client_put_in_server(self.engine, self.global_state, ent);
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

export_dll!(Dll);
