use core::cell::Cell;

use xash3d_entity_sprite::env_sprite::Sprite;
use xash3d_server::{
    engine::{TraceIgnore, TraceResult},
    entity::{
        delegate_entity, BaseEntity, EdictFlags, Effects, Entity, EntityHandle, EntityVars,
        KeyValue, Solid, UseType,
    },
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
    render::RenderMode,
    str::MapString,
};

use crate::beam::{Beam, BeamType, SpawnFlags};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Laser {
    base: Beam,

    sprite_name: Option<MapString>,
    sprite: Option<EntityHandle>,

    fire_position: Cell<vec3_t>,
}

impl CreateEntity for Laser {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Beam::create(base),

            sprite_name: None,
            sprite: None,

            fire_position: Cell::default(),
        }
    }
}

impl Laser {
    fn spawn_flags(&self) -> SpawnFlags {
        self.base.spawn_flags()
    }

    fn is_on(&self) -> bool {
        !self.vars().effects().intersects(Effects::NODRAW)
    }

    fn turn_off(&self) {
        if let Some(sprite) = self.sprite.downcast_ref::<Sprite>() {
            sprite.turn_off();
        }
        let v = self.vars();
        v.with_effects(|f| f | Effects::NODRAW);
        v.stop_thinking();
    }

    fn turn_on(&self) {
        if let Some(sprite) = self.sprite.downcast_ref::<Sprite>() {
            sprite.turn_on();
        }
        let v = self.vars();
        v.with_effects(|f| f.difference(Effects::NODRAW));
        let now = self.engine().globals.map_time();
        v.set_damage_time(now);
        v.set_next_think_time(now);
    }

    fn fire_at_point(&self, trace: &TraceResult) {
        let end = trace.end_position();
        self.base.set_end_pos(end);
        if let Some(sprite) = self.sprite {
            sprite.vars().set_origin_and_link(end);
        }
        self.base.beam_damage(trace);
        self.base.do_sparks(self.base.start_pos(), end);
    }
}

impl Entity for Laser {
    delegate_entity!(base not { key_value, precache, spawn, used, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        let engine = self.engine();
        let v = self.base.vars();
        match data.key_name().to_bytes() {
            b"LaserTarget" => v.set_message(engine.new_map_string(data.value())),
            b"width" => self.base.set_width(data.parse_or_default::<f32>() as u8),
            b"NoiseAmplitude" => self.base.set_noise(data.parse_or_default()),
            b"texture" => v.set_model_name(engine.new_map_string(data.value())),
            b"TextureScroll" => self.base.set_scroll_rate(data.parse_or_default()),
            b"EndSprite" => self.sprite_name = Some(engine.new_map_string(data.value())),
            b"framestart" => v.set_frame(data.parse_or_default::<i32>() as f32),
            b"damage" => v.set_damage(data.parse_or_default()),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        let engine = self.engine();
        let v = self.vars();
        if let Some(model_name) = v.model_name() {
            v.set_model_index_raw(engine.precache_model(model_name));
        }
        if let Some(sprite_name) = self.sprite_name {
            engine.precache_model(sprite_name);
        }
    }

    fn spawn(&mut self) {
        if self.vars().model_name().is_none() {
            self.remove_from_world();
            return;
        }

        self.precache();

        let engine = self.engine();
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.with_flags(|f| f | EdictFlags::CUSTOMENTITY);

        self.base.init(BeamType::Points(v.origin(), v.origin()));

        if let (true, Some(sprite_name)) = (self.sprite.is_none(), self.sprite_name) {
            let sprite = Sprite::new(&engine, sprite_name, v.origin(), true);
            self.sprite = Some(sprite.entity_handle());
        } else {
            self.sprite = None;
        }

        if let Some(sprite) = self.sprite.downcast_ref::<Sprite>() {
            sprite.set_transparency(RenderMode::Glow, v.render_color_to_rgba(), v.render_fx());
        }

        let sf = self.spawn_flags();
        if v.target_name().is_some() && !sf.intersects(SpawnFlags::START_ON) {
            self.turn_off();
        } else {
            self.turn_on();
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let active = self.is_on();
        if use_type.should_toggle(active) {
            if active {
                self.turn_off();
            } else {
                self.turn_on();
            }
        }
    }

    fn think(&self) {
        let name = self.pretty_name();
        let engine = self.engine();
        let v = self.vars();
        let Some(end_name) = v.message() else {
            error!("{name}: end entity name is none");
            return;
        };

        let end = if let Some(end) = self.base.random_target_name(&end_name) {
            self.fire_position.set(end.vars().origin());
            end.vars().origin()
        } else {
            self.fire_position.get()
        };

        let trace = engine.trace_line(v.origin(), end, TraceIgnore::NONE, None::<&EntityVars>);
        self.fire_at_point(&trace);
        v.set_next_think_time_from_now(0.1);
    }
}

impl_private!(Laser {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_env_laser {
    () => {
        $crate::export_entity!(env_laser, $crate::env_laser::Laser);
    };
}
#[doc(inline)]
pub use export_env_laser as export;
