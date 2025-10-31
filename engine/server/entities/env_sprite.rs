use core::cell::Cell;

use bitflags::bitflags;
use xash3d_shared::{
    entity::{Effects, MoveType},
    ffi::common::vec3_t,
};

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, EntityHandle, ObjectCaps, Solid, UseType},
    export::export_entity_default,
    prelude::*,
    time::MapTime,
};

#[derive(Copy, Clone, Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None,
    Animate,
}

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const START_ON  = 1 << 0;
        const ONCE      = 1 << 1;
        const TEMPORARY = 1 << 15;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Sprite {
    base: PointEntity,
    max_frame: f32,
    last_time: Cell<MapTime>,
    think: Cell<Think>,
}

impl CreateEntity for Sprite {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            max_frame: 0.0,
            last_time: Default::default(),
            think: Default::default(),
        }
    }
}

impl Sprite {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_attachment(&self, entity: EntityHandle, attachment: i32) {
        let v = self.vars();
        v.set_skin(entity.entity_index().to_i32());
        v.set_body(attachment);
        v.set_move_type(MoveType::Follow);
        v.set_aim_entity(&entity);
    }

    fn set_next_animate_time(&self, relative: f32) -> f32 {
        let now = self.engine().globals.map_time();
        self.vars().set_next_think_time(now + relative);
        now - self.last_time.replace(now)
    }

    fn is_on(&self) -> bool {
        self.vars().effects() != Effects::NODRAW
    }

    fn turn_on(&self) {
        let sf = self.spawn_flags();
        let v = self.vars();
        v.remove_effects();
        v.set_frame(0.0);
        if (v.framerate() != 0.0 && self.max_frame > 1.0) || sf.intersects(SpawnFlags::ONCE) {
            self.think.set(Think::Animate);
            self.set_next_animate_time(0.0);
        }
    }

    fn turn_off(&self) {
        let v = self.vars();
        v.set_effects(Effects::NODRAW);
        v.stop_thinking();
    }

    fn animate(&self, frames: f32) {
        let v = self.vars();
        v.set_frame(v.frame() + frames);
        if self.max_frame <= v.frame() {
            if self.spawn_flags().intersects(SpawnFlags::ONCE) {
                self.turn_off();
            } else if self.max_frame > 0.0 {
                v.set_frame(v.frame() % self.max_frame);
            }
        }
    }
}

impl Entity for Sprite {
    delegate_entity!(base not { object_caps, precache, spawn, used, think });

    fn object_caps(&self) -> ObjectCaps {
        let mut caps = self.base.object_caps();
        if self.spawn_flags().intersects(SpawnFlags::TEMPORARY) {
            caps = caps.union(ObjectCaps::DONT_SAVE);
        }
        caps.difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn precache(&mut self) {
        let v = self.base.vars();
        v.precache_model();

        if let Some(aiment) = v.aim_entity() {
            self.set_attachment(aiment, v.body());
        } else {
            v.set_skin(0);
            v.set_body(0);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.remove_effects();
        v.set_frame(0.0);

        self.precache();

        let sf = self.spawn_flags();
        let v = self.base.vars();
        v.reload_model();

        if v.target_name().is_some() & !sf.intersects(SpawnFlags::START_ON) {
            self.turn_off();
        } else {
            self.turn_on();
        }

        // worldcraft only sets y rotation, copy to Z
        let angles = v.angles();
        if angles.y != 0.0 && angles.z == 0.0 {
            v.set_angles(vec3_t::new(angles.x, 0.0, angles.y));
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let on = self.is_on();
        if use_type.should_toggle(on) {
            if on {
                self.turn_off();
            } else {
                self.turn_on();
            }
        }
    }

    fn think(&self) {
        match self.think.get() {
            Think::None => {}
            Think::Animate => {
                let delta = self.set_next_animate_time(0.1);
                self.animate(self.vars().framerate() * delta);
            }
        }
    }
}

export_entity_default!("export-env_sprite", env_sprite, Sprite {});
