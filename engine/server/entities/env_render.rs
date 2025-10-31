use bitflags::bitflags;

use crate::{
    entity::{delegate_entity, impl_entity_cast, BaseEntity, Solid, UseType},
    export::export_entity_default,
    prelude::*,
};

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const MASK_FX       = 1 << 0;
        const MASK_AMT      = 1 << 1;
        const MASK_MODE     = 1 << 2;
        const MASK_COLOR    = 1 << 3;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Render {
    base: BaseEntity,
}

impl_entity_cast!(Render);

impl CreateEntity for Render {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for Render {
    delegate_entity!(base not { spawn, used });

    fn spawn(&mut self) {
        self.vars().set_solid(Solid::Not);
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let v = self.vars();
        let Some(target) = v.target() else { return };
        let sf = SpawnFlags::from_bits_retain(v.spawn_flags());
        for target in self.engine().entities().by_target_name(target.as_thin()) {
            let target_v = target.vars();
            if !sf.intersects(SpawnFlags::MASK_FX) {
                target_v.set_render_fx(v.render_fx());
            }
            if !sf.intersects(SpawnFlags::MASK_AMT) {
                target_v.set_render_amount(v.render_amount());
            }
            if !sf.intersects(SpawnFlags::MASK_MODE) {
                target_v.set_render_mode(v.render_mode());
            }
            if !sf.intersects(SpawnFlags::MASK_COLOR) {
                target_v.set_render_color(v.render_color());
            }
        }
    }
}

export_entity_default!("export-env_render", env_render, Render {});
