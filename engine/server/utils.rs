use core::{cell::Cell, ffi::CStr, mem};

use xash3d_shared::{
    color::RGBA,
    csz::{self, CStrSlice, CStrThin},
    entity::EdictFlags,
    ffi::common::vec3_t,
    str::ToEngineStr,
};

use crate::{
    engine::TraceResult,
    entity::{EntityPlayer, EntityVars, KeyValue, ObjectCaps, UseType},
    prelude::*,
    save::{PositionVector, Restore, Save},
    str::MapString,
    user_message,
};

pub use xash3d_shared::utils::*;

#[cfg(feature = "save")]
use crate::save;

/// Used for view cone checking.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ViewField(f32);

impl ViewField {
    /// +-180 degrees
    pub const FULL: Self = Self(-1.0);
    /// +-135 degrees
    pub const WIDE: Self = Self(-0.7);
    /// +-85 degrees
    pub const FOV: Self = Self(0.09);
    /// +-45 degrees
    pub const NARROW: Self = Self(0.7);
    /// +-25 degrees
    pub const ULTRA_NARROW: Self = Self(0.9);

    pub fn from_degress(degrees: f32) -> Self {
        use xash3d_shared::math::cosf;
        Self(cosf(degrees.to_radians()))
    }

    pub fn to_dot(self) -> f32 {
        self.0
    }
}

#[cfg(feature = "save")]
impl Save for ViewField {
    fn save(&self, _: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        cur.write_f32(self.0)
    }
}

#[cfg(feature = "save")]
impl Restore for ViewField {
    fn restore(&mut self, _: &save::RestoreState, cur: &mut save::Cursor) -> save::SaveResult<()> {
        self.0 = cur.read_f32()?;
        Ok(())
    }
}

pub fn is_master_triggered(
    engine: &ServerEngine,
    master: Option<MapString>,
    activator: Option<&dyn Entity>,
) -> bool {
    if let Some(master) = master.as_deref() {
        if master.is_empty() {
            return true;
        }
        if let Some(master) = engine.entities().by_target_name(master).first() {
            if let Some(master) = master.get_entity() {
                if master.object_caps().intersects(ObjectCaps::MASTER) {
                    return master.is_triggered(activator);
                }
            }
        }
    }
    true
}

/// Fire targets by the given target name.
pub fn fire_targets(
    target_name: &CStrThin,
    use_type: UseType,
    activator: Option<&dyn Entity>,
    caller: &dyn Entity,
) {
    if target_name.is_empty() {
        return;
    }
    let engine = caller.engine();
    trace!(target: "fire_targets", "Fire targets {target_name} by {}", caller.pretty_name());
    for target in engine.entities().by_target_name(target_name) {
        if target.is_free() {
            continue;
        }
        if let Some(target) = target.get_entity() {
            trace!(target: "fire_targets", "Firing {}", target.pretty_name());
            target.used(use_type, activator, caller);
        }
    }
}

/// Fire targets by a target from the caller.
pub fn use_targets(use_type: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
    if let Some(target) = caller.vars().target() {
        fire_targets(&target, use_type, activator, caller);
    }
}

/// Kill entities by the given target name.
pub fn kill_targets(engine: &ServerEngine, kill_target: &CStrThin) {
    if kill_target.is_empty() {
        return;
    }
    trace!(target: "kill_targets", "Kill targets {kill_target}");
    for target in engine.entities().by_target_name(kill_target) {
        if let Some(target) = target.get_entity() {
            trace!(target: "kill_targets", "Killing {}", target.pretty_name());
            target.remove_from_world();
        }
    }
}

pub fn strip_token(key: &CStr, dest: &mut CStrSlice) -> Result<(), csz::CursorError> {
    let bytes = key.to_bytes();
    let head = bytes.split(|i| *i == b'#').next().unwrap_or(bytes);
    dest.cursor().write_bytes(head)
}

