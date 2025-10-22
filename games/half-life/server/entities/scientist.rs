use xash3d_server::{entities::stub::StubEntity, entity::Private, export::export_entity};

export_entity!(monster_scientist, Private<StubEntity>);
export_entity!(monster_scientist_dead, Private<StubEntity>);
export_entity!(monster_sitting_scientist, Private<StubEntity>);
