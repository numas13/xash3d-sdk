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

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Weapons: u32 {
        const NONE          = 0;
        const CROWBAR       = 1 << 1;
        const GLOCK         = 1 << 2;
        const PYTHON        = 1 << 3;
        const MP5           = 1 << 4;
        const CHAINGUN      = 1 << 5;
        const CROSSBOW      = 1 << 6;
        const SHOTGUN       = 1 << 7;
        const RPG           = 1 << 8;
        const GAUSS         = 1 << 9;
        const EGON          = 1 << 10;
        const HORNETGUN     = 1 << 11;
        const HANDGRENADE   = 1 << 12;
        const TRIPMINE      = 1 << 13;
        const SATCHEL       = 1 << 14;
        const SNARK         = 1 << 15;

        const ALL           = !Self::SUIT.bits();

        const SUIT          = 1 << 31;
    }
}
