use core::{any::Any, ffi::c_int};

use bitflags::bitflags;
use csz::CStrThin;
use sv::{
    consts::{SOLID_BSP, SOLID_NOT},
    entity::MoveType,
    ffi::{
        common::vec3_t,
        server::{edict_s, entvars_s, KeyValueData, TYPEDESCRIPTION},
    },
    math::fabsf,
    prelude::*,
    raw::{EdictFlags, Effects},
    str::MapString,
};

use crate::save::{self, SaveRestore};

/// A helper trait to downcast an Entity trait.
#[allow(dead_code)]
pub trait Cast {
    fn as_delay(&self) -> Option<&dyn Delay>;
    fn as_delay_mut(&mut self) -> Option<&mut dyn Delay>;

    fn as_animating(&self) -> Option<&dyn Animating>;
    fn as_animating_mut(&mut self) -> Option<&mut dyn Animating>;

    fn as_toggle(&self) -> Option<&dyn Toggle>;
    fn as_toggle_mut(&mut self) -> Option<&mut dyn Toggle>;

    fn as_monster(&self) -> Option<&dyn Monster>;
    fn as_monster_mut(&mut self) -> Option<&mut dyn Monster>;
}

#[doc(hidden)]
macro_rules! impl_cast {
    (cast $ty:ty, $trait:path, $value:expr $(,$mut:ident)?) => ({
        #[allow(dead_code)]
        trait DoNotCast {
            fn cast<V>(_: &$($mut)? V) -> Option<&$($mut)? dyn $trait> { None }
        }
        impl<T> DoNotCast for T {}

        struct Wrapper<V>(core::marker::PhantomData<V>);

        #[allow(dead_code)]
        impl<V: $trait> Wrapper<V> {
            fn cast(value: &$($mut)? V) -> Option<&$($mut)? dyn $trait> { Some(value) }
        }

        Wrapper::<$ty>::cast($value)
    });
    ($ty:ty, $trait:path, $meth:ident, $meth_mut:ident) => {
        fn $meth(&self) -> Option<&dyn $trait> {
            $crate::entity::impl_cast!(cast $ty, $trait, self)
        }

        fn $meth_mut(&mut self) -> Option<&mut dyn $trait> {
            $crate::entity::impl_cast!(cast $ty, $trait, self, mut)
        }
    };
    ($ty:ty) => {
        impl $crate::entity::Cast for $ty {
            $crate::entity::impl_cast!($ty, $crate::entity::Delay, as_delay, as_delay_mut);
            $crate::entity::impl_cast!($ty, $crate::entity::Animating, as_animating, as_animating_mut);
            $crate::entity::impl_cast!($ty, $crate::entity::Toggle, as_toggle, as_toggle_mut);
            $crate::entity::impl_cast!($ty, $crate::entity::Monster, as_monster, as_monster_mut);
        }
    };
}
#[doc(inline)]
pub(super) use impl_cast;

bitflags! {
    /// Flags to indicate an object's capabilities.
    ///
    /// Used for save/restore and level transitions.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct ObjectCaps: u32 {
        const NONE                  = 0x00000000;
        const CUSTOMSAVE            = 0x00000001;
        const ACROSS_TRANSITION     = 0x00000002;
        const MUST_SPAWN            = 0x00000004;
        const DONT_SAVE             = 0x80000000;
        const IMPULSE_USE           = 0x00000008;
        const CONTINUOUS_USE        = 0x00000010;
        const ONOFF_USE             = 0x00000020;
        const DIRECTIONAL_USE       = 0x00000040;
        const MASTER                = 0x00000080;
        const FORCE_TRANSITION      = 0x00000080;
    }
}

pub trait EntityVars {
    fn vars_ptr(&self) -> *mut entvars_s;
}

#[allow(dead_code)]
pub trait Entity: EntityVars + Cast + Any {
    fn vars(&self) -> &entvars_s {
        unsafe { &*self.vars_ptr() }
    }

    fn vars_mut(&mut self) -> &mut entvars_s {
        unsafe { &mut *self.vars_ptr() }
    }

    fn ent(&self) -> &edict_s {
        unsafe { &*self.vars().pContainingEntity }
    }

    fn ent_mut(&mut self) -> &mut edict_s {
        unsafe { &mut *self.vars().pContainingEntity }
    }

