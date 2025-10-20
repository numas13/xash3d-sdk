use core::{cell::Cell, ffi::CStr, mem};

use csz::{CStrSlice, CStrThin};
pub use xash3d_shared::utils::*;
use xash3d_shared::{entity::EdictFlags, ffi::common::vec3_t};

use crate::{
    engine::TraceResult,
    entity::{Entity, EntityVars, GetPrivateData, KeyValue, ObjectCaps, UseType},
    prelude::*,
    save::PositionVector,
    str::MapString,
    user_message,
};

#[cfg(feature = "save")]
use crate::save::{self, Restore, Save};

/// Used for view cone checking.
#[derive(Copy, Clone, PartialEq)]
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

pub fn is_master_triggered(
    engine: &ServerEngine,
    master: MapString,
    activator: &dyn Entity,
) -> bool {
    engine
        .entities()
        .by_target_name(&master)
        .filter_map(|ent| ent.get_entity())
        .find(|ent| ent.object_caps().intersects(ObjectCaps::MASTER))
        .map_or(true, |ent| ent.is_triggered(activator))
}

pub fn fire_targets(
    target_name: &CStrThin,
    use_type: UseType,
    activator: Option<&dyn Entity>,
    caller: &dyn Entity,
) {
    let engine = caller.engine();
    trace!("Firing: ({target_name})");
    for target in engine.entities().by_target_name(target_name) {
        if let Some(target) = target.get_entity() {
            if !target.vars().flags().intersects(EdictFlags::KILLME) {
                trace!("Found: {}, firing ({target_name})", target.classname());
                target.used(use_type, activator, caller);
            }
        }
    }
}

pub fn use_targets(
    kill_target: Option<MapString>,
    use_type: UseType,
    activator: Option<&dyn Entity>,
    caller: &dyn Entity,
) {
    if let Some(kill_target) = kill_target {
        let engine = caller.engine();
        trace!("KillTarget: {kill_target}");
        for target in engine.entities().by_target_name(&kill_target) {
            if let Some(target) = target.get_entity() {
                trace!("killing {}", target.classname());
                target.remove_from_world();
            }
        }
    }

    if let Some(target) = caller.vars().target() {
        fire_targets(&target, use_type, activator, caller);
    }
}

pub fn strip_token(key: &CStr, dest: &mut CStrSlice) -> Result<(), csz::CursorError> {
    if let Some(head) = key.to_bytes().split(|i| *i == b'#').next() {
        dest.cursor().write_bytes(head)
    } else {
        dest.clear();
        Ok(())
    }
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

    pub fn emit(&self, location: vec3_t, vars: &EntityVars) {
        let engine = self.engine;
        let pos = location + vars.size() * 0.5;
        engine.msg_pvs(pos, &user_message::Sparks::new(pos));
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
    Bottom = 0,
    Top,
    GoingDown,
    GoingUp,
}

impl MoveState {
    pub fn is_moving(&self) -> bool {
        matches!(self, Self::GoingUp | Self::GoingDown)
    }
}

#[cfg(feature = "save")]
impl Save for MoveState {
    fn save(&self, _: &mut save::SaveState, cur: &mut save::CursorMut) -> save::SaveResult<()> {
        cur.write_u8(*self as u8)?;
        Ok(())
    }
}

#[cfg(feature = "save")]
impl Restore for MoveState {
    fn restore(&mut self, _: &save::RestoreState, cur: &mut save::Cursor) -> save::SaveResult<()> {
        *self = match cur.read_u8()? {
            0 => Self::Bottom,
            1 => Self::Top,
            2 => Self::GoingDown,
            3 => Self::GoingUp,
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
    fn move_up(&self, v: &EntityVars, speed: f32, reverse: bool) -> bool;

    /// Returns `true` if movement is finished.
    fn move_down(&self, v: &EntityVars, speed: f32) -> bool;
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
    pub fn lip(&self) -> f32 {
        self.lip
    }

    pub fn set_lip(&mut self, lip: f32) {
        self.lip = lip;
    }

    fn start_move(&self, v: &EntityVars, speed: f32, dest: vec3_t) -> bool {
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

    fn move_up(&self, v: &EntityVars, speed: f32, _: bool) -> bool {
        self.start_move(v, speed, self.end.into())
    }

    fn move_down(&self, v: &EntityVars, speed: f32) -> bool {
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

    fn move_up(&self, v: &EntityVars, speed: f32, reverse: bool) -> bool {
        if reverse {
            self.start_move(v, speed, -self.end)
        } else {
            self.start_move(v, speed, self.end)
        }
    }

    fn move_down(&self, v: &EntityVars, speed: f32) -> bool {
        self.start_move(v, speed, self.start)
    }
}
