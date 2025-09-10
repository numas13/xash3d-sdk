use core::{
    ffi::{c_char, c_int, c_uchar, c_uint, c_void},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use csz::{CStrSlice, CStrThin};
use shared::{
    color::RGBA,
    consts::RefParm,
    ffi::{
        api::render::texFlags_t,
        common::uint,
        render::{
            ref_api_s, ref_globals_s, ref_interface_s, ref_screen_rotation_t, REF_API_VERSION,
        },
    },
    raw::{
        byte, cl_entity_s, colorVec, decal_s, decallist_s, lightstyle_t, model_s, msurface_s,
        particle_s, qboolean, ref_viewpass_s, vec2_t, vec3_t, TextureFlags, BEAM, MAX_LIGHTSTYLES,
        MAX_RENDER_DECALS, TRICULLSTYLE,
    },
    utils::{cstr_or_none, slice_from_raw_parts_or_empty},
};

use crate::{
    raw::{mstudioseqdesc_t, mstudiotex_s, rgbdata_t, SKYBOX_MAX_SIDES},
    texture::{TextureId, UNUSED_TEXTURE_NAME},
};

pub use shared::export::{impl_unsync_global, UnsyncGlobal};

#[allow(unused_variables)]
pub trait RefDll: UnsyncGlobal {
    fn new() -> Option<Self>;

    fn get_config_name(&self) -> Option<&'static CStrThin> {
        None
    }

    fn set_display_transform(
        &self,
        rotate: ref_screen_rotation_t,
        x: c_int,
        y: c_int,
        scale_x: f32,
        scale_y: f32,
    ) -> bool {
        true
    }

    fn gl_setup_attributes(&self, safe_gl: bool) {}

    fn gl_init_extensions(&self) {}

    fn gl_clear_extensions(&self) {}

    fn gamma_changed(&self, do_reset_gamma: bool) {}

    fn begin_frame(&self, clear_scene: bool) {}

    fn render_scene(&self) {}

    fn end_frame(&self) {}

    fn push_scene(&self) {}

    fn pop_scene(&self) {}

    fn gl_backend_start_frame(&self) {}

    fn gl_backend_end_frame(&self) {}

    fn clear_screen(&self) {}

    fn allow_fog(&self, allow: bool) {}

    fn gl_set_render_mode(&self, render_mode: c_int) {}

    fn add_entity(&self, ent: &mut cl_entity_s, type_: c_int) -> bool {
        true
    }

    fn add_custom_beam(&self, env_beam: &mut cl_entity_s) {}

    fn process_entity_data(&self, allocate: bool, entities: *mut cl_entity_s, max_entities: usize) {
    }

    fn flush(&self, flush_flags: c_uint) {}

    fn show_textures(&self) {}

    fn get_texture_original_buffer(&self, texture: TextureId) -> *const byte {
        ptr::null()
    }

    fn gl_load_texture_from_buffer(
        &self,
        name: &CStrThin,
        pic: *mut rgbdata_t,
        flags: TextureFlags,
        update: bool,
    ) -> Option<TextureId> {
        None
    }

    fn gl_process_texture(
        &self,
        texture: TextureId,
        gamma: f32,
        top_color: c_int,
        bottom_color: c_int,
    ) {
    }

    fn setup_skybox(&self, skybox_textures: &[Option<TextureId>; SKYBOX_MAX_SIDES]) {}

    fn unload_skybox(&self) {}

    fn set_2d_mode(&self, enable: bool) {}

    #[allow(clippy::too_many_arguments)]
    fn draw_stretch_raw(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        cols: c_int,
        rows: c_int,
        data: *const byte,
        dirty: bool,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_stretch_pic(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        texture: TextureId,
    ) {
    }

    fn fill_rgba(&self, render_mode: c_int, x: f32, y: f32, w: f32, h: f32, color: RGBA) {}

    /// Converts a point in world space coordinates to a screen space coordinates.
    ///
    /// Returns `Err` if the point is behind the screen.
    fn world_to_screen(&self, world: vec3_t) -> Result<vec3_t, vec3_t> {
        Err(vec3_t::ZERO)
    }

    fn screen_shot(&self, filename: &CStrThin, shot_type: c_int) -> bool {
        false
    }

    fn cubemap_shot(
        &self,
        base: &CStrThin,
        size: c_uint,
        vieworg: Option<&vec3_t>,
        skyshot: bool,
    ) -> bool {
        false
    }

    fn light_point(&self, point: vec3_t) -> RGBA {
        RGBA::splat(0)
    }

    fn decal_shoot(
        &self,
        texture: TextureId,
        entity_index: c_int,
        model_index: c_int,
        pos: &mut vec3_t,
        flags: c_int,
        scale: f32,
    ) {
    }

    fn decal_remove_all(&self, texture: TextureId) {}

    fn create_decal_list(&self, list: &mut [decallist_s]) -> usize {
        0
    }

    fn clear_all_decals(&self) {}

    fn studio_estimate_frame(
        &self,
        ent: &mut cl_entity_s,
        seq_desc: &mut mstudioseqdesc_t,
        time: f64,
    ) -> f32 {
        0.0
    }

    fn studio_lerp_movement(
        &self,
        ent: &mut cl_entity_s,
        time: f64,
        origin: &mut vec3_t,
        angles: &mut vec3_t,
    ) {
    }

    fn init_studio_api(&self) {}

    fn set_sky_clouds_textures(
        &self,
        solid_sky_texture: Option<TextureId>,
        alpha_sky_texture: Option<TextureId>,
    ) {
    }

    fn gl_subdivide_surface(&self, model: &mut model_s, fa: &mut msurface_s) {}

    fn run_light_styles(&self, ls: &mut [lightstyle_t]) {}

    fn get_sprite_parms(
        &self,
        frame_width: Option<&mut c_int>,
        frame_height: Option<&mut c_int>,
        num_frames: Option<&mut c_int>,
        current_frame: c_int,
        model: &model_s,
    ) {
        if let Some(frame_width) = frame_width {
            *frame_width = 0;
        }
        if let Some(frame_height) = frame_height {
            *frame_height = 0;
        }
        if let Some(num_frames) = num_frames {
            *num_frames = 0;
        }
    }

    fn get_sprite_texture(&self, sprite_model: &model_s, frame: c_int) -> Option<TextureId> {
        None
    }

    fn mod_process_render_data(
        &self,
        model: &mut model_s,
        create: bool,
        buffer: *const byte,
    ) -> bool {
        true
    }

    fn mod_studio_load_textures(&self, model: &mut model_s, data: NonNull<c_void>) {}

    // TODO: wrapper for particles list
    fn draw_particles(&self, frame_time: f64, particles: *mut particle_s, partsize: f32) {}

    // TODO: wrapper for particles list
    fn draw_tracers(&self, frame_time: f64, tracers: *mut particle_s) {}

    // TODO: wrapper for beams list
    fn draw_beams(&self, trans: bool, beams: *mut BEAM) {}

    fn beam_cull(&self, start: &vec3_t, end: &vec3_t, pvs_only: bool) -> bool {
        false
    }

    fn ref_get_parm(&self, parm: RefParm, arg: c_int) -> c_int {
        0
    }

    fn get_detail_scale_for_texture(&self, texture: TextureId) -> (f32, f32) {
        (1.0, 1.0)
    }

    fn get_extra_parms_for_texture(&self, texture: TextureId) -> RGBA {
        RGBA::splat(0)
    }

    fn get_frame_time(&self) -> f32 {
        0.0
    }

    fn set_current_entity(&self, ent: Option<&mut cl_entity_s>) {}

    fn set_current_model(&self, model: &mut model_s) {}

    fn gl_find_texture(&self, name: &CStrThin) -> Option<TextureId> {
        None
    }

    fn gl_texture_name(&self, texture: TextureId) -> *const c_char {
        ptr::null()
    }

    fn gl_texture_data(&self, texture: TextureId) -> *const byte {
        ptr::null()
    }

    fn gl_load_texture(&self, name: &CStrThin, buf: &[byte], flags: c_int) -> Option<TextureId> {
        None
    }

    fn gl_create_texture(
        &self,
        name: &CStrThin,
        width: c_int,
        height: c_int,
        buffer: &[RGBA],
        flags: TextureFlags,
    ) -> Option<TextureId> {
        None
    }

    fn gl_load_texture_array(&self, names: *mut *const c_char, flags: c_int) -> Option<TextureId> {
        None
    }

    fn gl_create_texture_array(
        &self,
        name: &CStrThin,
        width: c_int,
        height: c_int,
        depth: c_int,
        buffer: *const c_void,
        flags: TextureFlags,
    ) -> Option<TextureId> {
        None
    }

    fn gl_free_texture(&self, texture: TextureId) {}

    fn override_texture_source_size(
        &self,
        texture: TextureId,
        src_width: c_uint,
        src_height: c_uint,
    ) {
    }

    fn draw_single_decal(&self, decal: &mut decal_s, fa: &mut msurface_s) {}

    fn decal_setup_verts(
        &self,
        decal: &mut decal_s,
        surf: &mut msurface_s,
        texture: c_int,
        out_count: Option<&mut c_int>,
    ) -> *mut f32 {
        ptr::null_mut()
    }

    fn entity_remove_decals(&self, model: &mut model_s) {}

    fn avi_upload_raw_frame(
        &self,
        texture: TextureId,
        cols: c_int,
        rows: c_int,
        width: c_int,
        height: c_int,
        data: *const byte,
    ) {
    }

    fn gl_bind(&self, tmu: c_int, texture: Option<TextureId>) {}

    fn gl_select_texture(&self, tmu: c_int) {}

    fn gl_load_texture_matrix(&self, gl_matrix: *const f32) {}

    fn gl_tex_matrix_identity(&self) {}

    fn gl_clean_up_texture_units(&self, last: c_int) {}

    fn gl_tex_gen(&self, coord: c_uint, mode: c_uint) {}

    fn gl_texture_target(&self, target: c_uint) {}

    fn gl_tex_coord_array_mode(&self, mode: c_uint) {}

    fn gl_update_tex_size(&self, texture: TextureId, width: c_int, height: c_int, depth: c_int) {}

    fn gl_draw_particles(&self, rvp: &ref_viewpass_s, trans_pass: bool, frame_time: f32) {}

    fn light_vec(
        &self,
        start: vec3_t,
        end: vec3_t,
        light_spot: Option<&mut vec3_t>,
        light_vec: Option<&mut vec3_t>,
    ) -> RGBA {
        RGBA::default()
    }

    fn studio_get_texture(&self, ent: &mut cl_entity_s) -> *mut mstudiotex_s {
        ptr::null_mut()
    }

    fn gl_render_frame(&self, rvp: &ref_viewpass_s) {}

    fn gl_ortho_bounds(&self, mins: vec2_t, maxs: vec2_t) {}

    fn speeds_message(&self, out: &mut CStrSlice) -> bool {
        false
    }

    fn mod_get_current_vis(&self) -> *mut byte {
        ptr::null_mut()
    }

    fn new_map(&self) {}

    fn clear_scene(&self) {}

    fn get_proc_address(&self, name: &CStrThin) -> *mut c_void {
        ptr::null_mut()
    }

    fn tri_render_mode(&self, mode: c_int) {}

    fn begin(&self, primitive_code: c_int) {}

    fn end(&self) {}

    fn color4f(&self, r: f32, g: f32, b: f32, a: f32) {}

    fn color4ub(&self, color: RGBA) {}

    fn tex_coord2f(&self, u: f32, v: f32) {}

    fn vertex3fv(&self, point: &vec3_t) {
        self.vertex3f(point.x(), point.y(), point.z());
    }

    fn vertex3f(&self, x: f32, y: f32, z: f32) {}

    fn fog(&self, fog_color: &[f32; 3], start: f32, end: f32, on: bool) {}

    fn screen_to_world(&self, point: vec3_t) -> vec3_t {
        vec3_t::ZERO
    }

    fn get_matrix(&self, pname: c_int, matrix: *mut f32) {}

    fn fog_params(&self, density: f32, fog_skybox: c_int) {}

    fn cull_face(&self, mode: TRICULLSTYLE) {}

    fn vgui_setup_drawing(&self, rect: bool) {}

    fn vgui_upload_texture_block(
        &self,
        draw_x: c_int,
        draw_y: c_int,
        rgba: *const byte,
        block_width: c_int,
        block_height: c_int,
    ) {
    }
}

pub fn ref_functions<T: RefDll>() -> ref_interface_s {
    Export::<T>::ref_functions()
}

#[allow(clippy::missing_safety_doc)]
trait RefDllExport {
    fn ref_functions() -> ref_interface_s {
        ref_interface_s {
            R_Init: Some(Self::init),
            R_Shutdown: Some(Self::shutdown),
            R_GetConfigName: Some(Self::get_config_name),
            R_SetDisplayTransform: Some(Self::set_display_transform),
            GL_SetupAttributes: Some(Self::gl_setup_attributes),
            GL_InitExtensions: Some(Self::gl_init_extensions),
            GL_ClearExtensions: Some(Self::gl_clear_extensions),
            R_GammaChanged: Some(Self::gamma_changed),
            R_BeginFrame: Some(Self::begin_frame),
            R_RenderScene: Some(Self::render_scene),
            R_EndFrame: Some(Self::end_frame),
            R_PushScene: Some(Self::push_scene),
            R_PopScene: Some(Self::pop_scene),
            GL_BackendStartFrame: Some(Self::gl_backend_start_frame),
            GL_BackendEndFrame: Some(Self::gl_backend_end_frame),
            R_ClearScreen: Some(Self::clear_screen),
            R_AllowFog: Some(Self::allow_fog),
            GL_SetRenderMode: Some(Self::gl_set_render_mode),
            R_AddEntity: Some(Self::add_entity),
            CL_AddCustomBeam: Some(Self::add_custom_beam),
            R_ProcessEntData: Some(Self::process_entity_data),
            R_Flush: Some(Self::flush),
            R_ShowTextures: Some(Self::show_textures),
            R_GetTextureOriginalBuffer: Some(Self::get_texture_original_buffer),
            GL_LoadTextureFromBuffer: Some(Self::gl_load_texture_from_buffer),
            GL_ProcessTexture: Some(Self::gl_process_texture),
            R_SetupSky: Some(Self::setup_sky),
            R_Set2DMode: Some(Self::set_2d_mode),
            R_DrawStretchRaw: Some(Self::draw_stretch_raw),
            R_DrawStretchPic: Some(Self::draw_stretch_pic),
            FillRGBA: Some(Self::fill_rgba),
            WorldToScreen: Some(Self::world_to_screen),
            VID_ScreenShot: Some(Self::screen_shot),
            VID_CubemapShot: Some(Self::cubemap_shot),
            R_LightPoint: Some(Self::light_point),
            R_DecalShoot: Some(Self::decal_shoot),
            R_DecalRemoveAll: Some(Self::decal_remove_all),
            R_CreateDecalList: Some(Self::create_decal_list),
            R_ClearAllDecals: Some(Self::clear_all_decals),
            R_StudioEstimateFrame: Some(Self::studio_estimate_frame),
            R_StudioLerpMovement: Some(Self::studio_lerp_movement),
            CL_InitStudioAPI: Some(Self::init_studio_api),
            R_SetSkyCloudsTextures: Some(Self::set_sky_clouds_textures),
            GL_SubdivideSurface: Some(Self::gl_subdivide_surface),
            CL_RunLightStyles: Some(Self::run_light_styles),
            R_GetSpriteParms: Some(Self::get_sprite_parms),
            R_GetSpriteTexture: Some(Self::get_sprite_texture),
            Mod_ProcessRenderData: Some(Self::mod_process_render_data),
            Mod_StudioLoadTextures: Some(Self::mod_studio_load_textures),
            CL_DrawParticles: Some(Self::draw_particles),
            CL_DrawTracers: Some(Self::draw_tracers),
            CL_DrawBeams: Some(Self::draw_beams),
            R_BeamCull: Some(Self::beam_cull),
            RefGetParm: Some(Self::ref_get_parm),
            GetDetailScaleForTexture: Some(Self::get_detail_scale_for_texture),
            GetExtraParmsForTexture: Some(Self::get_extra_parms_for_texture),
            GetFrameTime: Some(Self::get_frame_time),
            R_SetCurrentEntity: Some(Self::set_current_entity),
            R_SetCurrentModel: Some(Self::set_current_model),
            GL_FindTexture: Some(Self::gl_find_texture),
            GL_TextureName: Some(Self::gl_texture_name),
            GL_TextureData: Some(Self::gl_texture_data),
            GL_LoadTexture: Some(Self::gl_load_texture),
            GL_CreateTexture: Some(Self::gl_create_texture),
            GL_LoadTextureArray: Some(Self::gl_load_texture_array),
            GL_CreateTextureArray: Some(Self::gl_create_texture_array),
            GL_FreeTexture: Some(Self::gl_free_texture),
            R_OverrideTextureSourceSize: Some(Self::override_texture_source_size),
            DrawSingleDecal: Some(Self::draw_single_decal),
            R_DecalSetupVerts: Some(Self::decal_setup_verts),
            R_EntityRemoveDecals: Some(Self::entity_remove_decals),
            AVI_UploadRawFrame: Some(Self::avi_upload_raw_frame),
            GL_Bind: Some(Self::gl_bind),
            GL_SelectTexture: Some(Self::gl_select_texture),
            GL_LoadTextureMatrix: Some(Self::gl_load_texture_matrix),
            GL_TexMatrixIdentity: Some(Self::gl_tex_matrix_identity),
            GL_CleanUpTextureUnits: Some(Self::gl_clean_up_texture_units),
            GL_TexGen: Some(Self::gl_tex_gen),
            GL_TextureTarget: Some(Self::gl_texture_target),
            GL_TexCoordArrayMode: Some(Self::gl_tex_coord_array_mode),
            GL_UpdateTexSize: Some(Self::gl_update_tex_size),
            GL_Reserved0: None,
            GL_Reserved1: None,
            GL_DrawParticles: Some(Self::gl_draw_particles),
            LightVec: Some(Self::light_vec),
            StudioGetTexture: Some(Self::studio_get_texture),
            GL_RenderFrame: Some(Self::gl_render_frame),
            GL_OrthoBounds: Some(Self::gl_ortho_bounds),
            R_SpeedsMessage: Some(Self::speeds_message),
            Mod_GetCurrentVis: Some(Self::mod_get_current_vis),
            R_NewMap: Some(Self::new_map),
            R_ClearScene: Some(Self::clear_scene),
            R_GetProcAddress: Some(Self::get_proc_address),
            TriRenderMode: Some(Self::tri_render_mode),
            Begin: Some(Self::begin),
            End: Some(Self::end),
            Color4f: Some(Self::color4f),
            Color4ub: Some(Self::color4ub),
            TexCoord2f: Some(Self::tex_coord2f),
            Vertex3fv: Some(Self::vertex3fv),
            Vertex3f: Some(Self::vertex3f),
            Fog: Some(Self::fog),
            ScreenToWorld: Some(Self::screen_to_world),
            GetMatrix: Some(Self::get_matrix),
            FogParams: Some(Self::fog_params),
            CullFace: Some(Self::cull_face),
            VGUI_SetupDrawing: Some(Self::vgui_setup_drawing),
            VGUI_UploadTextureBlock: Some(Self::vgui_upload_texture_block),
        }
    }

    unsafe extern "C" fn init() -> qboolean;

    unsafe extern "C" fn shutdown();

    unsafe extern "C" fn get_config_name() -> *const c_char;

    unsafe extern "C" fn set_display_transform(
        rotate: ref_screen_rotation_t,
        x: c_int,
        y: c_int,
        scale_x: f32,
        scale_y: f32,
    ) -> qboolean;

    unsafe extern "C" fn gl_setup_attributes(safe_gl: c_int);

    unsafe extern "C" fn gl_init_extensions();

    unsafe extern "C" fn gl_clear_extensions();

    unsafe extern "C" fn gamma_changed(do_reset_gamma: qboolean);

    unsafe extern "C" fn begin_frame(clear_scene: qboolean);

    unsafe extern "C" fn render_scene();

    unsafe extern "C" fn end_frame();

    unsafe extern "C" fn push_scene();

    unsafe extern "C" fn pop_scene();

    unsafe extern "C" fn gl_backend_start_frame();

    unsafe extern "C" fn gl_backend_end_frame();

    unsafe extern "C" fn clear_screen();

    unsafe extern "C" fn allow_fog(_allow: qboolean);

    unsafe extern "C" fn gl_set_render_mode(render_mode: c_int);

    unsafe extern "C" fn add_entity(ent: *mut cl_entity_s, type_: c_int) -> qboolean;

    unsafe extern "C" fn add_custom_beam(env_beam: *mut cl_entity_s);

    unsafe extern "C" fn process_entity_data(
        allocate: qboolean,
        entities: *mut cl_entity_s,
        max_entities: c_uint,
    );

    unsafe extern "C" fn flush(flush_flags: c_uint);

    unsafe extern "C" fn show_textures();

    unsafe extern "C" fn get_texture_original_buffer(idx: c_uint) -> *const byte;

    unsafe extern "C" fn gl_load_texture_from_buffer(
        name: *const c_char,
        pic: *mut rgbdata_t,
        flags: texFlags_t,
        update: qboolean,
    ) -> c_int;

    unsafe extern "C" fn gl_process_texture(
        texture: c_int,
        gamma: f32,
        top_color: c_int,
        bottom_color: c_int,
    );

    unsafe extern "C" fn setup_sky(skybox_textures: *mut c_int);

    unsafe extern "C" fn set_2d_mode(enable: qboolean);

    unsafe extern "C" fn draw_stretch_raw(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        cols: c_int,
        rows: c_int,
        data: *const byte,
        dirty: qboolean,
    );

    unsafe extern "C" fn draw_stretch_pic(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        texture: c_int,
    );

    unsafe extern "C" fn fill_rgba(
        render_mode: c_int,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: byte,
        g: byte,
        b: byte,
        a: byte,
    );

    unsafe extern "C" fn world_to_screen(world: *const vec3_t, screen: *mut vec3_t) -> c_int;

    unsafe extern "C" fn screen_shot(filename: *const c_char, shot_type: c_int) -> qboolean;

    unsafe extern "C" fn cubemap_shot(
        base: *const c_char,
        size: uint,
        vieworg: *const f32,
        skyshot: qboolean,
    ) -> qboolean;

    unsafe extern "C" fn light_point(point: *const f32) -> colorVec;

    unsafe extern "C" fn decal_shoot(
        texture: c_int,
        entity_index: c_int,
        model_index: c_int,
        pos: *mut vec3_t,
        flags: c_int,
        scale: f32,
    );

    unsafe extern "C" fn decal_remove_all(texture: c_int);

    unsafe extern "C" fn create_decal_list(list: *mut decallist_s) -> c_int;

    unsafe extern "C" fn clear_all_decals();

    unsafe extern "C" fn studio_estimate_frame(
        ent: *mut cl_entity_s,
        seq_desc: *mut mstudioseqdesc_t,
        time: f64,
    ) -> f32;

    unsafe extern "C" fn studio_lerp_movement(
        ent: *mut cl_entity_s,
        time: f64,
        origin: *mut vec3_t,
        angles: *mut vec3_t,
    );

    unsafe extern "C" fn init_studio_api();

    unsafe extern "C" fn set_sky_clouds_textures(
        solid_sky_texture: c_int,
        alpha_sky_texture: c_int,
    );

    unsafe extern "C" fn gl_subdivide_surface(model: *mut model_s, fa: *mut msurface_s);

    unsafe extern "C" fn run_light_styles(ls: *mut lightstyle_t);

    unsafe extern "C" fn get_sprite_parms(
        frame_width: *mut c_int,
        frame_height: *mut c_int,
        num_frames: *mut c_int,
        current_frame: c_int,
        model: *const model_s,
    );

    unsafe extern "C" fn get_sprite_texture(model: *const model_s, frame: c_int) -> c_int;

    unsafe extern "C" fn mod_process_render_data(
        model: *mut model_s,
        create: qboolean,
        buffer: *const byte,
    ) -> qboolean;

    unsafe extern "C" fn mod_studio_load_textures(model: *mut model_s, data: *mut c_void);

    unsafe extern "C" fn draw_particles(frame_time: f64, particles: *mut particle_s, partsize: f32);

    unsafe extern "C" fn draw_tracers(frame_time: f64, tracers: *mut particle_s);

    unsafe extern "C" fn draw_beams(trans: c_int, beams: *mut BEAM);

    unsafe extern "C" fn beam_cull(
        start: *const vec3_t,
        end: *const vec3_t,
        pvs_only: qboolean,
    ) -> qboolean;

    unsafe extern "C" fn ref_get_parm(parm: c_int, arg: c_int) -> c_int;

    unsafe extern "C" fn get_detail_scale_for_texture(
        texture: c_int,
        x_scale: *mut f32,
        y_scale: *mut f32,
    );

    unsafe extern "C" fn get_extra_parms_for_texture(
        texture: c_int,
        red: *mut byte,
        green: *mut byte,
        blue: *mut byte,
        alpha: *mut byte,
    );

    unsafe extern "C" fn get_frame_time() -> f32;

    unsafe extern "C" fn set_current_entity(ent: *mut cl_entity_s);

    unsafe extern "C" fn set_current_model(model: *mut model_s);

    unsafe extern "C" fn gl_find_texture(name: *const c_char) -> c_int;

    unsafe extern "C" fn gl_texture_name(texture: c_uint) -> *const c_char;

    unsafe extern "C" fn gl_texture_data(texture: c_uint) -> *const byte;

    unsafe extern "C" fn gl_load_texture(
        name: *const c_char,
        buf: *const byte,
        size: usize,
        flags: c_int,
    ) -> c_int;

    unsafe extern "C" fn gl_create_texture(
        name: *const c_char,
        width: c_int,
        height: c_int,
        buffer: *const c_void,
        flags: texFlags_t,
    ) -> c_int;

    unsafe extern "C" fn gl_load_texture_array(names: *mut *const c_char, flags: c_int) -> c_int;

    unsafe extern "C" fn gl_create_texture_array(
        name: *const c_char,
        width: c_int,
        height: c_int,
        depth: c_int,
        buffer: *const c_void,
        flags: texFlags_t,
    ) -> c_int;

    unsafe extern "C" fn gl_free_texture(texture: c_uint);

    unsafe extern "C" fn override_texture_source_size(
        texture: c_uint,
        src_width: c_uint,
        src_height: c_uint,
    );

    unsafe extern "C" fn draw_single_decal(decal: *mut decal_s, fa: *mut msurface_s);

    unsafe extern "C" fn decal_setup_verts(
        decal: *mut decal_s,
        surf: *mut msurface_s,
        texture: c_int,
        out_count: *mut c_int,
    ) -> *mut f32;

    unsafe extern "C" fn entity_remove_decals(model: *mut model_s);

    unsafe extern "C" fn avi_upload_raw_frame(
        texture: c_int,
        cols: c_int,
        rows: c_int,
        width: c_int,
        height: c_int,
        data: *const byte,
    );

    unsafe extern "C" fn gl_bind(tmu: c_int, texture: c_uint);

    unsafe extern "C" fn gl_select_texture(tmu: c_int);

    unsafe extern "C" fn gl_load_texture_matrix(gl_matrix: *const f32);

    unsafe extern "C" fn gl_tex_matrix_identity();

    unsafe extern "C" fn gl_clean_up_texture_units(last: c_int);

    unsafe extern "C" fn gl_tex_gen(coord: c_uint, mode: c_uint);

    unsafe extern "C" fn gl_texture_target(target: c_uint);

    unsafe extern "C" fn gl_tex_coord_array_mode(mode: c_uint);

    unsafe extern "C" fn gl_update_tex_size(
        texture: c_int,
        width: c_int,
        height: c_int,
        depth: c_int,
    );

    unsafe extern "C" fn gl_draw_particles(
        rvp: *const ref_viewpass_s,
        trans_pass: qboolean,
        frame_time: f32,
    );

    unsafe extern "C" fn light_vec(
        start: *const f32,
        end: *const f32,
        light_spot: *mut f32,
        light_vec: *mut f32,
    ) -> colorVec;

    unsafe extern "C" fn studio_get_texture(ent: *mut cl_entity_s) -> *mut mstudiotex_s;

    unsafe extern "C" fn gl_render_frame(rvp: *const ref_viewpass_s);

    unsafe extern "C" fn gl_ortho_bounds(mins: *const f32, maxs: *const f32);

    unsafe extern "C" fn speeds_message(out: *mut c_char, size: usize) -> qboolean;

    unsafe extern "C" fn mod_get_current_vis() -> *mut byte;

    unsafe extern "C" fn new_map();

    unsafe extern "C" fn clear_scene();

    unsafe extern "C" fn get_proc_address(name: *const c_char) -> *mut c_void;

    unsafe extern "C" fn tri_render_mode(mode: c_int);

    unsafe extern "C" fn begin(primitive_code: c_int);

    unsafe extern "C" fn end();

    unsafe extern "C" fn color4f(r: f32, g: f32, b: f32, a: f32);

    unsafe extern "C" fn color4ub(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar);

    unsafe extern "C" fn tex_coord2f(u: f32, v: f32);

    unsafe extern "C" fn vertex3fv(world_point: *const f32);

    unsafe extern "C" fn vertex3f(x: f32, y: f32, z: f32);

    unsafe extern "C" fn fog(fog_color: *mut [f32; 3], start: f32, end: f32, on: c_int);

    unsafe extern "C" fn screen_to_world(point: *const f32, ret: *mut f32);

    unsafe extern "C" fn get_matrix(pname: c_int, matrix: *mut f32);

    unsafe extern "C" fn fog_params(density: f32, fog_skybox: c_int);

    unsafe extern "C" fn cull_face(mode: TRICULLSTYLE);

    unsafe extern "C" fn vgui_setup_drawing(rect: qboolean);

    unsafe extern "C" fn vgui_upload_texture_block(
        draw_x: c_int,
        draw_y: c_int,
        rgba: *const byte,
        block_width: c_int,
        block_height: c_int,
    );
}

struct Export<T> {
    phantom: PhantomData<T>,
}

fn texture_name<'a>(name: *const c_char) -> Option<&'a CStrThin> {
    let name = unsafe { cstr_or_none(name) }?;
    if !name.is_empty() {
        Some(name)
    } else {
        None
    }
}

impl<T: RefDll> RefDllExport for Export<T> {
    unsafe extern "C" fn init() -> qboolean {
        match T::new() {
            Some(instance) => unsafe {
                (&mut *T::global_as_mut_ptr()).write(instance);
                1
            },
            None => 0,
        }
    }

    unsafe extern "C" fn shutdown() {
        unsafe {
            (&mut *T::global_as_mut_ptr()).assume_init_drop();
        }
    }

    unsafe extern "C" fn get_config_name() -> *const c_char {
        match unsafe { T::global_assume_init_ref() }.get_config_name() {
            Some(name) => name.as_ptr(),
            None => ptr::null(),
        }
    }

    unsafe extern "C" fn set_display_transform(
        rotate: ref_screen_rotation_t,
        x: c_int,
        y: c_int,
        scale_x: f32,
        scale_y: f32,
    ) -> qboolean {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.set_display_transform(rotate, x, y, scale_x, scale_y)
            .into()
    }

    unsafe extern "C" fn gl_setup_attributes(safe_gl: c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_setup_attributes(safe_gl != 0);
    }

    unsafe extern "C" fn gl_init_extensions() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_init_extensions();
    }

    unsafe extern "C" fn gl_clear_extensions() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_clear_extensions();
    }

    unsafe extern "C" fn gamma_changed(do_reset_gamma: qboolean) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gamma_changed(do_reset_gamma != 0);
    }

    unsafe extern "C" fn begin_frame(clear_scene: qboolean) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.begin_frame(clear_scene != 0);
    }

    unsafe extern "C" fn render_scene() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.render_scene();
    }

    unsafe extern "C" fn end_frame() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.end_frame();
    }

    unsafe extern "C" fn push_scene() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.push_scene();
    }

    unsafe extern "C" fn pop_scene() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.pop_scene();
    }

    unsafe extern "C" fn gl_backend_start_frame() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_backend_start_frame();
    }

    unsafe extern "C" fn gl_backend_end_frame() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_backend_end_frame();
    }

    unsafe extern "C" fn clear_screen() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.clear_screen();
    }

    unsafe extern "C" fn allow_fog(allow: qboolean) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.allow_fog(allow != 0);
    }

    unsafe extern "C" fn gl_set_render_mode(render_mode: c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_set_render_mode(render_mode);
    }

    unsafe extern "C" fn add_entity(ent: *mut cl_entity_s, type_: c_int) -> qboolean {
        if let Some(ent) = unsafe { ent.as_mut() } {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.add_entity(ent, type_).into()
        } else {
            0
        }
    }

    unsafe extern "C" fn add_custom_beam(env_beam: *mut cl_entity_s) {
        let env_beam = unsafe { env_beam.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.add_custom_beam(env_beam);
    }

    unsafe extern "C" fn process_entity_data(
        allocate: qboolean,
        entities: *mut cl_entity_s,
        max_entities: c_uint,
    ) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.process_entity_data(allocate != 0, entities, max_entities as usize);
    }

    unsafe extern "C" fn flush(flush_flags: c_uint) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.flush(flush_flags);
    }

    unsafe extern "C" fn show_textures() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.show_textures();
    }

    unsafe extern "C" fn get_texture_original_buffer(texture: c_uint) -> *const byte {
        if let Some(texture) = TextureId::new(texture as c_int) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.get_texture_original_buffer(texture)
        } else {
            ptr::null()
        }
    }

    unsafe extern "C" fn gl_load_texture_from_buffer(
        name: *const c_char,
        pic: *mut rgbdata_t,
        flags: texFlags_t,
        update: qboolean,
    ) -> c_int {
        let name = unsafe { cstr_or_none(name).unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        let flags = TextureFlags::from_bits_retain(flags);
        let res = dll.gl_load_texture_from_buffer(name, pic, flags, update != 0);
        TextureId::to_ffi(res)
    }

    unsafe extern "C" fn gl_process_texture(
        texture: c_int,
        gamma: f32,
        top_color: c_int,
        bottom_color: c_int,
    ) {
        if let Some(texture) = TextureId::new(texture) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.gl_process_texture(texture, gamma, top_color, bottom_color);
        }
    }

    unsafe extern "C" fn setup_sky(skybox_textures: *mut c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.unload_skybox();
        let skybox_textures = skybox_textures.cast::<[Option<TextureId>; SKYBOX_MAX_SIDES]>();
        if let Some(skybox_textures) = unsafe { skybox_textures.as_ref() } {
            dll.setup_skybox(skybox_textures);
        }
    }

    unsafe extern "C" fn set_2d_mode(enable: qboolean) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.set_2d_mode(enable != 0);
    }

    unsafe extern "C" fn draw_stretch_raw(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        cols: c_int,
        rows: c_int,
        data: *const byte,
        dirty: qboolean,
    ) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.draw_stretch_raw(x, y, w, h, cols, rows, data, dirty != 0);
    }

    unsafe extern "C" fn draw_stretch_pic(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        s1: f32,
        t1: f32,
        s2: f32,
        t2: f32,
        texture: c_int,
    ) {
        if let Some(texture) = TextureId::new(texture) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.draw_stretch_pic(x, y, w, h, s1, t1, s2, t2, texture);
        }
    }

    unsafe extern "C" fn fill_rgba(
        render_mode: c_int,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: byte,
        g: byte,
        b: byte,
        a: byte,
    ) {
        let color = RGBA::new(r, g, b, a);
        let dll = unsafe { T::global_assume_init_ref() };
        dll.fill_rgba(render_mode, x, y, w, h, color);
    }

    unsafe extern "C" fn world_to_screen(world: *const vec3_t, ret: *mut vec3_t) -> c_int {
        if world.is_null() || ret.is_null() {
            return 1;
        }
        let world = unsafe { *world };
        let ret = unsafe { &mut *ret };
        let dll = unsafe { T::global_assume_init_ref() };
        match dll.world_to_screen(world) {
            Ok(point) => {
                *ret = point;
                0
            }
            Err(point) => {
                *ret = point;
                1
            }
        }
    }

    unsafe extern "C" fn screen_shot(filename: *const c_char, shot_type: c_int) -> qboolean {
        let filename = unsafe { cstr_or_none(filename).unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.screen_shot(filename, shot_type).into()
    }

    unsafe extern "C" fn cubemap_shot(
        base: *const c_char,
        size: uint,
        vieworg: *const f32,
        skyshot: qboolean,
    ) -> qboolean {
        let base = unsafe { cstr_or_none(base).unwrap() };
        let vieworg = unsafe { vieworg.cast::<vec3_t>().as_ref() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.cubemap_shot(base, size, vieworg, skyshot != 0).into()
    }

    unsafe extern "C" fn light_point(point: *const f32) -> colorVec {
        let point = unsafe { point.cast::<vec3_t>().as_ref().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.light_point(*point).into()
    }

    unsafe extern "C" fn decal_shoot(
        texture: c_int,
        entity_index: c_int,
        model_index: c_int,
        pos: *mut vec3_t,
        flags: c_int,
        scale: f32,
    ) {
        if let Some(texture) = TextureId::new(texture) {
            let pos = unsafe { pos.as_mut().unwrap() };
            let dll = unsafe { T::global_assume_init_ref() };
            dll.decal_shoot(texture, entity_index, model_index, pos, flags, scale);
        }
    }

    unsafe extern "C" fn decal_remove_all(texture: c_int) {
        if let Some(texture) = TextureId::new(texture) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.decal_remove_all(texture);
        }
    }

    unsafe extern "C" fn create_decal_list(list: *mut decallist_s) -> c_int {
        assert!(!list.is_null());
        let list = unsafe { slice::from_raw_parts_mut(list, MAX_RENDER_DECALS * 2) };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.create_decal_list(list) as c_int
    }

    unsafe extern "C" fn clear_all_decals() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.clear_all_decals();
    }

    unsafe extern "C" fn studio_estimate_frame(
        ent: *mut cl_entity_s,
        seq_desc: *mut mstudioseqdesc_t,
        time: f64,
    ) -> f32 {
        let ent = unsafe { ent.as_mut().unwrap() };
        let seq_desc = unsafe { seq_desc.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.studio_estimate_frame(ent, seq_desc, time)
    }

    unsafe extern "C" fn studio_lerp_movement(
        ent: *mut cl_entity_s,
        time: f64,
        origin: *mut vec3_t,
        angles: *mut vec3_t,
    ) {
        let ent = unsafe { ent.as_mut().unwrap() };
        let origin = unsafe { origin.as_mut().unwrap() };
        let angles = unsafe { angles.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.studio_lerp_movement(ent, time, origin, angles);
    }

    unsafe extern "C" fn init_studio_api() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.init_studio_api();
    }

    unsafe extern "C" fn set_sky_clouds_textures(
        solid_sky_texture: c_int,
        alpha_sky_texture: c_int,
    ) {
        let solid_sky_texture = TextureId::new(solid_sky_texture);
        let alpha_sky_texture = TextureId::new(alpha_sky_texture);
        let dll = unsafe { T::global_assume_init_ref() };
        dll.set_sky_clouds_textures(solid_sky_texture, alpha_sky_texture);
    }

    unsafe extern "C" fn gl_subdivide_surface(model: *mut model_s, fa: *mut msurface_s) {
        let model = unsafe { model.as_mut().unwrap() };
        let fa = unsafe { fa.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_subdivide_surface(model, fa);
    }

    unsafe extern "C" fn run_light_styles(ls: *mut lightstyle_t) {
        assert!(!ls.is_null());
        let ls = unsafe { slice::from_raw_parts_mut(ls, MAX_LIGHTSTYLES) };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.run_light_styles(ls);
    }

    unsafe extern "C" fn get_sprite_parms(
        frame_width: *mut c_int,
        frame_height: *mut c_int,
        num_frames: *mut c_int,
        current_frame: c_int,
        model: *const model_s,
    ) {
        if let Some(model) = unsafe { model.as_ref() } {
            let frame_width = unsafe { frame_width.as_mut() };
            let frame_height = unsafe { frame_height.as_mut() };
            let num_frames = unsafe { num_frames.as_mut() };
            let dll = unsafe { T::global_assume_init_ref() };
            dll.get_sprite_parms(frame_width, frame_height, num_frames, current_frame, model);
        }
    }

    unsafe extern "C" fn get_sprite_texture(model: *const model_s, frame: c_int) -> c_int {
        if let Some(model) = unsafe { model.as_ref() } {
            let dll = unsafe { T::global_assume_init_ref() };
            let res = dll.get_sprite_texture(model, frame);
            TextureId::to_ffi(res)
        } else {
            0
        }
    }

    unsafe extern "C" fn mod_process_render_data(
        model: *mut model_s,
        create: qboolean,
        buffer: *const byte,
    ) -> qboolean {
        let model = unsafe { model.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.mod_process_render_data(model, create != 0, buffer)
            .into()
    }

    unsafe extern "C" fn mod_studio_load_textures(model: *mut model_s, data: *mut c_void) {
        let Some(data) = NonNull::new(data) else {
            return;
        };
        let model = unsafe { model.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.mod_studio_load_textures(model, data);
    }

    unsafe extern "C" fn draw_particles(
        frame_time: f64,
        particles: *mut particle_s,
        partsize: f32,
    ) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.draw_particles(frame_time, particles, partsize);
    }

    unsafe extern "C" fn draw_tracers(frame_time: f64, tracers: *mut particle_s) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.draw_tracers(frame_time, tracers);
    }

    unsafe extern "C" fn draw_beams(trans: c_int, beams: *mut BEAM) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.draw_beams(trans != 0, beams);
    }

    unsafe extern "C" fn beam_cull(
        start: *const vec3_t,
        end: *const vec3_t,
        pvs_only: qboolean,
    ) -> qboolean {
        let start = unsafe { start.as_ref().unwrap() };
        let end = unsafe { end.as_ref().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.beam_cull(start, end, pvs_only != 0).into()
    }

    unsafe extern "C" fn ref_get_parm(parm: c_int, arg: c_int) -> c_int {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.ref_get_parm(RefParm::new(parm), arg)
    }

    unsafe extern "C" fn get_detail_scale_for_texture(
        texture: c_int,
        x_scale: *mut f32,
        y_scale: *mut f32,
    ) {
        let (x, y) = if let Some(texture) = TextureId::new(texture) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.get_detail_scale_for_texture(texture)
        } else {
            (1.0, 1.0)
        };
        if let Some(x_scale) = unsafe { x_scale.as_mut() } {
            *x_scale = x;
        }
        if let Some(y_scale) = unsafe { y_scale.as_mut() } {
            *y_scale = y;
        }
    }

    unsafe extern "C" fn get_extra_parms_for_texture(
        texture: c_int,
        red: *mut byte,
        green: *mut byte,
        blue: *mut byte,
        alpha: *mut byte,
    ) {
        let color = if let Some(texture) = TextureId::new(texture) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.get_extra_parms_for_texture(texture)
        } else {
            RGBA::splat(0)
        };
        if let Some(r) = unsafe { red.as_mut() } {
            *r = color.r();
        }
        if let Some(g) = unsafe { green.as_mut() } {
            *g = color.g();
        }
        if let Some(b) = unsafe { blue.as_mut() } {
            *b = color.b();
        }
        if let Some(a) = unsafe { alpha.as_mut() } {
            *a = color.a();
        }
    }

    unsafe extern "C" fn get_frame_time() -> f32 {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.get_frame_time()
    }

    unsafe extern "C" fn set_current_entity(ent: *mut cl_entity_s) {
        let ent = unsafe { ent.as_mut() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.set_current_entity(ent);
    }

    unsafe extern "C" fn set_current_model(model: *mut model_s) {
        let model = unsafe { model.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.set_current_model(model);
    }

    unsafe extern "C" fn gl_find_texture(name: *const c_char) -> c_int {
        let Some(name) = texture_name(name) else {
            return 0;
        };
        let dll = unsafe { T::global_assume_init_ref() };
        let res = dll.gl_find_texture(name);
        TextureId::to_ffi(res)
    }

    unsafe extern "C" fn gl_texture_name(texture: c_uint) -> *const c_char {
        if let Some(texture) = TextureId::new(texture as c_int) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.gl_texture_name(texture)
        } else {
            UNUSED_TEXTURE_NAME.as_ptr()
        }
    }

    unsafe extern "C" fn gl_texture_data(texture: c_uint) -> *const byte {
        if let Some(texture) = TextureId::new(texture as c_int) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.gl_texture_data(texture)
        } else {
            ptr::null()
        }
    }

    unsafe extern "C" fn gl_load_texture(
        name: *const c_char,
        buf: *const byte,
        size: usize,
        flags: c_int,
    ) -> c_int {
        let Some(name) = texture_name(name) else {
            return 0;
        };
        let buf = unsafe { slice_from_raw_parts_or_empty(buf, size) };
        let dll = unsafe { T::global_assume_init_ref() };
        let res = dll.gl_load_texture(name, buf, flags);
        TextureId::to_ffi(res)
    }

    unsafe extern "C" fn gl_create_texture(
        name: *const c_char,
        width: c_int,
        height: c_int,
        buffer: *const c_void,
        flags: texFlags_t,
    ) -> c_int {
        assert!(!buffer.is_null());
        let Some(name) = texture_name(name) else {
            return 0;
        };
        let len = width as usize * height as usize;
        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), len) };
        let flags = TextureFlags::from_bits_retain(flags);
        let dll = unsafe { T::global_assume_init_ref() };
        let res = dll.gl_create_texture(name, width, height, buffer, flags);
        TextureId::to_ffi(res)
    }

    unsafe extern "C" fn gl_load_texture_array(names: *mut *const c_char, flags: c_int) -> c_int {
        let dll = unsafe { T::global_assume_init_ref() };
        let res = dll.gl_load_texture_array(names, flags);
        TextureId::to_ffi(res)
    }

    unsafe extern "C" fn gl_create_texture_array(
        name: *const c_char,
        width: c_int,
        height: c_int,
        depth: c_int,
        buffer: *const c_void,
        flags: texFlags_t,
    ) -> c_int {
        let Some(name) = texture_name(name) else {
            return 0;
        };
        let flags = TextureFlags::from_bits_retain(flags);
        let dll = unsafe { T::global_assume_init_ref() };
        let res = dll.gl_create_texture_array(name, width, height, depth, buffer, flags);
        TextureId::to_ffi(res)
    }

    unsafe extern "C" fn gl_free_texture(texture: c_uint) {
        if let Some(texture) = TextureId::new(texture as c_int) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.gl_free_texture(texture);
        }
    }

    unsafe extern "C" fn override_texture_source_size(
        texture: c_uint,
        src_width: c_uint,
        src_height: c_uint,
    ) {
        if let Some(texture) = TextureId::new(texture as c_int) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.override_texture_source_size(texture, src_width, src_height);
        }
    }

    unsafe extern "C" fn draw_single_decal(decal: *mut decal_s, fa: *mut msurface_s) {
        let decal = unsafe { decal.as_mut().unwrap() };
        let fa = unsafe { fa.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.draw_single_decal(decal, fa);
    }

    unsafe extern "C" fn decal_setup_verts(
        decal: *mut decal_s,
        surf: *mut msurface_s,
        texture: c_int,
        out_count: *mut c_int,
    ) -> *mut f32 {
        let decal = unsafe { decal.as_mut().unwrap() };
        let surf = unsafe { surf.as_mut().unwrap() };
        let out_count = unsafe { out_count.as_mut() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.decal_setup_verts(decal, surf, texture, out_count)
    }

    unsafe extern "C" fn entity_remove_decals(model: *mut model_s) {
        if let Some(model) = unsafe { model.as_mut() } {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.entity_remove_decals(model);
        }
    }

    unsafe extern "C" fn avi_upload_raw_frame(
        texture: c_int,
        cols: c_int,
        rows: c_int,
        width: c_int,
        height: c_int,
        data: *const byte,
    ) {
        if let Some(texture) = TextureId::new(texture) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.avi_upload_raw_frame(texture, cols, rows, width, height, data);
        }
    }

    unsafe extern "C" fn gl_bind(tmu: c_int, texture: c_uint) {
        let texture = TextureId::new(texture as c_int);
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_bind(tmu, texture);
    }

    unsafe extern "C" fn gl_select_texture(tmu: c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_select_texture(tmu);
    }

    unsafe extern "C" fn gl_load_texture_matrix(gl_matrix: *const f32) {
        assert!(!gl_matrix.is_null());
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_load_texture_matrix(gl_matrix);
    }

    unsafe extern "C" fn gl_tex_matrix_identity() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_tex_matrix_identity();
    }

    unsafe extern "C" fn gl_clean_up_texture_units(last: c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_clean_up_texture_units(last);
    }

    unsafe extern "C" fn gl_tex_gen(coord: c_uint, mode: c_uint) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_tex_gen(coord, mode);
    }

    unsafe extern "C" fn gl_texture_target(target: c_uint) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_texture_target(target);
    }

    unsafe extern "C" fn gl_tex_coord_array_mode(mode: c_uint) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_tex_coord_array_mode(mode);
    }

    unsafe extern "C" fn gl_update_tex_size(
        texture: c_int,
        width: c_int,
        height: c_int,
        depth: c_int,
    ) {
        if let Some(texture) = TextureId::new(texture) {
            let dll = unsafe { T::global_assume_init_ref() };
            dll.gl_update_tex_size(texture, width, height, depth);
        }
    }

    unsafe extern "C" fn gl_draw_particles(
        rvp: *const ref_viewpass_s,
        trans_pass: qboolean,
        frame_time: f32,
    ) {
        let rvp = unsafe { rvp.as_ref().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_draw_particles(rvp, trans_pass != 0, frame_time);
    }

    unsafe extern "C" fn light_vec(
        start: *const f32,
        end: *const f32,
        light_spot: *mut f32,
        light_vec: *mut f32,
    ) -> colorVec {
        let start = unsafe { *start.cast::<vec3_t>().as_ref().unwrap() };
        let end = unsafe { *end.cast::<vec3_t>().as_ref().unwrap() };
        let light_spot = unsafe { light_spot.cast::<vec3_t>().as_mut() };
        let light_vec = unsafe { light_vec.cast::<vec3_t>().as_mut() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.light_vec(start, end, light_spot, light_vec).into()
    }

    unsafe extern "C" fn studio_get_texture(ent: *mut cl_entity_s) -> *mut mstudiotex_s {
        let ent = unsafe { ent.as_mut().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.studio_get_texture(ent)
    }

    unsafe extern "C" fn gl_render_frame(rvp: *const ref_viewpass_s) {
        let rvp = unsafe { rvp.as_ref().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_render_frame(rvp);
    }

    unsafe extern "C" fn gl_ortho_bounds(mins: *const f32, maxs: *const f32) {
        let mins = unsafe { *mins.cast::<vec2_t>().as_ref().unwrap() };
        let maxs = unsafe { *maxs.cast::<vec2_t>().as_ref().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.gl_ortho_bounds(mins, maxs);
    }

    unsafe extern "C" fn speeds_message(out: *mut c_char, size: usize) -> qboolean {
        if out.is_null() || size == 0 {
            return 0;
        }
        let out = unsafe { slice::from_raw_parts_mut(out.cast(), size) };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.speeds_message(CStrSlice::new_in(out)).into()
    }

    unsafe extern "C" fn mod_get_current_vis() -> *mut byte {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.mod_get_current_vis()
    }

    unsafe extern "C" fn new_map() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.new_map();
    }

    unsafe extern "C" fn clear_scene() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.clear_scene();
    }

    unsafe extern "C" fn get_proc_address(name: *const c_char) -> *mut c_void {
        let Some(name) = (unsafe { cstr_or_none(name) }) else {
            return ptr::null_mut();
        };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.get_proc_address(name)
    }

    unsafe extern "C" fn tri_render_mode(mode: c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.tri_render_mode(mode);
    }

    unsafe extern "C" fn begin(primitive_code: c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.begin(primitive_code);
    }

    unsafe extern "C" fn end() {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.end();
    }

    unsafe extern "C" fn color4f(r: f32, g: f32, b: f32, a: f32) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.color4f(r, g, b, a);
    }

    unsafe extern "C" fn color4ub(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.color4ub(RGBA::new(r, g, b, a));
    }

    unsafe extern "C" fn tex_coord2f(u: f32, v: f32) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.tex_coord2f(u, v);
    }

    unsafe extern "C" fn vertex3fv(world_point: *const f32) {
        let world_point = unsafe { world_point.cast::<vec3_t>().as_ref().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.vertex3fv(world_point);
    }

    unsafe extern "C" fn vertex3f(x: f32, y: f32, z: f32) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.vertex3f(x, y, z);
    }

    unsafe extern "C" fn fog(fog_color: *mut [f32; 3], start: f32, end: f32, on: c_int) {
        let fog_color = unsafe { fog_color.as_ref().unwrap() };
        let dll = unsafe { T::global_assume_init_ref() };
        dll.fog(fog_color, start, end, on != 0);
    }

    unsafe extern "C" fn screen_to_world(point: *const f32, ret: *mut f32) {
        let Some(point) = (unsafe { point.cast::<vec3_t>().as_ref() }) else {
            return;
        };
        let Some(ret) = (unsafe { ret.cast::<vec3_t>().as_mut() }) else {
            return;
        };
        let dll = unsafe { T::global_assume_init_ref() };
        *ret = dll.screen_to_world(*point);
    }

    unsafe extern "C" fn get_matrix(pname: c_int, matrix: *mut f32) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.get_matrix(pname, matrix);
    }

    unsafe extern "C" fn fog_params(density: f32, fog_skybox: c_int) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.fog_params(density, fog_skybox);
    }

    unsafe extern "C" fn cull_face(mode: TRICULLSTYLE) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.cull_face(mode);
    }

    unsafe extern "C" fn vgui_setup_drawing(rect: qboolean) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.vgui_setup_drawing(rect != 0);
    }

    unsafe extern "C" fn vgui_upload_texture_block(
        draw_x: c_int,
        draw_y: c_int,
        rgba: *const byte,
        block_width: c_int,
        block_height: c_int,
    ) {
        let dll = unsafe { T::global_assume_init_ref() };
        dll.vgui_upload_texture_block(draw_x, draw_y, rgba, block_width, block_height);
    }
}

