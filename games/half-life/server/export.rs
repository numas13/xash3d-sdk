use core::ffi::{c_int, CStr};

use xash3d_server::{
    engine::RegisterUserMessageError,
    entities::world::World,
    entity::{BaseEntity, EntityHandle, EntityPlayer},
    export::{export_dll, impl_unsync_global, ServerDll},
    global_state::GlobalStateRef,
    prelude::*,
    private::Private,
    user_message::register_user_message,
};

use crate::{entities::player::TestPlayer, game_rules::install_game_rules, user_message};

struct Dll {
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
}

impl_unsync_global!(Dll);

impl Dll {
    fn register_user_messages(engine: ServerEngineRef) -> Result<(), RegisterUserMessageError> {
        register_user_message!(engine, user_message::SelAmmo)?;
        register_user_message!(engine, user_message::CurWeapon)?;
        register_user_message!(engine, user_message::Geiger)?;
        register_user_message!(engine, user_message::Flashlight)?;
        register_user_message!(engine, user_message::FlashBat)?;
        register_user_message!(engine, user_message::Health)?;
        register_user_message!(engine, user_message::Damage)?;
        register_user_message!(engine, user_message::Battery)?;
        register_user_message!(engine, user_message::Train)?;
        register_user_message!(engine, user_message::HudText)?;
        register_user_message!(engine, user_message::SayText)?;
        // register_user_message!(engine, user_message::TextMsg)?;
        register_user_message!(engine, user_message::WeaponList)?;
        register_user_message!(engine, user_message::ResetHUD)?;
        register_user_message!(engine, user_message::InitHUD)?;
        register_user_message!(engine, user_message::GameTitle)?;
        register_user_message!(engine, user_message::DeathMsg)?;
        register_user_message!(engine, user_message::ScoreInfo)?;
        // register_user_message!(engine, user_message::TeamInfo)?;
        // register_user_message!(engine, user_message::TeamScore)?;
        register_user_message!(engine, user_message::GameMode)?;
        // register_user_message!(engine, user_message::MOTD)?;
        register_user_message!(engine, user_message::ServerName)?;
        register_user_message!(engine, user_message::AmmoPickup)?;
        register_user_message!(engine, user_message::WeapPickup)?;
        register_user_message!(engine, user_message::ItemPickup)?;
        register_user_message!(engine, user_message::HideWeapon)?;
        register_user_message!(engine, user_message::SetFOV)?;
        register_user_message!(engine, user_message::ScreenShake)?;
        register_user_message!(engine, user_message::ScreenFade)?;
        register_user_message!(engine, user_message::AmmoX)?;
        // register_user_message!(engine, user_message::TeamNames)?;
        // register_user_message!(engine, user_message::StatusText)?;
        // register_user_message!(engine, user_message::StatusValue)?;

        Ok(())
    }
}

impl ServerDll for Dll {
    type World = Private<World>;
    type Player = Private<TestPlayer>;

    fn new(engine: ServerEngineRef, global_state: GlobalStateRef) -> Self {
        crate::cvar::init(engine);
        if let Err(err) = Self::register_user_messages(engine) {
            panic!("{err}");
        }
        Self {
            engine,
            global_state,
        }
    }

    fn create_world(base: BaseEntity) -> World {
        World::create(base, install_game_rules)
    }

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }

    fn get_game_description_static() -> &'static CStr {
        c"Half-Life"
    }

    fn client_command(&self, ent: EntityHandle) {
        use xash3d_server::{entity::UseType, utils};
        let engine = self.engine();
        let name = engine.cmd_argv(0);
        match name.to_bytes() {
            b"fullupdate" => {
                if let Some(player) = ent.downcast_ref::<TestPlayer>() {
                    player.force_update_client_data();
                }
            }
            b"fire" => {
                let target = engine.cmd_argv(1);
                if target.is_empty() {
                    info!("usage: fire target");
                    return;
                }
                let player = ent.get_entity().unwrap();
                utils::fire_targets(target, UseType::Toggle, Some(player), player);
            }
            b"give" => {
                if let Some(player) = ent.downcast_ref::<dyn EntityPlayer>() {
                    for item_name in engine.cmd_args().skip(1) {
                        player.give_named_item(item_name);
                    }
                }
            }
            b"find" => {
                if let Some(player) = ent.downcast_ref::<TestPlayer>() {
                    player.find_class_name(engine.cmd_argv(1));
                }
            }
            b"health" => {
                if let Some(player) = ent.downcast_ref::<TestPlayer>() {
                    if let Ok(arg) = engine.cmd_argv(1).to_str() {
                        if let Ok(health) = arg.parse() {
                            player.vars().set_health(health);
                        }
                    }
                }
            }
            _ => {
                if let Some(args) = self.engine.cmd_args_raw() {
                    warn!("unimplemented client command \"{name} {args}\"");
                }
            }
        }
    }
}

export_dll!(Dll);
