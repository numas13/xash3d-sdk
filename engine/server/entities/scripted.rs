#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(scripted_sequence, Private<StubEntity>);
    export_entity!(aiscripted_sequence, Private<StubEntity>);
    export_entity!(scripted_sentence, Private<StubEntity>);
    export_entity!(monster_furniture, Private<StubEntity>);
}
