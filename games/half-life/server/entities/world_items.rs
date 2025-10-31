use core::ffi::CStr;

use xash3d_server::{
    entities::world_items::{WorldItems, WorldItemsNames},
    export::export_entity,
    prelude::*,
    save::{Restore, Save},
};

#[derive(Save, Restore)]
struct Names;

impl WorldItemsNames for Names {
    fn create(_: ServerEngineRef) -> Self {
        Self
    }

    fn get_class_name(&self, ty: u16) -> Option<&CStr> {
        let name = match ty {
            42 => c"item_antidote",
            43 => c"item_security",
            44 => c"item_battery",
            45 => c"item_suit",
            _ => return None,
        };
        Some(name)
    }
}

export_entity!(world_items, WorldItems<Names>);
