use core::{
    ffi::{c_char, c_int, c_uchar, c_uint, c_void, CStr},
    fmt::Write,
    marker::PhantomData,
    ptr::NonNull,
    slice,
};

use csz::{CStrArray, CStrThin};
use shared::{engine::net::netadr_s, raw::playermove_s};

use crate::{
    prelude::*,
    raw::{
        self, clientdata_s, customization_s, edict_s, entity_state_s, qboolean, usercmd_s, vec3_t,
        weapon_data_s, KeyValueData, SAVERESTOREDATA, TYPEDESCRIPTION,
    },
    utils::slice_from_raw_parts_or_empty_mut,
};

pub use shared::export::{impl_unsync_global, UnsyncGlobal};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RestoreResult {
    Delete,
    Ok,
    Moved,
}

impl From<RestoreResult> for c_int {
    fn from(val: RestoreResult) -> Self {
        match val {
            RestoreResult::Delete => -1,
            RestoreResult::Ok => 0,
            RestoreResult::Moved => 1,
        }
    }
}

#[allow(unused_variables)]
#[allow(clippy::missing_safety_doc)]
pub trait ServerDll: UnsyncGlobal {
    fn dispatch_spawn(&self, ent: &mut edict_s) -> SpawnResult;

    fn dispatch_think(&self, ent: &mut edict_s);

    fn dispatch_use(&self, used: &mut edict_s, other: &mut edict_s);

    fn dispatch_touch(&self, touched: &mut edict_s, other: &mut edict_s);

    fn dispatch_blocked(&self, blocked: &mut edict_s, other: &mut edict_s);

    fn dispatch_key_value(&self, ent: &mut edict_s, data: &mut KeyValueData);

    fn dispatch_save(&self, ent: &mut edict_s, save_data: &mut SAVERESTOREDATA);

    fn dispatch_restore(
        &self,
        ent: &mut edict_s,
        save_data: &mut SAVERESTOREDATA,
        global_entity: bool,
    ) -> RestoreResult;

    fn dispatch_object_collsion_box(&self, ent: &mut edict_s);

    fn save_write_fields(
        &self,
        save_data: &mut SAVERESTOREDATA,
        name: &CStrThin,
        base_data: *mut c_void,
        fields: &mut [TYPEDESCRIPTION],
    );

    fn save_read_fields(
        &self,
        save_data: &mut SAVERESTOREDATA,
        name: &CStrThin,
        base_data: *mut c_void,
        fields: &mut [TYPEDESCRIPTION],
    );

    fn save_global_state(&self, save_data: &mut SAVERESTOREDATA);

    fn restore_global_state(&self, save_data: &mut SAVERESTOREDATA);

    fn reset_global_state(&self);

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

    fn client_put_in_server(&self, ent: &mut edict_s);

    fn client_command(&self, ent: &mut edict_s) {}

    fn client_user_info_changed(&self, ent: &mut edict_s, info_buffer: &CStrThin) {}

    fn server_activate(&self, list: &mut [edict_s], client_max: c_int) {}

    fn server_deactivate(&self) {}

    fn player_pre_think(&self, ent: &mut edict_s) {}

    fn player_post_think(&self, ent: &mut edict_s) {}

    fn start_frame(&self) {}

    fn parms_new_level(&self) {}

    fn parms_change_level(&self) {}

