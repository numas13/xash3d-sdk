use xash3d_shared::math::powf;

use crate::{
    entities::light::Light,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, KeyValue},
    export::export_entity_default,
    prelude::*,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct EnvLight {
    base: Light,
}

impl_entity_cast!(EnvLight);

impl CreateEntity for EnvLight {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Light::create(base),
        }
    }
}

impl Entity for EnvLight {
    delegate_entity!(base not { key_value, spawn });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"_light" {
            let mut rgb = [0; 3];
            for (i, s) in data.value_str().split_ascii_whitespace().enumerate() {
                let Ok(value) = s.parse() else {
                    let name = self.pretty_name();
                    let key = data.key_name();
                    let value = data.value();
                    error!("{name}: failed to parse key={key:?} value={value:?}");
                    return;
                };

                if i == 3 {
                    let f = value as f32 / 255.0;
                    rgb[0] = (rgb[0] as f32 * f) as u32;
                    rgb[1] = (rgb[1] as f32 * f) as u32;
                    rgb[2] = (rgb[2] as f32 * f) as u32;
                    break;
                }

                rgb[i] = value;
                if i == 0 {
                    rgb[1] = value;
                    rgb[2] = value;
                }
            }

            // simulate qrad direct, ambient and gamma adjustments
            rgb[0] = (powf(rgb[0] as f32 / 114.0, 0.6) * 264.0) as u32;
            rgb[1] = (powf(rgb[1] as f32 / 114.0, 0.6) * 264.0) as u32;
            rgb[2] = (powf(rgb[2] as f32 / 114.0, 0.6) * 264.0) as u32;

            let engine = self.engine();
            engine.set_cvar(c"sv_skycolor_r", rgb[0]);
            engine.set_cvar(c"sv_skycolor_g", rgb[1]);
            engine.set_cvar(c"sv_skycolor_b", rgb[2]);
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let v = self.base.vars();
        let angles = v.angles();
        let forward = angles.with_x(-angles.x).angle_vectors().forward();
        engine.set_cvar(c"sv_skyvec_x", forward.x);
        engine.set_cvar(c"sv_skyvec_y", forward.y);
        engine.set_cvar(c"sv_skyvec_z", forward.z);

        self.base.spawn();
    }
}

export_entity_default!("export-light_environment", light_environment, EnvLight);
