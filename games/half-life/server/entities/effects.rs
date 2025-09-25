use xash3d_server::export::export_entity;

use crate::{entities::subs::PointEntity, entity::Private};

// Lightning target, just alias landmark.
export_entity!(info_target, Private<PointEntity>);
