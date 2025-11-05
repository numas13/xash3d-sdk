use xash3d_shared::{
    entity::{Effects, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    entity::{BaseEntity, ObjectCaps, Solid, delegate_entity},
    prelude::*,
    private::impl_private,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Trigger {
    base: BaseEntity,
}

impl CreateEntity for Trigger {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Trigger {
    pub fn toggle_use(&self) {
        let v = self.vars();
        match v.solid() {
            Solid::Not => {
                v.set_solid(Solid::Trigger);
                self.engine().globals.force_retouch();
            }
            _ => {
                v.set_solid(Solid::Not);
            }
        }
        v.link();
    }
}

impl Entity for Trigger {
    delegate_entity!(base not { object_caps, spawn });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn spawn(&mut self) {
        let v = self.vars();
        if v.angles() != vec3_t::ZERO {
            v.set_move_dir_from_angles();
        }
        v.set_solid(Solid::Trigger);
        v.set_move_type(MoveType::None);
        v.reload_model();
        if !self.engine().get_cvar::<bool>(c"showtriggers") {
            v.with_effects(|f| f | Effects::NODRAW);
        }
    }
}

impl_private!(Trigger {});