    /// Returns `false` if entity should be deleted.
    fn spawn(&mut self) -> bool {
        true
    }

    fn precache(&mut self) {}

    fn key_value(&mut self, data: &mut KeyValueData) {
        let class_name = data.class_name();
        let key_name = data.key_name();
        let value = data.value();
        debug!(
            "{}::key_value({class_name:?}, {key_name}, {value})",
            self.classname()
        );
        data.set_handled(true);
    }

    fn fields(&self) -> &'static [TYPEDESCRIPTION] {
        // TODO:
        &[]
    }

    fn save(&mut self, _save: &mut SaveRestore) -> save::Result<()> {
        // TODO:
        debug!("TODO: save {:?}", self.classname());
        Ok(())
    }

    fn restore(&mut self, restore: &mut SaveRestore) -> save::Result<()> {
        restore.read_ent_vars(c"ENTVARS", self.vars_mut())?;

        let fields = self.fields();
        restore.read_fields(c"BASE", self as *mut _ as *mut _, fields)?;

        let ev = self.vars_mut();
        if let (true, Some(model)) = (ev.modelindex != 0, ev.model()) {
            let mins = ev.mins;
            let maxs = ev.maxs;
            let engine = engine();
            engine.precache_model(&model);
            engine.set_model(self.ent_mut(), &model);
            engine.set_size(self.ent_mut(), mins, maxs);
        }

        Ok(())
    }

    fn override_reset(&mut self) {}

    fn object_caps(&self) -> ObjectCaps {
        ObjectCaps::ACROSS_TRANSITION
    }

    fn activate(&mut self) {}

    fn think(&mut self) {
        // TODO:
    }

    fn touch(&mut self, _other: &mut dyn Entity) {
        // debug!("{self:?} touch {other:?}");
    }

    fn set_object_collision_box(&mut self) {
        set_object_collision_box(self.vars_mut());
    }

    // TODO: classify
    // TODO: death_notice

    fn make_dormant(&mut self) {
        let ev = self.vars_mut();
        ev.flags_mut().insert(EdictFlags::DORMANT);
        ev.solid = SOLID_NOT as c_int;
        ev.movetype = MoveType::None.into();
        ev.effects_mut().insert(Effects::NODRAW);
        ev.nextthink = 0.0;
    }

    fn is_dormant(&self) -> bool {
        self.vars().flags().intersects(EdictFlags::DORMANT)
    }

    fn globalname(&self) -> MapString {
        self.vars().globalname().unwrap()
    }

    fn is_globalname(&self, name: &CStrThin) -> bool {
        name == self.globalname().as_thin()
    }

    fn classname(&self) -> MapString {
        self.vars().classname().unwrap()
    }

    fn is_classname(&self, name: &CStrThin) -> bool {
        name == self.classname().as_thin()
    }

    fn intersects(&self, other: &dyn Entity) -> bool {
        let a = self.vars();
        let b = other.vars();
        !(b.absmin.x() > a.absmax.x()
            || b.absmin.y() > a.absmax.y()
            || b.absmin.z() > a.absmax.z()
            || b.absmax.x() < a.absmin.x()
            || b.absmax.y() < a.absmin.y()
            || b.absmax.z() < a.absmin.z())
    }
}

pub fn set_object_collision_box(ev: &mut entvars_s) {
    if ev.solid == SOLID_BSP && ev.angles != vec3_t::ZERO {
        let mut max = 0.0;
        for i in 0..3 {
            let v = fabsf(ev.mins[i]);
            if v > max {
                max = v;
            }
            let v = fabsf(ev.maxs[i]);
            if v > max {
                max = v;
            }
        }

        ev.absmin = ev.origin - vec3_t::splat(max);
        ev.absmax = ev.origin + vec3_t::splat(max);
    } else {
        ev.absmin = ev.origin + ev.mins; // TODO: should it be sub?
        ev.absmax = ev.origin + ev.maxs;
    }

    ev.absmin -= vec3_t::splat(1.0);
    ev.absmax += vec3_t::splat(1.0);
}

pub trait Delay: Entity {}
pub trait Animating: Delay {}
pub trait Toggle: Animating {}
pub trait Monster: Toggle {}
