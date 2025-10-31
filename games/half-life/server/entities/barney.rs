use xash3d_server::{entities::stub::StubEntity, export::export_entity, private::Private};

export_entity!(monster_barney, Private<StubEntity>);
export_entity!(monster_barney_dead, Private<StubEntity>);
