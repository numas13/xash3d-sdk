use core::cell::Cell;

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entities::subs::PointEntity,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue, UseType,
    },
    export::export_entity_default,
    str::MapString,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Light {
    base: PointEntity,
    style: Cell<i32>,
    pattern: Cell<Option<MapString>>,
}

impl Light {
    pub const SF_START_OFF: u32 = 1;
}

impl_entity_cast!(Light);

impl CreateEntity for Light {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            style: Cell::new(0),
            pattern: Cell::new(None),
        }
    }
}

impl Entity for Light {
    delegate_entity!(base not { key_value, spawn, used });

    fn key_value(&self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"style" => {
                self.style.set(data.value_str().parse().unwrap_or(0));
            }
            b"pitch" => {
                let v = self.vars();
                v.with_angles(|v| v.with_x(data.value_str().parse().unwrap_or(0.0)));
            }
            b"pattern" => {
                let engine = self.engine();
                self.pattern.set(engine.try_alloc_map_string(data.value()));
            }
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&self) {
        let engine = self.engine();
        if self.vars().target_name().is_none() {
            self.vars().delayed_remove();
        } else if self.style.get() >= 32 {
            if self.vars().spawn_flags() & Self::SF_START_OFF != 0 {
                engine.light_style(self.style.get(), c"a");
            } else if let Some(pattern) = self.pattern.get() {
                engine.light_style(self.style.get(), pattern);
            } else {
                engine.light_style(self.style.get(), c"m");
            }
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let style = self.style.get();
        if style < 32 {
            return;
        }

        let engine = self.engine();
        let v = self.base.vars();
        let is_start_off = v.spawn_flags() & Self::SF_START_OFF != 0;
        if !use_type.should_toggle(!is_start_off) {
            return;
        }

        if is_start_off {
            if let Some(pattern) = self.pattern.get() {
                engine.light_style(style, pattern);
            } else {
                engine.light_style(style, c"m");
            }
            v.with_spawn_flags(|f| f & !Self::SF_START_OFF);
        } else {
            engine.light_style(style, c"a");
            v.with_spawn_flags(|f| f | Self::SF_START_OFF);
        }
    }
}

export_entity_default!("export-light", light, Light);
export_entity_default!("export-light_spot", light_spot, Light);
