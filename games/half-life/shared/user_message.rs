use core::ffi::CStr;

use xash3d_shared::{
    ffi::common::vec3_t,
    user_message::{define_user_message, Coord},
};

pub use xash3d_shared::user_message::HudText;

define_user_message! {
    pub struct SelAmmo {
        pub ammo1_type: u8,
        pub ammo1: u8,
        pub ammo2_type: u8,
        pub ammo2: u8,
    }
}

define_user_message! {
    pub struct CurWeapon {
        pub state: u8,
        pub id: i8,
        pub clip: i8,
    }
}

define_user_message! {
    pub struct Geiger {
        pub range: u8,
    }
}

impl Geiger {
    pub const fn new(range: u8) -> Self {
        Self { range }
    }
}

define_user_message! {
    pub struct Flashlight {
        pub on: bool,
        pub battery: u8,
    }
}

impl Flashlight {
    pub const fn new(on: bool, battery: u8) -> Self {
        Self { on, battery }
    }
}

define_user_message! {
    pub struct FlashBat {
        pub battery: u8,
    }
}

impl FlashBat {
    pub const fn new(battery: u8) -> Self {
        Self { battery }
    }
}

define_user_message! {
    pub struct Health {
        pub health: u8,
    }
}

impl Health {
    pub const fn new(health: u8) -> Self {
        Self { health }
    }
}

define_user_message! {
    pub struct Damage {
        pub armor: u8,
        pub damage_taken: u8,
        pub damage_bits: u32,
        pub from: Coord<vec3_t>,
    }
}

define_user_message! {
    pub struct Battery {
        pub battery: i16,
    }
}

impl Battery {
    pub const fn new(battery: i16) -> Self {
        Self { battery }
    }
}

define_user_message! {
    pub struct Train {
        pub pos: u8,
    }
}

impl Train {
    pub const fn new(pos: u8) -> Self {
        Self { pos }
    }
}

define_user_message! {
    pub struct SayText<'a> {
        pub client_index: u8,
        pub text: &'a CStr,
    }
}

// TODO: define_user_message!(TextMsg)

define_user_message! {
    pub struct WeaponList<'a> {
        pub name: &'a CStr,
        pub ammo1: i8,
        pub max1: u8,
        pub ammo2: i8,
        pub max2: u8,
        pub slot: i8,
        pub slot_pos: i8,
        pub id: i8,
        pub flags: u8,
    }
}

define_user_message! {
    pub struct ResetHUD {
        pub x: u8,
    }
}

define_user_message! {
    pub struct InitHUD {}
}

define_user_message! {
    pub struct GameTitle {
        pub x: u8,
    }
}

define_user_message! {
    pub struct DeathMsg<'a> {
        pub killer: u8,
        pub victim: u8,
        pub killed_with: &'a CStr,
    }
}

define_user_message! {
    pub struct ScoreInfo {
        pub cl: u8,
        pub frags: i16,
        pub deaths: i16,
        pub player_class: i16,
        pub teamnumber: i16,
    }
}

// TODO: define_user_message!(TeamInfo)
// TODO: define_user_message!(TeamScore)

define_user_message! {
    pub struct GameMode {
        pub mode: u8,
    }
}

// TODO: define_user_message!(MOTD)

define_user_message! {
    pub struct ServerName<'a> {
        pub name: &'a CStr,
    }
}

define_user_message! {
    pub struct AmmoPickup {
        pub index: u8,
        pub count: u8,
    }
}

define_user_message! {
    pub struct WeapPickup {
        pub index: u8,
    }
}

define_user_message! {
    pub struct ItemPickup<'a> {
        pub classname: &'a CStr,
    }
}

define_user_message! {
    pub struct HideWeapon {
        pub hide: u8,
    }
}

define_user_message! {
    pub struct SetFOV {
        pub fov: u8,
    }
}

define_user_message! {
    pub struct AmmoX {
        pub ty: u8,
        pub count: u8,
    }
}

// TODO: define_user_message!(TeamNames);
// TODO: define_user_message!(StatusText);
// TODO: define_user_message!(StatusValue);
