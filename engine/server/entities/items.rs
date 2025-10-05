#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(world_items, Private<StubEntity>);
    export_entity!(item_suit, Private<StubEntity>);
    export_entity!(item_battery, Private<StubEntity>);
    export_entity!(item_antidote, Private<StubEntity>);
    export_entity!(item_security, Private<StubEntity>);
    export_entity!(item_longjump, Private<StubEntity>);
}
