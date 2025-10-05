#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(func_door, Private<StubEntity>);
    // func_water is the same as a door.
    export_entity!(func_water, Private<StubEntity>);
    export_entity!(func_door_rotating, Private<StubEntity>);
}
