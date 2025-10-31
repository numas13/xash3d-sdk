pub use xash3d_shared::prelude::*;

pub use crate::{
    engine::prelude::*,
    engine::{ServerEngine, ServerEngineRef},
};

pub use crate::{
    entity::{AsEntityHandle, CreateEntity, Entity},
    global_state::decals::Decals,
    private::{GetPrivateData, PrivateEntity},
};

// TODO: remove me
pub use crate::{engine::LevelListExt, save::TypeDescriptionExt};
