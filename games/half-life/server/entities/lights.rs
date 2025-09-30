use core::ffi::CStr;

use xash3d_server::{
    entity::{
        delegate_entity, impl_save_restore, BaseEntity, CreateEntity, Entity, KeyValue, UseType,
    },
    export::export_entity,
    ffi::server::TYPEDESCRIPTION,
    save::{define_fields, SaveFields},
    str::MapString,
};

use crate::{entities::subs::PointEntity, entity::Private, impl_cast};

pub struct Light {
    base: PointEntity,
    style: i32,
    pattern: Option<MapString>,
}

unsafe impl SaveFields for Light {
    const SAVE_NAME: &'static CStr = c"Light";

    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![style, pattern];
}

impl Light {
    pub const SF_START_OFF: i32 = 1;
}

impl_cast!(Light);

impl CreateEntity for Light {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            style: 0,
            pattern: None,
        }
    }
}

impl Entity for Light {
    delegate_entity!(base not { key_value, save, restore, spawn, used });
    impl_save_restore!(base);

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"style" => {
                self.style = data.value_str().parse().unwrap_or(0);
            }
            b"pitch" => {
                let ev = self.vars_mut().as_raw_mut();
                ev.angles.set_x(data.value_str().parse().unwrap_or(0.0));
            }
            b"pattern" => {
                let engine = self.engine();
                self.pattern = engine.try_alloc_map_string(data.value());
            }
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        if MapString::from_index(engine, self.vars().as_raw().targetname).is_none() {
            self.vars_mut().delayed_remove();
        } else if self.style >= 32 {
            let ev = self.vars_mut().as_raw_mut();
            if ev.spawnflags & Self::SF_START_OFF != 0 {
                engine.light_style(self.style, c"a");
            } else if let Some(pattern) = self.pattern {
                engine.light_style(self.style, &pattern);
            } else {
                engine.light_style(self.style, c"m");
            }
        }
    }

    fn used(&mut self, _: &mut dyn Entity, use_type: UseType, _: f32) {
        if self.style < 32 {
            return;
        }

        let engine = self.engine();
        let ev = self.vars_mut().as_raw_mut();
        let is_start_off = ev.spawnflags & Self::SF_START_OFF != 0;
        if !use_type.should_toggle(!is_start_off) {
            return;
        }

        if is_start_off {
            if let Some(pattern) = self.pattern {
                engine.light_style(self.style, &pattern);
            } else {
                engine.light_style(self.style, c"m");
            }
            let ev = self.vars_mut().as_raw_mut();
            ev.spawnflags &= !Self::SF_START_OFF;
        } else {
            engine.light_style(self.style, c"a");
            let ev = self.vars_mut().as_raw_mut();
            ev.spawnflags |= Self::SF_START_OFF;
        }
    }
}

export_entity!(light, Private<Light>);
export_entity!(light_spot, Private<Light>);
