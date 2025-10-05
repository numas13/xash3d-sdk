use core::ffi::CStr;

use xash3d_shared::{consts::SOLID_NOT, entity::MoveType, ffi::server::TYPEDESCRIPTION};

use crate::{
    entities::subs::PointEntity,
    entity::{
        delegate_entity, impl_entity_cast, impl_save_restore, BaseEntity, CreateEntity, Entity,
    },
    save::{define_fields, SaveFields},
    str::MapString,
    time::MapTime,
};

pub struct Glow {
    base: PointEntity,
    last_time: MapTime,
    max_frame: f32,
}

unsafe impl SaveFields for Glow {
    const SAVE_NAME: &'static CStr = c"Glow";
    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![last_time, max_frame];
}

impl Glow {
    fn animate(&mut self, frames: f32) {
        if self.max_frame > 0.0 {
            let ev = self.base.vars_mut().as_raw_mut();
            ev.frame = (ev.frame + frames) % self.max_frame;
        }
    }
}

impl_entity_cast!(Glow);

impl CreateEntity for Glow {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            last_time: MapTime::ZERO,
            max_frame: 0.0,
        }
    }
}

impl Entity for Glow {
    delegate_entity!(base not { save, restore, spawn, think });
    impl_save_restore!(base);

    fn spawn(&mut self) {
        let engine = self.engine();
        let ev = self.base.vars_mut().as_raw_mut();

        ev.solid = SOLID_NOT;
        ev.movetype = MoveType::None.into();
        ev.effects = 0;
        ev.frame = 0.0;

        if let Some(model) = MapString::from_index(engine, ev.model) {
            engine.precache_model(&model);
            engine.set_model(self, &model);
        }

        let ev = self.base.vars_mut().as_raw_mut();
        self.max_frame = (engine.model_frames(ev.modelindex) - 1) as f32;
        if self.max_frame > 1.0 && ev.framerate != 0.0 {
            self.vars_mut().set_next_think_time(0.1);
        }
        self.last_time = engine.globals.map_time();
    }

    fn think(&mut self) {
        let engine = self.base.engine();
        let ev = self.base.vars().as_raw();
        let now = engine.globals.map_time();
        self.animate(ev.framerate * now.duration_since(self.last_time).as_secs_f32());

        self.vars_mut().set_next_think_time(0.1);
        self.last_time = engine.globals.map_time();
    }
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(env_glow, Private<super::Glow>);

    export_entity!(env_bubbles, Private<StubEntity>);
    export_entity!(beam, Private<StubEntity>);
    export_entity!(env_lightning, Private<StubEntity>);
    export_entity!(env_beam, Private<StubEntity>);
    export_entity!(env_laser, Private<StubEntity>);
    export_entity!(env_sprite, Private<StubEntity>);
    export_entity!(gibshooter, Private<StubEntity>);
    export_entity!(env_shooter, Private<StubEntity>);
    export_entity!(test_effect, Private<StubEntity>);
    export_entity!(env_blood, Private<StubEntity>);
    export_entity!(env_shake, Private<StubEntity>);
    export_entity!(env_fade, Private<StubEntity>);
}