    fn get_game_description(&self) -> &'static CStr {
        c"Half-Life"
    }

    fn player_customization(&self, ent: &mut edict_s, custom: &mut raw::customization_s) {}

    fn spectator_connect(&self, ent: &mut edict_s) {}

    fn spectator_disconnect(&self, ent: &mut edict_s) {}

    fn spectator_think(&self, ent: &mut edict_s) {}

    /// Called when the engine has encountered an error.
    fn system_error(&self, error_string: &CStrThin) {}

    fn player_move_init(&self, pm: NonNull<raw::playermove_s>) {
        let pm = unsafe { pm.cast().as_mut() };
        pm::player_move_init(pm);
    }

    fn player_move(&self, pm: NonNull<raw::playermove_s>, is_server: bool) {
        let pm = unsafe { pm.cast().as_mut() };
        pm::player_move(pm, is_server);
    }

    fn player_move_find_texture_type(&self, name: &CStrThin) -> c_char {
        pm::find_texture_type(name)
    }

    fn setup_visibility(
        &self,
        view_entity: Option<&mut edict_s>,
        client: &mut edict_s,
        pvs: *mut *mut c_uchar,
        pas: *mut *mut c_uchar,
    );

    fn update_client_data(&self, ent: &edict_s, send_weapons: bool, cd: &mut raw::clientdata_s);

    #[allow(clippy::too_many_arguments)]
    fn add_to_full_pack(
        &self,
        state: &mut raw::entity_state_s,
        e: c_int,
        ent: &edict_s,
        host: &edict_s,
        hostflags: c_int,
        player: bool,
        set: *mut c_uchar,
    ) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn create_baseline(
        &self,
        player: bool,
        eindex: c_int,
        baseline: &mut raw::entity_state_s,
        entity: &mut edict_s,
        player_model_index: c_int,
        player_mins: vec3_t,
        player_maxs: vec3_t,
    );

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
        pm::get_hull_bounds_ffi(hullnumber, mins, maxs)
    }

    fn create_instanced_baselines(&self) {}

    fn inconsistent_file(
        &self,
        player: &edict_s,
        filename: &CStrThin,
        disconnect_message: &mut CStrArray<256>,
    ) -> bool {
        if !engine().get_cvar::<bool>(c"mp_consistency") {
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

    unsafe fn on_free_entity_private_data(&self, ent: *mut edict_s);

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

#[allow(non_snake_case)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ServerDllFunctions {
    pub pfnGameInit: Option<unsafe extern "C" fn()>,
    pub pfnSpawn: Option<unsafe extern "C" fn(pent: *mut edict_s) -> c_int>,
    pub pfnThink: Option<unsafe extern "C" fn(pent: *mut edict_s)>,
    pub pfnUse: Option<unsafe extern "C" fn(pentUsed: *mut edict_s, pentOther: *mut edict_s)>,
    pub pfnTouch: Option<unsafe extern "C" fn(pentTouched: *mut edict_s, pentOther: *mut edict_s)>,
    pub pfnBlocked:
        Option<unsafe extern "C" fn(pentBlocked: *mut edict_s, pentOther: *mut edict_s)>,
    pub pfnKeyValue:
        Option<unsafe extern "C" fn(pentKeyvalue: *mut edict_s, pkvd: *mut KeyValueData)>,
    pub pfnSave: Option<unsafe extern "C" fn(pent: *mut edict_s, pSaveData: *mut SAVERESTOREDATA)>,
    pub pfnRestore: Option<
        unsafe extern "C" fn(
            pent: *mut edict_s,
            pSaveData: *mut SAVERESTOREDATA,
            globalEntity: c_int,
        ) -> c_int,
    >,
    pub pfnSetAbsBox: Option<unsafe extern "C" fn(pent: *mut edict_s)>,
    pub pfnSaveWriteFields: Option<
        unsafe extern "C" fn(
            save_data: *mut SAVERESTOREDATA,
            name: *const c_char,
            base_data: *mut c_void,
            fields: *mut TYPEDESCRIPTION,
            fields_count: c_int,
        ),
    >,
    pub pfnSaveReadFields: Option<
        unsafe extern "C" fn(
            save_data: *mut SAVERESTOREDATA,
            name: *const c_char,
            base_data: *mut c_void,
            fields: *mut TYPEDESCRIPTION,
            fields_count: c_int,
        ),
    >,
    pub pfnSaveGlobalState: Option<unsafe extern "C" fn(save_data: *mut SAVERESTOREDATA)>,
    pub pfnRestoreGlobalState: Option<unsafe extern "C" fn(save_data: *mut SAVERESTOREDATA)>,
    pub pfnResetGlobalState: Option<unsafe extern "C" fn()>,
    pub pfnClientConnect: Option<
        unsafe extern "C" fn(
            pEntity: *mut edict_s,
            pszName: *const c_char,
            pszAddress: *const c_char,
            szRejectReason: *mut [c_char; 128],
        ) -> qboolean,
    >,
    pub pfnClientDisconnect: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientKill: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientPutInServer: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientCommand: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnClientUserInfoChanged:
        Option<unsafe extern "C" fn(pEntity: *mut edict_s, infobuffer: *mut c_char)>,
    pub pfnServerActivate:
        Option<unsafe extern "C" fn(pEdictList: *mut edict_s, edictCount: c_int, clientMax: c_int)>,
    pub pfnServerDeactivate: Option<unsafe extern "C" fn()>,
    pub pfnPlayerPreThink: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnPlayerPostThink: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnStartFrame: Option<unsafe extern "C" fn()>,
    pub pfnParmsNewLevel: Option<unsafe extern "C" fn()>,
    pub pfnParmsChangeLevel: Option<unsafe extern "C" fn()>,
    pub pfnGetGameDescription: Option<unsafe extern "C" fn() -> *const c_char>,
    pub pfnPlayerCustomization:
        Option<unsafe extern "C" fn(pEntity: *mut edict_s, pCustom: *mut customization_s)>,
    pub pfnSpectatorConnect: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnSpectatorDisconnect: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnSpectatorThink: Option<unsafe extern "C" fn(pEntity: *mut edict_s)>,
    pub pfnSys_Error: Option<unsafe extern "C" fn(error_string: *const c_char)>,
    pub pfnPM_Move: Option<unsafe extern "C" fn(ppmove: *mut playermove_s, server: qboolean)>,
    pub pfnPM_Init: Option<unsafe extern "C" fn(ppmove: *mut playermove_s)>,
    pub pfnPM_FindTextureType: Option<unsafe extern "C" fn(name: *const c_char) -> c_char>,
    pub pfnSetupVisibility: Option<
        unsafe extern "C" fn(
            pViewEntity: *mut edict_s,
            pClient: *mut edict_s,
            pvs: *mut *mut c_uchar,
            pas: *mut *mut c_uchar,
        ),
    >,
    pub pfnUpdateClientData: Option<
        unsafe extern "C" fn(ent: *const edict_s, sendweapons: c_int, cd: *mut clientdata_s),
    >,
    pub pfnAddToFullPack: Option<
        unsafe extern "C" fn(
            state: *mut entity_state_s,
            e: c_int,
            ent: *mut edict_s,
            host: *mut edict_s,
            hostflags: c_int,
            player: c_int,
            pSet: *mut c_uchar,
        ) -> c_int,
    >,
    pub pfnCreateBaseline: Option<
        unsafe extern "C" fn(
            player: c_int,
            eindex: c_int,
            baseline: *mut entity_state_s,
            entity: *mut edict_s,
            playermodelindex: c_int,
            player_mins: *const vec3_t,
            player_maxs: *const vec3_t,
        ),
    >,
    pub pfnRegisterEncoders: Option<unsafe extern "C" fn()>,
    pub pfnGetWeaponData:
        Option<unsafe extern "C" fn(player: *mut edict_s, info: *mut weapon_data_s) -> c_int>,
    pub pfnCmdStart: Option<
        unsafe extern "C" fn(player: *mut edict_s, cmd: *const usercmd_s, random_seed: c_uint),
    >,
    pub pfnCmdEnd: Option<unsafe extern "C" fn(player: *mut edict_s)>,
    pub pfnConnectionlessPacket: Option<
        unsafe extern "C" fn(
            net_from: *const netadr_s,
            args: *const c_char,
            response_buffer: *mut c_char,
            response_buffer_size: *mut c_int,
        ) -> c_int,
    >,
    pub pfnGetHullBounds: Option<
        unsafe extern "C" fn(hullnumber: c_int, mins: *mut vec3_t, maxs: *mut vec3_t) -> c_int,
    >,
    pub pfnCreateInstancedBaselines: Option<unsafe extern "C" fn()>,
    pub pfnInconsistentFile: Option<
        unsafe extern "C" fn(
            player: *const edict_s,
            filename: *const c_char,
            disconnect_message: *mut c_char,
        ) -> c_int,
    >,
    pub pfnAllowLagCompensation: Option<unsafe extern "C" fn() -> c_int>,
}

impl ServerDllFunctions {
    pub const VERSION: c_int = 140;

    pub fn new<T: ServerDll + Default>() -> ServerDllFunctions {
        Export::<T>::dll_functions()
    }
}

#[allow(non_snake_case)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ServerDllFunctions2 {
    pub pfnOnFreeEntPrivateData: Option<unsafe extern "C" fn(pEnt: *mut edict_s)>,
    pub pfnGameShutdown: Option<unsafe extern "C" fn()>,
    pub pfnShouldCollide:
        Option<unsafe extern "C" fn(pentTouched: *mut edict_s, pentOther: *mut edict_s) -> c_int>,
    pub pfnCvarValue: Option<unsafe extern "C" fn(pEnt: *const edict_s, value: *const c_char)>,
    pub pfnCvarValue2: Option<
        unsafe extern "C" fn(
            pEnt: *const edict_s,
            requestID: c_int,
            cvarName: *const c_char,
            value: *const c_char,
        ),
    >,
}

