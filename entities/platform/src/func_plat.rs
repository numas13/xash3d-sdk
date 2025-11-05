use core::cell::Cell;

use xash3d_server::{
    entity::{
        BaseEntity, DamageFlags, EntityHandle, KeyValue, MoveType, ObjectCaps, Solid, UseType,
        delegate_entity,
    },
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
    sound::PlatformSounds,
    utils::{LinearMove, Move, MoveState},
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
struct PlatformTrigger {
    base: BaseEntity,
    platform: EntityHandle,
}

impl Entity for PlatformTrigger {
    delegate_entity!(base not { object_caps, touched });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
            .union(ObjectCaps::DONT_SAVE)
    }

    fn touched(&self, other: &dyn Entity) {
        if !other.is_player() {
            return;
        }

        let Some(platform) = self.platform.get_entity() else {
            self.remove_from_world();
            return;
        };

        if !other.is_alive() {
            return;
        }

        // TODO: define platform/moving trait
        let platform = platform
            .downcast_ref::<Platform>()
            .or_else(|| platform.downcast_ref::<RotatingPlatform>().map(|i| &i.base))
            .expect("Platform or RotatingPlatform");

        match platform.state.get() {
            MoveState::AtStart => platform.go_end_with_delay(1.0),
            MoveState::AtEnd => {
                // delay going to start
                platform.vars().set_next_think_time_from_last(1.0);
            }
            _ => {}
        }
    }
}

impl_private!(PlatformTrigger {});

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    MoveDone,
    GoStart,
    GoEnd,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Platform {
    pub(crate) base: BaseEntity,

    platform_sounds: PlatformSounds,
    wait: f32,

    height: f32,
    linear: LinearMove,

    pub(crate) rotation: f32,
    pub(crate) start_angles: vec3_t,
    pub(crate) end_angles: vec3_t,

    state: Cell<MoveState>,
    enable_use: Cell<bool>,
    think: Cell<Think>,
}

impl CreateEntity for Platform {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            platform_sounds: Default::default(),
            wait: 0.0,

            height: 0.0,
            linear: LinearMove::default(),

            rotation: 0.0,
            start_angles: vec3_t::ZERO,
            end_angles: vec3_t::ZERO,

            state: Cell::default(),
            enable_use: Cell::default(),
            think: Cell::default(),
        }
    }
}

impl Platform {
    const SF_TOGGLE: u32 = 1 << 0;

    fn is_toggle_platform(&self) -> bool {
        self.vars().spawn_flags() & Self::SF_TOGGLE != 0
    }

    fn rot_move(&self, dest: vec3_t) {
        let v = self.vars();
        let delta = dest - v.angles();
        let time = v.next_think_time() - v.last_think_time();
        if time >= 0.1 {
            v.set_angular_velocity(delta / time);
        } else {
            v.set_angular_velocity(delta);
            v.set_next_think_time_from_last(1.0);
        }
    }

    fn go_start(&self) {
        let v = self.vars();
        self.platform_sounds.emit_moving_noise(v);

        assert!(matches!(
            self.state.get(),
            MoveState::AtEnd | MoveState::GoingToEnd
        ));
        self.state.set(MoveState::GoingToStart);

        if self.linear.move_to_start(v, v.speed()) {
            self.hit_start();
        } else {
            self.think.set(Think::MoveDone);
        }

        if self.rotation != 0.0 {
            self.rot_move(self.start_angles);
        }
    }

    fn hit_start(&self) {
        let v = self.vars();
        self.platform_sounds.emit_moving_stop_noise(v);

        assert_eq!(self.state.get(), MoveState::GoingToStart);
        self.state.set(MoveState::AtStart);

        v.set_velocity(vec3_t::ZERO);

        if self.rotation != 0.0 {
            v.set_angular_velocity(vec3_t::ZERO);
            v.set_angles(self.start_angles);
        }
    }

    fn go_end_with_delay(&self, delay: f32) {
        if self.state.get() == MoveState::AtStart && self.think.get() != Think::GoEnd {
            self.think.set(Think::GoEnd);
            self.vars().set_next_think_time_from_last(delay);
        }
    }

    fn go_end(&self) {
        let v = self.vars();
        self.platform_sounds.emit_moving_noise(v);

        assert!(matches!(
            self.state.get(),
            MoveState::AtStart | MoveState::GoingToStart
        ));
        self.state.set(MoveState::GoingToEnd);

        if self.linear.move_to_end(v, v.speed(), false) {
            self.hit_end();
        } else {
            self.think.set(Think::MoveDone);
        }

        if self.rotation != 0.0 {
            self.rot_move(self.end_angles);
        }
    }

