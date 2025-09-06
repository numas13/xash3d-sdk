use core::{
    ffi::{c_char, c_int, c_uint, c_void},
    marker::PhantomData,
    ptr, slice,
};

use csz::CStrThin;
use shared::{
    cvar::CVarPtr,
    raw::{
        byte, cl_entity_s, clientdata_s, engine_studio_api_s, entity_state_s, kbutton_t,
        mstudioevent_s, netadr_s, playermove_s, qboolean, r_studio_interface_s, usercmd_s, vec3_t,
        weapon_data_s,
    },
};

use crate::{
    collections::TempEntityList,
    engine::cl_enginefuncs_s,
    prelude::*,
    raw::{client_data_s, local_state_s, ref_params_s, EntityType, TEMPENTITY},
};

pub use shared::export::{impl_unsync_global, UnsyncGlobal};

#[allow(unused_variables)]
pub trait ClientDll: UnsyncGlobal {
    fn vid_init(&self) -> bool {
        true
    }

    fn redraw(&self, time: f32, intermission: bool) -> bool {
        true
    }

    fn frame(&self, time: f64) {}

    fn reset(&self) {}

    fn update_client_data(&self, data: &mut client_data_s, time: f32) -> bool {
        true
    }

    fn player_move_init(&self, pm: *mut playermove_s) {
        pm::player_move_init(unsafe { &mut *pm.cast() });
    }

    fn player_move(&self, pm: *mut playermove_s, is_server: bool) {
        pm::player_move(unsafe { &mut *pm.cast() }, is_server);
    }

    fn player_move_texture(&self, name: &CStrThin) -> c_char {
        pm::find_texture_type(name)
    }

    fn get_hull_bounds(&self, hullnumber: c_int, mins: &mut vec3_t, maxs: &mut vec3_t) -> bool {
        pm::get_hull_bounds_ffi(hullnumber, mins, maxs) != 0
    }

    fn activate_mouse(&self) {}

    fn deactivate_mouse(&self) {}

    fn mouse_event(&self, mstate: c_int) {}

    fn clear_states(&self) {}

    fn accumulate(&self) {}

    fn create_move(&self, frametime: f32, active: bool) -> usercmd_s {
        usercmd_s::default()
    }

    fn is_third_person(&self) -> bool {
        false
    }

    fn camera_offset(&self) -> vec3_t {
        vec3_t::ZERO
    }

    fn camera_think(&self) {}

    fn kb_find(&self, name: &CStrThin) -> *mut kbutton_t {
        ptr::null_mut()
    }

    fn calc_ref_def(&self, params: &mut ref_params_s) {}

    fn add_entity(&self, ty: EntityType, ent: &mut cl_entity_s, model_name: &CStrThin) -> bool {
        false
    }

    fn create_entities(&self) {}

    fn draw_normal_triangles(&self) {}

    fn draw_transparent_triangles(&self) {}

    fn studio_event(&self, event: &mstudioevent_s, entity: &cl_entity_s) {}

    fn post_run_cmd(
        &self,
        from: &mut local_state_s,
        to: &mut local_state_s,
        cmd: &mut usercmd_s,
        run_funcs: bool,
        time: f64,
        random_seed: c_uint,
    ) {
    }

    fn txfer_local_overrides(&self, state: &mut entity_state_s, client: &clientdata_s) {}

    fn process_player_state(&self, dst: &mut entity_state_s, src: &entity_state_s) {}

    fn txfer_prediction_data(
        &self,
        ps: &mut entity_state_s,
        pps: &entity_state_s,
        pcd: &mut clientdata_s,
        ppcd: &clientdata_s,
        wd: *mut weapon_data_s,
        pwd: *const weapon_data_s,
    ) {
    }

    fn demo_read_buffer(&self, buffer: &[u8]) {}

    fn connectionless_packet(
        &self,
        from: &netadr_s,
        args: &CStrThin,
        buffer: &mut [u8],
    ) -> Result<usize, ()> {
        // no response
        Ok(0)
    }

