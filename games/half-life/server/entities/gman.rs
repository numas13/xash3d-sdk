use xash3d_server::{
    entity::{Private, StubEntity},
    export::export_entity,
};

export_entity!(monster_gman, Private<StubEntity>);
