use xash3d_shared::sound::Attenuation;

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, KeyValue, UseType},
    export::export_entity_default,
    prelude::*,
    private::Private,
    str::MapString,
    utils,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Message {
    base: PointEntity,
}

impl_entity_cast!(Message);

impl CreateEntity for Message {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl Message {
    const SF_ONCE: u32 = 1 << 0;
    const SF_ALL: u32 = 1 << 1;

    pub fn show_once(engine: &ServerEngine, message: MapString) {
        engine
            .new_entity::<Private<Self>>()
            .vars(|v| {
                v.with_spawn_flags(|f| f | Message::SF_ONCE);
                v.set_message(message);
                v.set_next_think_time_from_now(0.3);
            })
            .build();
    }
}

impl Entity for Message {
    delegate_entity!(base not { key_value, precache, spawn, used, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        let v = self.base.vars();
        match data.key_name().to_bytes() {
            b"messagesound" => v.set_noise(self.engine().new_map_string(data.value())),
            b"messagevolume" => v.set_scale(data.parse_or_default::<f32>() * 0.1),
            b"messageattenuation" => v.set_impulse(data.parse_or_default()),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        if let Some(noise) = self.vars().noise() {
            self.engine().precache_sound(&*noise);
        }
    }

    fn spawn(&mut self) {
        self.precache();

        let v = self.base.vars();
        let attenuation = match v.impulse() {
            0 => Attenuation::IDLE,
            1 => Attenuation::STATIC,
            2 => Attenuation::NORM,
            3 => Attenuation::NONE,
            _ => Attenuation::IDLE,
        };
        v.set_speed(attenuation.into());

        if v.scale() <= 0.0 {
            v.set_scale(1.0);
        }
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, _: &dyn Entity) {
        let engine = self.engine();
        let v = self.vars();
        let sf = v.spawn_flags();

        if let Some(message) = v.message().as_deref() {
            if sf & Self::SF_ALL != 0 {
                utils::show_message_all(&engine, message.into());
            } else {
                let player = if activator.is_some_and(|i| i.is_player()) {
                    activator
                } else {
                    engine.get_single_player().get_entity()
                };

                if let Some(player) = player.and_then(|i| i.as_player()) {
                    utils::show_message(player, message.into())
                }
            }
        }

        if let Some(noise) = v.noise() {
            engine
                .build_sound()
                .channel_body()
                .volume(v.scale())
                .attenuation(v.speed())
                .emit_dyn(&*noise, v);
        }

        if sf & Self::SF_ONCE != 0 {
            self.remove_from_world();
        }

        utils::use_targets(UseType::Toggle, Some(self), self);
    }

    fn think(&self) {
        self.used(UseType::Toggle, Some(self), self);
    }
}

export_entity_default!("export-env_message", env_message, Message);