pub fn clamp_vector_to_box(mut v: vec3_t, clamp_size: vec3_t) -> vec3_t {
    if v.x > clamp_size.x {
        v.x -= clamp_size.x;
    } else if v.x < -clamp_size.x {
        v.x += clamp_size.x;
    } else {
        v.x = 0.0;
    }

    if v.y > clamp_size.y {
        v.y -= clamp_size.y;
    } else if v.y < -clamp_size.y {
        v.y += clamp_size.y;
    } else {
        v.y = 0.0;
    }

    if v.z > clamp_size.z {
        v.z -= clamp_size.z;
    } else if v.z < -clamp_size.z {
        v.z += clamp_size.z;
    } else {
        v.z = 0.0;
    }

    v.normalize()
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Blood {
    #[default]
    Red,
    Yellow,
}

impl Blood {
    pub fn should_show(&self, engine: &ServerEngine) -> bool {
        match self {
            Self::Red => engine.get_cvar(c"violence_hblood"),
            Self::Yellow => engine.get_cvar(c"violence_ablood"),
        }
    }

    pub fn color_index(&self) -> u8 {
        match self {
            Self::Red => 247,
            Self::Yellow => 195,
        }
    }

    pub fn emit_blood_stream(
        &self,
        engine: &ServerEngine,
        origin: vec3_t,
        direction: vec3_t,
        amount: u8,
    ) {
        if !self.should_show(engine) {
            return;
        }
        let msg = user_message::BloodStream {
            start: origin.into(),
            direction: direction.into(),
            color: self.color_index(),
            speed: amount,
        };
        engine.msg_pvs(origin, &msg);
    }

    pub fn emit_blood_drips(&self, engine: &ServerEngine, origin: vec3_t, mut amount: u8) {
        if !self.should_show(engine) || amount == 0 {
            return;
        }
        let global_state = engine.global_state_ref();
        if global_state.game_rules().is_multiplayer() {
            amount = amount.saturating_mul(2);
        }
        let sprites = global_state.sprites();
        let msg = user_message::BloodSprite {
            position: origin.into(),
            initial_sprite_index: sprites.blood_spray(),
            droplet_sprite_index: sprites.blood_drop(),
            color: self.color_index(),
            scale: (amount / 10).clamp(3, 16),
        };
        engine.msg_pvs(origin, &msg);
    }

    pub fn decal_trace(&self, engine: &ServerEngine, trace: &TraceResult) {
        if !self.should_show(engine) {
            return;
        }
        let global_state = engine.global_state_ref();
        let decals = global_state.decals();
        let decal_index = match self {
            Self::Red => decals.get_random_blood(),
            Self::Yellow => decals.get_random_yellow_blood(),
        };
        decal_trace(engine, trace, decal_index);
    }

    pub fn random_direction(engine: &ServerEngine) -> vec3_t {
        vec3_t::new(
            engine.random_float(-1.0, 1.0),
            engine.random_float(-1.0, 1.0),
            engine.random_float(0.0, 1.0),
        )
    }
}

pub fn decal_trace(engine: &ServerEngine, trace: &TraceResult, decal_index: u16) {
    if trace.fraction() == 1.0 {
        return;
    }

    let mut entity_index = trace.hit_entity().entity_index();
    if !entity_index.is_world_spawn() {
        if let Some(entity) = trace.hit_entity().get_entity() {
            if !entity.is_bsp_model() {
                return;
            }
            entity_index = engine.get_entity_index(&entity);
        }
    }

    if entity_index.is_world_spawn() {
        let msg = user_message::WorldDecal {
            position: trace.end_position().into(),
            texture_index: decal_index,
        };
        engine.msg_broadcast(&msg);
    } else {
        let msg = user_message::Decal {
            position: trace.end_position().into(),
            texture_index: decal_index,
            entity: entity_index,
        };
        engine.msg_broadcast(&msg);
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Sparks {
    #[cfg_attr(feature = "save", save(skip))]
    engine: ServerEngineRef,
}

impl Sparks {
    const SOUNDS: [&'static CStr; 6] = [
        res::valve::sound::buttons::SPARK1,
        res::valve::sound::buttons::SPARK2,
        res::valve::sound::buttons::SPARK3,
        res::valve::sound::buttons::SPARK4,
        res::valve::sound::buttons::SPARK5,
        res::valve::sound::buttons::SPARK6,
    ];

    pub fn new(engine: ServerEngineRef) -> Self {
        Self { engine }
    }

    pub fn get_random_sound(&self) -> &CStr {
        let max = Self::SOUNDS.len() - 1;
        let index = self.engine.random_int(0, max as i32);
        Self::SOUNDS[index as usize]
    }

    pub fn precache(&self) {
        for sound in Self::SOUNDS {
            self.engine.precache_sound(sound);
        }
    }

    /// Emit sparks without sound at exact location.
    pub fn emit_simple(&self, pos: vec3_t) {
        let msg = user_message::Sparks::new(pos);
        self.engine.msg_pvs(pos, &msg);
    }

    pub fn emit(&self, location: vec3_t, vars: &EntityVars) {
        let engine = self.engine;
        let pos = location + vars.size() * 0.5;
        self.emit_simple(pos);
        engine
            .build_sound()
            .channel_voice()
            .volume(engine.random_float(0.25, 0.75) * 0.4)
            .emit(self.get_random_sound(), vars);
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum MoveState {
    #[default]
    AtStart = 0,
    AtEnd,
    GoingToStart,
    GoingToEnd,
}

impl MoveState {
    pub fn is_moving(&self) -> bool {
        matches!(self, Self::GoingToEnd | Self::GoingToStart)
    }
}

#[cfg(feature = "save")]
impl Save for MoveState {
    fn save(&self, _: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        let id = match self {
            Self::AtStart => 0,
            Self::AtEnd => 1,
            Self::GoingToStart => 2,
            Self::GoingToEnd => 3,
        };
        cur.write_u8(id)?;
        Ok(())
    }
}

#[cfg(feature = "save")]
impl Restore for MoveState {
    fn restore(&mut self, _: &save::RestoreState, cur: &mut save::Cursor) -> save::SaveResult<()> {
        *self = match cur.read_u8()? {
            0 => Self::AtStart,
            1 => Self::AtEnd,
            2 => Self::GoingToStart,
            3 => Self::GoingToEnd,
            _ => return Err(save::SaveError::InvalidEnum),
        };
        Ok(())
    }
}

pub trait Move: Save + Restore + 'static {
    fn is_reversable(&self) -> bool {
        false
    }

    #[allow(unused_variables)]
    fn key_value(&mut self, data: &mut KeyValue) -> bool {
        false
    }

    fn init(&mut self, v: &EntityVars);

    fn swap(&mut self, v: &EntityVars);

    fn realign_to(&self, v: &EntityVars, other: &EntityVars);

    /// Returns `true` if movement is finished.
    fn move_done(&self, v: &EntityVars) -> bool;

    /// Returns `true` if movement is finished.
    fn move_to_end(&self, v: &EntityVars, speed: f32, reverse: bool) -> bool;

    /// Returns `true` if movement is finished.
    fn move_to_start(&self, v: &EntityVars, speed: f32) -> bool;
}

#[derive(Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct LinearMove {
    lip: f32,
    start: PositionVector,
    end: PositionVector,
    dest: Cell<PositionVector>,
}

impl LinearMove {
    pub fn start(&self) -> vec3_t {
        self.start.to_vec()
    }

    pub fn set_start(&mut self, start: vec3_t) {
        self.start = start.into();
    }

    pub fn end(&self) -> vec3_t {
        self.end.to_vec()
    }

    pub fn set_end(&mut self, end: vec3_t) {
        self.end = end.into();
    }

    pub fn lip(&self) -> f32 {
        self.lip
    }

    pub fn set_lip(&mut self, lip: f32) {
        self.lip = lip;
    }

    pub fn start_move(&self, v: &EntityVars, speed: f32, dest: vec3_t) -> bool {
        assert_ne!(speed, 0.0, "linear_move: speed is zero");

        self.dest.set(dest.into());
        if dest == v.origin() {
            return self.move_done(v);
        }

        let dest_delta = dest - v.origin();
        let travel_time = dest_delta.length() / speed;
        v.set_velocity(dest_delta / travel_time);
        v.set_next_think_time_from_last(travel_time);
        false
    }
}

impl Move for LinearMove {
    fn key_value(&mut self, data: &mut KeyValue) -> bool {
        if data.key_name() == c"lip" {
            self.lip = data.parse_or_default();
            data.set_handled(true);
            true
        } else {
            false
        }
    }

    fn init(&mut self, v: &EntityVars) {
        let start = v.origin();
        let tmp = (v.move_dir() * (v.size() - 2.0)).abs();
        let end = start + (v.move_dir() * (tmp.element_sum() - self.lip));
        self.start = start.into();
        self.end = end.into();
    }

    fn swap(&mut self, v: &EntityVars) {
        mem::swap(&mut self.start, &mut self.end);
        v.set_origin_and_link(self.start);
    }

    fn realign_to(&self, v: &EntityVars, other: &EntityVars) {
        if v.velocity() == other.velocity() {
            v.set_origin(other.origin());
            v.set_velocity(vec3_t::ZERO);
        }
    }

    fn move_done(&self, v: &EntityVars) -> bool {
        let dest = self.dest.get().to_vec();
        let delta = dest - v.origin();
        let error = delta.length();
        if error > 0.03125 {
            self.start_move(v, 100.0, dest);
            return false;
        }

        v.set_origin_and_link(dest);
        v.set_velocity(vec3_t::ZERO);
        v.stop_thinking();
        true
    }

    fn move_to_end(&self, v: &EntityVars, speed: f32, _: bool) -> bool {
        self.start_move(v, speed, self.end.into())
    }

    fn move_to_start(&self, v: &EntityVars, speed: f32) -> bool {
        self.start_move(v, speed, self.start.into())
    }
}

#[derive(Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct AngularMove {
    distance: f32,
    start: vec3_t,
    end: vec3_t,
    dest: Cell<vec3_t>,
}

impl AngularMove {
    pub fn start(&self) -> vec3_t {
        self.start
    }

    pub fn set_start(&mut self, start: vec3_t) {
        self.start = start;
    }

    pub fn end(&self) -> vec3_t {
        self.end
    }

    pub fn set_end(&mut self, end: vec3_t) {
        self.end = end;
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }

    fn start_move(&self, v: &EntityVars, speed: f32, dest: vec3_t) -> bool {
        assert_ne!(speed, 0.0, "angular_move: speed is zero");

        self.dest.set(dest);
        if dest == v.angles() {
            return self.move_done(v);
        }

        let delta = dest - v.angles();
        let travel_time = delta.length() / speed;
        v.set_angular_velocity(delta / travel_time);
        v.set_next_think_time_from_last(travel_time);
        false
    }
}

impl Move for AngularMove {
    fn is_reversable(&self) -> bool {
        true
    }

    fn key_value(&mut self, data: &mut KeyValue) -> bool {
        if data.key_name() == c"distance" {
            self.distance = data.parse_or_default();
            data.set_handled(true);
            true
        } else {
            false
        }
    }

    fn init(&mut self, v: &EntityVars) {
        self.start = v.angles();
        self.end = self.start + v.move_dir() * self.distance;
        assert_ne!(
            self.start, self.end,
            "rotating door start and end angles are equal"
        );
    }

    fn swap(&mut self, v: &EntityVars) {
        mem::swap(&mut self.start, &mut self.end);
        v.set_angles(self.start);
        v.with_move_dir(|x| -x);
    }

    fn realign_to(&self, v: &EntityVars, other: &EntityVars) {
        if v.angular_velocity() == other.angular_velocity() {
            v.set_angles(other.angles());
            v.set_angular_velocity(vec3_t::ZERO);
        }
    }

    fn move_done(&self, v: &EntityVars) -> bool {
        v.set_angles(self.dest.get());
        v.set_angular_velocity(vec3_t::ZERO);
        v.stop_thinking();
        true
    }

    fn move_to_end(&self, v: &EntityVars, speed: f32, reverse: bool) -> bool {
        if reverse {
            self.start_move(v, speed, -self.end)
        } else {
            self.start_move(v, speed, self.end)
        }
    }

    fn move_to_start(&self, v: &EntityVars, speed: f32) -> bool {
        self.start_move(v, speed, self.start)
    }
}

#[derive(Copy, Clone)]
pub struct ScreenShake<'a> {
    engine: &'a ServerEngine,
    amplitude: f32,
    frequency: f32,
    duration: f32,
    radius: f32,
    in_air: bool,
}

impl<'a> ScreenShake<'a> {
    pub fn new(engine: &'a ServerEngine) -> Self {
        Self {
            engine,
            amplitude: 8.0,
            frequency: 40.0,
            duration: 1.0,
            radius: 0.0,
            in_air: false,
        }
    }

    pub fn amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn in_air(mut self, in_air: bool) -> Self {
        self.in_air = in_air;
        self
    }

    pub fn emit(self, center: vec3_t) {
        for player in self.engine.players() {
            let v = player.vars();
            if !self.in_air && !v.flags().intersects(EdictFlags::ONGROUND) {
                continue;
            }

            let mut local_amplitude = 0.0;
            if self.radius <= 0.0 {
                local_amplitude = self.amplitude;
            } else {
                let distance = (center - v.origin()).length();
                if distance < self.radius {
                    local_amplitude = self.amplitude;
                }
            }

            if local_amplitude != 0.0 {
                let msg = user_message::ScreenShake {
                    amplitude: local_amplitude.into(),
                    duration: self.duration.into(),
                    frequence: self.frequency.into(),
                };
                self.engine.msg_one_reliable(v, &msg);
            }
        }
    }
}

pub use crate::user_message::ScreenFadeFlags;

pub struct ScreenFade {
    pub color: RGBA,
    pub duration: f32,
    pub hold_time: f32,
    pub flags: ScreenFadeFlags,
}

impl ScreenFade {
    pub fn emit_one(&self, v: &EntityVars) {
        let msg = user_message::ScreenFade {
            duration: self.duration.into(),
            hold_time: self.hold_time.into(),
            flags: self.flags,
            color: self.color,
        };
        v.engine().msg_one_reliable(v, &msg);
    }

    pub fn emit_all(&self, engine: &ServerEngine) {
        for player in engine.players() {
            self.emit_one(player.vars());
        }
    }
}

pub fn precache_other(engine: &ServerEngine, class_name: impl ToEngineStr) {
    let class_name = class_name.to_engine_str();
    let Some(mut entity) = engine.create_named_entity(class_name.as_ref()) else {
        let class_name = class_name.as_ref();
        error!("precache_other: failed to create entity {class_name}");
        return;
    };
    if let Some(entity) = unsafe { entity.get_entity_mut() } {
        entity.precache();
    }
    // SAFETY: entity is not used
    unsafe {
        engine.remove_entity_now(&entity);
    }
}

pub fn show_message(player: &dyn EntityPlayer, msg: &CStr) {
    if !player.is_net_client() {
        return;
    }
    trace!("show_message: send {msg:?} to {}", player.pretty_name());
    let msg = user_message::HudText::new(msg);
    player.engine().msg_one_reliable(player.vars(), &msg);
}

pub fn show_message_all(engine: &ServerEngine, msg: &CStr) {
    trace!("show_message_all: send {msg:?}");
    for player in engine.players().filter_map(|i| i.as_player()) {
        show_message(player, msg);
    }
}
