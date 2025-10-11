use core::{cmp, fmt::Write};

use alloc::vec::Vec;
use csz::{CStrArray, CStrSlice, CStrThin};
use xash3d_shared::str::ByteSliceExt;

use crate::{engine::ServerEngineRef, entity::EntityVars, str::MapString, time::MapTime};

pub use xash3d_shared::sound::*;

struct SentenceEntry {
    name_offset: u32,
    index: u16,
}

pub struct SentenceGroup {
    name_offset: u32,
    count: u16,
}

pub struct Sentences {
    engine: ServerEngineRef,
    strings: Vec<u8>,
    names: Vec<SentenceEntry>,
    groups: Vec<SentenceGroup>,
}

impl Sentences {
    pub(crate) fn new(engine: ServerEngineRef) -> Self {
        let mut ret = Self {
            engine,
            strings: Vec::new(),
            names: Vec::new(),
            groups: Vec::new(),
        };
        ret.init();
        ret
    }

    fn init(&mut self) {
        match self.engine.load_file("sound/sentences.txt") {
            Ok(file) => {
                self.parse(file.as_bytes());
            }
            Err(err) => {
                error!("sentences: failed to load sentences file, error: {err}");
            }
        }
    }

    fn parse(&mut self, bytes: &[u8]) {
        self.strings.clear();
        self.names.clear();
        self.groups.clear();

        let mut index = 0;
        let mut group_name = None;
        for line in bytes.split(|i| *i == b'\n') {
            let cur = line.bytes_trim_ascii_start();
            if cur.is_empty() || cur[0] == b'/' || !cur[0].is_ascii_alphabetic() {
                continue;
            }
            let (name, cur) = cur.bytes_take_while(|i| !i.is_ascii_whitespace());
            let cur = cur.bytes_trim_ascii_start();
            if cur.is_empty() {
                continue;
            }

            let name_offset = self.strings.len() as u32;
            self.strings.extend(name);
            self.strings.push(0);
            self.names.push(SentenceEntry { name_offset, index });
            index += 1;

            let group = name.bytes_trim_suffix(|i| i.is_ascii_digit());
            if name.len() == group.len() {
                continue;
            }

            if group_name != Some(group) {
                group_name = Some(group);
                let name_offset = self.strings.len() as u32;
                self.strings.extend(group);
                self.strings.push(0);
                self.groups.push(SentenceGroup {
                    name_offset,
                    count: 1,
                })
            } else {
                self.groups.last_mut().unwrap().count += 1;
            }
        }

        self.names.sort_unstable_by_key(|i| {
            let ptr = &self.strings[i.name_offset as usize] as *const u8;
            unsafe { CStrThin::from_ptr(ptr as *const i8) }
        });

        self.groups.sort_unstable_by_key(|i| {
            let ptr = &self.strings[i.name_offset as usize] as *const u8;
            unsafe { CStrThin::from_ptr(ptr as *const i8) }
        });

        self.strings.shrink_to_fit();
        self.names.shrink_to_fit();
        self.groups.shrink_to_fit();
    }

    pub fn find_sentence_index(&self, name: &CStrThin) -> Option<u16> {
        if !matches!(name.bytes().next(), Some(b'!')) {
            return None;
        }

        let name = unsafe { CStrThin::from_ptr(name.as_ptr().wrapping_add(1)) };
        self.names
            .binary_search_by_key(&name, |i| {
                let ptr = &self.strings[i.name_offset as usize] as *const u8;
                unsafe { CStrThin::from_ptr(ptr as *const i8) }
            })
            .map(|index| self.names[index].index)
            .ok()
    }

    pub fn find_sentence(&self, name: &CStrThin) -> Option<CStrArray<16>> {
        self.find_sentence_index(name).map(|index| {
            let mut buf = CStrArray::new();
            write!(buf.cursor(), "!{index}").ok();
            buf
        })
    }

    fn find_group(&self, name: &CStrThin) -> Option<&SentenceGroup> {
        if name.is_empty() {
            return None;
        }

        self.groups
            .binary_search_by_key(&name, |i| {
                let ptr = &self.strings[i.name_offset as usize] as *const u8;
                unsafe { CStrThin::from_ptr(ptr as *const i8) }
            })
            .map(|index| &self.groups[index])
            .ok()
    }

