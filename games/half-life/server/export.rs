use core::ffi::c_int;

use xash3d_server::{
    export::{export_dll, impl_unsync_global, ServerDll},
    global_state::GlobalStateRef,
    prelude::*,
};

struct Dll {
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
}

impl_unsync_global!(Dll);

impl ServerDll for Dll {
    fn new(engine: ServerEngineRef, global_state: GlobalStateRef) -> Self {
        crate::cvar::init(engine);
        Self {
            engine,
            global_state,
        }
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }
}

export_dll!(Dll);
