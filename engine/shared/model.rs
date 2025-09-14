use core::ffi::c_int;

use bitflags::bitflags;
use xash3d_ffi::common::{model_s, modtype_t};

use crate::ffi;

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum ModelType: modtype_t {
        Bad(ffi::common::modtype_t_mod_bad),
        Brush(ffi::common::modtype_t_mod_brush),
        Sprite(ffi::common::modtype_t_mod_sprite),
        Alias(ffi::common::modtype_t_mod_alias),
        Studio(ffi::common::modtype_t_mod_studio),
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct ModelFlags: c_int {
        const NONE              = 0;
        const CONVEYOR          = ffi::common::MODEL_CONVEYOR;
        const HAS_ORIGIN        = ffi::common::MODEL_HAS_ORIGIN;
        /// Model has only point hull.
        const LIQUID            = ffi::common::MODEL_LIQUID;
        /// Model has transparent surfaces.
        const TRANSPARENT       = ffi::common::MODEL_TRANSPARENT;
        /// Lightmaps stored as RGB.
        const COLORED_LIGHTING  = ffi::common::MODEL_COLORED_LIGHTING;

        /// uses 32-bit types.
        const QBSP2             = ffi::common::MODEL_QBSP2;
        /// It is a world model.
        const WORLD             = ffi::common::MODEL_WORLD;
        /// A client sprite.
        const CLIENT            = ffi::common::MODEL_CLIENT;
    }
}

// TODO: add safe wrapper for model_s and remove this trait
pub trait ModelExt {
    fn model_type(&self) -> ModelType;
}

impl ModelExt for model_s {
    fn model_type(&self) -> ModelType {
        ModelType::from_raw(self.type_).unwrap()
    }
}
