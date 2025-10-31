use core::ffi::CStr;

use xash3d_server::{
    entities::item::BaseItem,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, EntityItem},
    export::export_entity,
    prelude::*,
    private::Private,
    save::{Restore, Save},
};

use crate::{entities::player::WEAPON_SUIT, sound::emit_sound_suit};

#[derive(Save, Restore)]
pub struct ItemSuit {
    base: BaseItem,
}

impl_entity_cast!(ItemSuit);

impl CreateEntity for ItemSuit {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseItem::create(base),
        }
    }
}

impl ItemSuit {
    const SF_SHORT_LOGON: u32 = 1 << 0;

    const MODEL_NAME: &'static CStr = res::valve::models::W_SUIT;
}

impl Entity for ItemSuit {
    delegate_entity!(base not { precache, spawn, touched });

    fn precache(&mut self) {
        self.engine().precache_model(Self::MODEL_NAME);
    }

    fn spawn(&mut self) {
        self.precache();
        self.vars().set_model(Self::MODEL_NAME);
        self.base.spawn();
    }

    fn touched(&self, other: &dyn Entity) {
        self.try_give(other);
    }
}

impl EntityItem for ItemSuit {
    fn try_give(&self, other: &dyn Entity) -> bool {
        self.base.try_give_to_player(self, other, |player| {
            let v = self.vars();
            let player_v = player.vars();
            if player_v.weapons() & WEAPON_SUIT != 0 {
                return false;
            }
            if v.spawn_flags() & Self::SF_SHORT_LOGON != 0 {
                emit_sound_suit(player.as_entity(), c"!HEV_A0".into());
            } else {
                emit_sound_suit(player.as_entity(), c"!HEV_AAx".into());
            }
            player_v.with_weapons(|i| i | WEAPON_SUIT);
            true
        })
    }
}

export_entity!(item_suit, Private<ItemSuit>);
