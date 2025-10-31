use core::cell::Cell;

use xash3d_shared::{
    entity::{Effects, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    engine::DropToFloorResult,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, EntityItem, EntityPlayer, Solid, UseType,
    },
    prelude::*,
    private::impl_private,
    time::MapTime,
    utils,
};

pub const SF_ITEM_NO_RESPAWN: u32 = 1 << 30;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum State {
    #[default]
    None = 0,
    Spawned,
    Respawn,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct BaseItem {
    base: BaseEntity,
    state: Cell<State>,
}

impl_entity_cast!(BaseItem);

impl CreateEntity for BaseItem {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,
            state: Default::default(),
        }
    }
}

impl BaseItem {
    fn respawn(&self, time: MapTime, origin: vec3_t) {
        let v = self.vars();
        v.with_effects(|f| f.union(Effects::NODRAW));
        v.set_origin_and_link(origin);
        v.set_next_think_time(time);
        self.state.set(State::Respawn);
    }

    fn materialize(&self) {
        let v = self.vars();
        if v.effects().intersects(Effects::NODRAW) {
            self.engine()
                .build_sound()
                .channel_weapon()
                .pitch(150)
                .emit_dyn(res::valve::sound::items::SUITCHARGEOK1, v);
            v.with_effects(|f| f.difference(Effects::NODRAW).union(Effects::MUZZLEFLASH));
        }
        self.state.set(State::Spawned);
    }

    pub fn try_give_to_player(
        &self,
        item: &dyn Entity,
        other: &dyn Entity,
        give: impl FnOnce(&dyn EntityPlayer) -> bool,
    ) -> bool {
        if self.state.get() != State::Spawned {
            return false;
        }
        let Some(player) = other.as_player() else {
            return false;
        };
        let global_state = self.global_state();
        let game_rules = global_state.game_rules();
        if !game_rules.can_have_item(player, item) {
            return false;
        }

        if give(player) {
            utils::use_targets(UseType::Toggle, Some(player.as_entity()), item);
            game_rules.player_got_item(player, item);
            if let Some((time, origin)) = game_rules.item_respawn(item) {
                self.respawn(time, origin);
            } else {
                self.remove_from_world();
            }
            true
        } else {
            false
        }
    }
}

impl Entity for BaseItem {
    delegate_entity!(base not { spawn, think, remove_from_world });

    fn remove_from_world(&self) {
        self.state.set(State::None);
        self.base.remove_from_world();
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();
        v.set_solid(Solid::Trigger);
        v.set_move_type(MoveType::Toss);
        v.link();
        let min = vec3_t::new(-16.0, -16.0, 0.0);
        let max = vec3_t::new(16.0, 16.0, 16.0);
        engine.set_size(v, min, max);
        if engine.drop_to_floor(v) == DropToFloorResult::False {
            let name = self.pretty_name();
            error!("{name}: fell out of level at {}", v.origin());
            self.remove_from_world();
            return;
        }
        self.state.set(State::Spawned);
    }

    fn think(&self) {
        if self.state.get() == State::Respawn {
            self.materialize();
        }
    }
}

impl EntityItem for BaseItem {
    fn try_give(&self, _: &dyn Entity) -> bool {
        false
    }
}

impl_private!(BaseItem { EntityItem });
