use core::ffi::CStr;

use xash3d_hl_shared::user_message;
use xash3d_server::{
    engine::ServerEngineRef,
    entities::player::Player as BasePlayer,
    entity::{
        delegate_entity, delegate_player, impl_entity_cast, impl_save_restore, BaseEntity,
        CreateEntity, Effects, Entity, EntityPlayer, Private, PrivateData,
    },
    export::export_entity,
    ffi::server::{edict_s, TYPEDESCRIPTION},
    global_state::GlobalStateRef,
    prelude::EntityVarsExt,
    save::{define_fields, SaveFields},
};

pub struct TestPlayer {
    base: BasePlayer,
    init_hud: u8,
    game_hud_initialized: u8,
}

unsafe impl SaveFields for TestPlayer {
    const SAVE_NAME: &'static CStr = c"TestPlayer";
    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] =
        &define_fields![init_hud, game_hud_initialized,];
}

impl_entity_cast!(TestPlayer);

impl CreateEntity for TestPlayer {
    fn create(base: BaseEntity) -> Self {
        Self {
            base: BasePlayer::create(base),
            init_hud: 1,
            game_hud_initialized: 0,
        }
    }
}

impl Entity for TestPlayer {
    delegate_entity!(base not { save, restore, spawn });
    impl_save_restore!(base);

    fn spawn(&mut self) {
        self.base.spawn();

        // enable suit
        // TODO: move Weapons type to shared crate
        self.vars_mut().as_raw_mut().weapons |= 1 << 31;
    }
}

impl EntityPlayer for TestPlayer {
    delegate_player!(base not { pre_think });

    fn pre_think(&mut self) {
        let engine = self.engine();

        if self.init_hud != 0 {
            self.init_hud = 0;

            engine.msg_one(self, &user_message::ResetHUD::default());

            if self.game_hud_initialized == 0 {
                self.game_hud_initialized = 1;
                engine.msg_one(self, &user_message::InitHUD::default());
            }

            engine.msg_one(self, &user_message::Geiger::default());
            // TODO: move Hide to shared crate
            engine.msg_one(self, &user_message::HideWeapon::default());
            engine.msg_one(self, &user_message::Health { x: 13 });
            engine.msg_one(self, &user_message::Battery { x: 31 });

            engine.msg_one(self, &user_message::Flashlight { on: true, x: 50 });
            self.vars_mut().effects_mut().insert(Effects::DIMLIGHT);
        }
    }
}

pub fn client_put_in_server(
    engine: ServerEngineRef,
    global_state: GlobalStateRef,
    ent: &mut edict_s,
) {
    let player =
        unsafe { PrivateData::create::<Private<TestPlayer>>(engine, global_state, &mut ent.v) };

    player.spawn();

    ent.v.effects_mut().insert(Effects::NOINTERP);
    ent.v.iuser1 = 0;
    ent.v.iuser2 = 0;
}

export_entity!(player, Private<TestPlayer>);
