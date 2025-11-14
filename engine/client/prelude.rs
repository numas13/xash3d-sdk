pub use xash3d_shared::prelude::*;

pub use crate::{
    engine::prelude::*,
    engine::{ClientEngine, ClientEngineRef},
    instance::studio,
};

// TODO: remove me
#[allow(deprecated)]
pub use crate::misc::WRectExt;

pub use crate::entity::TempEntityExt;
pub use crate::render::RefParamsExt;
pub use crate::sprite::ClientSpriteExt;
