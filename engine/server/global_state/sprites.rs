use xash3d_shared::utils::AsAny;

use crate::prelude::*;

pub trait Sprites: AsAny {
    fn laser(&self) -> u16;

    fn laser_dot(&self) -> u16;

    fn fireball(&self) -> u16;

    fn smoke(&self) -> u16;

    fn wexplosion(&self) -> u16;

    fn bubbles(&self) -> u16;

    fn blood_drop(&self) -> u16;

    fn blood_spray(&self) -> u16;
}

pub struct StubSprites(());

impl StubSprites {
    pub fn new(_: ServerEngineRef) -> Self {
        Self(())
    }
}

#[rustfmt::skip]
impl Sprites for StubSprites {
    fn laser(&self) -> u16 { 0 }
    fn laser_dot(&self) -> u16 { 0 }
    fn fireball(&self) -> u16 { 0 }
    fn smoke(&self) -> u16 { 0 }
    fn wexplosion(&self) -> u16 { 0 }
    fn bubbles(&self) -> u16 { 0 }
    fn blood_drop(&self) -> u16 { 0 }
    fn blood_spray(&self) -> u16 { 0 }
}

pub struct DefaultSprites {
    laser: u16,
    laser_dot: u16,
    fireball: u16,
    smoke: u16,
    wexplosion: u16,
    bubbles: u16,
    blood_drop: u16,
    blood_spray: u16,
}

impl DefaultSprites {
    pub fn new(engine: ServerEngineRef) -> Self {
        Self {
            laser: engine.precache_model(c"sprites/laserbeam.spr") as u16,
            laser_dot: engine.precache_model(c"sprites/laserdot.spr") as u16,
            fireball: engine.precache_model(c"sprites/zerogxplode.spr") as u16,
            smoke: engine.precache_model(c"sprites/steam1.spr") as u16,
            wexplosion: engine.precache_model(c"sprites/WXplo1.spr") as u16,
            bubbles: engine.precache_model(c"sprites/bubble.spr") as u16,
            blood_drop: engine.precache_model(c"sprites/blood.spr") as u16,
            blood_spray: engine.precache_model(c"sprites/bloodspray.spr") as u16,
        }
    }
}

#[rustfmt::skip]
impl Sprites for DefaultSprites {
    fn laser(&self) -> u16 { self.laser }
    fn laser_dot(&self) -> u16 { self.laser_dot }
    fn fireball(&self) -> u16 { self.fireball }
    fn smoke(&self) -> u16 { self.smoke }
    fn wexplosion(&self) -> u16 { self.wexplosion }
    fn bubbles(&self) -> u16 { self.bubbles }
    fn blood_drop(&self) -> u16 { self.blood_drop }
    fn blood_spray(&self) -> u16 { self.blood_spray }
}
