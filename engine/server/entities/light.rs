use crate::{
    entities::point_entity::PointEntity,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, UseType,
    },
    export::export_entity_default,
    str::MapString,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Light {
    base: PointEntity,
    style: i32,
    pattern: Option<MapString>,
}

impl Light {
    pub const SF_START_OFF: u32 = 1;
}

impl_entity_cast!(Light);

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
    delegate_entity!(base not { key_value, spawn, used });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"style" => {
                self.style = data.value_str().parse().unwrap_or(0);
            }
            b"pitch" => {
                let v = self.vars();
                v.with_angles(|v| v.with_x(data.value_str().parse().unwrap_or(0.0)));
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
        if self.vars().target_name().is_none() {
            self.vars().delayed_remove();
        } else if self.style >= 32 {
            if self.vars().spawn_flags() & Self::SF_START_OFF != 0 {
                engine.light_style(self.style, c"a");
            } else if let Some(pattern) = self.pattern {
                engine.light_style(self.style, pattern);
            } else {
                engine.light_style(self.style, c"m");
            }
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        if self.style < 32 {
            return;
        }

        let engine = self.engine();
        let v = self.base.vars();
        let is_start_off = v.spawn_flags() & Self::SF_START_OFF != 0;
        if !use_type.should_toggle(!is_start_off) {
            return;
        }

        if is_start_off {
            if let Some(pattern) = self.pattern {
                engine.light_style(self.style, pattern);
            } else {
                engine.light_style(self.style, c"m");
            }
            v.with_spawn_flags(|f| f & !Self::SF_START_OFF);
        } else {
            engine.light_style(self.style, c"a");
            v.with_spawn_flags(|f| f | Self::SF_START_OFF);
        }
    }
}

export_entity_default!("export-light", light, Light);
export_entity_default!("export-light_spot", light_spot, Light);