    fn hit_end(&self) {
        let v = self.vars();
        self.platform_sounds.emit_moving_stop_noise(v);

        assert_eq!(self.state.get(), MoveState::GoingToEnd);
        self.state.set(MoveState::AtEnd);

        if !self.is_toggle_platform() {
            self.think.set(Think::GoStart);
            self.vars().set_next_think_time_from_last(self.wait);
        }

        v.set_velocity(vec3_t::ZERO);

        if self.rotation != 0.0 {
            v.set_angular_velocity(vec3_t::ZERO);
            v.set_angles(self.end_angles);
        }
    }

    fn create_trigger(&self) {
        self.engine()
            .new_entity_with::<PlatformTrigger>(|base| PlatformTrigger {
                base,
                platform: self.entity_handle(),
            })
            .vars(|v| {
                let platform = self.vars();
                v.set_solid(Solid::Trigger);
                v.set_move_type(MoveType::None);
                v.set_origin(platform.origin());

                let mut min = platform.min_size() + vec3_t::new(25.0, 25.0, 0.0);
                let mut max = platform.max_size() + vec3_t::new(25.0, 25.0, 8.0);
                min.z = max.z - (self.linear.start().z - self.linear.end().z + 8.0);

                if platform.size().x <= 50.0 {
                    min.x = (platform.min_size().x + platform.max_size().x) / 2.0;
                    max.x = min.x + 1.0;
                }
                if platform.size().y <= 50.0 {
                    min.y = (platform.min_size().y + platform.max_size().y) / 2.0;
                    max.y = min.y + 1.0;
                }
                v.set_size_and_link(min, max);
            });
    }
}

impl Entity for Platform {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, used, blocked, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"wait" => self.wait = data.parse_or_default(),
            b"height" => self.height = data.parse_or_default(),
            _ => {
                if self.platform_sounds.key_value(data) {
                    return;
                }
                return self.base.key_value(data);
            }
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        self.platform_sounds.precache(self.base.vars());

        if !self.is_toggle_platform() {
            self.create_trigger();
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_angles(vec3_t::ZERO);
        v.set_solid(Solid::Bsp);
        v.set_move_type(MoveType::Push);
        v.link();
        v.set_size_and_link(v.min_size(), v.max_size());
        v.reload_model();

        if v.speed() == 0.0 {
            v.set_speed(150.0);
        }

        self.platform_sounds.init();

        if self.wait == 0.0 {
            self.wait = 3.0;
        }

        self.linear.set_start(v.origin());
        let mut end = v.origin();
        if self.height != 0.0 {
            end.z -= self.height;
        } else {
            end.z = end.z - v.size().z + 8.0;
        }
        self.linear.set_end(end);

        self.precache();

        let v = self.base.vars();
        if v.target_name().is_some() {
            v.set_origin_and_link(self.linear.start());
            self.state.set(MoveState::AtStart);
            self.enable_use.set(true);
        } else {
            v.set_origin_and_link(self.linear.end());
            self.state.set(MoveState::AtEnd);
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        if self.is_toggle_platform() {
            let state = self.state.get();
            if !use_type.should_toggle(state == MoveState::AtEnd) {
                return;
            }
            match state {
                MoveState::AtStart => self.go_end(),
                MoveState::AtEnd => self.go_start(),
                _ => {}
            }
        } else {
            self.enable_use.set(false);
            if self.state.get() == MoveState::AtStart {
                self.go_end();
            }
        }
    }

    fn blocked(&self, other: &dyn Entity) {
        trace!("{}: blocked by {}", self.pretty_name(), other.pretty_name());

        let v = self.vars();
        other.take_damage(1.0, DamageFlags::CRUSH, v, Some(v));

        self.platform_sounds.stop_moving_noise(v);

        match self.state.get() {
            MoveState::GoingToStart => self.go_end(),
            MoveState::GoingToEnd => self.go_start(),
            state => unreachable!("blocked: unexpected state {state:?}"),
        }
    }

    fn think(&self) {
        match self.think.take() {
            Think::None => {}
            Think::MoveDone => match self.state.get() {
                MoveState::GoingToStart => self.hit_start(),
                MoveState::GoingToEnd => self.hit_end(),
                state => unreachable!("think: unexpected state {state:?}"),
            },
            Think::GoEnd => self.go_end(),
            Think::GoStart => self.go_start(),
        }
    }
}

impl_private!(Platform {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_func_plat {
    () => {
        $crate::export_entity!(func_plat, $crate::func_plat::Platform);
    };
}
#[doc(inline)]
pub use export_func_plat as export;

use crate::func_platrot::RotatingPlatform;
