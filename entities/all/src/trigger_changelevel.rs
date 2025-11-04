use csz::{cstr, CStrArray, CStrThin};
use xash3d_server::{
    change_level::{find_landmark, in_transition_volume},
    entities::trigger::Trigger,
    entity::{delegate_entity, BaseEntity, EntityChangeLevel, KeyValue, UseType},
    prelude::*,
    private::impl_private,
    str::MapString,
    utils,
};

const MAP_NAME_MAX: usize = 32;

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct ChangeLevel {
    base: Trigger,

    map_name: CStrArray<MAP_NAME_MAX>,
    landmark_name: CStrArray<MAP_NAME_MAX>,
    change_target: Option<MapString>,
    change_target_delay: f32,
}

impl CreateEntity for ChangeLevel {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: Trigger::create(base),
            map_name: Default::default(),
            landmark_name: Default::default(),
            change_target: None,
            change_target_delay: 0.0,
        }
    }
}

impl ChangeLevel {
    const SF_USE_ONLY: u32 = 1 << 1;

    fn change_level_now(&self, activator: Option<&dyn Entity>) {
        let name = self.pretty_name();
        let engine = self.engine();
        let v = self.vars();

        if self.map_name.is_empty() {
            panic!("{name}: Detected problems with save/restore!!!");
        }

        if self.global_state().game_rules().is_deathmatch() {
            return;
        }

        // do not fire multiple times per frame
        let now = engine.globals.map_time();
        if now == v.damage_time() {
            return;
        }
        v.set_damage_time(now);

        let player = engine.get_single_player().expect("player entity");
        if !in_transition_volume(&engine, player, &self.landmark_name) {
            let landmark = &self.landmark_name;
            debug!("{name}: player is not in the transition volume {landmark}, aborting");
            return;
        }

        if let Some(change_target) = self.change_target {
            warn!("{name}: change target ({change_target}) is not implemented yet");
        }

        utils::use_targets(UseType::Toggle, activator, self);

        let mut next_spot = cstr!("");
        if let Some(landmark) = find_landmark(&engine, &self.landmark_name) {
            next_spot = &self.landmark_name;
            engine.globals.set_landmark_offset(landmark.vars().origin());
        }

        info!("CHANGE LEVEL: {:?} {next_spot:?}", self.map_name);
        engine.change_level(&self.map_name, next_spot);
    }
}

impl Entity for ChangeLevel {
    delegate_entity!(base not { key_value, spawn, used, touched });

    fn key_value(&mut self, data: &mut KeyValue) {
        let engine = self.engine();
        let value = data.value();

        match data.key_name().to_bytes() {
            b"map" => {
                if self.map_name.cursor().write_c_str(value).is_err() {
                    let name = self.pretty_name();
                    error!("{name}: map name is too long ({value:?})");
                }
            }
            b"landmark" => {
                if self.landmark_name.cursor().write_c_str(value).is_err() {
                    let name = self.pretty_name();
                    error!("{name}: landmark name is too long ({value:?})");
                }
            }
            b"changetarget" => self.change_target = Some(engine.new_map_string(value)),
            b"changedelay" => self.change_target_delay = data.parse_or_default(),
            _ => return self.base.key_value(data),
        }
        data.set_handled(true);
    }

    fn spawn(&mut self) {
        let name = self.pretty_name();
        if self.map_name.is_empty() {
            warn!("{name}: map is empty");
        }
        if self.landmark_name.is_empty() {
            info!("{name}: does not have a landmark to map {}", self.map_name);
        }
        self.base.spawn();
    }

    fn used(&self, _: UseType, activator: Option<&dyn Entity>, _: &dyn Entity) {
        if self.vars().target_name().is_some() {
            self.change_level_now(activator);
        }
    }

    fn touched(&self, other: &dyn Entity) {
        if self.vars().spawn_flags() & Self::SF_USE_ONLY != 0 {
            return;
        }
        if other.vars().is_class_name(c"player") {
            self.change_level_now(Some(other));
        }
    }
}

impl EntityChangeLevel for ChangeLevel {
    fn map_name(&self) -> &CStrThin {
        &self.map_name
    }

    fn landmark_name(&self) -> &CStrThin {
        &self.landmark_name
    }
}

impl_private!(ChangeLevel { EntityChangeLevel });

define_export! {
    export_trigger_changelevel as export if "trigger-changelevel" {
        trigger_changelevel = trigger_changelevel::ChangeLevel,
    }
}