impl ServerDllFunctions2 {
    pub const VERSION: c_int = 1;

    pub fn new<T: ServerDll + Default>() -> ServerDllFunctions2 {
        Export::<T>::new_dll_functions()
    }
}

trait ServerDllExport {
    fn dll_functions() -> ServerDllFunctions {
        ServerDllFunctions {
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

    fn new_dll_functions() -> ServerDllFunctions2 {
        ServerDllFunctions2 {
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

    unsafe extern "C" fn player_move_find_texture_type(name: *const c_char) -> c_char;

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
        player_mins: *const vec3_t,
        player_maxs: *const vec3_t,
    );

    unsafe extern "C" fn register_encoders();

    unsafe extern "C" fn get_weapon_data(player: *mut edict_s, info: *mut weapon_data_s) -> c_int;

    unsafe extern "C" fn command_start(
        player: *mut edict_s,
        cmd: *const usercmd_s,
        random_seed: c_uint,
    );

    unsafe extern "C" fn command_end(player: *mut edict_s);

    unsafe extern "C" fn connectionless_packet(
        from: *const netadr_s,
        args: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: *mut c_int,
    ) -> c_int;

    extern "C" fn get_hull_bounds(hullnumber: c_int, mins: *mut vec3_t, maxs: *mut vec3_t)
        -> c_int;

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

impl<T: ServerDll + Default> ServerDllExport for Export<T> {
    unsafe extern "C" fn init() {
        unsafe {
            (&mut *T::global_as_mut_ptr()).write(T::default());
        }
    }

    unsafe extern "C" fn shutdown() {
        unsafe { (&mut *T::global_as_mut_ptr()).assume_init_drop() }
    }

    unsafe extern "C" fn dispatch_spawn(ent: *mut edict_s) -> c_int {
        if !ent.is_null() {
            let ent = unsafe { &mut *ent };
            return unsafe { T::global_assume_init_ref() }
                .dispatch_spawn(ent)
                .into();
        }
        SpawnResult::Delete.into()
    }

    unsafe extern "C" fn dispatch_think(ent: *mut edict_s) {
        if !ent.is_null() {
            let ent = unsafe { &mut *ent };
            unsafe { T::global_assume_init_ref() }.dispatch_think(ent);
        }
    }

    unsafe extern "C" fn dispatch_use(used: *mut edict_s, other: *mut edict_s) {
        if !used.is_null() && !other.is_null() {
            let used = unsafe { &mut *used };
            let other = unsafe { &mut *other };
            unsafe { T::global_assume_init_ref() }.dispatch_use(used, other)
        }
    }

    unsafe extern "C" fn dispatch_touch(touched: *mut edict_s, other: *mut edict_s) {
        if !touched.is_null() && !other.is_null() {
            let touched = unsafe { &mut *touched };
            let other = unsafe { &mut *other };
            unsafe { T::global_assume_init_ref() }.dispatch_touch(touched, other);
        }
    }

    unsafe extern "C" fn dispatch_blocked(blocked: *mut edict_s, other: *mut edict_s) {
        if !blocked.is_null() && !other.is_null() {
            let blocked = unsafe { &mut *blocked };
            let other = unsafe { &mut *other };
            unsafe { T::global_assume_init_ref() }.dispatch_blocked(blocked, other);
        }
    }

    unsafe extern "C" fn dispatch_key_value(ent: *mut edict_s, data: *mut KeyValueData) {
        if !ent.is_null() && !data.is_null() {
            let ent = unsafe { &mut *ent };
            let data = unsafe { &mut *data };
            unsafe { T::global_assume_init_ref() }.dispatch_key_value(ent, data);
        }
    }

    unsafe extern "C" fn dispatch_save(ent: *mut edict_s, save_data: *mut SAVERESTOREDATA) {
        if !ent.is_null() && !save_data.is_null() {
            let ent = unsafe { &mut *ent };
            let save_data = unsafe { &mut *save_data };
            unsafe { T::global_assume_init_ref() }.dispatch_save(ent, save_data);
        }
    }

    unsafe extern "C" fn dispatch_restore(
        ent: *mut edict_s,
        save_data: *mut SAVERESTOREDATA,
        global_entity: c_int,
    ) -> c_int {
        if !ent.is_null() && !save_data.is_null() {
            let ent = unsafe { &mut *ent };
            let save_data = unsafe { &mut *save_data };
            let global_entity = global_entity != 0;
            return unsafe { T::global_assume_init_ref() }
                .dispatch_restore(ent, save_data, global_entity)
                .into();
        }
        RestoreResult::Delete.into()
    }

    unsafe extern "C" fn dispatch_object_collsion_box(ent: *mut edict_s) {
        if !ent.is_null() {
            let ent = unsafe { &mut *ent };
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
        assert!(!save_data.is_null());
        assert!(!name.is_null());
        let save_data = unsafe { &mut *save_data };
        let name = unsafe { CStrThin::from_ptr(name) };
        let fields = unsafe { slice_from_raw_parts_or_empty_mut(fields, fields_count as usize) };
        unsafe { T::global_assume_init_ref() }
            .save_write_fields(save_data, name, base_data, fields);
    }

    unsafe extern "C" fn save_read_fields(
        save_data: *mut SAVERESTOREDATA,
        name: *const c_char,
        base_data: *mut c_void,
        fields: *mut TYPEDESCRIPTION,
        fields_count: c_int,
    ) {
        assert!(!save_data.is_null());
        assert!(!name.is_null());
        let save_data = unsafe { &mut *save_data };
        let name = unsafe { CStrThin::from_ptr(name) };
        let fields = unsafe { slice_from_raw_parts_or_empty_mut(fields, fields_count as usize) };
        unsafe { T::global_assume_init_ref() }.save_read_fields(save_data, name, base_data, fields);
    }

    unsafe extern "C" fn save_global_state(save_data: *mut SAVERESTOREDATA) {
        assert!(!save_data.is_null());
        let save_data = unsafe { &mut *save_data };
        unsafe { T::global_assume_init_ref() }.save_global_state(save_data);
    }

    unsafe extern "C" fn restore_global_state(save_data: *mut SAVERESTOREDATA) {
        assert!(!save_data.is_null());
        let save_data = unsafe { &mut *save_data };
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
        assert!(!ent.is_null());
        assert!(!name.is_null());
        assert!(!address.is_null());
        assert!(!reject_reason.is_null());
        let ent = unsafe { &mut *ent };
        let name = unsafe { CStrThin::from_ptr(name) };
        let address = unsafe { CStrThin::from_ptr(address) };
        let reject_reason = unsafe { &mut *reject_reason.cast::<CStrArray<128>>() };
        reject_reason.clear();
        unsafe { T::global_assume_init_ref() }
            .client_connect(ent, name, address, reject_reason)
            .into()
    }

    unsafe extern "C" fn client_disconnect(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.client_disconnect(ent);
    }

    unsafe extern "C" fn client_kill(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.client_kill(ent);
    }

    unsafe extern "C" fn client_put_in_server(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.client_put_in_server(ent);
    }

    unsafe extern "C" fn client_command(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.client_command(ent);
    }

    unsafe extern "C" fn client_user_info_changed(ent: *mut edict_s, info_buffer: *mut c_char) {
        assert!(!ent.is_null());
        assert!(!info_buffer.is_null());
        let ent = unsafe { &mut *ent };
        let info_buffer = unsafe { CStrThin::from_ptr(info_buffer) };
        unsafe { T::global_assume_init_ref() }.client_user_info_changed(ent, info_buffer);
    }

    unsafe extern "C" fn server_activate(
        edict_list: *mut edict_s,
        edict_count: c_int,
        client_max: c_int,
    ) {
        assert!(!edict_list.is_null());
        let list = unsafe { slice::from_raw_parts_mut(edict_list, edict_count as usize) };
        unsafe { T::global_assume_init_ref() }.server_activate(list, client_max)
    }

    unsafe extern "C" fn server_deactivate() {
        unsafe { T::global_assume_init_ref() }.server_deactivate();
    }

    unsafe extern "C" fn player_pre_think(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.player_pre_think(ent);
    }

    unsafe extern "C" fn player_post_think(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
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
        unsafe { T::global_assume_init_ref() }
            .get_game_description()
            .as_ptr()
    }

    unsafe extern "C" fn player_customization(ent: *mut edict_s, custom: *mut customization_s) {
        assert!(!ent.is_null());
        assert!(!custom.is_null());
        let ent = unsafe { &mut *ent };
        let custom = unsafe { &mut *custom };
        unsafe { T::global_assume_init_ref() }.player_customization(ent, custom);
    }

    unsafe extern "C" fn spectator_connect(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.spectator_connect(ent);
    }

    unsafe extern "C" fn spectator_disconnect(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.spectator_disconnect(ent);
    }

    unsafe extern "C" fn spectator_think(ent: *mut edict_s) {
        assert!(!ent.is_null());
        let ent = unsafe { &mut *ent };
        unsafe { T::global_assume_init_ref() }.spectator_think(ent);
    }

    unsafe extern "C" fn system_error(error_string: *const c_char) {
        assert!(!error_string.is_null());
        let error_string = unsafe { CStrThin::from_ptr(error_string) };
        unsafe { T::global_assume_init_ref() }.system_error(error_string);
    }

    unsafe extern "C" fn player_move_init(pm: *mut playermove_s) {
        let pm = NonNull::new(pm).unwrap();
        unsafe { T::global_assume_init_ref() }.player_move_init(pm);
    }

    unsafe extern "C" fn player_move(pm: *mut playermove_s, is_server: qboolean) {
        let pm = NonNull::new(pm).unwrap();
        unsafe { T::global_assume_init_ref() }.player_move(pm, is_server.to_bool());
    }

    unsafe extern "C" fn player_move_find_texture_type(name: *const c_char) -> c_char {
        assert!(!name.is_null());
        let name = unsafe { CStrThin::from_ptr(name) };
        unsafe { T::global_assume_init_ref() }.player_move_find_texture_type(name)
    }

    unsafe extern "C" fn setup_visibility(
        view_entity: *mut edict_s,
        client: *mut edict_s,
        pvs: *mut *mut c_uchar,
        pas: *mut *mut c_uchar,
    ) {
        assert!(!client.is_null());
        let view_entity = if view_entity.is_null() {
            None
        } else {
            Some(unsafe { &mut *view_entity })
        };
        let client = unsafe { &mut *client };
        unsafe { T::global_assume_init_ref() }.setup_visibility(view_entity, client, pvs, pas);
    }

    unsafe extern "C" fn update_client_data(
        ent: *const edict_s,
        send_weapons: c_int,
        cd: *mut clientdata_s,
    ) {
        assert!(!ent.is_null());
        assert!(!cd.is_null());
        let ent = unsafe { &*ent };
        let send_weapons = send_weapons != 0;
        let cd = unsafe { &mut *cd };
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
        assert!(!state.is_null());
        assert!(!ent.is_null());
        assert!(!host.is_null());
        let state = unsafe { &mut *state };
        let ent = unsafe { &*ent };
        let host = unsafe { &*host };
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
        player_mins: *const vec3_t,
        player_maxs: *const vec3_t,
    ) {
        assert!(!baseline.is_null());
        assert!(!entity.is_null());
        assert!(!player_mins.is_null());
        assert!(!player_maxs.is_null());
        let baseline = unsafe { &mut *baseline };
        let entity = unsafe { &mut *entity };
        let player_mins = unsafe { *player_mins };
        let player_maxs = unsafe { *player_maxs };
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
        assert!(!player.is_null());
        assert!(!info.is_null());
        let player = unsafe { &mut *player };
        let info = unsafe { &mut *info };
        match unsafe { T::global_assume_init_ref() }.get_weapon_data(player) {
            Some(x) => {
                *info = x;
                1
            }
            None => {
                *info = weapon_data_s::default();
                0
            }
        }
    }

    unsafe extern "C" fn command_start(
        player: *mut edict_s,
        cmd: *const usercmd_s,
        random_seed: c_uint,
    ) {
        assert!(!player.is_null());
        assert!(!cmd.is_null());
        let player = unsafe { &mut *player };
        let cmd = unsafe { &*cmd };
        unsafe { T::global_assume_init_ref() }.command_start(player, cmd, random_seed);
    }

    unsafe extern "C" fn command_end(player: *mut edict_s) {
        assert!(!player.is_null());
        let player = unsafe { &mut *player };
        unsafe { T::global_assume_init_ref() }.command_end(player);
    }

    unsafe extern "C" fn connectionless_packet(
        from: *const netadr_s,
        args: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: *mut c_int,
    ) -> c_int {
        assert!(!from.is_null());
        assert!(!args.is_null());
        assert!(!response_buffer.is_null());
        assert!(!response_buffer_size.is_null());
        let from = unsafe { &*from };
        let args = unsafe { CStrThin::from_ptr(args) };
        let max_buffer_size = unsafe { *response_buffer_size } as usize;
        let buffer = unsafe { slice::from_raw_parts_mut(response_buffer.cast(), max_buffer_size) };
        match unsafe { T::global_assume_init_ref() }.connectionless_packet(from, args, buffer) {
            Ok(len) => unsafe {
                *response_buffer_size = len as c_int;
                (len > 0) as c_int
            },
            Err(_) => 0,
        }
    }

    extern "C" fn get_hull_bounds(
        hullnumber: c_int,
        mins: *mut vec3_t,
        maxs: *mut vec3_t,
    ) -> c_int {
        assert!(!mins.is_null());
        assert!(!maxs.is_null());
        let mins = unsafe { &mut *mins };
        let maxs = unsafe { &mut *maxs };
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
        assert!(!player.is_null());
        assert!(!filename.is_null());
        assert!(!disconnect_message.is_null());
        let player = unsafe { &*player };
        let filename = unsafe { CStrThin::from_ptr(filename) };
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
        assert!(!touched.is_null());
        assert!(!other.is_null());
        let touched = unsafe { &mut *touched };
        let other = unsafe { &mut *other };
        unsafe { T::global_assume_init_ref() }.chould_collide(touched, other) as c_int
    }

    unsafe extern "C" fn cvar_value(ent: *const edict_s, value: *const c_char) {
        assert!(!ent.is_null());
        assert!(!value.is_null());
        let ent = unsafe { &*ent };
        let value = unsafe { CStrThin::from_ptr(value) };
        unsafe { T::global_assume_init_ref() }.cvar_value(ent, value);
    }

    unsafe extern "C" fn cvar_value2(
        ent: *const edict_s,
        request_id: c_int,
        cvar_name: *const c_char,
        value: *const c_char,
    ) {
        assert!(!ent.is_null());
        assert!(!cvar_name.is_null());
        assert!(!value.is_null());
        let ent = unsafe { &*ent };
        let cvar_name = unsafe { CStrThin::from_ptr(cvar_name) };
        let value = unsafe { CStrThin::from_ptr(value) };
        unsafe { T::global_assume_init_ref() }.cvar_value2(ent, request_id, cvar_name, value);
    }
}

// pub type NewDllFunctionsFn =
//     unsafe extern "C" fn(dll_funcs: *mut DllFunctions2, version: *mut c_int) -> c_int;
//
// pub type GetEntityApiFn =
//     unsafe extern "C" fn(dll_funcs: *mut DllFunctions, version: c_int) -> c_int;
//
// pub type GetEntityApi2Fn =
//     unsafe extern "C" fn(dll_funcs: *mut DllFunctions, version: *mut c_int) -> c_int;

#[doc(hidden)]
#[macro_export]
macro_rules! export_dll {
    ($server_dll:ty $($init:block)?) => {
        #[no_mangle]
        unsafe extern "C" fn GiveFnptrsToDll(
            eng_funcs: Option<&$crate::engine::ServerEngineFunctions>,
            globals: *mut $crate::globals::globalvars_t,
        ) {
            let eng_funcs = eng_funcs.unwrap();
            unsafe {
                $crate::instance::init_engine(eng_funcs, globals);
            }
            $crate::cvar::init(|name, _, _| $crate::instance::engine().get_cvar_ptr(name));
        }

        #[no_mangle]
        unsafe extern "C" fn GetEntityAPI(
            dll_funcs: *mut $crate::export::ServerDllFunctions,
            mut version: core::ffi::c_int,
        ) -> core::ffi::c_int {
            unsafe { GetEntityAPI2(dll_funcs, &mut version) }
        }

        #[no_mangle]
        unsafe extern "C" fn GetEntityAPI2(
            dll_funcs: *mut $crate::export::ServerDllFunctions,
            version: *mut core::ffi::c_int,
        ) -> core::ffi::c_int {
            use $crate::export::ServerDllFunctions;
            unsafe {
                if dll_funcs.is_null() || *version != ServerDllFunctions::VERSION {
                    *version = ServerDllFunctions::VERSION;
                    return 0;
                }
            }
            unsafe {
                *dll_funcs = ServerDllFunctions::new::<$server_dll>();
            }
            $($init)?
            1
        }

        #[no_mangle]
        unsafe extern "C" fn GetNewDLLFunctions(
            dll_funcs: *mut $crate::export::ServerDllFunctions2,
            version: *mut core::ffi::c_int,
        ) -> core::ffi::c_int {
            use $crate::export::ServerDllFunctions2;
            unsafe {
                if dll_funcs.is_null() || *version != ServerDllFunctions2::VERSION {
                    *version = ServerDllFunctions2::VERSION;
                    return 0;
                }
                *dll_funcs = ServerDllFunctions2::new::<$server_dll>();
                1
            }
        }
    };
}
#[doc(inline)]
pub use export_dll;
