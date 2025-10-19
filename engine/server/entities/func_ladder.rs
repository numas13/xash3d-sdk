use xash3d_shared::{
    entity::{Effects, MoveType},
    ffi,
    render::RenderMode,
};

use crate::{
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, ObjectCaps, Solid,
    },
    export::export_entity_default,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Ladder {
    base: BaseEntity,
}

impl_entity_cast!(Ladder);

impl CreateEntity for Ladder {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for Ladder {
    delegate_entity!(base not { object_caps, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_skin(ffi::common::CONTENTS_LADDER);
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::Push);
        v.set_render_mode(RenderMode::TransTexture);
        v.set_render_amount(0.0);
        v.with_effects(|f| f.difference(Effects::NODRAW));
        v.reload_model();
    }
}

export_entity_default!("export-func_ladder", func_ladder, Ladder);