    fn key_event(&self, down: c_int, keynum: c_int, current_binding: Option<&CStrThin>) -> bool {
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn update_temp_entities(
        &self,
        frametime: f64,
        client_time: f64,
        cl_gravity: f64,
        list: &mut TempEntityList,
        add_visible_entity: impl FnMut(&mut cl_entity_s) -> c_int,
        play_sound: impl FnMut(&mut TEMPENTITY, f32),
    ) {
    }

    fn get_user_entity(&self, index: c_int) -> *mut cl_entity_s {
        ptr::null_mut()
    }

    fn voice_status(&self, ent_index: c_int, talking: bool) {}

    fn director_message(&self, buf: &[u8]) {}

    fn get_studio_model_interface(
        &self,
        version: c_int,
        interface: *mut *mut r_studio_interface_s,
        studio: *mut engine_studio_api_s,
    ) -> bool {
        false
    }

    fn chat_input_position(&self, x: &mut c_int, y: &mut c_int) {
        *x = 0;
        *y = 0;
    }
}

#[allow(non_camel_case_types)]
pub type cldll_func_s = ClientDllFunctions;

#[allow(non_snake_case)]
#[allow(clippy::type_complexity)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ClientDllFunctions {
    pub pfnInitialize: Option<
        unsafe extern "C" fn(pEnginefuncs: Option<&cl_enginefuncs_s>, iVersion: c_int) -> c_int,
    >,
    pub pfnInit: Option<unsafe extern "C" fn()>,
    pub pfnVidInit: Option<unsafe extern "C" fn() -> c_int>,
    pub pfnRedraw: Option<unsafe extern "C" fn(flTime: f32, intermission: c_int) -> c_int>,
    pub pfnUpdateClientData:
        Option<unsafe extern "C" fn(cdata: Option<&mut client_data_s>, flTime: f32) -> c_int>,
    pub pfnReset: Option<unsafe extern "C" fn()>,
    pub pfnPlayerMove: Option<unsafe extern "C" fn(ppmove: *mut playermove_s, server: c_int)>,
    pub pfnPlayerMoveInit: Option<unsafe extern "C" fn(ppmove: *mut playermove_s)>,
    pub pfnPlayerMoveTexture: Option<unsafe extern "C" fn(name: *const c_char) -> c_char>,
    pub IN_ActivateMouse: Option<unsafe extern "C" fn()>,
    pub IN_DeactivateMouse: Option<unsafe extern "C" fn()>,
    pub IN_MouseEvent: Option<unsafe extern "C" fn(mstate: c_int)>,
    pub IN_ClearStates: Option<unsafe extern "C" fn()>,
    pub IN_Accumulate: Option<unsafe extern "C" fn()>,
    pub CL_CreateMove:
        Option<unsafe extern "C" fn(frametime: f32, cmd: *mut usercmd_s, active: c_int)>,
    pub CL_IsThirdPerson: Option<unsafe extern "C" fn() -> c_int>,
    pub CL_CameraOffset: Option<unsafe extern "C" fn(ofs: *mut vec3_t)>,
    pub KB_Find: Option<unsafe extern "C" fn(name: *const c_char) -> *mut kbutton_t>,
    pub CAM_Think: Option<unsafe extern "C" fn()>,
    pub pfnCalcRefdef: Option<unsafe extern "C" fn(pparams: Option<&mut ref_params_s>)>,
    pub pfnAddEntity: Option<
        unsafe extern "C" fn(
            entity_type: EntityType,
            entity: Option<&mut cl_entity_s>,
            model_name: *const c_char,
        ) -> c_int,
    >,
    pub pfnCreateEntities: Option<unsafe extern "C" fn()>,
    pub pfnDrawNormalTriangles: Option<unsafe extern "C" fn()>,
    pub pfnDrawTransparentTriangles: Option<unsafe extern "C" fn()>,
    pub pfnStudioEvent:
        Option<unsafe extern "C" fn(event: *const mstudioevent_s, entity: *const cl_entity_s)>,
    pub pfnPostRunCmd: Option<
        unsafe extern "C" fn(
            from: Option<&mut local_state_s>,
            to: Option<&mut local_state_s>,
            cmd: Option<&mut usercmd_s>,
            runfuncs: c_int,
            time: f64,
            random_seed: c_uint,
        ),
    >,
    pub pfnShutdown: Option<unsafe extern "C" fn()>,
    pub pfnTxferLocalOverrides: Option<
        unsafe extern "C" fn(state: Option<&mut entity_state_s>, client: Option<&clientdata_s>),
    >,
    pub pfnProcessPlayerState: Option<
        unsafe extern "C" fn(dst: Option<&mut entity_state_s>, src: Option<&entity_state_s>),
    >,
    pub pfnTxferPredictionData: Option<
        unsafe extern "C" fn(
            ps: Option<&mut entity_state_s>,
            pps: Option<&entity_state_s>,
            pcd: Option<&mut clientdata_s>,
            ppcd: Option<&clientdata_s>,
            wd: *mut weapon_data_s,
            pwd: *const weapon_data_s,
        ),
    >,
    pub pfnDemo_ReadBuffer: Option<unsafe extern "C" fn(size: c_int, buffer: *mut byte)>,
    pub pfnConnectionlessPacket: Option<
        unsafe extern "C" fn(
            net_from: *const netadr_s,
            args: *const c_char,
            buffer: *mut c_char,
            size: *mut c_int,
        ) -> c_int,
    >,
    pub pfnGetHullBounds: Option<
        unsafe extern "C" fn(
            hullnumber: c_int,
            mins: Option<&mut vec3_t>,
            maxs: Option<&mut vec3_t>,
        ) -> c_int,
    >,
    pub pfnFrame: Option<unsafe extern "C" fn(time: f64)>,
    pub pfnKey_Event: Option<
        unsafe extern "C" fn(
            eventcode: c_int,
            keynum: c_int,
            pszCurrentBinding: *const c_char,
        ) -> c_int,
    >,
    pub pfnTempEntUpdate: Option<
        unsafe extern "C" fn(
            frametime: f64,
            client_time: f64,
            cl_gravity: f64,
            ppTempEntFree: *mut *mut TEMPENTITY,
            ppTempEntActive: *mut *mut TEMPENTITY,
            AddVisibleEntity: unsafe extern "C" fn(pEntity: *mut cl_entity_s) -> c_int,
            TempEntPlaySound: unsafe extern "C" fn(pTemp: *mut TEMPENTITY, damp: f32),
        ),
    >,
    pub pfnGetUserEntity: Option<unsafe extern "C" fn(index: c_int) -> *mut cl_entity_s>,
    pub pfnVoiceStatus: Option<unsafe extern "C" fn(entindex: c_int, bTalking: qboolean)>,
    pub pfnDirectorMessage: Option<unsafe extern "C" fn(iSize: c_int, pbuf: *const c_void)>,
    pub pfnGetStudioModelInterface: Option<
        unsafe extern "C" fn(
            version: c_int,
            ppinterface: *mut *mut r_studio_interface_s,
            pstudio: *mut engine_studio_api_s,
        ) -> c_int,
    >,
    pub pfnChatInputPosition: Option<unsafe extern "C" fn(x: *mut c_int, y: *mut c_int)>,
    // TODO:
    // pub pfnGetRenderInterface: Option<
    //     unsafe extern "C" fn(
    //         version: c_int,
    //         renderfuncs: *mut render_api_t,
    //         callback: *mut render_interface_t,
    //     ) -> c_int,
    // >,
    // pub pfnClipMoveToEntity: Option<
    //     unsafe extern "C" fn(
    //         pe: *mut physent_s,
    //         start: *mut vec3_t,
    //         mins: *mut vec3_t,
    //         maxs: *mut vec3_t,
    //         end: *mut vec3_t,
    //         tr: *mut pmtrace_s,
    //     ),
    // >,
    // pub pfnTouchEvent: Option<
    //     unsafe extern "C" fn(
    //         type_: c_int,
    //         fingerID: c_int,
    //         x: f32,
    //         y: f32,
    //         dx: f32,
    //         dy: f32,
    //     ) -> c_int,
    // >,
    // pub pfnMoveEvent: Option<unsafe extern "C" fn(forwardmove: f32, sidemove: f32)>,
    // pub pfnLookEvent: Option<unsafe extern "C" fn(relyaw: f32, relpitch: f32)>,
}

