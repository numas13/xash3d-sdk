use xash3d_server::{
    entity::{Private, StubEntity},
    export::export_entity,
};

export_entity!(item_healthkit, Private<StubEntity>);
export_entity!(func_healthcharger, Private<StubEntity>);
