use core::cmp::min;

use bitflags::bitflags;
use xash3d_shared::{
    consts::SOLID_NOT,
    entity::MoveType,
    ffi::common::PITCH_NORM,
    sound::{Attenuation, Pitch, SoundFlags},
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};
use crate::{
    entity::{
        delegate_entity, impl_entity_cast, AsEdict, BaseEntity, CreateEntity, Entity, KeyValue,
        ObjectCaps, UseType,
    },
    prelude::*,
    str::MapString,
};

const LFO_SQUARE: i32 = 1;
const LFO_TRIANGLE: i32 = 2;
const LFO_RANDOM: i32 = 3;

trait Fixup {
    fn fixup(self) -> Self;
}

impl Fixup for i32 {
    fn fixup(self) -> Self {
        if self > 0 {
            (101 - self) * 64
        } else {
            self
        }
    }
}

/// Runtime pitch shift and volume fadein/out.
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
struct DynamicPitchVolume {
    preset: i32,

    // pitch shift % when sound is running 0 - 255
    pitchrun: i32,
    // pitch shift % when sound stops or starts 0 - 255
    pitchstart: i32,
    // spinup time 0 - 100
    spinup: i32,
    // spindown time 0 - 100
    spindown: i32,

    // volume change % when sound is running 0 - 10
    volrun: i32,
    // volume change % when sound stops or starts 0 - 10
    volstart: i32,
    // volume fade in time 0 - 100
    fadein: i32,
    // volume fade out time 0 - 100
    fadeout: i32,

    // Low Frequency Oscillator

    // 0) off 1) square 2) triangle 3) random
    lfotype: i32,
    // 0 - 1000, how fast lfo osciallates
    lforate: i32,

    // 0-100 mod of current pitch. 0 is off.
    lfomodpitch: i32,
    // 0-100 mod of current volume. 0 is off.
    lfomodvol: i32,

    // each trigger hit increments counter and spinup pitch
    cspinup: i32,

    cspincount: i32,

    pitch: i32,
    spinupsav: i32,
    spindownsav: i32,
    pitchfrac: i32,

    vol: i32,
    fadeinsav: i32,
    fadeoutsav: i32,
    volfrac: i32,

    lfofrac: i32,
    lfomult: i32,
}

macro_rules! define_presets {
    ($({
        $preset:expr,
        $pitch_run:expr,
        $pitch_start:expr,
        $spinup:expr,
        $spindwn:expr,
        $volrun:expr,
        $fadein:expr,
        $fadeout:expr,
        $lfotype:expr,
        $lforate:expr,
        $modptch:expr,
        $modvol:expr,
        $cspinup:expr $(,)?
    }),* $(,)?) => {
        &[$(
            DynamicPitchVolume {
                preset: $preset,
                pitchrun: $pitch_run,
                pitchstart: $pitch_start,
                spinup: $spinup,
                spindown: $spindwn,
                volrun: $volrun,
                volstart: 1,
                fadein: $fadein,
                fadeout: $fadeout,
                lfotype: $lfotype,
                lforate: $lforate,
                lfomodpitch: $modptch,
                lfomodvol: $modvol,
                cspinup: $cspinup,
                ..unsafe { core::mem::zeroed::<DynamicPitchVolume>() }
            }
        ),*]
    };
}