impl ClientDllFunctions {
    pub fn new<T: ClientDll + Default>() -> Self {
        Export::<T>::client_functions()
    }
}

trait ClientDllExport {
    fn client_functions() -> ClientDllFunctions {
        ClientDllFunctions {
            pfnInitialize: Some(Self::initialize),
            pfnInit: Some(Self::init),
            pfnVidInit: Some(Self::vid_init),
            pfnRedraw: Some(Self::redraw),
            pfnUpdateClientData: Some(Self::update_client_data),
            pfnReset: Some(Self::reset),
            pfnPlayerMove: Some(Self::player_move),
            pfnPlayerMoveInit: Some(Self::player_move_init),
            pfnPlayerMoveTexture: Some(Self::player_move_texture),
            IN_ActivateMouse: Some(Self::activate_mouse),
            IN_DeactivateMouse: Some(Self::deactivate_mouse),
            IN_MouseEvent: Some(Self::mouse_event),
            IN_ClearStates: Some(Self::input_clear_states),
            IN_Accumulate: Some(Self::input_accumulate),
            CL_CreateMove: Some(Self::create_move),
            CL_IsThirdPerson: Some(Self::is_third_person),
            CL_CameraOffset: Some(Self::camera_offset),
            KB_Find: Some(Self::kb_find),
            CAM_Think: Some(Self::camera_think),
            pfnCalcRefdef: Some(Self::calc_ref_def),
            pfnAddEntity: Some(Self::add_entity),
            pfnCreateEntities: Some(Self::create_entities),
            pfnDrawNormalTriangles: Some(Self::draw_normal_triangles),
            pfnDrawTransparentTriangles: Some(Self::draw_transparent_triangles),
            pfnStudioEvent: Some(Self::studio_event),
            pfnPostRunCmd: Some(Self::post_run_cmd),
            pfnShutdown: Some(Self::shutdown),
            pfnTxferLocalOverrides: Some(Self::txfer_local_overrides),
            pfnProcessPlayerState: Some(Self::process_player_state),
            pfnTxferPredictionData: Some(Self::txfer_prediction_data),
            pfnDemo_ReadBuffer: Some(Self::demo_read_buffer),
            pfnConnectionlessPacket: Some(Self::connectionless_packet),
            pfnGetHullBounds: Some(Self::get_hull_bounds),
            pfnFrame: Some(Self::frame),
            pfnKey_Event: Some(Self::key_event),
            pfnTempEntUpdate: Some(Self::update_temp_entities),
            pfnGetUserEntity: Some(Self::get_user_entity),
            pfnVoiceStatus: Some(Self::voice_status),
            pfnDirectorMessage: Some(Self::director_message),
            pfnGetStudioModelInterface: Some(Self::get_studio_model_interface),
            pfnChatInputPosition: Some(Self::chat_input_position),
            // TODO:
            // pfnGetRenderInterface: None,
            // pfnClipMoveToEntity: None,
            // pfnTouchEvent: None,
            // pfnMoveEvent: None,
            // pfnLookEvent: None,
        }
    }

