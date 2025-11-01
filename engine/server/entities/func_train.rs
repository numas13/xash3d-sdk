use core::cell::Cell;

use xash3d_shared::{
    entity::{DamageFlags, Effects, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    entities::path_corner::PathCorner,
    entity::{delegate_entity, BaseEntity, EntityHandle, KeyValue, ObjectCaps, Solid, UseType},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
    sound::PlatformSounds,
    utils::{self, LinearMove, Move},
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    Next,
    Wait,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Train {
    base: BaseEntity,

    platform_sounds: PlatformSounds,
    // TODO: split LinearMove/AngularMove to multiple structs?
    linear: LinearMove,

    activated: Cell<bool>,
    current_target: Cell<Option<EntityHandle>>,
    wait: Cell<f32>,
    think: Cell<Think>,
}

impl CreateEntity for Train {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            platform_sounds: Default::default(),
            linear: Default::default(),

            activated: Cell::default(),
            current_target: Cell::default(),
            wait: Cell::default(),
            think: Cell::default(),
        }
    }
}

impl Train {
    const SF_WAIT_FOR_RETRIGGER: u32 = 1 << 0;
    const SF_NOT_SOLID: u32 = 1 << 3;

    fn wait_for_retrigger(&self) -> bool {
        self.vars().spawn_flags() & Self::SF_WAIT_FOR_RETRIGGER != 0
    }

    fn set_wait_for_retrigger(&self, wait: bool) {
        self.vars().with_spawn_flags(|f| {
            if wait {
                f | Self::SF_WAIT_FOR_RETRIGGER
            } else {
                f & !Self::SF_WAIT_FOR_RETRIGGER
            }
        })
    }

    fn next_target(&self) -> Option<&PathCorner> {
        let target = self.target_entity()?;
        match target.downcast_ref::<PathCorner>() {
            Some(target) => Some(target),
            None => {
                let name = self.pretty_name();
                let target_name = target.pretty_name();
                error!("{name}: target {target_name} is not a path corner");
                None
            }
        }
    }

    fn next(&self) {
        let name = self.pretty_name();
        let v = self.vars();

        let Some(target) = self.next_target() else {
            self.platform_sounds.emit_moving_stop_noise(v);
            return;
        };
        let target_v = target.vars();

        // save last target in case we need to find it again
        v.set_message(v.target());
        v.set_target(target_v.target());
        self.wait.set(target.delay());

        if let Some(current_target) = self.current_target.get() {
            let speed = current_target.vars().speed();
            if speed != 0.0 {
                v.set_speed(speed);
                trace!("{name}: set speed to {speed:4.2}");
            }
        }
        self.current_target.set(Some(target.entity_handle()));

        // HACK: used to restore previous target
        v.set_enemy(&target.entity_handle());

        let dest = target_v.origin() - v.center();
        if target.spawn_flags().has_teleport() {
            trace!("{name}: teleport to {}", target.pretty_name());
            v.with_effects(|f| f.union(Effects::NOINTERP));
            v.set_origin_and_link(dest);
            self.wait();
        } else {
            trace!("{name}: move to {}", target.pretty_name());

            self.platform_sounds.emit_moving_noise(v);

            v.with_effects(|f| f.difference(Effects::NOINTERP));
            if self.linear.start_move(v, v.speed(), dest) {
                self.wait();
            } else {
                self.think.set(Think::Wait);
            }
        }
    }

    fn wait(&self) {
        let target = self
            .current_target
            .get()
            .downcast_ref::<PathCorner>()
            .expect("current target must be a valid path corner");

        let v = self.vars();
        let target_v = target.vars();

        if let Some(message) = target_v.message().as_deref() {
            utils::fire_targets(message, UseType::Toggle, Some(self), self);
            if target.spawn_flags().has_fire_once() {
                target_v.set_message(None);
            }
        }

        if target.spawn_flags().has_wait_for_retrigger() || self.wait_for_retrigger() {
            self.set_wait_for_retrigger(true);
            self.platform_sounds.emit_moving_stop_noise(v);
            v.stop_thinking();
            return;
        }

        let wait = self.wait.get();
        if wait != 0.0 {
            trace!("{}: wait", self.pretty_name());

            self.platform_sounds.stop_moving_noise(v);
            self.think.set(Think::Next);
            v.set_next_think_time_from_last(wait);
        } else {
            self.next();
        }
    }
}

