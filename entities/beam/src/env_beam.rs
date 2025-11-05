use core::cell::Cell;

use csz::CStrThin;
use xash3d_server::{
    color::RGBA,
    engine::TraceIgnore,
    entity::{
        BaseEntity, EdictFlags, Effects, Entity, EntityVars, KeyValue, Solid, UseType,
        delegate_entity,
    },
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
    render::RenderMode,
    str::MapString,
    user_message::{self, FixedU8},
};

use crate::beam::{Beam, BeamFlags, BeamType, SpawnFlags};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Think {
    #[default]
    None = 0,
    Damage,
    Strike,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u8)]
enum Used {
    #[default]
    None = 0,
    Toggle,
    Strike,
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct EnvBeam {
    base: Beam,

    sprite_name: Option<MapString>,
    start_entity: Option<MapString>,
    end_entity: Option<MapString>,
    radius: f32,
    life: f32,
    bolt_width: u8,
    noise_amplitude: u8,
    scroll_speed: u8,
    start_frame: u8,
    restrike: f32,

    sprite_texture: u16,

    active: Cell<bool>,
    think: Cell<Think>,
    used: Cell<Used>,
}

impl CreateEntity for EnvBeam {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Beam::create(base),

            sprite_name: None,
            start_entity: None,
            end_entity: None,
            radius: 0.0,
            life: 0.0,
            bolt_width: 0,
            noise_amplitude: 0,
            scroll_speed: 0,
            start_frame: 0,
            restrike: 0.0,

            sprite_texture: 0,

            active: Cell::default(),
            think: Cell::default(),
            used: Cell::default(),
        }
    }
}

impl EnvBeam {
    fn spawn_flags(&self) -> SpawnFlags {
        self.base.spawn_flags()
    }

    fn is_server_side(&self) -> bool {
        let sf = self.spawn_flags();
        self.life == 0.0 && !sf.intersects(SpawnFlags::RING)
    }

    fn update_vars(&self) {
        let engine = self.engine();
        let v = self.vars();
        v.set_skin(0);
        v.set_sequence(0);
        v.set_render_mode(RenderMode::Normal);
        v.with_flags(|f| f | EdictFlags::CUSTOMENTITY);
        v.set_model_name(self.sprite_name);
        self.base.set_texture(self.sprite_texture);

        let entities = engine.entities();
        let start = self
            .start_entity
            .and_then(|i| entities.by_target_name(i).first())
            .unwrap_or_else(|| engine.get_world_spawn_entity());
        let end = self
            .end_entity
            .and_then(|i| entities.by_target_name(i).first())
            .unwrap_or_else(|| engine.get_world_spawn_entity());

        self.base
            .set_beam_type(BeamType::new(&start.vars(), &end.vars()));
        self.base.relink();
        self.base.set_width(self.bolt_width);
        self.base.set_noise(self.noise_amplitude);
        self.base.set_frame_start(self.start_frame);
        self.base.set_scroll_rate(self.scroll_speed);
        let sf = self.spawn_flags();
        if sf.intersects(SpawnFlags::SHADE_IN) {
            self.base.set_beam_flags(BeamFlags::SHADE_IN);
        } else if sf.intersects(SpawnFlags::SHADE_OUT) {
            self.base.set_beam_flags(BeamFlags::SHADE_OUT);
        }
    }

    fn damage(&self) {
        let engine = self.engine();
        let v = self.vars();
        v.set_next_think_time_from_now(0.1);
        let start = self.base.start_pos();
        let end = self.base.end_pos();
        let trace = engine.trace_line(start, end, TraceIgnore::NONE, None::<&EntityVars>);
        self.base.beam_damage(&trace);
    }

    fn zap(&self, start: vec3_t, end: vec3_t) {
        let engine = self.engine();
        let v = self.vars();
        let &[r, g, b] = v.render_color().as_ref();
        let a = v.render_amount();
        let color = RGBA::new(r as u8, g as u8, b as u8, a as u8);
        engine.msg_broadcast(&user_message::BeamPoints {
            start: start.into(),
            end: end.into(),
            sprite_index: self.sprite_texture,
            start_frame: self.start_frame,
            frame_rate: FixedU8::from_bits(v.framerate() as u8),
            duration: self.life.into(),
            line_width: FixedU8::from_bits(self.bolt_width),
            noise_amplitude: FixedU8::from_bits(self.noise_amplitude),
            color,
            scroll_speed: FixedU8::from_bits(self.scroll_speed),
        });
        self.base.do_sparks(start, end);
    }

