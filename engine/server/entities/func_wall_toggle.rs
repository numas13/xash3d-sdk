use xash3d_shared::entity::Effects;

use crate::{
    entities::func_wall::FuncWall,
    entity::{delegate_entity, BaseEntity, Solid, UseType},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct WallToggle {
    base: FuncWall,
}

impl CreateEntity for WallToggle {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: FuncWall::create(base),
        }
    }
}

impl WallToggle {
    const SF_START_OFF: u32 = 1 << 0;

    fn is_on(&self) -> bool {
        self.vars().solid() != Solid::Not
    }

    fn turn_on(&self) {
        let v = self.vars();
        v.set_solid(Solid::Bsp);
        v.with_effects(|f| f.difference(Effects::NODRAW));
        v.link();
    }

    fn turn_off(&self) {
        let v = self.vars();
        v.set_solid(Solid::Not);
        v.with_effects(|f| f.union(Effects::NODRAW));
        v.link();
    }
}

impl Entity for WallToggle {
    delegate_entity!(base not { spawn, used });

    fn spawn(&mut self) {
        self.base.spawn();
        if self.vars().spawn_flags() & Self::SF_START_OFF != 0 {
            self.turn_off();
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let status = self.is_on();
        if use_type.should_toggle(status) {
            if status {
                self.turn_off();
            } else {
                self.turn_on();
            }
        }
    }
}

impl_private!(WallToggle {});

export_entity_default!("export-func_wall_toggle", func_wall_toggle, WallToggle);
