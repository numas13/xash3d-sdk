use core::{
    ffi::{c_char, c_int, c_uint, c_void},
    ptr,
};

use cl::{
    self as engine, engine, engine_set,
    raw::kbutton_t,
    raw::{
        client_data_s, local_state_s, ref_params_s, EntityType, CLDLL_INTERFACE_VERSION, TEMPENTITY,
    },
    studio_set,
};
use csz::CStrThin;
use math::vec3_t;
use shared::{
    cvar::CVarPtr,
    raw::{
        byte, cl_entity_s, clientdata_s, entity_state_s, netadr_s, playermove_s, qboolean,
        usercmd_s, weapon_data_s,
    },
};

use crate::{
    camera::{camera, camera_mut},
    entity::{self, entities, entities_mut},
    events,
    hud::hud_mut,
    input::{input, input_mut},
    studio::{self, renderer_mut},
    view::view_mut,
    weapons::weapons_mut,
};

#[no_mangle]
unsafe extern "C" fn Initialize(
    engine_funcs: Option<&mut engine::raw::cl_enginefuncs_s>,
    version: c_int,
) -> c_int {
    if version != CLDLL_INTERFACE_VERSION {
        return 0;
    }

    let engine_funcs = engine_funcs.unwrap();
    engine_set(*engine_funcs);
    let dev = engine().cvar_get_float(c"developer") as i32;
    utils::logger::init(dev, |s| engine().console_print(s));

    shared::cvar::init(|name, value, flags| {
        let engine = engine();
        let ptr = engine.get_cvar(name);
        if ptr.is_null() {
            engine
                .register_variable(name, value, flags)
                .unwrap_or(CVarPtr::null())
        } else {
            ptr
        }
    });

    events::init();

    // TODO: CL_LoadParticleMan();

    1
}

#[no_mangle]
extern "C" fn HUD_Init() {
    crate::input::init();
    crate::camera::init();
    crate::view::init();
    crate::hud::init();
}

#[no_mangle]
unsafe extern "C" fn HUD_VidInit() -> c_int {
    hud_mut().vid_init();
    1
}

#[no_mangle]
unsafe extern "C" fn HUD_Redraw(time: f32, intermission: c_int) -> c_int {
    hud_mut().draw(time, intermission != 0) as c_int
}

#[no_mangle]
unsafe extern "C" fn HUD_Frame(_time: f64) {
    // TODO:
}

#[no_mangle]
unsafe extern "C" fn HUD_UpdateClientData(data: Option<&mut client_data_s>, time: f32) -> c_int {
    input_mut().in_commands();
    hud_mut().update_client_data(data.unwrap(), time) as c_int
}

#[no_mangle]
unsafe extern "C" fn HUD_Reset() {
    hud_mut().reset();
}

#[no_mangle]
extern "C" fn HUD_Shutdown() {
    input_mut().shutdown();
}

#[no_mangle]
extern "C" fn HUD_DrawNormalTriangles() {
    // TODO:
}

#[no_mangle]
extern "C" fn HUD_DrawTransparentTriangles() {
    // TODO:
}

#[no_mangle]
unsafe extern "C" fn HUD_ConnectionlessPacket(
    _net_from: *const netadr_s,
    _args: *const c_char,
    _response_buffer: *mut c_char,
    response_buffer_size: *mut c_int,
) -> c_int {
    let _max_buffer_size = unsafe { *response_buffer_size };

    // set to zero if there is no response
    unsafe {
        *response_buffer_size = 0;
    }

    0
}

#[no_mangle]
extern "C" fn HUD_GetHullBounds(
    hullnumber: c_int,
    mins: Option<&mut vec3_t>,
    maxs: Option<&mut vec3_t>,
) -> c_int {
    pm::get_hull_bounds_ffi(hullnumber, mins.unwrap(), maxs.unwrap())
}

#[no_mangle]
unsafe extern "C" fn HUD_PlayerMove(pm: *mut playermove_s, is_server: c_int) {
    pm::player_move(unsafe { &mut *pm.cast() }, is_server != 0);
}

#[no_mangle]
unsafe extern "C" fn HUD_PlayerMoveInit(pm: *mut playermove_s) {
    pm::player_move_init(unsafe { &mut *pm.cast() });
}

#[no_mangle]
unsafe extern "C" fn HUD_PlayerMoveTexture(name: *const c_char) -> c_char {
    pm::find_texture_type(unsafe { CStrThin::from_ptr(name) })
}

#[no_mangle]
unsafe extern "C" fn HUD_TxferLocalOverrides(
    state: Option<&mut entity_state_s>,
    client: Option<&clientdata_s>,
) {
    entities().txfer_local_overrides(state.unwrap(), client.unwrap());
}

#[no_mangle]
unsafe extern "C" fn HUD_ProcessPlayerState(
    dst: Option<&mut entity_state_s>,
    src: Option<&entity_state_s>,
) {
    entities_mut().process_player_state(dst.unwrap(), src.unwrap());
}

