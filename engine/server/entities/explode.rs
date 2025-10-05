#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(spark_shower, Private<StubEntity>);
    export_entity!(env_explosion, Private<StubEntity>);
}
