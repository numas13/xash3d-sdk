#![allow(non_snake_case)]

use core::{
    ffi::{c_char, c_int, c_uchar, c_uint, c_void, CStr},
    ptr, slice,
};

use csz::CStrThin;
use pm::{VEC_DUCK_HULL_MIN, VEC_HULL_MIN};
use sv::{
    engine, globals,
    raw::{
        self, clientdata_s, customization_s, edict_s, entity_state_s, netadr_s, playermove_s,
        qboolean, usercmd_s, vec3_t, weapon_data_s, EdictFlags, INTERFACE_VERSION,
        NEW_DLL_FUNCTIONS_VERSION,
    },
};

use crate::{
    gamerules::game_rules,
    global_state::{global_state, EntityState},
    player,
    private_data::{Private, PrivateDataRef},
    save, triggers,
};

#[no_mangle]
unsafe extern "C" fn GameDLLInit() {
    crate::cvar::init();
}

#[no_mangle]
unsafe extern "C" fn DispatchSpawn(ent: *mut edict_s) -> c_int {
    if ent.is_null() {
        return 0;
    }

    let Some(entity) = (unsafe { (*ent).private_mut() }) else {
        return 0;
    };
    let ev = unsafe { &mut (*ent).v };

    ev.absmin = ev.origin - vec3_t::splat(1.0);
    ev.absmax = ev.origin + vec3_t::splat(1.0);

    if !entity.spawn() {
        return -1;
    }

    if let Some(false) = game_rules().map(|rules| rules.is_allowed_to_spawn(&**entity)) {
        return -1;
    }

    if ev.flags.intersects(EdictFlags::KILLME) {
        return -1;
    }

    if !ev.globalname.is_null() {
        let global_state = global_state();
        let mut entities = global_state.entities.borrow_mut();
        if let Some(global) = entities.find_string(ev.globalname) {
            if global.is_dead() {
                return -1;
            }
            let map_name = globals().string(globals().mapname);
            if map_name != global.map_name() {
                entity.make_dormant();
            }
        } else {
            entities.add_string(ev.globalname, globals().mapname, EntityState::On);
        }
    }
    0
}

#[no_mangle]
unsafe extern "C" fn DispatchThink(ent: *mut edict_s) {
    if !ent.is_null() {
        let ent = unsafe { &mut *ent };
        if let Some(entity) = ent.private_mut() {
            if entity.vars().flags.intersects(EdictFlags::DORMANT) {
                let classname = entity.classname();
                error!("Dormant entity {classname:?} is thinkng");
            }
            entity.think();
        }
    }
}

#[no_mangle]
unsafe extern "C" fn DispatchUse(_ent_used: *mut edict_s, _ent_other: *mut edict_s) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn DispatchTouch(touched: *mut edict_s, other: *mut edict_s) {
    let touched = unsafe { &mut *touched };
    let other = unsafe { &mut *other };
    crate::todo::dispatch_touch(touched, other);
}

#[no_mangle]
unsafe extern "C" fn DispatchBlocked(_ent_blocked: *mut edict_s, _ent_other: *mut edict_s) {
    debug!("TODO: DispatchBlocked");
}

#[no_mangle]
unsafe extern "C" fn DispatchKeyValue(ent: *mut edict_s, data: *mut raw::KeyValueData) {
    if !ent.is_null() && !data.is_null() {
        unsafe {
            save::dispatch_key_value(&mut *ent, &mut *data);
        }
    }
}

#[no_mangle]
unsafe extern "C" fn DispatchSave(ent: *mut edict_s, save_data: *mut raw::SAVERESTOREDATA) {
    if !ent.is_null() && !save_data.is_null() {
        let ent = unsafe { &mut *ent };
        let save_data = unsafe { &mut *save_data };
        save::dispatch_save(ent, save_data);
    }
}

