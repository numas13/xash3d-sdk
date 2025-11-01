use xash3d_server::{
    entity::{Effects, MoveType, delegate_entity, BaseEntity, ObjectCaps, Solid},
    prelude::*,
    private::impl_private,
    ffi,
    render::RenderMode,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Ladder {
    base: BaseEntity,
}

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

impl_private!(Ladder {});

define_export! {
    export_func_ladder as export if "func-ladder" {
        func_ladder = func_ladder::Ladder,
    }
}
