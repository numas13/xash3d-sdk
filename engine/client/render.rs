use shared::ffi::common::{movevars_s, ref_params_s, usercmd_s};

pub use shared::render::*;

pub trait RefParamsExt {
    fn movevars(&self) -> &movevars_s;

    fn cmd(&self) -> &usercmd_s;
}

impl RefParamsExt for ref_params_s {
    fn movevars(&self) -> &movevars_s {
        unsafe { &*self.movevars }
    }

    fn cmd(&self) -> &usercmd_s {
        unsafe { &*self.cmd }
    }
}
