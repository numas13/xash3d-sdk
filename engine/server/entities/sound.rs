#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(ambient_generic, Private<StubEntity>);
    export_entity!(env_sound, Private<StubEntity>);
    export_entity!(speaker, Private<StubEntity>);
}