    unsafe extern "C" fn initialize(
        engine_funcs: Option<&cl_enginefuncs_s>,
        version: c_int,
    ) -> c_int;

    unsafe extern "C" fn init();

    unsafe extern "C" fn shutdown();

    unsafe extern "C" fn vid_init() -> c_int;

    unsafe extern "C" fn redraw(time: f32, intermission: c_int) -> c_int;

    unsafe extern "C" fn frame(time: f64);

    unsafe extern "C" fn reset();

    unsafe extern "C" fn update_client_data(data: Option<&mut client_data_s>, time: f32) -> c_int;

    unsafe extern "C" fn player_move_init(pm: *mut playermove_s);

    unsafe extern "C" fn player_move(pm: *mut playermove_s, is_server: c_int);

    unsafe extern "C" fn player_move_texture(name: *const c_char) -> c_char;

    unsafe extern "C" fn get_hull_bounds(
        hullnumber: c_int,
        ret_mins: Option<&mut vec3_t>,
        ret_maxs: Option<&mut vec3_t>,
    ) -> c_int;

    unsafe extern "C" fn activate_mouse();

    unsafe extern "C" fn deactivate_mouse();

    unsafe extern "C" fn mouse_event(mstate: c_int);

    unsafe extern "C" fn input_clear_states();

