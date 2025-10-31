use xash3d_server::{
    entity::{
        create_entity, delegate_entity, impl_entity_cast, BaseEntity, Effects, Solid, UseType,
    },
    export::export_entity,
    prelude::*,
    save::{Restore, Save},
};

use crate::entities::item_sodacan::ItemSodaCan;

#[derive(Save, Restore)]
pub struct Beverage {
    base: BaseEntity,
}

impl_entity_cast!(Beverage);

impl CreateEntity for Beverage {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for Beverage {
    delegate_entity!(base not { precache, spawn, used });

    fn precache(&mut self) {
        ItemSodaCan::precache(&self.engine());
    }

    fn spawn(&mut self) {
        self.precache();

        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_effects(Effects::NODRAW);
        v.set_frags(0.0);

        if v.health() == 0.0 {
            v.set_health(10.0);
        }
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let engine = self.engine();
        let v = self.vars();

        if v.frags() != 0.0 || v.health() <= 0.0 {
            return;
        }

        let item = ItemSodaCan::CLASS_NAME;
        let owner = Some(self.entity_handle());
        if let Ok(can) = create_entity(&engine, item, v.origin(), v.angles(), owner) {
            if v.skin() == 6 {
                can.vars().set_skin(engine.random_int(0, 5));
            } else {
                can.vars().set_skin(v.skin());
            }
        } else {
            error!("{}: failed to create {item:?}", self.pretty_name());
        }

        v.set_frags(1.0);
        v.with_health(|health| health - 1.0);
    }
}

export_entity!(env_beverage, Beverage {});
