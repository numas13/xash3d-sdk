use core::{cell::Cell, ffi::CStr};

use xash3d_server::{
    entity::{BaseEntity, DamageFlags, Effects, EntityItem, MoveType, Solid, delegate_entity},
    export::export_entity,
    ffi::common::vec3_t,
    prelude::*,
    save::{Restore, Save},
};

#[derive(Copy, Clone, Default, PartialEq, Eq, Save, Restore)]
#[repr(u8)]
enum State {
    #[default]
    None = 0,
    Think,
    Touch,
}

#[derive(Save, Restore)]
pub struct ItemSodaCan {
    base: BaseEntity,
    state: Cell<State>,
}

impl CreateEntity for ItemSodaCan {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            state: Cell::default(),
        }
    }
}

impl ItemSodaCan {
    pub const CLASS_NAME: &'static CStr = c"item_sodacan";

    const MODEL_NAME: &'static CStr = res::valve::models::CAN;
    const BOUNCE_SOUND: &'static CStr = res::valve::sound::weapons::G_BOUNCE3;

    pub fn precache(engine: &ServerEngine) {
        engine.precache_model(Self::MODEL_NAME);
        engine.precache_sound(Self::BOUNCE_SOUND);
    }
}

impl Entity for ItemSodaCan {
    delegate_entity!(base not { precache, spawn, think, touched });

    fn precache(&mut self) {
        Self::precache(&self.engine());
    }

    fn spawn(&mut self) {
        self.precache();

        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::Toss);
        v.set_model(Self::MODEL_NAME);
        v.set_size_and_link(vec3_t::ZERO, vec3_t::ZERO);
        v.set_next_think_time_from_now(0.5);
        self.state.set(State::Think);
    }

    fn think(&self) {
        if self.state.get() != State::Think {
            return;
        }

        let engine = self.engine();
        let v = self.vars();

        engine
            .build_sound()
            .channel_weapon()
            .emit_dyn(Self::BOUNCE_SOUND, v);

        v.set_solid(Solid::Trigger);
        v.set_size_and_link(vec3_t::new(-8.0, -8.0, 0.0), vec3_t::splat(8.0));

        self.state.set(State::Touch);
    }

    fn touched(&self, other: &dyn Entity) {
        if self.state.get() != State::Touch {
            return;
        }

        if self.try_give(other) {
            self.state.set(State::None);
            self.remove_from_world();
        }
    }
}

impl EntityItem for ItemSodaCan {
    fn try_give(&self, other: &dyn Entity) -> bool {
        if other.is_player() {
            other.take_health(1.0, DamageFlags::GENERIC);
            let v = self.vars();
            if let Some(owner) = v.owner() {
                // tell the machine that the can was taken
                owner.vars().set_frags(0.0);
            }
            v.set_solid(Solid::Not);
            v.set_move_type(MoveType::None);
            v.set_effects(Effects::NODRAW);
            v.stop_thinking();
            self.state.set(State::None);
            true
        } else {
            false
        }
    }
}

export_entity!(item_sodacan, ItemSodaCan { EntityItem });
