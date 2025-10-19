use crate::{
    engine::TraceIgnore,
    entities::point_entity::PointEntity,
    entity::{
        delegate_entity, impl_entity_cast, BaseEntity, CreateEntity, Entity, EntityPlayer,
        EntityVars, KeyValue, LastSound,
    },
    export::export_entity_default,
    prelude::*,
    user_message,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

// TODO: enum derive(Save, Restore)
// #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
// #[repr(u16)]
// pub enum RoomType {
//     #[default]
//     /// The default, echo-less sound style.
//     Normal = 0,
//     /// A slightly more closed in sound than default.
//     Generic = 1,
//     /// Quite similar to Generic, with slightly more ring.
//     MetalSmall = 2,
//     /// As above, but with slightly longer echo.
//     MetalMedium = 3,
//     /// As above, but with longer echo.
//     MetalLarge = 4,
//     /// A drawn out, tinny sound.
//     TunnelSmall = 5,
//     /// As above, by with more drawn out echo.
//     TunnelMedium = 6,
//     /// As above, but with a very drawn out echo.
//     TunnelLarge = 7,
//     /// Similar to Generic, but with more echo.
//     ChamberSmall = 8,
//     /// As above, but with slightly longer echo.
//     ChamberMedium = 9,
//     /// As above, but with a long echo.
//     ChamberLarge = 10,
//     /// Very similar to Generic.
//     BrightSmall = 11,
//     /// As above, but more open-sounding.
//     BrightMedium = 12,
//     /// As above, but more open-sounding.
//     BrightLarge = 13,
//     /// A claustrophobic, muffled sound.
//     Water1 = 14,
//     /// As above, but with an echo.
//     Water2 = 15,
//     /// As above, but with a longer, ringing echo.
//     Water3 = 16,
//     /// Similar to Generic, but with a short echo.
//     ConcreteSmall = 17,
//     /// As above, but with a longer echo.
//     ConcreteMedium = 18,
//     /// As above, but with a longer echo.
//     ConcreteLarge = 19,
//     /// An open sound with a spaced out, ringing echo.
//     Big1 = 20,
//     /// As above, but with a longer-lingering echo.
//     Big2 = 21,
//     /// As above, but with a much longer-lingering echo.
//     Big3 = 22,
//     /// A closed in sound with a fast-ringing echo.
//     CavernSmall = 23,
//     /// As above, but with a longer-lingering echo.
//     CavernMedium = 24,
//     /// As above, but with a much longer-lingering echo.
//     CavernLarge = 25,
//     /// Similar to Generic, but with a sharper sound.
//     Weirdo1 = 26,
//     /// As above, but with a high, ringing echo.
//     Weirdo2 = 27,
//     /// As above, but with a strange, high-pitched echo.
//     Weirdo3 = 28,
// }

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct EnvSound {
    base: PointEntity,
    radius: f32,
    room_type: f32,
}

impl EnvSound {
    fn set_next_think_fast(&self) {
        self.vars().set_next_think_time_from_now(0.25);
    }

    fn set_next_think_slow(&self) {
        self.vars().set_next_think_time_from_now(0.75);
    }

    fn in_range(&self, player: &EntityVars) -> Option<f32> {
        let engine = self.engine();
        let v = self.base.vars();
        let start = v.origin() + v.view_ofs();
        let end = player.origin() + player.view_ofs();
        let trace = engine.trace_line(start, end, TraceIgnore::MONSTERS, Some(v));

        if (trace.in_open() && trace.in_water()) || trace.fraction() != 1.0 {
            return None;
        }

        let range = (trace.end_position() - start).length();
        if range <= self.radius {
            Some(range)
        } else {
            None
        }
    }
}

impl_entity_cast!(EnvSound);

impl CreateEntity for EnvSound {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            radius: 0.0,
            room_type: 0.0,
        }
    }
}

impl Entity for EnvSound {
    delegate_entity!(base not { key_value, spawn, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"radius" => self.radius = data.parse_or_default(),
            b"roomtype" => self.room_type = data.parse_or_default(),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        self.vars()
            .set_next_think_time_from_now(self.engine().random_float(0.0, 0.5));
    }

    fn think(&self) {
        let engine = self.engine();

        let Some(player) = engine
            .find_client_in_pvs(self)
            .downcast_ref::<dyn EntityPlayer>()
        else {
            return self.set_next_think_slow();
        };

        if let Some(last) = player.env_sound() {
            if *self.vars() == last.entity().vars() {
                // this is the last entity that modified the player room type
                match self.in_range(player.vars()) {
                    Some(range) => {
                        // player is in the range and visible
                        player.set_env_sound(Some(last.with_range(range)));
                        return self.set_next_think_fast();
                    }
                    None => {
                        // player is not in the range or not visible
                        player.set_env_sound(None);
                        return self.set_next_think_slow();
                    }
                }
            }
        }

        if let Some(range) = self.in_range(player.vars()) {
            if player.env_sound().map_or(true, |i| i.range() < range) {
                player.set_env_sound(Some(LastSound::new(self.entity_handle(), range)));

                let msg = user_message::RoomType::new(self.room_type as u16);
                engine.msg_one_reliable(player.vars(), &msg);
            }
        }

        self.set_next_think_fast();
    }
}

export_entity_default!("export-env_sound", env_sound, EnvSound);