#[rustfmt::skip]
const DPV_PRESET: &[DynamicPitchVolume] = define_presets![
    //  pitch pstart spinup spindwn volrun fadein fadeout lfotype lforate modptch modvol cspnup
    { 1, 255,    75,    95,     95,    10,    50,     95,      0,      0,      0,     0,     0},
    { 2, 255,    85,    70,     88,    10,    20,     88,      0,      0,      0,     0,     0},
    { 3, 255,   100,    50,     75,    10,    10,     75,      0,      0,      0,     0,     0},
    { 4, 100,   100,     0,      0,    10,    90,     90,      0,      0,      0,     0,     0},
    { 5, 100,   100,     0,      0,    10,    80,     80,      0,      0,      0,     0,     0},
    { 6, 100,   100,     0,      0,    10,    50,     70,      0,      0,      0,     0,     0},
    { 7, 100,   100,     0,      0,     5,    40,     50,      1,     50,      0,    10,     0},
    { 8, 100,   100,     0,      0,     5,    40,     50,      1,    150,      0,    10,     0},
    { 9, 100,   100,     0,      0,     5,    40,     50,      1,    750,      0,    10,     0},
    {10, 128,   100,    50,     75,    10,    30,     40,      2,      8,     20,     0,     0},
    {11, 128,   100,    50,     75,    10,    30,     40,      2,     25,     20,     0,     0},
    {12, 128,   100,    50,     75,    10,    30,     40,      2,     70,     20,     0,     0},
    {13,  50,    50,     0,      0,    10,    20,     50,      0,      0,      0,     0,     0},
    {14,  70,    70,     0,      0,    10,    20,     50,      0,      0,      0,     0,     0},
    {15,  90,    90,     0,      0,    10,    20,     50,      0,      0,      0,     0,     0},
    {16, 120,   120,     0,      0,    10,    20,     50,      0,      0,      0,     0,     0},
    {17, 180,   180,     0,      0,    10,    20,     50,      0,      0,      0,     0,     0},
    {18, 255,   255,     0,      0,    10,    20,     50,      0,      0,      0,     0,     0},
    {19, 200,    75,    90,     90,    10,    50,     90,      2,    100,     20,     0,     0},
    {20, 255,    75,    97,     90,    10,    50,     90,      1,     40,     50,     0,     0},
    {21, 100,   100,     0,      0,    10,    30,     50,      3,     15,     20,     0,     0},
    {22, 160,   160,     0,      0,    10,    50,     50,      3,    500,     25,     0,     0},
    {23, 255,    75,    88,      0,    10,    40,      0,      0,      0,      0,     0,     5},
    {24, 200,    20,    95,     70,    10,    70,     70,      3,     20,     50,     0,     0},
    {25, 180,   100,    50,     60,    10,    40,     60,      2,     90,    100,   100,     0},
    {26,  60,    60,     0,      0,    10,    40,     70,      3,     80,     20,    50,     0},
    {27, 128,    90,    10,     10,    10,    20,     40,      1,      5,     10,    20,     0},
];

impl DynamicPitchVolume {
    fn key_value(&mut self, data: &mut KeyValue) {
        let value = data.value_str();
        match data.key_name().to_bytes() {
            b"preset" => {
                self.preset = value.parse().unwrap_or(0);
            }
            b"pitch" => {
                self.pitchrun = value.parse().unwrap_or(0).clamp(0, 255);
            }
            b"pitchstart" => {
                self.pitchstart = value.parse().unwrap_or(0).clamp(0, 255);
            }
            b"spinup" => {
                self.spinup = value.parse().unwrap_or(0).clamp(0, 100).fixup();
                self.spinupsav = self.spinup;
            }
            b"spindown" => {
                self.spindown = value.parse().unwrap_or(0).clamp(0, 100).fixup();
                self.spindownsav = self.spindown;
            }
            b"volstart" => {
                self.volstart = value.parse().unwrap_or(0).clamp(0, 10) * 10;
            }
            b"fadein" => {
                self.fadein = value.parse().unwrap_or(0).clamp(0, 100).fixup();
                self.fadeinsav = self.fadein;
            }
            b"fadeout" => {
                self.fadeout = value.parse().unwrap_or(0).clamp(0, 100).fixup();
                self.fadeoutsav = self.fadeout;
            }
            b"lfotype" => {
                self.lfotype = value.parse().unwrap_or(0);
                if self.lfotype > 4 {
                    self.lfotype = LFO_TRIANGLE;
                }
            }
            b"lforate" => {
                self.lforate = value.parse().unwrap_or(0).clamp(0, 1000) * 256;
            }
            b"lfomodpitch" => {
                self.lfomodpitch = value.parse().unwrap_or(0).clamp(0, 100);
            }
            b"lfomodvol" => {
                self.lfomodvol = value.parse().unwrap_or(0).clamp(0, 100);
            }
            b"cspinup" => {
                self.cspinup = value.parse().unwrap_or(0).clamp(0, 100);
            }
            _ => return,
        }
        data.set_handled(true);
    }

