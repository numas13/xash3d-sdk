use xash3d_server::{
    engine::TraceIgnore,
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, EntityPlayer, EntityVars, KeyValue, LastSound},
    prelude::*,
    private::impl_private,
    user_message::{self, RoomType},
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct EnvSound {
    base: PointEntity,
    radius: f32,
    room_type: RoomType,
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

impl CreateEntity for EnvSound {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
            radius: 0.0,
            room_type: RoomType::default(),
        }
    }
}

impl Entity for EnvSound {
    delegate_entity!(base not { key_value, spawn, think });

    fn key_value(&mut self, data: &mut KeyValue) {
        match data.key_name().to_bytes() {
            b"radius" => self.radius = data.parse_or_default(),
            b"roomtype" => {
                let value = data.parse_or_default();
                self.room_type = RoomType::from_raw(value).unwrap_or_default();
            }
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

                trace!("set room type {:?}", self.room_type);
                let msg = user_message::SetRoomType::new(self.room_type);
                engine.msg_one_reliable(player.vars(), &msg);
            }
        }

        self.set_next_think_fast();
    }
}

impl_private!(EnvSound {});

define_export! {
    export_env_sound as export if "env-sound" {
        env_sound = env_sound::EnvSound,
    }
}
