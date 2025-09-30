use core::ffi::CStr;

use xash3d_server::{
    consts::SOLID_NOT,
    entity::{
        delegate_entity, impl_save_restore, AsEdict, BaseEntity, CreateEntity, Entity, MoveType,
    },
    export::export_entity,
    ffi::server::TYPEDESCRIPTION,
    save::{define_fields, SaveFields, Time},
    str::MapString,
};

use crate::{entities::subs::PointEntity, entity::Private, impl_cast};

pub struct Glow {
    base: PointEntity,
    last_time: Time,
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

impl_cast!(Glow);

impl CreateEntity for Glow {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            last_time: Time(0.0),
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
            engine.set_model(self.as_edict_mut(), &model);
        }

        let ev = self.base.vars_mut().as_raw_mut();
        self.max_frame = (engine.model_frames(ev.modelindex) - 1) as f32;
        if self.max_frame > 1.0 && ev.framerate != 0.0 {
            self.vars_mut().set_next_think_time(0.1);
        }
        self.last_time = Time(engine.globals.map_time_f32());
    }

    fn think(&mut self) {
        let engine = self.base.engine();
        let ev = self.base.vars().as_raw();
        self.animate(ev.framerate * (engine.globals.map_time_f32() - self.last_time.0));

        self.vars_mut().set_next_think_time(0.1);
        self.last_time = Time(engine.globals.map_time_f32());
    }
}

export_entity!(env_glow, Private<Glow>);

// Lightning target, just alias landmark.
export_entity!(info_target, Private<PointEntity>);