#[no_mangle]
unsafe extern "C" fn DispatchRestore(
    ent: *mut edict_s,
    save_data: *mut raw::SAVERESTOREDATA,
    global_entity: c_int,
) -> c_int {
    if !ent.is_null() && !save_data.is_null() {
        let save_data = unsafe { &mut *save_data };
        save::dispatch_restore(ent, save_data, global_entity != 0)
    } else {
        0
    }
}

#[no_mangle]
unsafe extern "C" fn DispatchObjectCollsionBox(ent: *mut edict_s) {
    crate::todo::dispatch_object_collision_box(unsafe { &mut *ent });
}

#[no_mangle]
unsafe extern "C" fn SaveWriteFields(
    save_data: *mut raw::SAVERESTOREDATA,
    name: *const c_char,
    base_data: *mut c_void,
    fields: *mut raw::TYPEDESCRIPTION,
    fields_count: c_int,
) {
    assert!(!save_data.is_null());

    unsafe {
        save::write_fields(
            &mut *save_data,
            CStr::from_ptr(name),
            base_data,
            if !fields.is_null() {
                slice::from_raw_parts(fields, fields_count as usize)
            } else {
                &mut []
            },
        );
    }
}

#[no_mangle]
unsafe extern "C" fn SaveReadFields(
    save_data: *mut raw::SAVERESTOREDATA,
    name: *const c_char,
    base_data: *mut c_void,
    fields: *mut raw::TYPEDESCRIPTION,
    fields_count: c_int,
) {
    assert!(!save_data.is_null());

    unsafe {
        save::read_fields(
            &mut *save_data,
            CStr::from_ptr(name),
            base_data,
            if !fields.is_null() {
                slice::from_raw_parts(fields, fields_count as usize)
            } else {
                &mut []
            },
        );
    }
}

#[no_mangle]
unsafe extern "C" fn SaveGlobalState(save_data: *mut raw::SAVERESTOREDATA) {
    if !save_data.is_null() {
        if let Err(e) = global_state().save(unsafe { &mut *save_data }) {
            error!("Failed to save global state: {e:?}");
        }
    }
}

#[no_mangle]
unsafe extern "C" fn RestoreGlobalState(save_data: *mut raw::SAVERESTOREDATA) {
    if !save_data.is_null() {
        if let Err(e) = global_state().restore(unsafe { &mut *save_data }) {
            error!("Failed to restore global state: {e:?}");
        }
    }
}

#[no_mangle]
unsafe extern "C" fn ResetGlobalState() {
    global_state().reset();
}

#[no_mangle]
unsafe extern "C" fn ClientConnect(
    _pEntity: *mut edict_s,
    _pszName: *const c_char,
    _pszAddress: *const c_char,
    _szRejectReason: *mut [c_char; 128usize],
) -> qboolean {
    debug!("TODO: ClientConnect");
    qboolean::TRUE
}

#[no_mangle]
unsafe extern "C" fn ClientDisconnect(_ent: *mut edict_s) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn ClientKill(_ent: *mut edict_s) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn ClientPutInServer(ent: *mut edict_s) {
    let ent = unsafe { &mut *ent };
    player::client_put_in_server(ent);
}

#[no_mangle]
unsafe extern "C" fn ClientCommand(_ent: *mut edict_s) {
    let engine = engine();
    let cmd = engine.cmd_argv(0);
    debug!("client command {cmd:?}");
}

#[no_mangle]
unsafe extern "C" fn ClientUserInfoChanged(_ent: *mut edict_s, _infobuffer: *mut c_char) {
    debug!("TODO: ClientUserInfoChanged");
}

#[no_mangle]
unsafe extern "C" fn ServerActivate(
    _pEdictList: *mut edict_s,
    _edictCount: c_int,
    _clientMax: c_int,
) {
    debug!("TODO: ServerActivate");
}

#[no_mangle]
unsafe extern "C" fn ServerDeactivate() {
    debug!("TODO: ServerDeactivate");
}

#[no_mangle]
unsafe extern "C" fn PlayerPreThink(_pEntity: *mut edict_s) {
    // debug!("TODO: PlayerPreThink");
}

