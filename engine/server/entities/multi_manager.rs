use core::{cell::Cell, ptr};

use alloc::vec::Vec;
use bitflags::bitflags;
use csz::{CStrArray, CStrThin};

use crate::{
    entity::{delegate_entity, BaseEntity, EntityHandle, KeyValue, ObjectCaps, Solid, UseType},
    export::export_entity_default,
    prelude::*,
    private::impl_private,
    str::MapString,
    time::MapTime,
    utils,
};

#[derive(Copy, Clone, Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
struct MultiManagerTarget {
    name: Option<MapString>,
    delay: f32,
}

impl MultiManagerTarget {
    fn new(name: MapString, delay: f32) -> Self {
        Self {
            name: Some(name),
            delay,
        }
    }
}

bitflags! {
    struct MultiManagerSpawnFlags: u32 {
        /// Create clones when triggered.
        const THREAD = 1 << 0;
        /// This is a clone for a threaded execution.
        const CLONE = 1 << 31;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct MultiManager {
    base: BaseEntity,
    targets: Vec<MultiManagerTarget>,
    wait: f32,
    start_time: Cell<MapTime>,
    activator: Cell<EntityHandle>,
    index: Cell<u32>,
    enable_use: Cell<bool>,
    enable_think: Cell<bool>,
}

impl MultiManager {
    fn spawn_flags(&self) -> MultiManagerSpawnFlags {
        MultiManagerSpawnFlags::from_bits_retain(self.vars().spawn_flags())
    }

    fn is_clone(&self) -> bool {
        self.spawn_flags().intersects(MultiManagerSpawnFlags::CLONE)
    }

    fn should_clone(&self) -> bool {
        !self.is_clone()
            && self
                .spawn_flags()
                .intersects(MultiManagerSpawnFlags::THREAD)
    }

    fn clone_me(&self) -> *mut Self {
        let engine = self.engine();
        let multi = engine.new_entity::<Self>().build();
        let edict = multi.vars().containing_entity_raw();
        unsafe {
            ptr::copy_nonoverlapping(self.vars().as_ptr(), multi.vars().as_mut_ptr(), 1);
        }
        let v = multi.vars();
        unsafe {
            v.set_containing_entity_raw(edict);
        }
        v.with_spawn_flags(|f| f | MultiManagerSpawnFlags::CLONE.bits());
        multi.targets = self.targets.clone();
        multi
    }

    pub fn has_target(&self, target: &CStrThin) -> bool {
        self.targets
            .iter()
            .filter_map(|i| i.name)
            .any(|i| i.as_thin() == target)
    }
}

impl CreateEntity for MultiManager {
    fn create(base: BaseEntity) -> Self {
        let engine = base.engine();
        Self {
            base,
            targets: Default::default(),
            wait: 0.0,
            start_time: Default::default(),
            activator: Cell::new(engine.get_world_spawn_entity()),
            index: Default::default(),
            enable_use: Default::default(),
            enable_think: Default::default(),
        }
    }
}

impl Entity for MultiManager {
    delegate_entity!(base not { object_caps, key_value, spawn, used, think });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        let key = data.key_name();
        if key == c"wait" {
            self.wait = data.parse_or(0.0);
            data.set_handled(true);
        } else {
            let mut tmp = CStrArray::<128>::new();
            match utils::strip_token(key.into(), &mut tmp) {
                Ok(()) => {
                    let name = self.engine().new_map_string(&tmp);
                    let delay = data.parse_or_default();
                    self.targets.push(MultiManagerTarget::new(name, delay))
                }
                Err(_) => {
                    error!("{}: failed to strip token {key:?}", self.classname());
                }
            }
        }
    }

    fn spawn(&mut self) {
        self.vars().set_solid(Solid::Not);
        self.targets
            .sort_by(|a, b| a.delay.partial_cmp(&b.delay).unwrap());
        self.enable_use.set(true);
    }

    fn used(&self, _use_type: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
        if !self.enable_use.get() {
            return;
        }

        if self.should_clone() {
            let clone = unsafe { &*self.clone_me() };
            clone.used(_use_type, activator, caller);
            return;
        }

        let engine = self.engine();
        self.activator
            .set(activator.unwrap_or(caller).entity_handle());
        self.index.set(0);
        self.start_time.set(engine.globals.map_time());
        self.enable_use.set(false);
        self.enable_think.set(true);
        self.vars().set_next_think_time_from_now(0.0);
    }

    fn think(&self) {
        if !self.enable_think.get() {
            return;
        }

        let engine = self.engine();
        let time = engine.globals.map_time() - self.start_time.get();
        let activator = self.activator.get().get_entity();

        for target in self.targets.iter().skip(self.index.get() as usize) {
            if target.delay > time {
                break;
            }
            if let Some(target_name) = target.name {
                utils::fire_targets(&target_name, UseType::Toggle, activator, self);
            }
            self.index.set(self.index.get() + 1);
        }

        if self.index.get() as usize >= self.targets.len() {
            self.enable_think.set(false);
            if self.is_clone() {
                self.remove_from_world();
            }
            self.enable_use.set(true);
        } else if let Some(target) = self.targets.get(self.index.get() as usize) {
            let next_time = self.start_time.get() + target.delay;
            self.base.vars().set_next_think_time(next_time);
        }
    }
}

impl_private!(MultiManager {});

export_entity_default!("export-multi_manager", multi_manager, MultiManager);
