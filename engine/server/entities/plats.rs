#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(func_plat, Private<StubEntity>);
    export_entity!(func_platrot, Private<StubEntity>);
    export_entity!(func_train, Private<StubEntity>);
    export_entity!(func_tracktrain, Private<StubEntity>);
    export_entity!(func_traincontrols, Private<StubEntity>);
    export_entity!(func_trackchange, Private<StubEntity>);
    export_entity!(func_trackautochange, Private<StubEntity>);
    export_entity!(func_guntarget, Private<StubEntity>);
}