#[no_mangle]
unsafe extern "C" fn HUD_TxferPredictionData(
    ps: Option<&mut entity_state_s>,
    pps: Option<&entity_state_s>,
    pcd: Option<&mut clientdata_s>,
    ppcd: Option<&clientdata_s>,
    wd: *mut weapon_data_s,
    pwd: *const weapon_data_s,
) {
    let ps = ps.unwrap();
    let pps = pps.unwrap();
    let pcd = pcd.unwrap();
    let ppcd = ppcd.unwrap();
    let wd = unsafe { &mut *wd.cast() };
    let pwd = unsafe { &*pwd.cast() };

    entities_mut().txfer_prediction_data(ps, pps, pcd, ppcd, wd, pwd);
}

#[no_mangle]
unsafe extern "C" fn CL_CreateMove(frametime: f32, cmd: *mut usercmd_s, active: c_int) {
    let cmd = unsafe { &mut *cmd };
    *cmd = input_mut().create_move(frametime, active != 0);
}

#[no_mangle]
unsafe extern "C" fn HUD_GetUserEntity(_index: c_int) -> *mut cl_entity_s {
    ptr::null_mut()
}

#[no_mangle]
extern "C" fn HUD_DirectorMessage(size: c_int, buf: *const c_void) {
    debug!("HUD_DirectorMessage({size}, {buf:?})");
}

#[no_mangle]
extern "C" fn HUD_VoiceStatus(_entindex: c_int, _talking: qboolean) {
    // TODO:
}

#[no_mangle]
unsafe extern "C" fn Demo_ReadBuffer(_size: c_int, _buffer: *mut byte) {
    debug!("TODO: Demo_ReadBuffer");
}

#[no_mangle]
unsafe extern "C" fn HUD_StudioEvent(
    _event: *const engine::raw::mstudioevent_s,
    _entity: *const cl_entity_s,
) {
    // TODO:
}

#[no_mangle]
unsafe extern "C" fn HUD_PostRunCmd(
    from: Option<&mut local_state_s>,
    to: Option<&mut local_state_s>,
    cmd: Option<&mut usercmd_s>,
    runfuncs: c_int,
    time: f64,
    random_seed: c_uint,
) {
    let from = from.unwrap();
    let to = to.unwrap();
    let cmd = cmd.unwrap();
    weapons_mut().post_run_cmd(from, to, cmd, runfuncs != 0, time, random_seed);
}

#[allow(non_snake_case)]
unsafe extern "C" fn StudioDrawModel(flags: c_int) -> c_int {
    renderer_mut().draw_model(flags)
}

#[allow(non_snake_case)]
unsafe extern "C" fn StudioDrawPlayer(flags: c_int, player: *mut entity_state_s) -> c_int {
    let player = unsafe { &mut *player };
    renderer_mut().draw_player(flags, player)
}

static mut STUDIO: engine::raw::r_studio_interface_s = engine::raw::r_studio_interface_s {
    version: shared::consts::STUDIO_INTERFACE_VERSION,
    StudioDrawModel: Some(StudioDrawModel),
    StudioDrawPlayer: Some(StudioDrawPlayer),
};

#[no_mangle]
unsafe extern "C" fn HUD_GetStudioModelInterface(
    version: c_int,
    ppinterface: *mut *mut engine::raw::r_studio_interface_s,
    pstudio: *mut engine::raw::engine_studio_api_s,
) -> c_int {
    // TODO:
    if true {
        return 0;
    }

    if version != shared::consts::STUDIO_INTERFACE_VERSION {
        return 0;
    }

    unsafe {
        ptr::write(ppinterface, ptr::addr_of_mut!(STUDIO));
    }

    studio_set(unsafe { *pstudio });

    studio::init();

    1
}

#[no_mangle]
extern "C" fn HUD_ChatInputPosition(_x: *mut c_int, _y: *mut c_int) {
    // TODO:
}

#[no_mangle]
unsafe extern "C" fn V_CalcRefdef(params: Option<&mut ref_params_s>) {
    view_mut().calc_ref_def(params.unwrap());
}

#[no_mangle]
unsafe extern "C" fn CL_IsThirdPerson() -> c_int {
    camera().is_third_person() as c_int
}

#[no_mangle]
unsafe extern "C" fn CL_CameraOffset(ofs: *mut vec3_t) {
    unsafe {
        *ofs = camera().offset();
    }
}

#[no_mangle]
unsafe extern "C" fn CAM_Think() {
    camera_mut().think();
}

#[no_mangle]
unsafe extern "C" fn KB_Find(name: *const c_char) -> *mut kbutton_t {
    let name = unsafe { CStrThin::from_ptr(name) };
    input().keys.find(name).unwrap_or(ptr::null_mut())
}

#[no_mangle]
unsafe extern "C" fn HUD_Key_Event(
    _down: c_int,
    _keynum: c_int,
    _current_binding: *const c_char,
) -> c_int {
    1
}

#[no_mangle]
unsafe extern "C" fn IN_MouseEvent(mstate: c_int) {
    input_mut().mouse_event(mstate);
}

