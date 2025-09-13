use core::ffi::{c_char, c_int};

use shared::{
    ffi::{
        api::render::{decallist_s, render_interface_s},
        common::{cl_entity_s, model_s, vec2_t},
    },
    render::ViewPass,
};

pub enum Renderer {
    Engine,
    Client,
}

pub struct Draw<'a> {
    raw: &'a render_interface_s,
}

impl Draw<'_> {
    pub(crate) fn new(raw: &render_interface_s) -> Draw<'_> {
        Draw { raw }
    }

    pub fn version(&self) -> c_int {
        self.raw.version
    }

    pub fn gl_render_frame(&self, rvp: &ViewPass) -> Option<Renderer> {
        self.raw
            .GL_RenderFrame
            .map(|f| match unsafe { f(rvp.as_ref()) } {
                0 => Renderer::Engine,
                1 => Renderer::Client,
                n => {
                    error!("expected GL_RenderFrame result {n}");
                    Renderer::Engine
                }
            })
    }

    pub fn gl_build_lightmaps(&self) -> Option<()> {
        self.raw.GL_BuildLightmaps.map(|f| unsafe { f() })
    }

    pub fn gl_ortho_bounds(&self, mins: vec2_t, maxs: vec2_t) -> Option<()> {
        self.raw
            .GL_OrthoBounds
            .map(|f| unsafe { f(mins.as_ptr(), maxs.as_ptr()) })
    }

    pub fn r_create_studio_decal_list(&self, list: &mut [decallist_s]) -> Option<usize> {
        self.raw
            .R_CreateStudioDecalList
            .map(|f| unsafe { f(list.as_mut_ptr(), list.len() as c_int) as usize })
    }

    pub fn r_clear_studio_decals(&self) -> Option<()> {
        self.raw.R_ClearStudioDecals.map(|f| unsafe { f() })
    }

    pub fn r_speeds_message(&self, out: &mut [c_char]) -> Option<bool> {
        self.raw
            .R_SpeedsMessage
            .map(|f| unsafe { f(out.as_mut_ptr(), out.len()) != 0 })
    }

    // XXX: temporary silence clippy
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn mod_process_user_data(
        &self,
        model: &mut model_s,
        create: bool,
        buffer: *const u8,
    ) -> Option<()> {
        self.raw
            .Mod_ProcessUserData
            .map(|f| unsafe { f(model, create.into(), buffer) })
    }

    pub fn r_process_ent_data(&self, allocate: bool) -> Option<()> {
        self.raw
            .R_ProcessEntData
            .map(|f| unsafe { f(allocate.into()) })
    }

    pub fn mod_get_current_vis(&self) -> Option<*mut u8> {
        self.raw.Mod_GetCurrentVis.map(|f| unsafe { f() })
    }

    pub fn r_new_map(&self) -> Option<()> {
        self.raw.R_NewMap.map(|f| unsafe { f() })
    }

    pub fn r_clear_scene(&self) -> Option<()> {
        self.raw.R_ClearScene.map(|f| unsafe { f() })
    }

    pub fn cl_update_latched_vars(&self, ent: &mut cl_entity_s, reset: bool) -> Option<()> {
        self.raw
            .CL_UpdateLatchedVars
            .map(|f| unsafe { f(ent, reset.into()) })
    }
}