    fn random_area(&self) {
        let engine = self.engine();
        let v = self.vars();
        let start = v.origin();
        for _ in 0..10 {
            let dir1 = engine.random_vec3(-1.0, 1.0).normalize();
            let end1 = start + dir1 * self.radius;
            let tr1 = engine.trace_line(start, end1, TraceIgnore::MONSTERS, Some(v));
            if tr1.fraction() == 1.0 {
                continue;
            }

            let mut dir2;
            loop {
                dir2 = engine.random_vec3(-1.0, 1.0);
                if dir1.dot(dir2) > 0.0 {
                    break;
                }
            }
            dir2 = dir2.normalize();
            let end2 = start + dir2 * self.radius;
            let tr2 = engine.trace_line(start, end2, TraceIgnore::MONSTERS, Some(v));
            if tr2.fraction() == 1.0 {
                continue;
            }
            if (tr1.end_position() - tr2.end_position()).length() < self.radius * 0.1 {
                continue;
            }

            let tr2 = engine.trace_line(
                tr1.end_position(),
                tr2.end_position(),
                TraceIgnore::MONSTERS,
                Some(v),
            );
            if tr2.fraction() != 1.0 {
                continue;
            }

            self.zap(tr1.end_position(), tr2.end_position());
            break;
        }
    }

    fn random_point(&self, start: vec3_t) {
        let engine = self.engine();
        let v = self.vars();
        for _ in 0..10 {
            let end = start + engine.random_vec3(-1.0, 1.0).normalize() * self.radius;
            let tr = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(v));
            if (tr.end_position() - start).length() < self.radius * 0.1 {
                continue;
            }
            if tr.fraction() == 1.0 {
                continue;
            }
            self.zap(start, tr.end_position());
            break;
        }
    }

    fn strike_entities(&self, start_name: &CStrThin, end_name: &CStrThin) {
        let Some(start) = self.base.random_target_name(start_name).get_entity() else {
            return;
        };
        let Some(end) = self.base.random_target_name(end_name).get_entity() else {
            return;
        };
        let start_v = start.vars();
        let end_v = end.vars();

        let engine = self.engine();
        let v = self.vars();
        let sf = self.spawn_flags();

        let sprite_index = self.sprite_texture;
        let start_frame = self.start_frame;
        let frame_rate = FixedU8::from_bits(v.framerate() as u8);
        let duration = self.life.into();
        let line_width = FixedU8::from_bits(self.bolt_width);
        let noise_amplitude = FixedU8::from_bits(self.noise_amplitude);
        let color = v.render_color_to_rgba();
        let scroll_speed = FixedU8::from_bits(self.scroll_speed);

        match BeamType::new(start_v, end_v) {
            BeamType::Points(start, end) => {
                if sf.intersects(SpawnFlags::RING) {
                    // do not work
                    return;
                }
                let msg = user_message::BeamPoints {
                    start: start.into(),
                    end: end.into(),
                    sprite_index,
                    start_frame,
                    frame_rate,
                    duration,
                    line_width,
                    noise_amplitude,
                    color,
                    scroll_speed,
                };
                engine.msg_broadcast(&msg);
            }
            BeamType::EntityPoint(start, end) => {
                if sf.intersects(SpawnFlags::RING) {
                    // do not work
                    return;
                }
                let msg = user_message::BeamEntPoint {
                    start: start.into(),
                    end: end.into(),
                    sprite_index,
                    start_frame,
                    frame_rate,
                    duration,
                    line_width,
                    noise_amplitude,
                    color,
                    scroll_speed,
                };
                engine.msg_broadcast(&msg);
            }
            BeamType::Entities(start, end) if sf.intersects(SpawnFlags::RING) => {
                let msg = user_message::BeamRing {
                    start: start.into(),
                    end: end.into(),
                    sprite_index,
                    start_frame,
                    frame_rate,
                    duration,
                    line_width,
                    noise_amplitude,
                    color,
                    scroll_speed,
                };
                engine.msg_broadcast(&msg);
            }
            BeamType::Entities(start, end) => {
                let msg = user_message::BeamEnts {
                    start: start.into(),
                    end: end.into(),
                    sprite_index,
                    start_frame,
                    frame_rate,
                    duration,
                    line_width,
                    noise_amplitude,
                    color,
                    scroll_speed,
                };
                engine.msg_broadcast(&msg);
            }
            BeamType::Hose(..) => unreachable!("beam type hose"),
        }

        self.base.do_sparks(start_v.origin(), end_v.origin());

        if v.damage() > 0.0 {
            let start = start_v.origin();
            let end = end_v.origin();
            let trace = engine.trace_line(start, end, TraceIgnore::NONE, None::<&EntityVars>);
            self.base.beam_damage_instant(&trace, v.damage());
        }
    }

    fn strike(&self) {
        let engine = self.engine();
        let v = self.vars();
        let sf = self.spawn_flags();

        if self.life != 0.0 {
            let restrike = if sf.intersects(SpawnFlags::RANDOM) {
                engine.random_float(0.0, self.restrike)
            } else {
                self.restrike
            };
            v.set_next_think_time_from_now(self.life + restrike)
        }

        self.active.set(true);

        if let Some(end_name) = self.end_entity.as_deref() {
            if let Some(start) = self.start_entity.as_deref() {
                self.strike_entities(start, end_name);
            } else {
                let name = self.pretty_name();
                error!("{name}: start entity name is none");
            }
        } else if let Some(start_name) = self.start_entity.as_deref() {
            if let Some(start) = self.base.random_target_name(start_name) {
                self.random_point(start.vars().origin());
            } else {
                let name = self.pretty_name();
                warn!("{name}: failed to find a start entity {start_name:?}");
            }
        } else {
            self.random_area();
        }
    }
}