    unsafe extern "C" fn input_accumulate();

    unsafe extern "C" fn create_move(frametime: f32, cmd: *mut usercmd_s, active: c_int);

    unsafe extern "C" fn is_third_person() -> c_int;

    unsafe extern "C" fn camera_offset(ofs: *mut vec3_t);

    unsafe extern "C" fn camera_think();

    unsafe extern "C" fn kb_find(name: *const c_char) -> *mut kbutton_t;

    unsafe extern "C" fn calc_ref_def(params: Option<&mut ref_params_s>);

    unsafe extern "C" fn add_entity(
        ty: EntityType,
        ent: Option<&mut cl_entity_s>,
        modelname: *const c_char,
    ) -> c_int;

    unsafe extern "C" fn create_entities();

    unsafe extern "C" fn draw_normal_triangles();

    unsafe extern "C" fn draw_transparent_triangles();

    unsafe extern "C" fn studio_event(event: *const mstudioevent_s, entity: *const cl_entity_s);

    unsafe extern "C" fn post_run_cmd(
        from: Option<&mut local_state_s>,
        to: Option<&mut local_state_s>,
        cmd: Option<&mut usercmd_s>,
        runfuncs: c_int,
        time: f64,
        random_seed: c_uint,
    );

    unsafe extern "C" fn txfer_local_overrides(
        state: Option<&mut entity_state_s>,
        client: Option<&clientdata_s>,
    );

    unsafe extern "C" fn process_player_state(
        dst: Option<&mut entity_state_s>,
        src: Option<&entity_state_s>,
    );

    unsafe extern "C" fn txfer_prediction_data(
        ps: Option<&mut entity_state_s>,
        pps: Option<&entity_state_s>,
        pcd: Option<&mut clientdata_s>,
        ppcd: Option<&clientdata_s>,
        wd: *mut weapon_data_s,
        pwd: *const weapon_data_s,
    );

    unsafe extern "C" fn demo_read_buffer(size: c_int, buffer: *mut byte);

    unsafe extern "C" fn connectionless_packet(
        from: *const netadr_s,
        args: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: *mut c_int,
    ) -> c_int;

    unsafe extern "C" fn key_event(
        down: c_int,
        keynum: c_int,
        current_binding: *const c_char,
    ) -> c_int;

    unsafe extern "C" fn update_temp_entities(
        frametime: f64,
        client_time: f64,
        cl_gravity: f64,
        temp_ent_free: *mut *mut TEMPENTITY,
        temp_ent_active: *mut *mut TEMPENTITY,
        add_visible_entity: unsafe extern "C" fn(entity: *mut cl_entity_s) -> c_int,
        temp_ent_play_sound: unsafe extern "C" fn(temp: *mut TEMPENTITY, damp: f32),
    );

    unsafe extern "C" fn get_user_entity(index: c_int) -> *mut cl_entity_s;

    unsafe extern "C" fn voice_status(ent_index: c_int, talking: qboolean);

    unsafe extern "C" fn director_message(size: c_int, buf: *const c_void);

    unsafe extern "C" fn get_studio_model_interface(
        version: c_int,
        interface: *mut *mut r_studio_interface_s,
        studio: *mut engine_studio_api_s,
    ) -> c_int;

    unsafe extern "C" fn chat_input_position(x: *mut c_int, y: *mut c_int);
}

struct Export<T> {
    dll: PhantomData<T>,
}

