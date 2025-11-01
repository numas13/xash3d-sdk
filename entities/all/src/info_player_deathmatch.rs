use xash3d_server::{
    entities::point_entity::PointEntity,
    entity::{delegate_entity, BaseEntity, KeyValue},
    prelude::*,
    private::impl_private,
    utils,
};

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct DeathMatchStart {
    base: PointEntity,
}

impl CreateEntity for DeathMatchStart {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: PointEntity::create(base),
        }
    }
}

impl Entity for DeathMatchStart {
    delegate_entity!(base not { key_value, is_triggered });

    fn key_value(&mut self, data: &mut KeyValue) {
        if data.key_name() == c"master" {
            let engine = self.engine();
            self.vars()
                .set_net_name(engine.new_map_string(data.value()));
            data.set_handled(true);
        } else {
            self.base.key_value(data);
        }
    }

    fn is_triggered(&self, activator: Option<&dyn Entity>) -> bool {
        let engine = self.engine();
        utils::is_master_triggered(&engine, self.vars().net_name(), activator)
    }
}

impl_private!(DeathMatchStart {});

define_export! {
    export_info_player_deathmatch as export if "info-player-deathmatch" {
        info_player_deathmatch = info_player_deathmatch::DeathMatchStart,
    }
}
