use xash3d_server::{
    entity::{BaseEntity, KeyValue, MoveType, Solid, UseType, delegate_entity},
    prelude::*,
    private::impl_private,
    utils::{LinearMove, Move, Sparks},
};

use crate::base_button::{BaseButton, SpawnFlags};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct Button {
    base: BaseButton<LinearMove>,
}

impl CreateEntity for Button {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BaseButton::create(base),
        }
    }
}

impl Button {
    fn spawn_flags(&self) -> SpawnFlags {
        self.base.spawn_flags()
    }

    fn spark_if_off(&self) -> bool {
        self.spawn_flags().intersects(SpawnFlags::SPARK_IF_OFF) && self.base.is_off()
    }
}

impl Entity for Button {
    delegate_entity!(base not { key_value, precache, spawn, used, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        if self.base.lock_sounds.key_value(data) {
            return;
        }
        self.base.key_value(data);
    }

    fn precache(&mut self) {
        if self.spawn_flags().intersects(SpawnFlags::SPARK_IF_OFF) {
            Sparks::new(self.engine()).precache();
        }

        self.base.lock_sounds.precache();
    }

    fn spawn(&mut self) {
        self.base.spawn();
        self.precache();

        let sf = self.spawn_flags();
        let v = self.base.base.vars();
        v.set_move_dir_from_angles();
        v.set_move_type(MoveType::Push);
        v.set_solid(Solid::Bsp);
        v.reload_model();

        if self.base.button_move.lip() == 0.0 {
            self.base.button_move.set_lip(4.0);
        }
        self.base.button_move.init(v);

        let button_move = &mut self.base.button_move;
        let distance = (button_move.end() - button_move.start()).length();
        if distance < 1.0 || sf.intersects(SpawnFlags::DONT_MOVE) {
            button_move.set_end(button_move.start());
        }

        if sf.intersects(SpawnFlags::SPARK_IF_OFF) {
            v.set_next_think_time_from_now(0.5);
        }
    }

    fn used(&self, use_type: UseType, activator: Option<&dyn Entity>, caller: &dyn Entity) {
        self.base.used(use_type, activator, caller);

        if self.spark_if_off() {
            self.vars().set_next_think_time_from_now(0.5);
        }
    }

    fn think(&self) {
        self.base.think();

        if self.spark_if_off() {
            let engine = self.engine();
            let v = self.base.vars();
            Sparks::new(engine).emit(v.abs_min(), v);
            let time = engine.random_float(0.0, 1.5);
            v.set_next_think_time_from_last(0.1 + time);
        }
    }
}

impl_private!(Button {});

#[doc(hidden)]
#[macro_export]
macro_rules! export_func_button {
    () => {
        $crate::export_entity!(func_button, $crate::func_button::Button);
    };
}
#[doc(inline)]
pub use export_func_button as export;
