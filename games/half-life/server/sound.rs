use xash3d_server::{csz::CStrThin, entity::Entity, prelude::*, sound::Pitch};

pub fn emit_sound_suit(entity: &dyn Entity, sample: &CStrThin) {
    let engine = entity.engine();
    let volume = engine.get_cvar(c"suitvolume");
    if volume > 0.05 {
        engine
            .build_sound()
            .channel_static()
            .volume(volume)
            .pitch(if engine.random_bool() {
                Pitch::from(engine.random_int(0, 6) + 98)
            } else {
                Pitch::NORM
            })
            .emit_dyn(sample, &entity)
    }
}
