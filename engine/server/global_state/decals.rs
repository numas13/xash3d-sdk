use core::ffi::CStr;

use xash3d_shared::utils::AsAny;

use crate::prelude::*;

pub trait Decals: AsAny {
    fn get_random_gunshot(&self) -> u16;

    fn get_random_bigshot(&self) -> u16;

    fn get_random_blood(&self) -> u16;

    fn get_random_yellow_blood(&self) -> u16;

    fn get_random_scorch(&self) -> u16;

    fn get_random_small_scorch(&self) -> u16;

    fn get_random_glass_break(&self) -> u16;

    fn get_random_spit(&self) -> u16;
}

pub struct StubDecals(());

impl StubDecals {
    pub fn new(_: ServerEngineRef) -> Self {
        Self(())
    }
}

#[rustfmt::skip]
impl Decals for StubDecals {
    fn get_random_gunshot(&self) -> u16 { 0 }
    fn get_random_bigshot(&self) -> u16 { 0 }
    fn get_random_blood(&self) -> u16 { 0 }
    fn get_random_yellow_blood(&self) -> u16 { 0 }
    fn get_random_scorch(&self) -> u16 { 0 }
    fn get_random_small_scorch(&self) -> u16 { 0 }
    fn get_random_glass_break(&self) -> u16 { 0 }
    fn get_random_spit(&self) -> u16 { 0 }
}

macro_rules! define_decals {
    ($( #[$attr:meta] )* $vis:vis enum $name:ident {
        $( $variant:ident($decal:expr) $(= $discriminant:expr)? ),+ $(,)?
    }) => {
        $( #[$attr] )*
        $vis enum $name {
            $($variant $(= $discriminant)? ),+
        }

        impl $name {
            fn as_c_str(&self) -> &CStr {
                match self {
                    $( Self::$variant => $decal ),+
                }
            }

            fn all() -> &'static [$name] {
                &[$( Self::$variant ),+]
            }
        }
    };
}

define_decals! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    pub enum Decal {
        GunShot1(c"{shot1") = 0,
        GunShot2(c"{shot2"),
        GunShot3(c"{shot3"),
        GunShot4(c"{shot4"),
        GunShot5(c"{shot5"),
        Lambda1(c"{lambda01"),
        Lambda2(c"{lambda02"),
        Lambda3(c"{lambda03"),
        Lambda4(c"{lambda04"),
        Lambda5(c"{lambda05"),
        Lambda6(c"{lambda06"),
        Scorch1(c"{scorch1"),
        Scorch2(c"{scorch2"),
        Blood1(c"{blood1"),
        Blood2(c"{blood2"),
        Blood3(c"{blood3"),
        Blood4(c"{blood4"),
        Blood5(c"{blood5"),
        Blood6(c"{blood6"),
        YellowBlood1(c"{yblood1"),
        YellowBlood2(c"{yblood2"),
        YellowBlood3(c"{yblood3"),
        YellowBlood4(c"{yblood4"),
        YellowBlood5(c"{yblood5"),
        YellowBlood6(c"{yblood6"),
        GlassBreak1(c"{break1"),
        GlassBreak2(c"{break2"),
        GlassBreak3(c"{break3"),
        BigShot1(c"{bigshot1"),
        BigShot2(c"{bigshot2"),
        BigShot3(c"{bigshot3"),
        BigShot4(c"{bigshot4"),
        BigShot5(c"{bigshot5"),
        Spit1(c"{spit1"),
        Spit2(c"{spit2"),
        BulletProofGlass1(c"{bproof1"),
        GargStomp1(c"{gargstomp"),
        SmallScorch1(c"{smscorch1"),
        SmallScorch2(c"{smscorch2"),
        SmallScorch3(c"{smscorch3"),
        MommaBirth(c"{mommablob"),
        MommaSplat(c"{mommablob"),
    }
}

impl Decal {
    const COUNT: usize = Self::MommaSplat as usize + 1;
}

pub struct DefaultDecals {
    engine: ServerEngineRef,
    list: [u16; Decal::COUNT],
}

impl DefaultDecals {
    pub fn new(engine: ServerEngineRef) -> Self {
        let mut list = [0; Decal::COUNT];
        for &decal in Decal::all() {
            list[decal as usize] = engine.decal_index(decal.as_c_str()).unwrap_or(0);
        }
        Self { engine, list }
    }

    fn get_index(&self, decal: Decal) -> u16 {
        self.list[decal as usize]
    }

    fn get_random(&self, decals: &[Decal]) -> u16 {
        let index = self.engine.random_int(0, decals.len() as i32 - 1);
        self.get_index(decals[index as usize])
    }
}

impl Decals for DefaultDecals {
    fn get_random_gunshot(&self) -> u16 {
        self.get_random(&[
            Decal::GunShot1,
            Decal::GunShot2,
            Decal::GunShot3,
            Decal::GunShot4,
            Decal::GunShot5,
        ])
    }

    fn get_random_bigshot(&self) -> u16 {
        self.get_random(&[
            Decal::BigShot1,
            Decal::BigShot2,
            Decal::BigShot3,
            Decal::BigShot4,
            Decal::BigShot5,
        ])
    }

    fn get_random_blood(&self) -> u16 {
        self.get_random(&[
            Decal::Blood1,
            Decal::Blood2,
            Decal::Blood3,
            Decal::Blood4,
            Decal::Blood5,
            Decal::Blood6,
        ])
    }

    fn get_random_yellow_blood(&self) -> u16 {
        self.get_random(&[
            Decal::YellowBlood1,
            Decal::YellowBlood2,
            Decal::YellowBlood3,
            Decal::YellowBlood4,
            Decal::YellowBlood5,
            Decal::YellowBlood6,
        ])
    }

    fn get_random_scorch(&self) -> u16 {
        self.get_random(&[Decal::Scorch1, Decal::Scorch2])
    }

    fn get_random_small_scorch(&self) -> u16 {
        self.get_random(&[
            Decal::SmallScorch1,
            Decal::SmallScorch2,
            Decal::SmallScorch3,
        ])
    }

    fn get_random_glass_break(&self) -> u16 {
        self.get_random(&[Decal::GlassBreak1, Decal::GlassBreak2, Decal::GlassBreak3])
    }

    fn get_random_spit(&self) -> u16 {
        self.get_random(&[Decal::Spit1, Decal::Spit2])
    }
}