impl<T: ClientDll + Default> ClientDllExport for Export<T> {
    unsafe extern "C" fn initialize(
        engine_funcs: Option<&cl_enginefuncs_s>,
        version: c_int,
    ) -> c_int {
        if version != crate::CLDLL_INTERFACE_VERSION {
            return 0;
        }
        let Some(engine_funcs) = engine_funcs else {
            return 0;
        };
        unsafe {
            crate::instance::init_engine(engine_funcs);
        }

        crate::cvar::init(|name, value, flags| {
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

        // TODO: CL_LoadParticleMan();

        1
    }

    unsafe extern "C" fn init() {
        unsafe {
            (&mut *T::global_as_mut_ptr()).write(T::default());
        }
    }

    unsafe extern "C" fn shutdown() {
        unsafe {
            (&mut *T::global_as_mut_ptr()).assume_init_drop();
        }
    }

    unsafe extern "C" fn vid_init() -> c_int {
        unsafe { T::global_assume_init_ref() }.vid_init() as c_int
    }

    unsafe extern "C" fn redraw(time: f32, intermission: c_int) -> c_int {
        unsafe { T::global_assume_init_ref() }.redraw(time, intermission != 0) as c_int
    }

    unsafe extern "C" fn frame(time: f64) {
        unsafe { T::global_assume_init_ref() }.frame(time);
    }
    unsafe extern "C" fn reset() {
        unsafe { T::global_assume_init_ref() }.reset();
    }

    unsafe extern "C" fn update_client_data(data: Option<&mut client_data_s>, time: f32) -> c_int {
        let Some(data) = data else { return 0 };
        unsafe { T::global_assume_init_ref() }.update_client_data(data, time) as c_int
    }

    unsafe extern "C" fn player_move_init(pm: *mut playermove_s) {
        unsafe { T::global_assume_init_ref() }.player_move_init(pm)
    }

    unsafe extern "C" fn player_move(pm: *mut playermove_s, is_server: c_int) {
        unsafe { T::global_assume_init_ref() }.player_move(pm, is_server != 0)
    }

    unsafe extern "C" fn player_move_texture(name: *const c_char) -> c_char {
        assert!(!name.is_null());
        let name = unsafe { CStrThin::from_ptr(name) };
        unsafe { T::global_assume_init_ref() }.player_move_texture(name)
    }

    unsafe extern "C" fn get_hull_bounds(
        hullnumber: c_int,
        ret_mins: Option<&mut vec3_t>,
        ret_maxs: Option<&mut vec3_t>,
    ) -> c_int {
        let ret_mins = ret_mins.unwrap();
        let ret_maxs = ret_maxs.unwrap();
        unsafe { T::global_assume_init_ref() }.get_hull_bounds(hullnumber, ret_mins, ret_maxs)
            as c_int
    }

    unsafe extern "C" fn activate_mouse() {
        unsafe { T::global_assume_init_ref() }.activate_mouse();
    }

    unsafe extern "C" fn deactivate_mouse() {
        unsafe { T::global_assume_init_ref() }.deactivate_mouse();
    }

    unsafe extern "C" fn mouse_event(mstate: c_int) {
        // TODO: net type for mstate
        unsafe { T::global_assume_init_ref() }.mouse_event(mstate);
    }

    unsafe extern "C" fn input_clear_states() {
        unsafe { T::global_assume_init_ref() }.clear_states();
    }

    unsafe extern "C" fn input_accumulate() {
        unsafe { T::global_assume_init_ref() }.accumulate();
    }

    unsafe extern "C" fn create_move(frametime: f32, cmd: *mut usercmd_s, active: c_int) {
        assert!(!cmd.is_null());
        let cmd = unsafe { &mut *cmd };
        *cmd = unsafe { T::global_assume_init_ref() }.create_move(frametime, active != 0);
    }

    unsafe extern "C" fn is_third_person() -> c_int {
        unsafe { T::global_assume_init_ref() }.is_third_person() as c_int
    }

    unsafe extern "C" fn camera_offset(ofs: *mut vec3_t) {
        assert!(!ofs.is_null());
        let ofs = unsafe { &mut *ofs };
        *ofs = unsafe { T::global_assume_init_ref() }.camera_offset();
    }

    unsafe extern "C" fn camera_think() {
        unsafe { T::global_assume_init_ref() }.camera_think();
    }

    unsafe extern "C" fn kb_find(name: *const c_char) -> *mut kbutton_t {
        assert!(!name.is_null());
        let name = unsafe { CStrThin::from_ptr(name) };
        unsafe { T::global_assume_init_ref() }.kb_find(name)
    }

    unsafe extern "C" fn calc_ref_def(params: Option<&mut ref_params_s>) {
        unsafe { T::global_assume_init_ref() }.calc_ref_def(params.unwrap());
    }

    unsafe extern "C" fn add_entity(
        ty: EntityType,
        ent: Option<&mut cl_entity_s>,
        model_name: *const c_char,
    ) -> c_int {
        assert!(!model_name.is_null());
        let model_name = unsafe { CStrThin::from_ptr(model_name) };
        unsafe { T::global_assume_init_ref() }.add_entity(ty, ent.unwrap(), model_name) as c_int
    }

    unsafe extern "C" fn create_entities() {
        unsafe { T::global_assume_init_ref() }.create_entities();
    }

    unsafe extern "C" fn draw_normal_triangles() {
        unsafe { T::global_assume_init_ref() }.draw_normal_triangles();
    }

    unsafe extern "C" fn draw_transparent_triangles() {
        unsafe { T::global_assume_init_ref() }.draw_transparent_triangles();
    }

    unsafe extern "C" fn studio_event(event: *const mstudioevent_s, entity: *const cl_entity_s) {
        assert!(!event.is_null() && !entity.is_null());
        let event = unsafe { &*event };
        let entity = unsafe { &*entity };
        unsafe { T::global_assume_init_ref() }.studio_event(event, entity);
    }

    unsafe extern "C" fn post_run_cmd(
        from: Option<&mut local_state_s>,
        to: Option<&mut local_state_s>,
        cmd: Option<&mut usercmd_s>,
        run_funcs: c_int,
        time: f64,
        random_seed: c_uint,
    ) {
        let from = from.unwrap();
        let to = to.unwrap();
        let cmd = cmd.unwrap();
        unsafe { T::global_assume_init_ref() }.post_run_cmd(
            from,
            to,
            cmd,
            run_funcs != 0,
            time,
            random_seed,
        );
    }

    unsafe extern "C" fn txfer_local_overrides(
        state: Option<&mut entity_state_s>,
        client: Option<&clientdata_s>,
    ) {
        let state = state.unwrap();
        let client = client.unwrap();
        unsafe { T::global_assume_init_ref() }.txfer_local_overrides(state, client);
    }

    unsafe extern "C" fn process_player_state(
        dst: Option<&mut entity_state_s>,
        src: Option<&entity_state_s>,
    ) {
        let dst = dst.unwrap();
        let src = src.unwrap();
        unsafe { T::global_assume_init_ref() }.process_player_state(dst, src);
    }

    unsafe extern "C" fn txfer_prediction_data(
        ps: Option<&mut entity_state_s>,
        pps: Option<&entity_state_s>,
        pcd: Option<&mut clientdata_s>,
        ppcd: Option<&clientdata_s>,
        wd: *mut weapon_data_s,
        pwd: *const weapon_data_s,
    ) {
        assert!(!wd.is_null() && !pwd.is_null());
        let ps = ps.unwrap();
        let pps = pps.unwrap();
        let pcd = pcd.unwrap();
        let ppcd = ppcd.unwrap();
        unsafe { T::global_assume_init_ref() }.txfer_prediction_data(ps, pps, pcd, ppcd, wd, pwd);
    }

    unsafe extern "C" fn demo_read_buffer(size: c_int, buffer: *mut byte) {
        assert!(!buffer.is_null());
        let buffer = unsafe { slice::from_raw_parts(buffer, size as usize) };
        unsafe { T::global_assume_init_ref() }.demo_read_buffer(buffer);
    }

    unsafe extern "C" fn connectionless_packet(
        from: *const netadr_s,
        args: *const c_char,
        response_buffer: *mut c_char,
        response_buffer_size: *mut c_int,
    ) -> c_int {
        assert!(
            !from.is_null()
                && !args.is_null()
                && !response_buffer.is_null()
                && !response_buffer_size.is_null()
        );
        let from = unsafe { &*from };
        let args = unsafe { CStrThin::from_ptr(args) };
        let max_buffer_size = unsafe { *response_buffer_size } as usize;
        let buffer = unsafe { slice::from_raw_parts_mut(response_buffer.cast(), max_buffer_size) };
        match unsafe { T::global_assume_init_ref() }.connectionless_packet(from, args, buffer) {
            Ok(len) => {
                unsafe {
                    *response_buffer_size = len as c_int;
                }
                (len > 0) as c_int
            }
            Err(_) => 0,
        }
    }

    unsafe extern "C" fn key_event(
        down: c_int,
        keynum: c_int,
        current_binding: *const c_char,
    ) -> c_int {
        let current_binding = if current_binding.is_null() {
            None
        } else {
            Some(unsafe { CStrThin::from_ptr(current_binding) })
        };
        unsafe { T::global_assume_init_ref() }.key_event(down, keynum, current_binding) as c_int
    }

    unsafe extern "C" fn update_temp_entities(
        frametime: f64,
        client_time: f64,
        cl_gravity: f64,
        temp_ent_free: *mut *mut TEMPENTITY,
        temp_ent_active: *mut *mut TEMPENTITY,
        add_visible_entity: unsafe extern "C" fn(entity: *mut cl_entity_s) -> c_int,
        play_sound: unsafe extern "C" fn(temp: *mut TEMPENTITY, damp: f32),
    ) {
        let mut list = unsafe { TempEntityList::from_raw_parts(temp_ent_active, temp_ent_free) };
        let add_visible_entity = |ent: &mut cl_entity_s| unsafe { add_visible_entity(ent) };
        let play_sound = |temp: &mut TEMPENTITY, damp: f32| unsafe { play_sound(temp, damp) };
        unsafe { T::global_assume_init_ref() }.update_temp_entities(
            frametime,
            client_time,
            cl_gravity,
            &mut list,
            add_visible_entity,
            play_sound,
        );
    }

    unsafe extern "C" fn get_user_entity(index: c_int) -> *mut cl_entity_s {
        unsafe { T::global_assume_init_ref() }.get_user_entity(index)
    }

    unsafe extern "C" fn voice_status(ent_index: c_int, talking: qboolean) {
        unsafe { T::global_assume_init_ref() }.voice_status(ent_index, talking.to_bool())
    }

    unsafe extern "C" fn director_message(size: c_int, buf: *const c_void) {
        assert!(!buf.is_null());
        let buf = unsafe { slice::from_raw_parts(buf.cast(), size as usize) };
        unsafe { T::global_assume_init_ref() }.director_message(buf)
    }

    unsafe extern "C" fn get_studio_model_interface(
        version: c_int,
        interface: *mut *mut r_studio_interface_s,
        studio: *mut engine_studio_api_s,
    ) -> c_int {
        unsafe { T::global_assume_init_ref() }
            .get_studio_model_interface(version, interface, studio) as c_int
    }

    unsafe extern "C" fn chat_input_position(x: *mut c_int, y: *mut c_int) {
        assert!(!x.is_null() && !y.is_null());
        let x = unsafe { &mut *x };
        let y = unsafe { &mut *y };
        unsafe { T::global_assume_init_ref() }.chat_input_position(x, y);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! export_dll {
    ($client_dll:ty $($init:block)?) => {
        #[no_mangle]
        unsafe extern "C" fn F(dll_funcs: Option<&mut $crate::export::ClientDllFunctions>) {
            if let Some(dll_funcs) = dll_funcs {
                *dll_funcs = $crate::export::ClientDllFunctions::new::<$client_dll>();
                $($init)?
            }
        }
    };
}
#[doc(inline)]
pub use export_dll;
