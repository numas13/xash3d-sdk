use core::{cell::Cell, ffi::CStr};

use bitflags::bitflags;
use xash3d_server::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, EntityHandle, KeyValue, Solid, UseType},
    ffi::common::vec3_t,
    prelude::*,
    private::impl_private,
    str::MapString,
    utils,
};

bitflags! {
    #[derive(Copy, Clone)]
    struct SpawnFlags: u32 {
        const DISABLED          = 1 << 0;
        const FIRE_ONCE         = 1 << 1;
        const ALT_REVERSE       = 1 << 2;
        const DISABLE_TRAIN     = 1 << 3;
        const ALTERNATE         = 1 << 15;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct PathTrack {
    base: PointEntity,

    alt_name: Option<MapString>,

    previous: Cell<Option<EntityHandle>>,
    next: Cell<Option<EntityHandle>>,
    alt: Cell<Option<EntityHandle>>,
}

impl CreateEntity for PathTrack {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),

            alt_name: None,

            previous: Cell::default(),
            next: Cell::default(),
            alt: Cell::default(),
        }
    }
}

impl PathTrack {
    pub const CLASS_NAME: &'static CStr = c"path_track";

    fn spawn_flags(&self) -> SpawnFlags {
        SpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn set_spawn_flags(&self, flags: SpawnFlags, state: bool) {
        self.vars().with_spawn_flags(|f| {
            if state {
                f | flags.bits()
            } else {
                f & !flags.bits()
            }
        });
    }

    fn fire_once(&self) -> bool {
        self.spawn_flags().intersects(SpawnFlags::FIRE_ONCE)
    }

    pub fn disable_train(&self) -> bool {
        self.spawn_flags().intersects(SpawnFlags::DISABLE_TRAIN)
    }

    fn alternate(&self) -> bool {
        self.spawn_flags().intersects(SpawnFlags::ALTERNATE)
    }

    fn set_alternate(&self, alt: bool) {
        self.set_spawn_flags(SpawnFlags::ALTERNATE, alt);
    }

    fn alternate_reverse(&self) -> bool {
        self.spawn_flags().intersects(SpawnFlags::ALT_REVERSE)
    }

    fn disabled(&self) -> bool {
        self.spawn_flags().intersects(SpawnFlags::DISABLED)
    }

    fn set_disabled(&self, disabled: bool) {
        self.set_spawn_flags(SpawnFlags::DISABLED, disabled);
    }

    fn set_previous(&self, path: &Self) {
        self.previous.set(Some(path.entity_handle()));
    }

    fn set_next(&self, path: &Self) {
        self.next.set(Some(path.entity_handle()));
    }

    fn set_alt(&self, path: &Self) {
        self.alt.set(Some(path.entity_handle()));
    }

    fn link(&self) {
        let name = self.pretty_name();

        if let Some(target_name) = self.vars().target() {
            match self.target_entity() {
                Some(target) => match target.downcast_ref::<Self>() {
                    Some(target) => {
                        trace!("{}: link {target_name}", self.pretty_name());
                        self.set_next(target);
                        target.set_previous(self);
                    }
                    None => error!("{name}: target {target_name} is not PathTrack"),
                },
                None => debug!("{name}: link {target_name} (dead end)"),
            }
        }

        if let Some(target_name) = self.alt_name {
            let engine = self.engine();
            match engine.entities().by_target_name(target_name).first() {
                Some(target) => match target.downcast_ref::<Self>() {
                    Some(target) => {
                        trace!("{}: link {target_name}", self.pretty_name());
                        self.set_alt(target);
                        target.set_previous(self);
                    }
                    None => error!("{name}: alt target {target_name} is not PathTrack"),
                },
                None => trace!("{name}: alt target {target_name} not found"),
            }
        }
    }

    pub fn previous(&self) -> Option<&Self> {
        if let Some(alt) = self.alt.get() {
            if self.alternate() && self.alternate_reverse() {
                return alt.downcast_ref();
            }
        }
        self.previous.get().downcast_ref()
    }

    pub fn next(&self) -> Option<&Self> {
        if let Some(alt) = self.alt.get() {
            if self.alternate() && !self.alternate_reverse() {
                return alt.downcast_ref();
            }
        }
        self.next.get().downcast_ref()
    }

    pub fn first(&self, is_move: bool) -> &Self {
        let mut cur = self;
        while let Some(next) = valid_path(cur.previous(), is_move) {
            cur = next;
        }
        cur
    }

    pub fn last(&self, is_move: bool) -> &Self {
        let mut cur = self;
        while let Some(next) = valid_path(cur.next(), is_move) {
            cur = next;
        }
        cur
    }

    fn look_forward(&self, origin: vec3_t, dist: f32, is_move: bool) -> (vec3_t, Option<&Self>) {
        let original_dist = dist;
        let mut dist = dist;
        let mut cur = self;
        let mut pos = origin;
        while dist > 0.0 {
            let Some(next) = valid_path(cur.next(), is_move) else {
                if !is_move {
                    pos = project(cur.previous(), cur, dist).unwrap_or(pos);
                }
                return (pos, None);
            };

            let next_pos = next.vars().origin();
            let dir = next_pos - pos;
            let length = dir.length();
            if length == 0.0 && valid_path(next.next(), is_move).is_none() {
                if dist == original_dist {
                    return (pos, None);
                }
                break;
            } else if length > dist {
                pos += dir * (dist / length);
                break;
            }

            dist -= length;
            pos = next_pos;
            cur = next;
        }
        (pos, Some(cur))
    }

    fn look_backward(&self, origin: vec3_t, dist: f32, is_move: bool) -> (vec3_t, Option<&Self>) {
        let mut dist = dist;
        let mut cur = self;
        let mut pos = origin;
        while dist > 0.0 {
            let dir = cur.vars().origin() - pos;
            let length = dir.length();
            if length == 0.0 {
                match valid_path(cur.previous(), is_move) {
                    Some(prev) => cur = prev,
                    None => {
                        if !is_move {
                            pos = project(cur.next(), cur, dist).unwrap_or(pos);
                        }
                        return (pos, None);
                    }
                }
            } else if length > dist {
                pos += dir * (dist / length);
                break;
            }

            dist -= length;
            pos = cur.vars().origin();

            match valid_path(cur.previous(), is_move) {
                Some(prev) => cur = prev,
                None => return (pos, None),
            };
        }
        (pos, Some(cur))
    }

    pub fn look_ahead(&self, origin: vec3_t, dist: f32, is_move: bool) -> (vec3_t, Option<&Self>) {
        if dist > 0.0 {
            self.look_forward(origin, dist, is_move)
        } else if dist < 0.0 {
            self.look_backward(origin, -dist, is_move)
        } else {
            (origin, Some(self))
        }
    }

    pub fn fire_targets(&self, activator: &dyn Entity) {
        let v = self.vars();
        if let Some(message) = v.message().as_deref() {
            utils::fire_targets(message, UseType::Toggle, Some(activator), activator);
            if self.fire_once() {
                v.set_message(None);
            }
        }
    }
}

fn valid_path(path: Option<&PathTrack>, is_move: bool) -> Option<&PathTrack> {
    path.and_then(|path| {
        if is_move && path.disabled() {
            None
        } else {
            Some(path)
        }
    })
}

fn project(start: Option<&PathTrack>, end: &PathTrack, dist: f32) -> Option<vec3_t> {
    let start = start?.vars().origin();
    let end = end.vars().origin();
    Some(end + (end - start).normalize() * dist)
}

impl Entity for PathTrack {
    delegate_entity!(base not { key_value, spawn, activate, used });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"altpath" {
            self.alt_name = Some(self.engine().new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn spawn(&mut self) {
        let v = self.base.vars();
        v.set_solid(Solid::Trigger);
        v.set_size_and_link(vec3_t::splat(-8.0), vec3_t::splat(8.0));

        self.previous.set(None);
        self.next.set(None);
    }

    fn activate(&self) {
        if self.vars().target_name().is_some() {
            self.link();
        }
    }

    fn used(&self, use_type: UseType, _: Option<&dyn Entity>, _: &dyn Entity) {
        if self.alt.get().is_some() {
            // toggle between two paths
            let on = self.alternate();
            if use_type.should_toggle(on) {
                self.set_alternate(!on);
            }
        } else {
            // enable/disable
            let on = !self.disabled();
            if use_type.should_toggle(on) {
                self.set_disabled(!on);
            }
        }
    }
}

impl_private!(PathTrack {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_path_track {
    () => {
        $crate::export_entity!(path_track, $crate::path_track::PathTrack);
    };
}
#[doc(inline)]
pub use export_path_track as export;
