use core::{ffi::CStr, fmt::Write};

use csz::CStrArray;
use xash3d_hl_shared::user_message;
use xash3d_server::{
    entities::item::BaseItem,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Dead, Entity, EntityItem,
        Private,
    },
    export::export_entity,
    save::{Restore, Save},
};

use crate::{
    entities::player::{MAX_NORMAL_BATTERY, WEAPON_SUIT},
    game_rules::SkillData,
    sound::emit_sound_suit,
};

#[derive(Save, Restore)]
pub struct ItemBattery {
    base: BaseItem,
}

impl_entity_cast!(ItemBattery);

impl CreateEntity for ItemBattery {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseItem::create(base),
        }
    }
}

impl ItemBattery {
    const MODEL_NAME: &'static CStr = res::valve::models::W_BATTERY;
    const PICKUP_SOUND: &'static CStr = res::valve::sound::items::GUNPICKUP2;
}

impl Entity for ItemBattery {
    delegate_entity!(base not { precache, spawn, touched });

    fn precache(&mut self) {
        let engine = self.engine();
        engine.precache_model(Self::MODEL_NAME);
        engine.precache_sound(Self::PICKUP_SOUND);
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

impl EntityItem for ItemBattery {
    fn try_give(&self, other: &dyn Entity) -> bool {
        self.base.try_give_to_player(self, other, |player| {
            let player_v = player.vars();
            if player_v.dead() != Dead::No {
                return false;
            }
            if player_v.weapons() & WEAPON_SUIT == 0 {
                return false;
            }
            if player_v.armor_value() >= MAX_NORMAL_BATTERY {
                return false;
            }
            let engine = self.engine();
            let global_state = self.global_state();
            let skill_data = global_state.get::<SkillData>();
            let armor = player_v.armor_value() + skill_data.battery_capacity;
            player_v.set_armor_value(armor.min(MAX_NORMAL_BATTERY));

            engine
                .build_sound()
                .channel_item()
                .emit_dyn(Self::PICKUP_SOUND, player_v);

            let classname = self.classname();
            let msg = user_message::ItemPickup {
                classname: classname.as_thin().into(),
            };
            engine.msg_one_reliable(player_v, &msg);

            let p = (player_v.armor_value() as i32 / 5).clamp(1, 20) - 1;

            let mut buffer = CStrArray::<64>::new();
            write!(buffer.cursor(), "!HEV_{p}P").ok();
            emit_sound_suit(player.as_entity(), &buffer);

            debug!(
                "{}: set suit update {buffer} is not implemented yet",
                self.pretty_name()
            );

            true
        })
    }
}

export_entity!(item_battery, Private<ItemBattery>);
