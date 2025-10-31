use xash3d_shared::entity::MoveType;

use crate::{
    entity::{delegate_entity, BaseEntity, KeyValue, ObjectCaps, Solid, UseType},
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct StubEntity {
    base: BaseEntity,
    dump_key_value: bool,
}

impl StubEntity {
    pub fn new(base: BaseEntity, dump_key_value: bool) -> Self {
        Self {
            base,
            dump_key_value,
        }
    }
}

impl CreateEntity for StubEntity {
    fn create(base: BaseEntity) -> Self {
        Self::new(base, false)
    }
}

impl Entity for StubEntity {
    delegate_entity!(base not { object_caps, key_value, spawn, touched, used, blocked });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        self.base.key_value(data);

        if self.dump_key_value && !data.handled() {
            let name = self.pretty_name();
            let key = data.key_name();
            let value = data.value();
            trace!("{name}: key={key} value={value}");
        }
    }

    fn spawn(&mut self) {
        let name = self.pretty_name();
        let target = self.vars().target();
        trace!("spawn {name}, target={target:?}");

        let v = self.vars();
        v.set_move_dir_from_angles();
        v.set_solid(Solid::Trigger);
        v.set_move_type(MoveType::Push);
        v.reload_model();
    }

    fn touched(&self, other: &dyn Entity) {
        let name = self.pretty_name();
        trace!("{name} touched by {}", other.pretty_name());
    }

    fn used(&self, use_type: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
        let name = self.pretty_name();
        let caller_name = caller.pretty_name();
        let activator_name = activator.map(|i| i.pretty_name());
        trace!("{name} used({use_type:?}) by {caller_name}, activator {activator_name:?}");
    }

    fn blocked(&self, other: &dyn Entity) {
        trace!("{} blocked by {}", self.pretty_name(), other.pretty_name());
    }
}

impl_private!(StubEntity {});