#[no_mangle]
unsafe extern "C" fn IN_ClearStates() {
    input_mut().clear_states();
}

#[no_mangle]
unsafe extern "C" fn IN_Accumulate() {
    input_mut().accumulate();
}

#[no_mangle]
unsafe extern "C" fn IN_ActivateMouse() {
    input_mut().activate_mouse();
}

#[no_mangle]
unsafe extern "C" fn IN_DeactivateMouse() {
    input_mut().deactivate_mouse();
}

#[no_mangle]
pub unsafe extern "C" fn HUD_AddEntity(
    ty: EntityType,
    ent: Option<&mut cl_entity_s>,
    modelname: *const c_char,
) -> c_int {
    let modelname = unsafe { CStrThin::from_ptr(modelname) };
    entities().add_entity(ty, ent.unwrap(), modelname) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn HUD_CreateEntities() {
    entities().create_entities();
}

#[no_mangle]
pub unsafe extern "C" fn HUD_TempEntUpdate(
    frametime: f64,
    client_time: f64,
    cl_gravity: f64,
    temp_ent_free: *mut *mut TEMPENTITY,
    temp_ent_active: *mut *mut TEMPENTITY,
    add_visible_entity: unsafe extern "C" fn(pEntity: *mut cl_entity_s) -> c_int,
    temp_ent_play_sound: unsafe extern "C" fn(pTemp: *mut TEMPENTITY, damp: f32),
) {
    let mut list = unsafe { entity::TempEntityList::new(temp_ent_active, temp_ent_free) };
    let add = |ent: &mut cl_entity_s| unsafe { add_visible_entity(ent) };
    let play = |temp: &mut TEMPENTITY, damp: f32| unsafe { temp_ent_play_sound(temp, damp) };

    entities().update_temp_entities(frametime, client_time, cl_gravity, &mut list, add, play);
}

#[no_mangle]
unsafe extern "C" fn F(cldll_func: Option<&mut engine::raw::cldll_func_s>) {
    let Some(cldll_func) = cldll_func else { return };

    cldll_func.clone_from(&engine::raw::cldll_func_s {
        pfnInitialize: Some(Initialize),
        pfnInit: Some(HUD_Init),
        pfnVidInit: Some(HUD_VidInit),
        pfnRedraw: Some(HUD_Redraw),
        pfnUpdateClientData: Some(HUD_UpdateClientData),
        pfnReset: Some(HUD_Reset),
        pfnPlayerMove: Some(HUD_PlayerMove),
        pfnPlayerMoveInit: Some(HUD_PlayerMoveInit),
        pfnPlayerMoveTexture: Some(HUD_PlayerMoveTexture),
        IN_ActivateMouse: Some(IN_ActivateMouse),
        IN_DeactivateMouse: Some(IN_DeactivateMouse),
        IN_MouseEvent: Some(IN_MouseEvent),
        IN_ClearStates: Some(IN_ClearStates),
        IN_Accumulate: Some(IN_Accumulate),
        CL_CreateMove: Some(CL_CreateMove),
        CL_IsThirdPerson: Some(CL_IsThirdPerson),
        CL_CameraOffset: Some(CL_CameraOffset),
        KB_Find: Some(KB_Find),
        CAM_Think: Some(CAM_Think),
        pfnCalcRefdef: Some(V_CalcRefdef),
        pfnAddEntity: Some(HUD_AddEntity),
        pfnCreateEntities: Some(HUD_CreateEntities),
        pfnDrawNormalTriangles: Some(HUD_DrawNormalTriangles),
        pfnDrawTransparentTriangles: Some(HUD_DrawNormalTriangles),
        pfnStudioEvent: Some(HUD_StudioEvent),
        pfnPostRunCmd: Some(HUD_PostRunCmd),
        pfnShutdown: Some(HUD_Shutdown),
        pfnTxferLocalOverrides: Some(HUD_TxferLocalOverrides),
        pfnProcessPlayerState: Some(HUD_ProcessPlayerState),
        pfnTxferPredictionData: Some(HUD_TxferPredictionData),
        pfnDemo_ReadBuffer: Some(Demo_ReadBuffer),
        pfnConnectionlessPacket: Some(HUD_ConnectionlessPacket),
        pfnGetHullBounds: Some(HUD_GetHullBounds),
        pfnFrame: Some(HUD_Frame),
        pfnKey_Event: Some(HUD_Key_Event),
        pfnTempEntUpdate: Some(HUD_TempEntUpdate),
        pfnGetUserEntity: Some(HUD_GetUserEntity),
        pfnVoiceStatus: Some(HUD_VoiceStatus),
        pfnDirectorMessage: Some(HUD_DirectorMessage),
        pfnGetStudioModelInterface: Some(HUD_GetStudioModelInterface),
        pfnChatInputPosition: Some(HUD_ChatInputPosition),
        // TODO:
        // pfnGetRenderInterface: None,
        // pfnClipMoveToEntity: None,
        // pfnTouchEvent: None,
        // pfnMoveEvent: None,
        // pfnLookEvent: None,
    });
}
