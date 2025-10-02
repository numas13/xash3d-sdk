use core::ffi::c_int;

use xash3d_server::{
    engine::RegisterUserMessageError,
    export::{export_dll, impl_unsync_global, ServerDll},
    global_state::GlobalStateRef,
    prelude::*,
    user_message::register_user_message,
};

struct Dll {
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
}

impl_unsync_global!(Dll);

impl Dll {
    fn register_user_messages(engine: ServerEngineRef) -> Result<(), RegisterUserMessageError> {
        use xash3d_hl_shared::user_message;

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
        // register_user_message!(engine, user_message::ScreenShake)?;
        // register_user_message!(engine, user_message::ScreenFade)?;
        register_user_message!(engine, user_message::AmmoX)?;
        // register_user_message!(engine, user_message::TeamNames)?;
        // register_user_message!(engine, user_message::StatusText)?;
        // register_user_message!(engine, user_message::StatusValue)?;
        register_user_message!(engine, user_message::SetFOV)?;
        register_user_message!(engine, user_message::ScoreInfo)?;

        Ok(())
    }
}

impl ServerDll for Dll {
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

    fn engine(&self) -> ServerEngineRef {
        self.engine
    }

    fn global_state(&self) -> GlobalStateRef {
        self.global_state
    }
}

export_dll!(Dll);
