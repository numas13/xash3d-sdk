use core::ffi::c_int;

use xash3d_client::{ffi::common::entity_state_s, prelude::*};

pub struct StudioRenderer {}

impl StudioRenderer {
    pub fn new(_: ClientEngineRef) -> Self {
        Self {}
    }

    pub fn draw_player(&self, _flags: c_int, _player: &mut entity_state_s) -> c_int {
        // TODO:
        0
    }

    pub fn draw_model(&self, _flags: c_int) -> c_int {
        // TODO:
        0
    }
}
