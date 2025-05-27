use core::{cell::RefCell, ffi::c_int};

use cl::{cell::SyncOnceCell, raw::entity_state_s};

pub struct StudioRenderer {}

impl StudioRenderer {
    fn new() -> Self {
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

impl Default for StudioRenderer {
    fn default() -> Self {
        Self::new()
    }
}

static RENDERER: SyncOnceCell<RefCell<StudioRenderer>> = unsafe { SyncOnceCell::new() };

fn renderer_global() -> &'static RefCell<StudioRenderer> {
    RENDERER.get_or_init(|| RefCell::new(StudioRenderer::new()))
}

pub fn renderer_mut<'a>() -> core::cell::RefMut<'a, StudioRenderer> {
    renderer_global().borrow_mut()
}

pub fn init() {
    renderer_global();
}
