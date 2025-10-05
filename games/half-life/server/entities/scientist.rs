use xash3d_server::{
    entity::{Private, StubEntity},
    export::export_entity,
};

export_entity!(monster_scientist, Private<StubEntity>);
export_entity!(monster_scientist_dead, Private<StubEntity>);
export_entity!(monster_sitting_scientist, Private<StubEntity>);