/// Initialize the global engine instance and returns exported functions.
///
/// # Safety
///
/// Must be called only once.
pub unsafe fn get_ref_api<T: RefDll>(
    version: c_int,
    dll_funcs: Option<&mut ref_interface_s>,
    eng_funcs: Option<&ref_api_s>,
    globals: *mut ref_globals_s,
) -> c_int {
    if version != REF_API_VERSION as c_int {
        return 0;
    }
    let Some(dll_funcs) = dll_funcs else { return 0 };
    let Some(eng_funcs) = eng_funcs else { return 0 };
    unsafe {
        crate::instance::init_engine(eng_funcs, globals);
    }
    *dll_funcs = ref_functions::<T>();
    REF_API_VERSION as c_int
}

#[doc(hidden)]
#[macro_export]
macro_rules! export_dll {
    ($ref_dll:ty $($init:block)?) => {
        #[no_mangle]
        unsafe extern "C" fn GetRefAPI(
            version: core::ffi::c_int,
            dll_funcs: Option<&mut $crate::ffi::render::ref_interface_t>,
            eng_funcs: Option<&$crate::ffi::render::ref_api_t>,
            globals: *mut $crate::ffi::render::ref_globals_t,
        ) -> core::ffi::c_int {
            let result = unsafe {
                $crate::export::get_ref_api::<$ref_dll>(version, dll_funcs, eng_funcs, globals)
            };
            if result != 0 {
                $($init)?
            }
            result
        }
    };
}
#[doc(inline)]
pub use export_dll;