impl Entity for Train {
    delegate_entity!(base not {
        object_caps, key_value, precache, spawn, activate, used, blocked, think, override_reset
    });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        if self.platform_sounds.key_value(data) {
            return;
        }
        self.base.key_value(data);
    }

    fn precache(&mut self) {
        self.platform_sounds.precache(self.base.vars());
    }

    fn spawn(&mut self) {
        self.platform_sounds.init();

        let v = self.base.vars();
        if v.target().is_none() {
            error!("{}: no target", self.pretty_name());
        }

        if v.speed() == 0.0 {
            v.set_speed(100.0);
        }

        if v.damage() == 0.0 {
            v.set_damage(2.0);
        }

        if v.spawn_flags() & Self::SF_NOT_SOLID != 0 {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }
        v.set_move_type(MoveType::Push);

        v.reload_model();
        v.set_size_and_link(v.min_size(), v.max_size());
        v.link();

        self.precache();

        self.activated.set(false);
    }

    fn activate(&self) {
        if self.activated.get() {
            return;
        }

        let name = self.pretty_name();
        let v = self.vars();

        let Some(target) = v.target() else {
            error!("{name}: no target");
            return;
        };
        let Some(target) = self.next_target() else {
            error!("{name}: target {target} not found");
            return;
        };
        let target_v = target.vars();

        v.set_target(target_v.target());
        self.current_target.set(Some(target.entity_handle()));

        v.set_origin_and_link(target_v.origin() - v.center());

        if v.target_name().is_none() {
            trace!("{name}: start immediately");
            self.think.set(Think::Next);
            v.set_next_think_time_from_last(0.1);
        } else {
            trace!("{name}: wait for retrigger");
            v.with_spawn_flags(|f| f | Self::SF_WAIT_FOR_RETRIGGER);
        }

        self.activated.set(true);
    }

    fn used(&self, _: UseType, _: Option<&dyn Entity>, caller: &dyn Entity) {
        let name = self.pretty_name();
        trace!("{name}: used by {}", caller.pretty_name());

        if self.wait_for_retrigger() {
            self.set_wait_for_retrigger(false);
            self.next();
        } else {
            self.set_wait_for_retrigger(true);
            let v = self.vars();
            if let Some(enemy) = v.enemy() {
                v.set_target(enemy.vars().target_name());
            }
            v.set_velocity(vec3_t::ZERO);
            v.stop_thinking();
            self.platform_sounds.emit_moving_stop_noise(v);
        }
    }

    fn blocked(&self, other: &dyn Entity) {
        debug!("{}: blocked is not implemented yet", self.pretty_name());

        let v = self.vars();
        other.take_damage(v.damage(), DamageFlags::CRUSH, v, Some(v));
    }

    fn think(&self) {
        match self.think.get() {
            Think::None => {}
            Think::Next => {
                self.think.set(Think::None);
                self.next();
            }
            Think::Wait => {
                if self.linear.move_done(self.vars()) {
                    self.think.set(Think::None);
                    self.wait();
                }
            }
        }
    }

    fn override_reset(&self) {
        let v = self.vars();
        if v.velocity() != vec3_t::ZERO && v.is_thinking() {
            // restore previous target
            v.set_target(v.message());

            if self.target_entity().is_some() {
                self.think.set(Think::Next);
                v.set_next_think_time_from_last(0.1);
            } else {
                v.set_velocity(vec3_t::ZERO);
                v.stop_thinking();
            }
        }
    }
}

impl_private!(Train {});

export_entity_default!("export-func_train", func_train, Train);
