pub use xash3d_shared::prelude::*;

pub use crate::{
    engine::prelude::*,
    engine::{ServerEngine, ServerEngineRef},
};

pub use crate::{
    entity::{AsEntityHandle, CreateEntity, Entity, EntityCast},
    global_state::decals::Decals,
    private::{GetPrivateData, PrivateEntity},
};

#[cfg(feature = "save")]
pub use crate::save::{Restore, Save};
