use core::ffi::c_int;

use bitflags::bitflags;

use crate::ffi;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Channel {
    #[default]
    Auto,
    Weapon,
    Voice,
    Item,
    Body,
    /// Allocate a stream channel from the static or dynamic area.
    Stream,
    /// Allocate a channel from the static area.
    Static,
    /// A voice data comming across the network.
    NetworkVoice(u16),
}

impl Channel {
    pub fn from_raw(raw: c_int) -> Option<Channel> {
        Some(match raw {
            ffi::common::CHAN_AUTO => Self::Auto,
            ffi::common::CHAN_WEAPON => Self::Weapon,
            ffi::common::CHAN_VOICE => Self::Voice,
            ffi::common::CHAN_ITEM => Self::Item,
            ffi::common::CHAN_BODY => Self::Body,
            ffi::common::CHAN_STREAM => Self::Stream,
            ffi::common::CHAN_STATIC => Self::Static,
            ffi::common::CHAN_NETWORKVOICE_BASE..=ffi::common::CHAN_NETWORKVOICE_END => {
                Self::NetworkVoice((raw - ffi::common::CHAN_NETWORKVOICE_BASE) as u16)
            }
            _ => return None,
        })
    }

    pub fn into_raw(self) -> c_int {
        match self {
            Channel::Auto => ffi::common::CHAN_AUTO,
            Channel::Weapon => ffi::common::CHAN_WEAPON,
            Channel::Voice => ffi::common::CHAN_VOICE,
            Channel::Item => ffi::common::CHAN_ITEM,
            Channel::Body => ffi::common::CHAN_BODY,
            Channel::Stream => ffi::common::CHAN_STREAM,
            Channel::Static => ffi::common::CHAN_STATIC,
            Channel::NetworkVoice(n) => ffi::common::CHAN_NETWORKVOICE_BASE + n as c_int,
        }
    }
}

impl From<Channel> for c_int {
    fn from(value: Channel) -> Self {
        value.into_raw()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Attenuation(f32);

impl Default for Attenuation {
    fn default() -> Self {
        Self::NORM
    }
}

impl Attenuation {
    pub const NONE: Self = Self(ffi::common::ATTN_NONE);
    pub const NORM: Self = Self(ffi::common::ATTN_NORM);
    pub const STATIC: Self = Self(ffi::common::ATTN_STATIC);
    pub const IDLE: Self = Self(ffi::common::ATTN_IDLE);
}

impl From<f32> for Attenuation {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<Attenuation> for f32 {
    fn from(value: Attenuation) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Pitch(c_int);

impl Default for Pitch {
    fn default() -> Self {
        Self::NORM
    }
}

impl Pitch {
    pub const LOW: Self = Self(ffi::common::PITCH_LOW);
    pub const NORM: Self = Self(ffi::common::PITCH_NORM);
    pub const HIGH: Self = Self(ffi::common::PITCH_HIGH);
}

impl From<c_int> for Pitch {
    fn from(value: c_int) -> Self {
        Self(value)
    }
}

impl From<Pitch> for c_int {
    fn from(value: Pitch) -> Self {
        value.0
    }
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct SoundFlags: c_int {
        const NONE              = 0;
        /// A scaled byte.
        const VOLUME            = ffi::common::SND_VOLUME;
        /// A byte.
        const ATTENUATION       = ffi::common::SND_ATTENUATION;
        /// Get sentence from a script.
        const SEQUENCE          = ffi::common::SND_SEQUENCE;
        /// A byte.
        const PITCH             = ffi::common::SND_PITCH;
        /// Set if sound num is actually a sentence num.
        const SENTENCE          = ffi::common::SND_SENTENCE;
        /// Stop the sound.
        const STOP              = ffi::common::SND_STOP;
        /// Change sound vol.
        const CHANGE_VOL        = ffi::common::SND_CHANGE_VOL;
        /// Change sound pitch.
        const CHANGE_PITCH      = ffi::common::SND_CHANGE_PITCH;
        /// We're spawning, used in some cases for ambients (not sent across network).
        const SPAWNING          = ffi::common::SND_SPAWNING;
        /// Not paused, not looped, for internal use.
        const LOCALSOUND        = ffi::common::SND_LOCALSOUND;
        /// Stop all looping sounds on the entity.
        const STOP_LOOPING      = ffi::common::SND_STOP_LOOPING;
        /// Don't send sound from local player if prediction was enabled.
        const FILTER_CLIENT     = ffi::common::SND_FILTER_CLIENT;
        /// Passed playing position and the forced end.
        const RESTORE_POSITION  = ffi::common::SND_RESTORE_POSITION;
    }
}
