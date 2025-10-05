#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(func_breakable, Private<StubEntity>);
    export_entity!(func_pushable, Private<StubEntity>);
}