    fn is_active(&self) -> bool {
        self.spinup != 0
            || self.spindown != 0
            || self.fadein != 0
            || self.fadeout != 0
            || self.lfotype != 0
    }

    fn init(&mut self, health: f32) {
        self.volrun = ((health * 10.0) as i32).clamp(0, 100);

        if let Some(preset) = DPV_PRESET.get(self.preset as usize) {
            self.clone_from(preset);
            self.spindown = self.spindown.fixup();
            self.spinup = self.spinup.fixup();
            self.volstart *= 10;
            self.volrun *= 10;
            self.fadein = self.fadein.fixup();
            self.fadeout = self.fadeout.fixup();
            self.lforate *= 256;
            self.fadeinsav = self.fadein;
            self.fadeoutsav = self.fadeout;
            self.spinupsav = self.spinup;
            self.spindownsav = self.spindown;
        }

        self.fadein = self.fadeinsav;
        self.fadeout = 0;

        if self.fadein != 0 {
            self.vol = self.volstart;
        } else {
            self.vol = self.volrun;
        }

        self.spinup = self.spinupsav;
        self.spindown = 0;

        if self.spinup != 0 {
            self.pitch = self.pitchstart;
        } else {
            self.pitch = self.pitchrun;
        }

        if self.pitch == 0 {
            self.pitch = PITCH_NORM;
        }

        self.pitchfrac = self.pitch << 8;
        self.volfrac = self.vol << 8;

        self.lfofrac = 0;
        self.lforate = self.lforate.abs();

        self.cspincount = 1;

        if self.cspinup != 0 {
            let pitchinc = (255 - self.pitchstart) / self.cspinup;
            self.pitchrun = min(self.pitchstart + pitchinc, 255);
        }

        if (self.spinupsav != 0
            || self.spindownsav != 0
            || (self.lfotype != 0 && self.lfomodpitch != 0))
            && self.pitch == PITCH_NORM
        {
            self.pitch = PITCH_NORM + 1;
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct AmbientSound: i32 {
        // medium radius attenuation
        const STATIC          = 0;
        const EVERYWHERE      = 1 << 0;
        const SMALL_RADIUS    = 1 << 1;
        const MEDIUM_RADIUS   = 1 << 2;
        const LARGE_RADIUS    = 1 << 3;
        const START_SILENT    = 1 << 4;
        const NOT_LOOPING     = 1 << 5;
    }
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct AmbientGeneric {
    base: BaseEntity,

    attenuation: Attenuation,
    active: bool,
    looping: bool,
    remove_me: bool,
    dpv: DynamicPitchVolume,
}

impl AmbientGeneric {
    fn init_modulation_parms(&mut self) {
        self.dpv.init(self.vars().as_raw().health);
    }
}

impl_entity_cast!(AmbientGeneric);

impl CreateEntity for AmbientGeneric {
    fn create(base: BaseEntity) -> Self {
        Self {
            base,

            attenuation: Attenuation::default(),
            active: false,
            looping: false,
            remove_me: false,
            dpv: DynamicPitchVolume::default(),
        }
    }
}

impl Entity for AmbientGeneric {
    delegate_entity!(base not { object_caps, key_value, precache, spawn, think, used });

    fn object_caps(&self) -> ObjectCaps {
        self.base
            .object_caps()
            .difference(ObjectCaps::ACROSS_TRANSITION)
    }

    fn key_value(&mut self, data: &mut KeyValue) {
        self.dpv.key_value(data);
        if !data.handled() {
            self.base.key_value(data);
        }
    }

    fn precache(&mut self) {
        let engine = self.engine();

        let ev = self.base.vars.as_raw();
        if let Some(sample) = MapString::from_index(engine, ev.message) {
            let sound_file = sample.as_thin();
            if !sound_file.is_empty() && sound_file.to_bytes_with_nul()[0] != b'!' {
                engine.precache_sound(sound_file);
            }
        }

        self.init_modulation_parms();

        let ev = self.base.vars.as_raw_mut();
        let spawn_flags = AmbientSound::from_bits_retain(ev.spawnflags);
        if !spawn_flags.intersects(AmbientSound::START_SILENT) && self.looping {
            self.active = true;
        }
        if self.active {
            if let Some(sample) = MapString::from_index(engine, ev.message) {
                engine
                    .build_sound()
                    .volume(self.dpv.vol as f32 * 0.01)
                    .attenuation(self.attenuation)
                    .flags(SoundFlags::SPAWNING)
                    .pitch(self.dpv.pitch)
                    .ambient_emit_dyn(&sample, ev.origin, self.as_edict_mut());
                self.vars_mut()
                    .set_next_think_time(engine.globals.map_time_f32() + 0.1);
            }
        }
    }

    fn spawn(&mut self) {
        let engine = self.engine();
        let ev = self.base.vars.as_raw_mut();

        let spawn_flags = AmbientSound::from_bits_retain(ev.spawnflags);
        if spawn_flags.intersects(AmbientSound::EVERYWHERE) {
            self.attenuation = Attenuation::NONE;
        } else if spawn_flags.intersects(AmbientSound::SMALL_RADIUS) {
            self.attenuation = Attenuation::IDLE;
        } else if spawn_flags.intersects(AmbientSound::MEDIUM_RADIUS) {
            self.attenuation = Attenuation::STATIC;
        } else if spawn_flags.intersects(AmbientSound::LARGE_RADIUS) {
            self.attenuation = Attenuation::NORM;
        } else {
            self.attenuation = Attenuation::STATIC;
        }

        if MapString::is_null_or_empty(engine, ev.message) {
            error!(
                "Empty ambient at {}, {}, {}",
                ev.origin.x(),
                ev.origin.y(),
                ev.origin.z()
            );
            self.remove_me = true;
            self.vars_mut().set_next_think_time(0.1);
            return;
        };

        ev.solid = SOLID_NOT;
        ev.movetype = MoveType::None.into();
        ev.nextthink = 0.0;

        self.active = false;
        self.looping = !spawn_flags.intersects(AmbientSound::NOT_LOOPING);
        self.precache();
    }

    fn think(&mut self) {
        if self.remove_me {
            self.remove_from_world();
            return;
        }

        if !self.dpv.is_active() {
            return;
        }

        let engine = self.engine();
        let Some(sample) = MapString::from_index(engine, self.vars().as_raw().message) else {
            return;
        };
        let mut pitch = self.dpv.pitch;
        let mut vol = self.dpv.vol;
        let mut flags = SoundFlags::NONE;
        let mut changed = false;

        // pitch envelope
        if self.dpv.spinup != 0 || self.dpv.spindown != 0 {
            let prev = self.dpv.pitchfrac >> 8;

            if self.dpv.spinup > 0 {
                self.dpv.pitchfrac += self.dpv.spinup;
            } else if self.dpv.spindown > 0 {
                self.dpv.pitchfrac -= self.dpv.spindown;
            }

            pitch = self.dpv.pitchfrac >> 8;
            if pitch > self.dpv.pitchrun {
                pitch = self.dpv.pitchrun;
                // done with ramp up
                self.dpv.spinup = 0;
            }

            if pitch < self.dpv.pitchstart {
                // done with ramp down
                self.dpv.spindown = 0;

                engine
                    .build_sound()
                    .flags(SoundFlags::STOP)
                    .ambient_emit_dyn(&sample, self.vars().origin(), self);

                return;
            }

            pitch = pitch.clamp(1, 255);
            self.dpv.pitch = pitch;

            changed |= prev != pitch;
            flags.insert(SoundFlags::CHANGE_PITCH);
        }

        // amplitude envelope
        if self.dpv.fadein != 0 || self.dpv.fadeout != 0 {
            let prev = self.dpv.volfrac >> 8;

            if self.dpv.fadein > 0 {
                self.dpv.volfrac += self.dpv.fadein;
            } else if self.dpv.fadeout > 0 {
                self.dpv.volfrac -= self.dpv.fadeout;
            }

            vol = self.dpv.volfrac >> 8;

            if vol > self.dpv.volrun {
                vol = self.dpv.volrun;
                self.dpv.fadein = 0; // done with ramp up
            }

            if vol < self.dpv.volstart {
                self.dpv.fadeout = 0; // done with ramp down

                engine
                    .build_sound()
                    .flags(SoundFlags::STOP)
                    .ambient_emit_dyn(&sample, self.vars().as_raw().origin, self.as_edict_mut());

                return;
            }

            vol = vol.clamp(1, 100);
            self.dpv.vol = vol;

            changed |= prev != vol;
            flags |= SoundFlags::CHANGE_VOL;
        }

        // pitch/amplitude LFO
        if self.dpv.lfotype != 0 {
            if self.dpv.lfofrac > 0x6fffffff {
                self.dpv.lfofrac = 0;
            }

            // update lfo, lfofrac/255 makes a triangle wave 0-255
            self.dpv.lfofrac += self.dpv.lforate;
            let mut pos = self.dpv.lfofrac >> 8;

            if self.dpv.lfofrac < 0 {
                self.dpv.lfofrac = 0;
                self.dpv.lforate = self.dpv.lforate.abs();
                pos = 0;
            } else if pos > 255 {
                pos = 255;
                self.dpv.lfofrac = 255 << 8;
                self.dpv.lforate = -self.dpv.lforate.abs();
            }

            match self.dpv.lfotype {
                LFO_SQUARE => {
                    if pos < 128 {
                        self.dpv.lfomult = 255;
                    } else {
                        self.dpv.lfomult = 0;
                    }
                }
                LFO_RANDOM => {
                    if pos == 255 {
                        self.dpv.lfomult = engine.random_int(0, 255);
                    }
                }
                _ => {
                    self.dpv.lfomult = pos;
                }
            }

            if self.dpv.lfomodpitch != 0 {
                let prev = pitch;
                pitch += ((self.dpv.lfomult - 128) * self.dpv.lfomodpitch) / 100;
                pitch = pitch.clamp(1, 255);
                changed |= prev != pitch;
                flags |= SoundFlags::CHANGE_PITCH;
            }

            if self.dpv.lfomodvol != 0 {
                let prev = vol;
                vol += ((self.dpv.lfomult - 128) * self.dpv.lfomodvol) / 100;
                vol = vol.clamp(0, 100);
                changed |= prev != vol;
                flags |= SoundFlags::CHANGE_VOL;
            }
        }

        if !flags.is_empty() && changed {
            if pitch == PITCH_NORM {
                pitch = PITCH_NORM + 1; // do not send 'no pitch'!
            }

            engine
                .build_sound()
                .volume(vol as f32 * 0.01)
                .attenuation(self.attenuation)
                .pitch(Pitch::from(pitch))
                .ambient_emit_dyn(&sample, self.vars().as_raw().origin, self.as_edict_mut());
        }

        self.vars_mut()
            .set_next_think_time(engine.globals.map_time_f32() + 0.2);
    }

    #[allow(unused_variables)]
    fn used(&mut self, other: &mut dyn Entity, use_type: UseType, value: f32) {
        warn!("AmbientGeneric::used is not implemented");
    }
}

#[cfg(feature = "export-default-entities")]
mod exports {
    use crate::{
        entity::{Private, StubEntity},
        export::export_entity,
    };

    export_entity!(ambient_generic, Private<super::AmbientGeneric>);
    export_entity!(env_sound, Private<StubEntity>);
    export_entity!(speaker, Private<StubEntity>);
}
