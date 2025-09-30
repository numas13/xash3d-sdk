use core::ffi::CStr;

use xash3d_server::{
    engine::TraceIgnore,
    entity::{delegate_entity, impl_save_restore, BaseEntity, CreateEntity, Entity, UseType},
    export::export_entity,
    ffi::{
        common::vec3_t,
        server::{KeyValueData, TYPEDESCRIPTION},
    },
    prelude::*,
    save::{define_fields, SaveFields},
    str::MapString,
};

use crate::{
    entity::{impl_cast, Private},
    game_rules::install_game_rules,
};

pub struct Decal {
    base: BaseEntity,
    state: u8,
}

unsafe impl SaveFields for Decal {
    const SAVE_NAME: &'static CStr = c"Decal";

    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![state];
}

impl Decal {
    const SF_NOTINDEATHMATCH: i32 = 1 << 11;

    const STATE_STATIC: u8 = 1;
    const STATE_TRIGGER: u8 = 2;
    const STATE_REMOVE: u8 = 3;

    fn static_decal(&mut self) {
        let engine = self.engine();
        let ev = self.base.vars().as_raw();
        let mut trace = engine.trace_line(
            ev.origin - vec3_t::splat(5.0),
            ev.origin + vec3_t::splat(5.0),
            TraceIgnore::MONSTERS,
            Some(self),
        );
        let entity = engine.ent_index(trace.hit_entity_mut());
        let model_index = if !entity.is_zero() {
            trace.hit_entity().v.modelindex
        } else {
            0
        };
        let ev = self.base.vars().as_raw();
        engine.static_decal(ev.origin, ev.skin as u16, entity, model_index as u16);
    }
}

impl_cast!(Decal);

impl CreateEntity for Decal {
    fn create(base: BaseEntity) -> Self {
        Self { base, state: 0 }
    }
}

impl Entity for Decal {
    delegate_entity!(base not { key_value, save, restore, spawn, think, used });
    impl_save_restore!(base);

    fn key_value(&mut self, data: &mut KeyValueData) {
        if data.key_name() == c"texture" {
            let engine = self.engine();
            let ev = self.vars_mut().as_raw_mut();
            if let Some(skin) = engine.decal_index(data.value()) {
                ev.skin = skin.into();
                data.set_handled(true);
            } else {
                warn!("failed to find decal {}", data.value());
            }
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let ev = self.vars().as_raw();
        if ev.skin < 0
            || (engine.globals.is_deathmatch() && ev.spawnflags & Self::SF_NOTINDEATHMATCH != 0)
        {
            self.vars_mut().delayed_remove();
            return;
        }

        if MapString::is_null_or_empty(engine, ev.targetname) {
            self.state = Self::STATE_STATIC;
            // spawn the decal as soon as the world is done spawning
            self.vars_mut().set_next_think_time(0.0);
        } else {
            self.state = Self::STATE_TRIGGER;
        }
    }

    fn think(&mut self) {
        match self.state {
            Self::STATE_STATIC => {
                self.state = 0;
                self.static_decal();
                self.remove_from_world();
            }
            Self::STATE_REMOVE => {
                self.state = 0;
                self.remove_from_world();
            }
            _ => {}
        }
    }

    #[allow(unused_variables)]
    fn used(&mut self, other: &mut dyn Entity, use_type: UseType, value: f32) {
        if self.state != Self::STATE_TRIGGER {
            return;
        }

        // TODO: decal trigger
        warn!("{}: used is not implemented", self.classname());

        self.state = Self::STATE_REMOVE;
        self.vars_mut().set_next_think_time(0.1);
    }
}

export_entity!(infodecal, Private<Decal>);

pub struct World {
    base: BaseEntity,
}

impl_cast!(World);

impl CreateEntity for World {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Entity for World {
    delegate_entity!(base not { key_value, precache, spawn });

    fn key_value(&mut self, data: &mut KeyValueData) {
        let class_name = data.class_name();
        let key_name = data.key_name();
        let value = data.value();
        let handled = data.handled();
        debug!("World::key_value({class_name:?}, {key_name}, {value}, {handled})");
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.base.engine;
        engine.set_cvar(c"sv_gravity", c"800");
        engine.set_cvar(c"sv_stepsize", c"18");
        engine.set_cvar(c"room_type", c"0");
        install_game_rules(engine);
    }

    fn spawn(&mut self) {
        // TODO: global_game_over = false;
        self.precache();
    }
}

export_entity!(worldspawn, Private<World>);
