use bitflags::bitflags;
use xash3d_shared::{color::RGBA, entity::MoveType};

use crate::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, impl_entity_cast, BaseEntity, KeyValue, Solid, UseType},
    export::export_entity_default,
    prelude::*,
    utils::{self, ScreenFadeFlags},
};

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const FADE_IN   = 1 << 0;
        const MODULATE  = 1 << 1;
        const ONLY_ONE  = 1 << 2;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Fade {
    base: PointEntity,
}

impl_entity_cast!(Fade);

impl CreateEntity for Fade {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl Fade {
    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn duration(&self) -> f32 {
        self.vars().damage_take()
    }

    fn set_duration(&self, duration: f32) {
        self.vars().set_damage_take(duration)
    }

    fn hold_time(&self) -> f32 {
        self.vars().damage_save()
    }

    fn set_hold_time(&self, hold_time: f32) {
        self.vars().set_damage_take(hold_time);
    }
}

impl Entity for Fade {
    delegate_entity!(base not { key_value, spawn, used });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"duration" => self.set_duration(data.parse_or_default()),
            b"holdtime" => self.set_hold_time(data.parse_or_default()),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_move_type(MoveType::None);
        v.remove_effects();
        v.set_frame(0.0);
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, _: &dyn Entity) {
        let sf = self.spawn_flags();
        let v = self.vars();
        let render_color = v.render_color();
        let mut fade = utils::ScreenFade {
            duration: self.duration(),
            hold_time: self.hold_time(),
            flags: ScreenFadeFlags::empty(),
            color: RGBA::new(
                render_color.x as u8,
                render_color.y as u8,
                render_color.z as u8,
                v.render_amount() as u8,
            ),
        };
        if !sf.intersects(SpawnFlags::FADE_IN) {
            fade.flags.insert(ScreenFadeFlags::OUT);
        }
        if sf.intersects(SpawnFlags::MODULATE) {
            fade.flags.insert(ScreenFadeFlags::MODULATE);
        }

        if sf.intersects(SpawnFlags::ONLY_ONE) {
            if let Some(activator) = activator.and_then(|i| i.as_player()) {
                if activator.is_net_client() {
                    fade.emit_one(activator.vars());
                }
            }
        } else {
            fade.emit_all(&self.engine());
        }

        utils::use_targets(UseType::Toggle, Some(self), self);
    }
}

export_entity_default!("export-env_fade", env_fade, Fade {});
