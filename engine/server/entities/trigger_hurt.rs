use bitflags::bitflags;
use xash3d_shared::{entity::DamageFlags, ffi::common::vec3_t};

use crate::{
    entities::{delayed_use::DelayedUse, trigger::Trigger},
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, Dead, EntityPlayer, KeyValue, Solid,
        TakeDamage, UseType,
    },
    export::export_entity_default,
    prelude::*,
};

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct TriggerHurtSpawnFlags: u32 {
        /// Only fire hurt target once.
        const TARGET_ONCE = 1 << 0;
        /// Spawnflag that makes trigger_push spawn turned OFF.
        const START_OFF = 1 << 1;
        /// Players not allowed to fire this trigger.
        const NO_CLIENTS = 1 << 3;
        /// Trigger hurt will only fire its target if it is hurting a client.
        const CLIENT_ONLY_FIRE = 1 << 4;
        /// Only clients may touch this trigger.
        const CLIENT_ONLY_TOUCH = 1 << 4;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct TriggerHurt {
    base: Trigger,
    delayed: DelayedUse,
    damage_type: DamageFlags,
}

impl_entity_cast!(TriggerHurt);

impl CreateEntity for TriggerHurt {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base: Trigger::create(base),
            delayed: DelayedUse::new(engine),
            damage_type: DamageFlags::default(),
        }
    }
}

impl TriggerHurt {
    fn spawn_flags(&self) -> TriggerHurtSpawnFlags {
        TriggerHurtSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }
}

impl Entity for TriggerHurt {
    delegate_entity!(base not { key_value, spawn, used, touched, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"damagetype" => {
                let bits = data.parse_or_default();
                self.damage_type = DamageFlags::from_bits_retain(bits);
            }
            _ => {
                if !self.delayed.key_value(data) {
                    self.base.key_value(data);
                }
                return;
            }
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        self.base.spawn();

        let spawn_flags = self.spawn_flags();
        let engine = self.base.engine();
        let v = self.base.vars();

        if self.damage_type.intersects(DamageFlags::RADIATION) {
            v.set_next_think_time_from_now(engine.random_float(0.0, 0.5));
        }

        if spawn_flags.intersects(TriggerHurtSpawnFlags::START_OFF) {
            v.set_solid(Solid::Not);
        }

        v.link();
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        if self.vars().target_name().is_some() {
            self.base.toggle_use();
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if other.vars().take_damage() == TakeDamage::No {
            return;
        }

        let engine = self.base.engine();
        let global_state = self.base.global_state();
        let spawn_flags = self.spawn_flags();

        let is_player = other.is_player();
        if spawn_flags.intersects(TriggerHurtSpawnFlags::NO_CLIENTS) && is_player {
            return;
        }

        let v = self.base.vars();
        let now = engine.globals.map_time();
        let is_multiplayer = global_state.game_rules().is_multiplayer();
        if is_multiplayer {
            warn!(
                "{}: touched is not implemented in multiplayer",
                self.classname()
            );
            return;
        } else if now <= v.damage_time() && now != v.pain_finished_time() {
            return;
        }

        let dmg = v.damage() * 0.5;
        if dmg < 0.0 {
            if !(is_multiplayer && is_player && other.vars().dead() != Dead::No) {
                other.take_health(-dmg, self.damage_type);
            }
        } else {
            other.take_damage(dmg, self.damage_type, v, None);
        }

        v.set_pain_finished_time(now);
        v.set_damage_time(now + 0.5);

        if v.target().is_some() {
            if spawn_flags.intersects(TriggerHurtSpawnFlags::CLIENT_ONLY_FIRE) && !is_player {
                return;
            }
            self.delayed.use_targets(UseType::Toggle, Some(other), self);
            if spawn_flags.intersects(TriggerHurtSpawnFlags::TARGET_ONCE) {
                self.vars().set_target(None);
            }
        }
    }

    fn think(&self) {
        if !self.damage_type.intersects(DamageFlags::RADIATION) {
            return;
        }

        let engine = self.base.engine();
        let v = self.base.vars();

        // set origin to center of trigger so that this check works
        let orig_origin = v.origin();
        let orig_view_ofs = v.view_ofs();
        v.set_origin(v.abs_center());
        v.set_view_ofs(vec3_t::ZERO);

        let player = engine.find_client_in_pvs(v);

        v.set_origin(orig_origin);
        v.set_view_ofs(orig_view_ofs);

        if let Some(player) = player.downcast_ref::<dyn EntityPlayer>() {
            let spot1 = v.abs_center();
            let spot2 = player.vars().abs_center();
            let range = (spot1 - spot2).length();
            player.set_geiger_range(range);
        }

        v.set_next_think_time_from_now(0.25);
    }
}

export_entity_default!("export-trigger_hurt", trigger_hurt, TriggerHurt {});