impl Entity for EnvBeam {
    delegate_entity!(base not { key_value, precache, spawn, activate, used, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        let engine = self.engine();
        let v = self.base.vars();
        match data.key_name().to_bytes() {
            b"texture" => self.sprite_name = Some(engine.new_map_string(data.value())),
            b"LightningStart" => self.start_entity = Some(engine.new_map_string(data.value())),
            b"LightningEnd" => self.end_entity = Some(engine.new_map_string(data.value())),
            b"Radius" => self.radius = data.parse_or_default(),
            b"life" => self.life = data.parse_or_default(),
            b"BoltWidth" => self.bolt_width = data.parse_or_default(),
            b"NoiseAmplitude" => {
                // FIXME: some maps (like c1a0c) have invalid values for this field
                self.noise_amplitude = data.parse_or_default();
            }
            b"TextureScroll" => self.scroll_speed = data.parse_or_default(),
            b"framestart" => self.start_frame = data.parse_or_default(),
            b"StrikeTime" => self.restrike = data.parse_or_default(),
            b"damage" => v.set_damage(data.parse_or_default()),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn precache(&mut self) {
        if let Some(sprite_name) = self.sprite_name {
            self.sprite_texture = self.engine().precache_model(sprite_name) as u16;
        }
        self.base.precache();
    }

    fn spawn(&mut self) {
        if self.sprite_name.is_none() {
            self.remove_from_world();
            return;
        }

        self.precache();

        let sf = self.spawn_flags();
        let engine = self.engine();
        let v = self.base.vars();
        v.set_solid(Solid::Not);
        v.set_damage_time(engine.globals.map_time());

        if self.is_server_side() {
            if v.damage() > 0.0 {
                self.think.set(Think::Damage);
                v.set_next_think_time_from_now(0.1);
            }
            if v.target_name().is_some() {
                if sf.intersects(SpawnFlags::START_ON) {
                    self.active.set(true);
                } else {
                    v.set_effects(Effects::NODRAW);
                    v.stop_thinking();
                }
                self.used.set(Used::Toggle);
            }
        } else {
            if v.target_name().is_some() {
                self.used.set(Used::Strike);
            }
            if v.target_name().is_none() || sf.intersects(SpawnFlags::START_ON) {
                self.think.set(Think::Strike);
                v.set_next_think_time_from_now(1.0);
            }
        }
    }

    fn activate(&self) {
        if self.is_server_side() {
            self.update_vars();
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        let v = self.vars();
        match self.used.get() {
            Used::None => {}
            Used::Toggle => {
                if !use_type.should_toggle(self.active.get()) {
                    return;
                }

                if self.active.get() {
                    self.active.set(false);
                    v.with_effects(|f| f.union(Effects::NODRAW));
                    v.stop_thinking();
                } else {
                    self.active.set(true);
                    v.with_effects(|f| f.difference(Effects::NODRAW));
                    let start = self.base.start_pos();
                    let end = self.base.end_pos();
                    self.base.do_sparks(start, end);
                    if v.damage() > 0.0 {
                        let now = self.engine().globals.map_time();
                        v.set_next_think_time(now);
                        v.set_damage_time(now);
                    }
                }
            }
            Used::Strike => {
                if !use_type.should_toggle(self.active.get()) {
                    return;
                }

                if self.active.get() {
                    self.active.set(false);
                    self.think.set(Think::None);
                    v.stop_thinking();
                } else {
                    self.think.set(Think::Strike);
                    v.set_next_think_time_from_now(0.1);
                }

                let sf = self.spawn_flags();
                if !sf.intersects(SpawnFlags::TOGGLE) {
                    self.used.set(Used::None);
                }
            }
        }
    }

    fn think(&self) {
        match self.think.get() {
            Think::None => {}
            Think::Damage => self.damage(),
            Think::Strike => self.strike(),
        }
    }
}

impl_private!(EnvBeam {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_env_beam {
    () => {
        $crate::export_entity!(env_beam, $crate::env_beam::EnvBeam);
    };
}
#[doc(inline)]
pub use export_env_beam as export;
