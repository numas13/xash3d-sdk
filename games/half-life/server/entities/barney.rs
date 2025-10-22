use xash3d_server::{entities::stub::StubEntity, entity::Private, export::export_entity};

export_entity!(monster_barney, Private<StubEntity>);
export_entity!(monster_barney_dead, Private<StubEntity>);
