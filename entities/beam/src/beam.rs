use core::ffi::CStr;

use bitflags::bitflags;
use csz::CStrThin;
use xash3d_server::{
    color::RGB,
    engine::TraceResult,
    entity::{
        BaseEntity, BeamEntity, EdictFlags, EntityHandle, EntityIndex, EntityVars, ObjectCaps,
        TakeDamage, delegate_entity,
    },
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
    str::MapString,
    utils,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct BeamKind(u8);

impl BeamKind {
    const POINTS: Self = Self(0);
    const POINT_ENTITY: Self = Self(1);
    const ENTITIES: Self = Self(2);
    const HOSE: Self = Self(3);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BeamType {
    // start pos => end pos
    Points(vec3_t, vec3_t),
    // start entity => end pos
    EntityPoint(EntityIndex, vec3_t),
    // start entity => end entity
    Entities(EntityIndex, EntityIndex),
    // start pos => direction
    Hose(vec3_t, vec3_t),
}

impl BeamType {
    pub fn new(start: &EntityVars, end: &EntityVars) -> Self {
        let is_start_point = is_point_entity(start) as u8;
        let is_end_point = is_point_entity(end) as u8;
        match (is_start_point << 1) | is_end_point {
            0b11 => Self::Points(start.origin(), end.origin()),
            0b10 => Self::EntityPoint(end.entity_index(), start.origin()),
            0b01 => Self::EntityPoint(start.entity_index(), end.origin()),
            _ => Self::Entities(start.entity_index(), end.entity_index()),
        }
    }
}

fn is_point_entity(v: &EntityVars) -> bool {
    if v.model_index().is_none() {
        return true;
    }
    if let Some(class_name) = v.classname() {
        const POINT_CLASS_NAMES: &[&CStr] = &[c"info_target", c"info_landmark", c"path_corner"];
        POINT_CLASS_NAMES.contains(&class_name.as_c_str())
    } else {
        false
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct BeamFlags: u8 {
        const SINE          = 1 << 4;
        const SOLID         = 1 << 5;
        const SHADE_IN      = 1 << 6;
        const SHADE_OUT     = 1 << 7;
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct SpawnFlags: u32 {
        const START_ON      = 1 << 0;
        const TOGGLE        = 1 << 1;
        const RANDOM        = 1 << 2;
        const RING          = 1 << 3;
        const SPARK_START   = 1 << 4;
        const SPARK_END     = 1 << 5;
        const DECALS        = 1 << 6;
        const SHADE_IN      = 1 << 7;
        const SHADE_OUT     = 1 << 8;
        const TEMPORARY     = 1 << 15;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Beam {
    base: BaseEntity,
}

impl CreateEntity for Beam {
    fn create(base: BaseEntity) -> Self {
        Self { base }
    }
}

impl Beam {
    pub const CLASS_NAME: &'static CStr = c"beam";

    pub fn new(engine: &ServerEngine, sprite_name: MapString, width: u8) -> &Beam {
        let beam = engine
            .new_entity_with::<Self>(|base| Self { base })
            .class_name(Self::CLASS_NAME)
            .vars(|v| {
                v.with_flags(|f| f.union(EdictFlags::CUSTOMENTITY));
                v.set_model_name(sprite_name);
                // reset start beam entity
                v.set_sequence(0);
                // reset end beam entity
                v.set_skin(0);
                // reset beam type
                v.set_render_mode_raw(0);
            })
            .build();
        beam.set_color(RGB::WHITE);
        beam.set_brightness(255);
        beam.set_noise(0);
        beam.set_frame_start(0);
        beam.set_scroll_rate(0);
        beam.set_texture(engine.precache_model(sprite_name) as u16);
        beam.set_width(width);
        beam
    }

    pub fn init(&self, ty: BeamType) {
        self.set_beam_type(ty);
        self.set_start_attachment(0);
        self.set_end_attachment(0);
        self.relink();
    }

    pub fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    pub fn start_entity(&self) -> EntityIndex {
        BeamEntity::from_bits(self.vars().sequence() as u16).index()
    }

    pub fn set_start_entity(&self, index: EntityIndex) {
        let v = self.vars();
        let start = BeamEntity::from_bits(v.sequence() as u16);
        v.set_sequence(start.copy_with_index(index).bits() as i32);
        v.set_owner(self.engine().get_entity_by_index(index).as_ref());
    }

    pub fn end_entity(&self) -> EntityIndex {
        BeamEntity::from_bits(self.vars().skin() as u16).index()
    }

    pub fn set_end_entity(&self, index: EntityIndex) {
        let v = self.vars();
        let end = BeamEntity::from_bits(v.skin() as u16);
        v.set_skin(end.copy_with_index(index).bits() as i32);
        v.set_aim_entity(self.engine().get_entity_by_index(index).as_ref());
    }

    pub fn set_start_attachment(&self, attachment: u16) {
        let v = self.vars();
        let beam = BeamEntity::from_bits(v.sequence() as u16);
        match BeamEntity::with_attachment(beam.index(), attachment) {
            Some(beam) => v.set_sequence(beam.bits() as i32),
            None => panic!("invalid start beam entity attachment {attachment}"),
        }
    }

    pub fn set_end_attachment(&self, attachment: u16) {
        let v = self.vars();
        let beam = BeamEntity::from_bits(v.skin() as u16);
        match BeamEntity::with_attachment(beam.index(), attachment) {
            Some(beam) => v.set_skin(beam.bits() as i32),
            None => panic!("invalid end beam entity attachment {attachment}"),
        }
    }

    pub fn start_pos(&self) -> vec3_t {
        if let BeamKind::ENTITIES = self.beam_kind() {
            if let Some(start) = self.engine().get_entity_by_index(self.start_entity()) {
                return start.vars().origin();
            } else {
                let name = self.pretty_name();
                error!("{name}: beam start entity does not exist");
            }
        }
        self.vars().origin()
    }

    pub fn set_start_pos(&self, start: vec3_t) {
        self.vars().set_origin(start);
    }

    pub fn end_pos(&self) -> vec3_t {
        if !matches!(self.beam_kind(), BeamKind::POINTS | BeamKind::HOSE) {
            if let Some(end) = self.engine().get_entity_by_index(self.end_entity()) {
                return end.vars().origin();
            }
        }
        self.vars().angles()
    }

    pub fn set_end_pos(&self, end: vec3_t) {
        self.vars().set_angles(end);
    }

    fn beam_kind(&self) -> BeamKind {
        BeamKind(self.vars().render_mode_raw() as u8 & 0x0f)
    }

    fn set_beam_kind(&self, kind: BeamKind) {
        let v = self.vars();
        v.set_render_mode_raw((v.render_mode_raw() & 0xf0) | kind.0 as i32);
    }

    pub fn set_beam_type(&self, ty: BeamType) {
        match ty {
            BeamType::Points(start, end) => {
                self.set_beam_kind(BeamKind::POINTS);
                self.set_start_pos(start);
                self.set_end_pos(end);
            }
            BeamType::EntityPoint(start, end) => {
                self.set_beam_kind(BeamKind::POINT_ENTITY);
                self.set_start_entity(start);
                self.set_end_pos(end);
            }
            BeamType::Entities(start, end) => {
                self.set_beam_kind(BeamKind::ENTITIES);
                self.set_start_entity(start);
                self.set_end_entity(end);
            }
            BeamType::Hose(start, direction) => {
                self.set_beam_kind(BeamKind::HOSE);
                self.set_start_pos(start);
                self.set_end_pos(direction);
            }
        }
    }

    pub fn texture(&self) -> u16 {
        self.vars().model_index_raw() as u16
    }

    pub fn set_texture(&self, sprite_index: u16) {
        self.vars().set_model_index_raw(sprite_index as i32);
    }

    pub fn width(&self) -> u8 {
        self.vars().scale() as u8
    }

    pub fn set_width(&self, width: u8) {
        self.vars().set_scale(width as f32);
    }

    pub fn noise(&self) -> u8 {
        self.vars().body() as u8
    }

    pub fn set_noise(&self, noise: u8) {
        self.vars().set_body(noise as i32);
    }

    pub fn frame_start(&self) -> u8 {
        self.vars().frame() as u8
    }

    pub fn set_frame_start(&self, frame_start: u8) {
        self.vars().set_frame(frame_start as f32);
    }

    pub fn scroll_rate(&self) -> u8 {
        self.vars().animation_time() as u8
    }

    pub fn set_scroll_rate(&self, scroll_rate: u8) {
        self.vars().set_animation_time(scroll_rate as f32);
    }

    pub fn beam_flags(&self) -> BeamFlags {
        BeamFlags::from_bits_truncate(self.vars().render_mode_raw() as u8)
    }

    pub fn set_beam_flags(&self, flags: BeamFlags) {
        let v = self.vars();
        let m = v.render_mode_raw();
        v.set_render_mode_raw((flags.bits() as i32 & 0xf0) | (m & 0x0f));
    }

    pub fn color(&self) -> RGB {
        self.vars().render_color_to_rgb()
    }

    pub fn set_color(&self, color: RGB) {
        self.vars().set_render_color_from_rgb(color);
    }

    pub fn brightness(&self) -> u8 {
        self.vars().render_amount() as u8
    }

    pub fn set_brightness(&self, brightness: u8) {
        self.vars().set_render_amount(brightness as f32);
    }

    pub fn relink(&self) {
        let start = self.start_pos();
        let end = self.end_pos();
        let v = self.vars();
        v.set_min_size(start.min(end) - v.origin());
        v.set_max_size(start.max(end) - v.origin());
        v.set_size_and_link(v.min_size(), v.max_size());
        v.link();
    }

    pub fn random_target_name(&self, target_name: &CStrThin) -> Option<EntityHandle> {
        let engine = self.engine();
        engine
            .entities()
            .by_target_name(target_name)
            .enumerate()
            .reduce(|a, b @ (index, _)| {
                if engine.random_int(0, index as i32) < 1 {
                    b
                } else {
                    a
                }
            })
            .map(|(_, entity)| entity.into())
    }

    pub fn beam_damage(&self, trace: &TraceResult) {
        self.relink();

        let engine = self.engine();
        let v = self.vars();
        let now = self.engine().globals.map_time();
        if trace.fraction() != 1.0 {
            if let Some(hit) = trace.hit_entity().get_entity() {
                // TODO: do beam damage
                // TODO: multi damage

                if hit.vars().take_damage() != TakeDamage::No {
                    let name = self.pretty_name();
                    warn!("{name}: beam damage is not implemented yet");
                }

                let sf = self.spawn_flags();
                if sf.intersects(SpawnFlags::DECALS) && hit.is_bsp_model() {
                    let global_state = self.global_state();
                    let decals = global_state.decals();
                    utils::decal_trace(&engine, trace, decals.get_random_bigshot());
                }
            }
        }
        v.set_damage_time(now);
    }

    pub fn beam_damage_instant(&self, trace: &TraceResult, damage: f32) {
        let v = self.vars();
        v.set_damage(damage);
        v.set_damage_time(self.engine().globals.map_time() - 1.0);
        self.beam_damage(trace);
    }

    pub fn do_sparks(&self, start: vec3_t, end: vec3_t) {
        let sf = self.spawn_flags();
        if sf.intersects(SpawnFlags::SPARK_START | SpawnFlags::SPARK_END) {
            let sparks = utils::Sparks::new(self.engine());
            if sf.intersects(SpawnFlags::SPARK_START) {
                sparks.emit_simple(start);
            }
            if sf.intersects(SpawnFlags::SPARK_END) {
                sparks.emit_simple(end);
            }
        }
    }
}

impl Entity for Beam {
    delegate_entity!(base not { object_caps, precache, spawn });

    fn object_caps(&self) -> ObjectCaps {
        let mut caps = self.base.object_caps();
        if self.spawn_flags().intersects(SpawnFlags::TEMPORARY) {
            caps.insert(ObjectCaps::DONT_SAVE);
        }
        caps.difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn precache(&mut self) {
        let v = self.base.vars();
        if let Some(owner) = v.owner() {
            self.set_start_entity(owner.entity_index());
        }
        if let Some(aim) = v.aim_entity() {
            self.set_end_entity(aim.entity_index());
        }
    }

    fn spawn(&mut self) {
        self.precache();
    }
}

impl_private!(Beam {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_beam {
    () => {
        $crate::export_entity!(beam, $crate::beam::Beam);
    };
}
#[doc(inline)]
pub use export_beam as export;