    /// Returns `None` if the group does not exists or buffer is not larger enough to hold the sample
    /// name.
    pub fn pick_sequential<'a>(
        &self,
        group_name: &CStrThin,
        pick: u16,
        reset: bool,
        buffer: &'a mut CStrSlice,
    ) -> Option<(u16, &'a CStrThin)> {
        let group = self.find_group(group_name)?;
        let max_pick = group.count.checked_sub(1)?;
        let pick = cmp::min(pick, max_pick);
        write!(buffer.cursor(), "!{group_name}{pick}").ok()?;

        let next = if pick < max_pick {
            pick + 1
        } else if reset {
            0
        } else {
            max_pick
        };

        Some((next, buffer.as_thin()))
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct LockSounds {
    pub locked_sound: Option<MapString>,
    pub locked_sentence: Option<MapString>,
    pub unlocked_sound: Option<MapString>,
    pub unlocked_sentence: Option<MapString>,

    pub locked_sentence_index: u16,
    pub unlocked_sentence_index: u16,

    pub wait_sound: MapTime,
    pub wait_sentence: MapTime,
    pub eof_locked: bool,
    pub eof_unlocked: bool,
}

impl LockSounds {
    const DOOR_SENTENCE_WAIT: f32 = 6.0;
    const DOOR_SOUND_WAIT: f32 = 3.0;
    const BUTTON_SOUND_WAIT: f32 = 0.5;

    fn play_lock(&mut self, v: &mut EntityVars, sound_wait: f32) {
        let engine = v.engine();
        let now = engine.globals.map_time();

        let play_sentence =
            self.locked_sentence.is_some() && !self.eof_locked && self.wait_sentence < now;

        if let (true, Some(locked_sound)) = (self.wait_sound < now, self.locked_sound) {
            engine
                .build_sound()
                .channel_item()
                .volume(if play_sentence { 0.25 } else { 1.0 })
                .emit_dyn(&locked_sound, v);
            self.wait_sound = now + sound_wait;
        }

        if let (true, Some(locked_sentence)) = (play_sentence, self.locked_sentence) {
            let prev = self.locked_sentence_index;

            self.locked_sentence_index = engine
                .build_sound()
                .volume(0.85)
                .emit_sequential(&locked_sentence, self.locked_sentence_index, false, v)
                .unwrap_or(0);
            self.unlocked_sentence_index = 0;

            self.eof_locked = prev == self.locked_sentence_index;
            self.wait_sentence = now + Self::DOOR_SENTENCE_WAIT;
        }
    }

    fn play_unlock(&mut self, v: &mut EntityVars, sound_wait: f32) {
        let engine = v.engine();
        let now = engine.globals.map_time();

        let play_sentence =
            self.unlocked_sentence.is_some() && !self.eof_unlocked && self.wait_sentence < now;

        if let (true, Some(unlocked_sound)) = (self.wait_sound < now, self.unlocked_sound) {
            engine
                .build_sound()
                .channel_item()
                .volume(if play_sentence { 0.25 } else { 1.0 })
                .emit_dyn(&unlocked_sound, v);
            self.wait_sound = now + sound_wait;
        }

        if let (true, Some(unlocked_sentence)) = (play_sentence, self.unlocked_sentence) {
            let prev = self.unlocked_sentence_index;

            self.unlocked_sentence_index = engine
                .build_sound()
                .volume(0.85)
                .emit_sequential(&unlocked_sentence, self.unlocked_sentence_index, false, v)
                .unwrap_or(0);
            self.locked_sentence_index = 0;

            self.eof_unlocked = prev == self.unlocked_sentence_index;
            self.wait_sentence = now + Self::DOOR_SENTENCE_WAIT;
        }
    }

    pub fn play_door(&mut self, locked: bool, v: &mut EntityVars) {
        if locked {
            self.play_lock(v, Self::DOOR_SOUND_WAIT);
        } else {
            self.play_unlock(v, Self::DOOR_SOUND_WAIT);
        }
    }

    pub fn play_button(&mut self, locked: bool, v: &mut EntityVars) {
        if locked {
            self.play_lock(v, Self::BUTTON_SOUND_WAIT);
        } else {
            self.play_unlock(v, Self::BUTTON_SOUND_WAIT);
        }
    }
}
