use core::{
    ffi::{c_char, c_int, c_void},
    fmt,
    ops::{Deref, DerefMut},
    slice,
};

use crate::{
    prelude::*,
    raw::{self, vec2_t},
};

pub enum Renderer {
    Engine,
    Client,
}

pub struct Draw<'a> {
    raw: &'a raw::render_interface_t,
}

impl Draw<'_> {
    pub(crate) fn new(raw: &raw::render_interface_t) -> Draw<'_> {
        Draw { raw }
    }

    pub fn version(&self) -> c_int {
        self.raw.version
    }

    pub fn gl_render_frame(&self, rvp: &raw::ref_viewpass_s) -> Option<Renderer> {
        self.raw.GL_RenderFrame.map(|f| match unsafe { f(rvp) } {
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

    pub fn r_create_studio_decal_list(&self, list: &mut [raw::decallist_s]) -> Option<usize> {
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
            .map(|f| unsafe { f(out.as_mut_ptr(), out.len()).to_bool() })
    }

    // XXX: temporary silence clippy
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn mod_process_user_data(
        &self,
        model: &mut raw::model_s,
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

    pub fn cl_update_latched_vars(&self, ent: &mut raw::cl_entity_s, reset: bool) -> Option<()> {
        self.raw
            .CL_UpdateLatchedVars
            .map(|f| unsafe { f(ent, reset.into()) })
    }
}

#[derive(Default)]
pub struct SwBuffer {
    pub(crate) width: c_int,
    pub(crate) height: c_int,
    pub(crate) stride: u32,
    pub(crate) bpp: u32,
    pub(crate) r_mask: u32,
    pub(crate) g_mask: u32,
    pub(crate) b_mask: u32,
}

impl SwBuffer {
    pub fn width(&self) -> usize {
        self.width as usize
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }

    pub fn stride(&self) -> usize {
        self.stride as usize
    }

    pub fn bpp(&self) -> usize {
        self.bpp as usize
    }

    pub fn r_mask(&self) -> u32 {
        self.r_mask
    }

    pub fn g_mask(&self) -> u32 {
        self.g_mask
    }

    pub fn b_mask(&self) -> u32 {
        self.b_mask
    }

    pub fn stride_bytes(&self) -> usize {
        self.stride() * self.bpp()
    }

    pub fn row_bytes(&self) -> usize {
        self.width() * self.bpp()
    }

    pub fn len_bytes(&self) -> usize {
        self.stride_bytes() * self.height()
    }

    pub fn is_empty(&self) -> bool {
        self.stride == 0 || self.width == 0 || self.height == 0
    }

    pub fn len(&self) -> usize {
        self.stride() * self.height()
    }

    pub fn lock(&mut self, width: c_int, height: c_int) -> Option<SwBufferLock<'_>> {
        let engine = engine();
        let data = unsafe { engine.sw_lock_buffer() };
        if !data.is_null() && width == self.width && height == self.height {
            Some(SwBufferLock { buf: self, data })
        } else {
            None
        }
    }
}

pub struct SwBufferLock<'a> {
    buf: &'a mut SwBuffer,
    data: *mut c_void,
}

impl SwBufferLock<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.cast(), self.len_bytes()) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data.cast(), self.len_bytes()) }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.cast()
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.data.cast()
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        let stride = self.stride_bytes();
        let row_len = self.row_bytes();
        self.as_bytes_mut()
            .chunks_exact_mut(stride)
            .map(move |row| &mut row[..row_len])
    }
}

impl Deref for SwBufferLock<'_> {
    type Target = SwBuffer;

    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

impl DerefMut for SwBufferLock<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf
    }
}

impl Drop for SwBufferLock<'_> {
    fn drop(&mut self) {
        unsafe {
            engine().sw_unlock_buffer();
        }
    }
}

pub struct RgbData {
    pub(crate) raw: *mut raw::rgbdata_t,
}

impl Clone for RgbData {
    fn clone(&self) -> Self {
        let raw = unsafe { engine().fs_copy_image(self.raw) };
        assert!(!raw.is_null());
        Self { raw }
    }
}

impl Drop for RgbData {
    fn drop(&mut self) {
        unsafe {
            engine().fs_free_image(self.raw);
        }
    }
}

impl Deref for RgbData {
    type Target = raw::rgbdata_t;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw }
    }
}

impl DerefMut for RgbData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw }
    }
}

pub struct SaveImageError(pub(crate) ());

impl fmt::Display for SaveImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("failed to save an image")
    }
}

impl fmt::Debug for SaveImageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SaveImageError").finish()
    }
}

pub struct FatPvsError(pub(crate) ());

impl fmt::Display for FatPvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("The buffer size must be greater or equal to {MAX_MAP_LEAFS_BYTES}")
    }
}

impl fmt::Debug for FatPvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FatPvsError").finish()
    }
}
