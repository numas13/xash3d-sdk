use core::cell::Cell;

use bitflags::bitflags;
use xash3d_shared::{entity::MoveType, ffi::common::vec3_t};

use crate::{
    entities::delayed_use::DelayedUse,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, DamageFlags, EntityCast, EntityHandle,
        EntityVars, KeyValue, ObjectCaps, Solid, TakeDamage, UseType,
    },
    export::export_entity_default,
    prelude::*,
    sound::{button_sound_or_default, LockSounds},
    str::MapString,
    utils::{self, AngularMove, LinearMove, Move, MoveState, Sparks},
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

bitflags! {
    #[derive(Copy, Clone, Debug)]
    struct SpawnFlags: u32 {
        const DONT_MOVE     = 1 << 0;
        const TOGGLE        = 1 << 5;
        const SPARK_IF_OFF  = 1 << 6;
        const TOUCH_ONLY    = 1 << 8;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    struct RotSpawnFlags: u32 {
        const NOT_SOLID         = 1 << 0;
        const ROTATE_BACKWARDS  = 1 << 1;
        const TOGGLE            = 1 << 5;
        const ROTATE_Z          = 1 << 6;
        const ROTATE_X          = 1 << 7;
        const TOUCH_ONLY        = 1 << 8;
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ButtonCode {
    None,
    Activate,
    Return,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    MoveDone,
    ButtonReturn,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
struct BaseButton<T> {
    base: BaseEntity,
    delayed: DelayedUse,
    master: Option<MapString>,
    wait: f32,

    button_move: T,
    state: Cell<MoveState>,
    activator: Cell<Option<EntityHandle>>,
    enable_touch: Cell<bool>,
    think: Cell<Think>,

    sounds: u8,
    // TODO: move to Button
    lock_sounds: LockSounds,
}

impl<T: Move> EntityCast for BaseButton<T> {
    impl_entity_cast!(cast BaseButton<T>);
}

impl<T: Move + Default> CreateEntity for BaseButton<T> {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base,
            delayed: DelayedUse::new(engine),
            master: None,
            wait: 0.0,

            button_move: T::default(),
            state: Default::default(),
            activator: Default::default(),
            enable_touch: Default::default(),
            think: Default::default(),

            sounds: 0,
            lock_sounds: LockSounds::new(engine),
        }
    }
}

impl<T: Move> BaseButton<T> {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn stay_pushed(&self) -> bool {
        self.wait == -1.0
    }

    fn is_off(&self) -> bool {
        self.state.get() == MoveState::AtStart
    }

    fn move_done(&self) {
        match self.state.get() {
            MoveState::GoingToEnd => self.button_activated(),
            MoveState::GoingToStart => self.button_returned(),
            state => unreachable!("unexpected state {state:?}"),
        }
    }

    fn button_activate(&self) {
        let engine = self.engine();
        let v = self.base.vars();
        if let Some(noise) = v.noise() {
            engine.build_sound().channel_voice().emit_dyn(noise, v);
        }

        if !utils::is_master_triggered(&engine, self.master, self.activator.get().get_entity()) {
            self.lock_sounds.play_button(true, v);
            return;
        } else {
            self.lock_sounds.play_button(false, v);
        }

        assert_eq!(self.state.get(), MoveState::AtStart);
        self.state.set(MoveState::GoingToEnd);

        if self.button_move.move_to_end(v, v.speed(), false) {
            self.button_activated();
        } else {
            self.think.set(Think::MoveDone);
        }
    }

    fn button_activated(&self) {
        assert_eq!(self.state.get(), MoveState::GoingToEnd);

        let engine = self.engine();
        let activator = self.activator.get().get_entity();
        if !utils::is_master_triggered(&engine, self.master, activator) {
            return;
        }
        self.state.set(MoveState::AtEnd);

        let v = self.vars();
        let sf = self.spawn_flags();
        if self.stay_pushed() || sf.intersects(SpawnFlags::TOGGLE) {
            self.enable_touch.set(sf.intersects(SpawnFlags::TOUCH_ONLY));
        } else {
            self.think.set(Think::ButtonReturn);
            v.set_next_think_time_from_last(self.wait);
        }

        self.delayed.use_targets(UseType::Toggle, activator, self);

        // use alternate textures
        v.set_frame(1.0);
    }

    fn button_return(&self) {
        assert_eq!(self.state.get(), MoveState::AtEnd);
        self.state.set(MoveState::GoingToStart);
        let v = self.vars();
        if self.button_move.move_to_start(v, v.speed()) {
            self.button_returned();
        } else {
            self.think.set(Think::MoveDone);
        }
        // use normal textures
        v.set_frame(0.0);
    }

    fn button_returned(&self) {
        assert_eq!(self.state.get(), MoveState::GoingToStart);
        self.state.set(MoveState::AtStart);

        let engine = self.engine();
        let sf = self.spawn_flags();
        if sf.intersects(SpawnFlags::TOGGLE) {
            let activator = self.activator.get().get_entity();
            self.delayed.use_targets(UseType::Toggle, activator, self);
        }

        if let Some(target) = self.target() {
            let activator = self.activator.get().get_entity();
            for target in engine.entities().by_target_name(&*target) {
                if !target.vars().is_class_name(c"multisource") {
                    continue;
                }
                if let Some(target) = target.get_entity() {
                    target.used(UseType::Toggle, activator, self);
                }
            }
        }

        self.enable_touch.set(sf.intersects(SpawnFlags::TOUCH_ONLY));
    }

    fn response_to_touch(&self) -> ButtonCode {
        match self.state.get() {
            MoveState::AtEnd => {
                let sf = self.spawn_flags();
                if sf.intersects(SpawnFlags::TOGGLE) && !self.stay_pushed() {
                    ButtonCode::Return
                } else {
                    ButtonCode::None
                }
            }
            MoveState::AtStart => ButtonCode::Activate,
            _ => ButtonCode::None,
        }
    }
}

impl<T: Move> Entity for BaseButton<T> {
    delegate_entity!(base not {
        object_caps, key_value, spawn, used, touched, think, take_damage,
    });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
            .union(if self.vars().take_damage() == TakeDamage::No {
                ObjectCaps::IMPULSE_USE
            } else {
                ObjectCaps::NONE
            })
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        let engine = self.engine();
        match data.key_name().to_bytes() {
            b"master" => self.master = Some(engine.new_map_string(data.value())),
            b"wait" => self.wait = data.parse_or_default(),
            b"changetarget" => warn!("{}: changetarget is not implemented", self.pretty_name()),
            b"sounds" => self.sounds = data.parse_or_default(),
            _ => {
                if self.button_move.key_value(data) {
                    return;
                }
                if self.delayed.key_value(data) {
                    return;
                }
                self.base.key_value(data);
                return;
            }
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let sf = self.spawn_flags();
        let v = self.base.vars();

        let sound = button_sound_or_default(self.sounds as usize);
        engine.precache_sound(sound);
        v.set_noise(engine.new_map_string(sound));

        if v.speed() == 0.0 {
            v.set_speed(40.0);
        }
        if v.health() > 0.0 {
            v.set_take_damage(TakeDamage::Yes);
        }

        if self.wait == 0.0 {
            self.wait = 1.0;
        }

        if sf.intersects(SpawnFlags::TOUCH_ONLY) {
            self.enable_touch.set(true);
        }
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, _: &dyn Entity) {
        let sf = self.spawn_flags();
        if sf.intersects(SpawnFlags::TOUCH_ONLY) {
            return;
        }
        if self.state.get().is_moving() {
            return;
        }
        self.activator.set(activator.map(|e| e.entity_handle()));
        if self.state.get() == MoveState::AtEnd {
            if !self.stay_pushed() && sf.intersects(SpawnFlags::TOGGLE) {
                let v = self.base.vars();
                if let Some(noise) = v.noise() {
                    let engine = self.engine();
                    engine.build_sound().channel_voice().emit_dyn(noise, v);
                }
                self.button_return();
            }
        } else {
            self.button_activate();
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if !self.enable_touch.get() {
            return;
        }

        self.activator.set(Some(other.entity_handle()));

        let code = self.response_to_touch();
        if code == ButtonCode::None {
            return;
        }

        let engine = self.engine();
        let v = self.vars();
        if !utils::is_master_triggered(&engine, self.master, Some(other)) {
            self.lock_sounds.play_button(true, v);
            return;
        }

        self.enable_touch.take();

        if code == ButtonCode::Return {
            if let Some(noise) = v.noise() {
                engine.build_sound().channel_voice().emit_dyn(&*noise, v);
            }
            self.delayed.use_targets(UseType::Toggle, Some(other), self);
            self.button_return();
        } else {
            self.button_activate();
        }
    }

    fn think(&self) {
        match self.think.get() {
            Think::None => {}
            Think::MoveDone => {
                self.think.take();
                self.move_done();
            }
            Think::ButtonReturn => {
                self.think.take();
                self.button_return();
            }
        }
    }

    fn take_damage(
        &self,
        _: f32,
        _: DamageFlags,
        _: &EntityVars,
        attacker: Option<&EntityVars>,
    ) -> bool {
        let code = self.response_to_touch();
        if code == ButtonCode::None {
            return false;
        }

        self.enable_touch.set(false);

        let Some(activator) = attacker.and_then(|v| v.entity_handle().get_entity()) else {
            return false;
        };
        self.activator.set(Some(activator.entity_handle()));

        if code == ButtonCode::Return {
            let v = self.vars();
            if let Some(noise) = v.noise() {
                self.engine()
                    .build_sound()
                    .channel_voice()
                    .emit_dyn(&*noise, v);
            }
            let sf = self.spawn_flags();
            if !sf.intersects(SpawnFlags::TOGGLE) {
                self.delayed
                    .use_targets(UseType::Toggle, Some(activator), self);
            }
            self.button_return();
        } else {
            self.button_activate();
        }

        false
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Button {
    base: BaseButton<LinearMove>,
}

impl_entity_cast!(Button);

impl CreateEntity for Button {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseButton::create(base),
        }
    }
}

impl Button {
    fn spawn_flags(&self) -> SpawnFlags {
        self.base.spawn_flags()
    }

    fn spark_if_off(&self) -> bool {
        self.spawn_flags().intersects(SpawnFlags::SPARK_IF_OFF) && self.base.is_off()
    }
}

impl Entity for Button {
    delegate_entity!(base not { key_value, precache, spawn, used, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        if self.base.lock_sounds.key_value(data) {
            return;
        }
        self.base.key_value(data);
    }

    fn precache(&mut self) {
        if self.spawn_flags().intersects(SpawnFlags::SPARK_IF_OFF) {
            Sparks::new(self.engine()).precache();
        }

        self.base.lock_sounds.precache();
    }

    fn spawn(&mut self) {
        self.base.spawn();
        self.precache();

        let sf = self.spawn_flags();
        let v = self.base.base.vars();
        v.set_move_dir_from_angles();
        v.set_move_type(MoveType::Push);
        v.set_solid(Solid::Bsp);
        v.reload_model();

        if self.base.button_move.lip() == 0.0 {
            self.base.button_move.set_lip(4.0);
        }
        self.base.button_move.init(v);

        let button_move = &mut self.base.button_move;
        let distance = (button_move.end() - button_move.start()).length();
        if distance < 1.0 || sf.intersects(SpawnFlags::DONT_MOVE) {
            button_move.set_end(button_move.start());
        }

        if sf.intersects(SpawnFlags::SPARK_IF_OFF) {
            v.set_next_think_time_from_now(0.5);
        }
    }

    fn used(&self, use_type: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
        self.base.used(use_type, activator, caller);

        if self.spark_if_off() {
            self.vars().set_next_think_time_from_now(0.5);
        }
    }

    fn think(&self) {
        self.base.think();

        if self.spark_if_off() {
            let engine = self.engine();
            let v = self.base.vars();
            Sparks::new(engine).emit(v.abs_min(), v);
            let time = engine.random_float(0.0, 1.5);
            v.set_next_think_time_from_last(0.1 + time);
        }
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct RotButton {
    base: BaseButton<AngularMove>,
}

impl_entity_cast!(RotButton);

impl CreateEntity for RotButton {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseButton::create(base),
        }
    }
}

impl RotButton {
    fn spawn_flags(&self) -> RotSpawnFlags {
        RotSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_move_dir_from_spawn_flags(&self) {
        let v = self.vars();
        let flags = self.spawn_flags();
        if flags.intersects(RotSpawnFlags::ROTATE_Z) {
            v.set_move_dir(vec3_t::Z);
        } else if flags.intersects(RotSpawnFlags::ROTATE_X) {
            v.set_move_dir(vec3_t::X);
        } else {
            v.set_move_dir(vec3_t::Y);
        }
    }
}

impl Entity for RotButton {
    delegate_entity!(base not { spawn });

    fn spawn(&mut self) {
        self.base.spawn();
        self.precache();

        let sf = self.spawn_flags();
        let v = self.base.base.vars();

        self.set_move_dir_from_spawn_flags();
        if sf.intersects(RotSpawnFlags::ROTATE_BACKWARDS) {
            v.with_move_dir(|x| -x);
        }
        v.set_move_type(MoveType::Push);

        if sf.intersects(RotSpawnFlags::NOT_SOLID) {
            v.set_solid(Solid::Not);
        } else {
            v.set_solid(Solid::Bsp);
        }
        v.reload_model();

        self.base.button_move.init(v);
    }
}

export_entity_default!("export-func_button", func_button, Button);
export_entity_default!("export-func_rot_button", func_rot_button, RotButton);