#[no_mangle]
unsafe extern "C" fn PlayerPostThink(_pEntity: *mut edict_s) {
    // debug!("TODO: PlayerPostThink");
}

#[no_mangle]
unsafe extern "C" fn StartFrame() {
    // debug!("TODO: StartFrame");
}

#[no_mangle]
unsafe extern "C" fn ParmsNewLevel() {
    debug!("TODO: ParmsNewLevel");
}

#[no_mangle]
unsafe extern "C" fn ParmsChangeLevel() {
    let save_data = globals().pSaveData.cast::<raw::SAVERESTOREDATA>();
    if !save_data.is_null() {
        let save_data = unsafe { &mut *save_data };
        save_data.connection_count =
            triggers::build_change_list(&mut save_data.level_list) as c_int;
    }
}

#[no_mangle]
unsafe extern "C" fn GetGameDescription() -> *const c_char {
    game_rules()
        .map_or(c"Half-Life", |rules| rules.get_game_description())
        .as_ptr()
}

#[no_mangle]
unsafe extern "C" fn PlayerCustomization(_pEntity: *mut edict_s, _pCustom: *mut customization_s) {
    debug!("TODO: PlayerCustomization");
}

#[no_mangle]
unsafe extern "C" fn SpectatorConnect(_pEntity: *mut edict_s) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn SpectatorDisconnect(_pEntity: *mut edict_s) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn SpectatorThink(_pEntity: *mut edict_s) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn Sys_Error(_error_string: *const c_char) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn PM_Move(pm: *mut playermove_s, is_server: qboolean) {
    pm::player_move(unsafe { &mut *pm.cast() }, is_server.to_bool());
}

#[no_mangle]
unsafe extern "C" fn PM_Init(pm: *mut playermove_s) {
    pm::player_move_init(unsafe { &mut *pm.cast() });
}

#[no_mangle]
unsafe extern "C" fn PM_FindTextureType(name: *const c_char) -> c_char {
    pm::find_texture_type(unsafe { CStrThin::from_ptr(name) })
}

#[no_mangle]
unsafe extern "C" fn SetupVisibility(
    view_entity: *mut edict_s,
    client: *mut edict_s,
    pvs: *mut *mut c_uchar,
    pas: *mut *mut c_uchar,
) {
    let client = unsafe { &mut *client };
    if client.v.flags.intersects(EdictFlags::PROXY) {
        unsafe {
            *pvs = ptr::null_mut();
            *pas = ptr::null_mut();
        }
        return;
    }

    let view = if !view_entity.is_null() {
        unsafe { &mut *view_entity }
    } else {
        client
    };

    let mut org = view.v.origin + view.v.view_ofs;
    if view.v.flags.intersects(EdictFlags::DUCKING) {
        org += VEC_HULL_MIN - VEC_DUCK_HULL_MIN;
    }

    let engine = engine();
    unsafe {
        *pvs = engine.set_pvs(org);
        *pas = engine.set_pas(org);
    }
}

#[no_mangle]
unsafe extern "C" fn UpdateClientData(
    ent: *const edict_s,
    sendweapons: c_int,
    cd: *mut clientdata_s,
) {
    if !ent.is_null() {
        unsafe {
            crate::todo::update_client_data(&*ent, sendweapons != 0, &mut *cd);
        }
    }
}

#[no_mangle]
unsafe extern "C" fn AddToFullPack(
    state: *mut entity_state_s,
    e: c_int,
    ent: *mut edict_s,
    host: *mut edict_s,
    hostflags: c_int,
    player: c_int,
    set: *mut c_uchar,
) -> c_int {
    let state = unsafe { &mut *state };
    let ent = unsafe { &*ent };
    let host = unsafe { &*host };
    let player = player != 0;
    let set = unsafe { &mut *set };
    crate::todo::add_to_full_pack(state, e, ent, host, hostflags, player, set) as c_int
}

