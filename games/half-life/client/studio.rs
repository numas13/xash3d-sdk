use core::ffi::c_int;

use cl::ffi::common::entity_state_s;

pub struct StudioRenderer {}

impl Default for StudioRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl StudioRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw_player(&mut self, _flags: c_int, _player: &mut entity_state_s) -> c_int {
        // TODO:
        0
    }

    pub fn draw_model(&mut self, _flags: c_int) -> c_int {
        // TODO:
        0
    }
}
