pub mod crossbow;
pub mod crowbar;
pub mod egon;
pub mod gauss;
pub mod glock;
pub mod hgun;
pub mod mp5;
pub mod python;
pub mod rpg;
pub mod shotgun;
pub mod snark;
pub mod tripmine;

use bitflags::bitflags;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Items {
    Healthkit = 1,
    Antidote,
    Security,
    Battery,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Weapon {
    #[default]
    None = 0,
    Crowbar = 1,
    Glock = 2,
    Python = 3,
    Mp5 = 4,
    Chaingun = 5,
    Crossbow = 6,
    Shotgun = 7,
    Rpg = 8,
    Gauss = 9,
    Egon = 10,
    HornetGun = 11,
    HandGrenade = 12,
    Tripmine = 13,
    Satchel = 14,
    Snark = 15,

    Suit = 31,
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Weapons: u32 {
        const NONE          = 0;
        const CROWBAR       = 1 << Weapon::Crowbar as u32;
        const GLOCK         = 1 << Weapon::Glock as u32;
        const PYTHON        = 1 << Weapon::Python as u32;
        const MP5           = 1 << Weapon::Mp5 as u32;
        const CHAINGUN      = 1 << Weapon::Chaingun as u32;
        const CROSSBOW      = 1 << Weapon::Crossbow as u32;
        const SHOTGUN       = 1 << Weapon::Shotgun as u32;
        const RPG           = 1 << Weapon::Rpg as u32;
        const GAUSS         = 1 << Weapon::Gauss as u32;
        const EGON          = 1 << Weapon::Egon as u32;
        const HORNETGUN     = 1 << Weapon::HornetGun as u32;
        const HANDGRENADE   = 1 << Weapon::HandGrenade as u32;
        const TRIPMINE      = 1 << Weapon::Tripmine as u32;
        const SATCHEL       = 1 << Weapon::Satchel as u32;
        const SNARK         = 1 << Weapon::Snark as u32;

        const ALL           = !Self::SUIT.bits();

        const SUIT          = 1 << Weapon::Suit as u32;
    }
}