#[no_mangle]
unsafe extern "C" fn CreateBaseline(
    player: c_int,
    eindex: c_int,
    baseline: *mut entity_state_s,
    entity: *mut edict_s,
    playermodelindex: c_int,
    player_mins: *mut vec3_t,
    player_maxs: *mut vec3_t,
) {
    crate::todo::create_baseline(
        player != 0,
        eindex,
        unsafe { &mut *baseline },
        unsafe { &*entity },
        playermodelindex,
        unsafe { *player_mins },
        unsafe { *player_maxs },
    );
}

#[no_mangle]
unsafe extern "C" fn RegisterEncoders() {
    debug!("TODO: RegisterEncoders");
}

#[no_mangle]
unsafe extern "C" fn GetWeaponData(_player: *mut edict_s, _info: *mut weapon_data_s) -> c_int {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn CmdStart(
    _player: *const edict_s,
    _cmd: *const usercmd_s,
    _random_seed: c_uint,
) {
    // debug!("TODO: CmdStart");
}

#[no_mangle]
unsafe extern "C" fn CmdEnd(_player: *const edict_s) {
    // debug!("TODO: CmdEnd");
}

#[no_mangle]
unsafe extern "C" fn ConnectionlessPacket(
    _net_from: *const netadr_s,
    _args: *const c_char,
    _response_buffer: *mut c_char,
    _response_buffer_size: *mut c_int,
) -> c_int {
    todo!();
}

#[no_mangle]
extern "C" fn GetHullBounds(hullnumber: c_int, mins: *mut vec3_t, maxs: *mut vec3_t) -> c_int {
    unsafe { pm::get_hull_bounds_ffi(hullnumber, &mut *mins, &mut *maxs) }
}

#[no_mangle]
unsafe extern "C" fn CreateInstancedBaselines() {
    debug!("TODO: CreateInstancedBaselines");
}

#[no_mangle]
unsafe extern "C" fn InconsistentFile(
    _player: *const edict_s,
    _filename: *const c_char,
    _disconnect_message: *mut c_char,
) -> c_int {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn AllowLagCompensation() -> c_int {
    todo!();
}

static DLL_FUNCTIONS: raw::DLL_FUNCTIONS = raw::DLL_FUNCTIONS {
    pfnGameInit: Some(GameDLLInit),
    pfnSpawn: Some(DispatchSpawn),
    pfnThink: Some(DispatchThink),
    pfnUse: Some(DispatchUse),
    pfnTouch: Some(DispatchTouch),
    pfnBlocked: Some(DispatchBlocked),
    pfnKeyValue: Some(DispatchKeyValue),
    pfnSave: Some(DispatchSave),
    pfnRestore: Some(DispatchRestore),
    pfnSetAbsBox: Some(DispatchObjectCollsionBox),
    pfnSaveWriteFields: Some(SaveWriteFields),
    pfnSaveReadFields: Some(SaveReadFields),
    pfnSaveGlobalState: Some(SaveGlobalState),
    pfnRestoreGlobalState: Some(RestoreGlobalState),
    pfnResetGlobalState: Some(ResetGlobalState),
    pfnClientConnect: Some(ClientConnect),
    pfnClientDisconnect: Some(ClientDisconnect),
    pfnClientKill: Some(ClientKill),
    pfnClientPutInServer: Some(ClientPutInServer),
    pfnClientCommand: Some(ClientCommand),
    pfnClientUserInfoChanged: Some(ClientUserInfoChanged),
    pfnServerActivate: Some(ServerActivate),
    pfnServerDeactivate: Some(ServerDeactivate),
    pfnPlayerPreThink: Some(PlayerPreThink),
    pfnPlayerPostThink: Some(PlayerPostThink),
    pfnStartFrame: Some(StartFrame),
    pfnParmsNewLevel: Some(ParmsNewLevel),
    pfnParmsChangeLevel: Some(ParmsChangeLevel),
    pfnGetGameDescription: Some(GetGameDescription),
    pfnPlayerCustomization: Some(PlayerCustomization),
    pfnSpectatorConnect: Some(SpectatorConnect),
    pfnSpectatorDisconnect: Some(SpectatorDisconnect),
    pfnSpectatorThink: Some(SpectatorThink),
    pfnSys_Error: Some(Sys_Error),
    pfnPM_Move: Some(PM_Move),
    pfnPM_Init: Some(PM_Init),
    pfnPM_FindTextureType: Some(PM_FindTextureType),
    pfnSetupVisibility: Some(SetupVisibility),
    pfnUpdateClientData: Some(UpdateClientData),
    pfnAddToFullPack: Some(AddToFullPack),
    pfnCreateBaseline: Some(CreateBaseline),
    pfnRegisterEncoders: Some(RegisterEncoders),
    pfnGetWeaponData: Some(GetWeaponData),
    pfnCmdStart: Some(CmdStart),
    pfnCmdEnd: Some(CmdEnd),
    pfnConnectionlessPacket: Some(ConnectionlessPacket),
    pfnGetHullBounds: Some(GetHullBounds),
    pfnCreateInstancedBaselines: Some(CreateInstancedBaselines),
    pfnInconsistentFile: Some(InconsistentFile),
    pfnAllowLagCompensation: Some(AllowLagCompensation),
};

#[no_mangle]
unsafe extern "C" fn pfnOnFreeEntPrivateData(ent: *mut edict_s) {
    unsafe {
        PrivateDataRef::free(ent);
    }
}

#[no_mangle]
unsafe extern "C" fn GameShutdown() {
    debug!("TODO: GameShutdown");
}

#[no_mangle]
unsafe extern "C" fn ShouldCollide(_pentTouched: *mut edict_s, _pentOther: *mut edict_s) -> c_int {
    debug!("TODO: ShouldCollide");
    0
}

#[no_mangle]
unsafe extern "C" fn CvarValue(_pEnt: *const edict_s, _value: *const c_char) {
    todo!();
}

#[no_mangle]
unsafe extern "C" fn CvarValue2(
    _pEnt: *const edict_s,
    _requestID: c_int,
    _cvarName: *const c_char,
    _value: *const c_char,
) {
    todo!();
}

static NEW_DLL_FUNCTIONS: raw::NEW_DLL_FUNCTIONS = raw::NEW_DLL_FUNCTIONS {
    pfnOnFreeEntPrivateData: Some(pfnOnFreeEntPrivateData),
    pfnGameShutdown: Some(GameShutdown),
    // TODO: pfnShouldCollide: Some(ShouldCollide),
    // TODO: pfnCvarValue: Some(CvarValue),
    // TODO: pfnCvarValue2: Some(CvarValue2),
    pfnShouldCollide: None,
    pfnCvarValue: None,
    pfnCvarValue2: None,
};

#[no_mangle]
unsafe extern "C" fn GetEntityAPI(funcs: *mut raw::DLL_FUNCTIONS, mut version: c_int) -> c_int {
    unsafe { GetEntityAPI2(funcs, &mut version) }
}

#[no_mangle]
unsafe extern "C" fn GetEntityAPI2(funcs: *mut raw::DLL_FUNCTIONS, version: *mut c_int) -> c_int {
    unsafe {
        if funcs.is_null() || *version != INTERFACE_VERSION {
            *version = INTERFACE_VERSION;
            return 0;
        }
        *funcs = DLL_FUNCTIONS;
        1
    }
}

#[no_mangle]
unsafe extern "C" fn GetNewDLLFunctions(
    funcs: *mut raw::NEW_DLL_FUNCTIONS,
    version: *mut c_int,
) -> c_int {
    unsafe {
        if funcs.is_null() || *version != NEW_DLL_FUNCTIONS_VERSION {
            *version = NEW_DLL_FUNCTIONS_VERSION;
            return 0;
        }
        *funcs = NEW_DLL_FUNCTIONS;
        1
    }
}

#[no_mangle]
unsafe extern "C" fn GiveFnptrsToDll(
    funcs: Option<&sv::raw::enginefuncs_s>,
    globals: *mut sv::raw::globalvars_t,
) {
    unsafe {
        sv::init_engine(funcs.unwrap(), globals);
    }
}
