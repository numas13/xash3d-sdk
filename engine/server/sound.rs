use core::{cell::RefCell, cmp, ffi::CStr, fmt::Write};

use alloc::vec::Vec;
use csz::{CStrArray, CStrSlice, CStrThin};
use xash3d_shared::str::ByteSliceExt;

use crate::{
    entity::{EntityVars, KeyValue},
    prelude::*,
    str::MapString,
    time::MapTime,
};

pub use xash3d_shared::sound::*;

struct SentenceEntry {
    name_offset: u32,
    index: u16,
}

pub struct SentenceGroup {
    name_offset: u32,
    count: u16,
    lru: RefCell<Vec<u16>>,
}

impl SentenceGroup {
    fn new(name_offset: u32, count: u16) -> Self {
        Self {
            name_offset,
            count,
            lru: RefCell::new(Vec::new()),
        }
    }

    fn random_lru_pick(&self, engine: &ServerEngine) -> u16 {
        let mut lru = self.lru.borrow_mut();
        if lru.is_empty() {
            if lru.capacity() == 0 {
                lru.reserve_exact(self.count as usize);
            }
            lru.extend(0..self.count);
        }
        let max = (lru.len() - 1) as i32;
        lru.swap_remove(engine.random_int(0, max) as usize)
    }
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
                self.groups.push(SentenceGroup::new(name_offset, 1));
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

    fn get_name(&self, offset: usize) -> &CStrThin {
        let ptr = &self.strings[offset] as *const u8;
        unsafe { CStrThin::from_ptr(ptr as *const i8) }
    }

    pub fn find_sentence_index(&self, name: &CStrThin) -> Option<u16> {
        if !matches!(name.bytes().next(), Some(b'!')) {
            return None;
        }

        let name = unsafe { CStrThin::from_ptr(name.as_ptr().wrapping_add(1)) };
        self.names
            .binary_search_by(|i| self.get_name(i.name_offset as usize).cmp_ignore_case(name))
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
            .binary_search_by(|i| self.get_name(i.name_offset as usize).cmp_ignore_case(name))
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

    /// Returns `None` if the group does not exists or buffer is not larger enough to hold the sample
    /// name.
    pub fn pick_random<'a>(
        &self,
        group_name: &CStrThin,
        buffer: &'a mut CStrSlice,
    ) -> Option<(u16, &'a CStrThin)> {
        let group = self.find_group(group_name)?;
        let pick = group.random_lru_pick(&self.engine);
        write!(buffer.cursor(), "!{group_name}{pick}").ok()?;
        Some((pick, buffer.as_thin()))
    }
}

#[derive(Default)]
struct LockSoundsState {
    locked_sound: Option<MapString>,
    locked_sentence: Option<MapString>,
    unlocked_sound: Option<MapString>,
    unlocked_sentence: Option<MapString>,

    locked_sentence_index: u16,
    unlocked_sentence_index: u16,

    wait_sound: MapTime,
    wait_sentence: MapTime,
    eof_locked: bool,
    eof_unlocked: bool,
}

impl LockSoundsState {
    const DOOR_SENTENCE_WAIT: f32 = 6.0;

    fn play_lock(&mut self, v: &EntityVars, sound_wait: f32) {
        let engine = v.engine();
        let now = engine.globals.map_time();

        let play_sentence =
            self.locked_sentence.is_some() && !self.eof_locked && self.wait_sentence < now;

        if let (true, Some(locked_sound)) = (self.wait_sound < now, self.locked_sound) {
            engine
                .build_sound()
                .channel_item()
                .volume(if play_sentence { 0.25 } else { 1.0 })
                .emit_dyn(locked_sound, v);
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

    fn play_unlock(&mut self, v: &EntityVars, sound_wait: f32) {
        let engine = v.engine();
        let now = engine.globals.map_time();

        let play_sentence =
            self.unlocked_sentence.is_some() && !self.eof_unlocked && self.wait_sentence < now;

        if let (true, Some(unlocked_sound)) = (self.wait_sound < now, self.unlocked_sound) {
            engine
                .build_sound()
                .channel_item()
                .volume(if play_sentence { 0.25 } else { 1.0 })
                .emit_dyn(unlocked_sound, v);
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
}

#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct LockSounds {
    #[cfg_attr(feature = "save", save(skip))]
    engine: ServerEngineRef,

    locked_sound: u8,
    locked_sentence: u8,
    unlocked_sound: u8,
    unlocked_sentence: u8,

    #[cfg_attr(feature = "save", save(skip))]
    state: RefCell<LockSoundsState>,
}

impl LockSounds {
    const DOOR_SOUND_WAIT: f32 = 3.0;
    const BUTTON_SOUND_WAIT: f32 = 0.5;

    pub fn new(engine: ServerEngineRef) -> Self {
        Self {
            engine,

            locked_sound: 0,
            locked_sentence: 0,
            unlocked_sound: 0,
            unlocked_sentence: 0,

            state: Default::default(),
        }
    }

    pub fn key_value(&mut self, data: &mut KeyValue) -> bool {
        match data.key_name().to_bytes() {
            b"locked_sound" => self.locked_sound = data.parse_or_default(),
            b"locked_sentence" => self.locked_sentence = data.parse_or_default(),
            b"unlocked_sound" => self.unlocked_sound = data.parse_or_default(),
            b"unlocked_sentence" => self.unlocked_sentence = data.parse_or_default(),
            _ => return false,
        }
        data.set_handled(true);
        true
    }

    pub fn precache(&mut self) {
        let engine = self.engine;
        let state = self.state.get_mut();

        state.locked_sound = if self.locked_sound > 0 {
            let sound = button_sound_or_default(self.locked_sound as usize);
            engine.precache_sound(sound);
            Some(engine.new_map_string(sound))
        } else {
            None
        };

        state.unlocked_sound = if self.unlocked_sound > 0 {
            let sound = button_sound_or_default(self.unlocked_sound as usize);
            engine.precache_sound(sound);
            Some(engine.new_map_string(sound))
        } else {
            None
        };

        state.locked_sentence = if self.locked_sentence > 0 {
            LOCK_SENTENCES
                .get(self.locked_sentence as usize - 1)
                .map(|&s| engine.new_map_string(s))
        } else {
            None
        };

        state.unlocked_sentence = if self.unlocked_sentence > 0 {
            UNLOCK_SENTENCES
                .get(self.unlocked_sentence as usize - 1)
                .map(|&s| engine.new_map_string(s))
        } else {
            None
        };
    }

    pub fn play_door(&self, locked: bool, v: &EntityVars) {
        let mut state = self.state.borrow_mut();
        if locked {
            state.play_lock(v, Self::DOOR_SOUND_WAIT);
        } else {
            state.play_unlock(v, Self::DOOR_SOUND_WAIT);
        }
    }

    pub fn play_button(&self, locked: bool, v: &EntityVars) {
        let mut state = self.state.borrow_mut();
        if locked {
            state.play_lock(v, Self::BUTTON_SOUND_WAIT);
        } else {
            state.play_unlock(v, Self::BUTTON_SOUND_WAIT);
        }
    }
}

pub const LOCK_SENTENCES: &[&CStr] = &[
    c"NA",    // access denied
    c"ND",    // security lockout
    c"NF",    // blast door
    c"NFIRE", // fire door
    c"NCHEM", // chemical door
    c"NRAD",  // radiation door
    c"NCON",  // gen containment
    c"NH",    // maintenance door
    c"NG",    // broken door
];

pub const UNLOCK_SENTENCES: &[&CStr] = &[
    c"EA",    // access granted
    c"ED",    // security door
    c"EF",    // blast door
    c"EFIRE", // fire door
    c"ECHEM", // chemical door
    c"ERAD",  // radiation door
    c"ECON",  // gen containment
    c"EH",    // maintenance door
];

const BUTTON_SOUNDS: &[&CStr] = &[
    res::valve::sound::common::NULL,
    res::valve::sound::buttons::BUTTON1,
    res::valve::sound::buttons::BUTTON2,
    res::valve::sound::buttons::BUTTON3,
    res::valve::sound::buttons::BUTTON4,
    res::valve::sound::buttons::BUTTON5,
    res::valve::sound::buttons::BUTTON6,
    res::valve::sound::buttons::BUTTON7,
    res::valve::sound::buttons::BUTTON8,
    res::valve::sound::buttons::BUTTON9,
    res::valve::sound::buttons::BUTTON10,
    res::valve::sound::buttons::BUTTON11,
    res::valve::sound::buttons::LATCHLOCKED1,
    res::valve::sound::buttons::LATCHUNLOCKED1,
    res::valve::sound::buttons::LIGHTSWITCH2,
    res::valve::sound::buttons::BUTTON9, // reserved
    res::valve::sound::buttons::BUTTON9, // reserved
    res::valve::sound::buttons::BUTTON9, // reserved
    res::valve::sound::buttons::BUTTON9, // reserved
    res::valve::sound::buttons::BUTTON9, // reserved
    res::valve::sound::buttons::BUTTON9, // reserved
    res::valve::sound::buttons::LEVER1,
    res::valve::sound::buttons::LEVER2,
    res::valve::sound::buttons::LEVER3,
    res::valve::sound::buttons::LEVER4,
    res::valve::sound::buttons::LEVER5,
];

const BUTTON_DEFAULT_SOUND: &CStr = res::valve::sound::buttons::BUTTON9;

pub fn button_sound(index: usize) -> Option<&'static CStr> {
    BUTTON_SOUNDS.get(index).copied()
}

pub fn button_sound_or_default(index: usize) -> &'static CStr {
    button_sound(index).unwrap_or(BUTTON_DEFAULT_SOUND)
}

const PLATFORM_MOVE_SOUNDS: &[&CStr] = &[
    res::valve::sound::plats::BIGMOVE1,
    res::valve::sound::plats::BIGMOVE2,
    res::valve::sound::plats::ELEVMOVE1,
    res::valve::sound::plats::ELEVMOVE2,
    res::valve::sound::plats::ELEVMOVE3,
    res::valve::sound::plats::FREIGHTMOVE1,
    res::valve::sound::plats::FREIGHTMOVE2,
    res::valve::sound::plats::HEAVYMOVE1,
    res::valve::sound::plats::RACKMOVE1,
    res::valve::sound::plats::RAILMOVE1,
    res::valve::sound::plats::SQUEEKMOVE1,
    res::valve::sound::plats::TALKMOVE1,
    res::valve::sound::plats::TALKMOVE2,
];

const PLATFORM_STOP_SOUNDS: &[&CStr] = &[
    res::valve::sound::plats::BIGSTOP1,
    res::valve::sound::plats::BIGSTOP2,
    res::valve::sound::plats::FREIGHTSTOP1,
    res::valve::sound::plats::HEAVYSTOP2,
    res::valve::sound::plats::RACKSTOP1,
    res::valve::sound::plats::RAILSTOP1,
    res::valve::sound::plats::SQUEEKSTOP1,
    res::valve::sound::plats::TALKSTOP1,
];

pub fn platform_move_sound(index: usize) -> Option<&'static CStr> {
    let index = index.checked_sub(1)?;
    PLATFORM_MOVE_SOUNDS.get(index).copied()
}

pub fn platform_stop_sound(index: usize) -> Option<&'static CStr> {
    let index = index.checked_sub(1)?;
    PLATFORM_STOP_SOUNDS.get(index).copied()
}

pub trait EntityVarsPlatformSounds {
    fn moving_noise(&self) -> Option<MapString>;

    fn set_moving_noise(&self, sound: MapString);

    fn moving_stop_noise(&self) -> Option<MapString>;

    fn set_moving_stop_noise(&self, sound: MapString);
}

impl EntityVarsPlatformSounds for EntityVars {
    fn moving_noise(&self) -> Option<MapString> {
        self.noise()
    }

    fn set_moving_noise(&self, sound: MapString) {
        self.set_noise(Some(sound));
    }

    fn moving_stop_noise(&self) -> Option<MapString> {
        self.noise1()
    }

    fn set_moving_stop_noise(&self, sound: MapString) {
        self.set_noise1(Some(sound));
    }
}

/// Platform moving sounds.
///
/// [EntityVars::noise] and [EntityVars::noise1] are used to store sound names.
#[derive(Default)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
pub struct PlatformSounds {
    volume: f32,
    move_sound: u8,
    stop_sound: u8,
}

impl PlatformSounds {
    pub fn key_value(&mut self, data: &mut KeyValue) -> bool {
        match data.key_name().to_bytes() {
            b"volume" => self.volume = data.parse_or_default(),
            b"movesnd" => self.move_sound = data.parse_or_default(),
            b"stopsnd" => self.stop_sound = data.parse_or_default(),
            _ => return false,
        }
        data.set_handled(true);
        true
    }

    pub fn precache(&mut self, v: &EntityVars) {
        let engine = v.engine();

        let move_sound = platform_move_sound(self.move_sound as usize)
            .inspect(|&sound| {
                engine.precache_sound(sound);
            })
            .unwrap_or(res::valve::sound::common::NULL);
        v.set_moving_noise(engine.new_map_string(move_sound));

        let stop_sound = platform_stop_sound(self.stop_sound as usize)
            .inspect(|&sound| {
                engine.precache_sound(sound);
            })
            .unwrap_or(res::valve::sound::common::NULL);
        v.set_moving_stop_noise(engine.new_map_string(stop_sound));
    }

    pub fn init(&mut self) {
        if self.volume == 0.0 {
            self.volume = 0.85;
        }
    }

    pub fn emit_moving_noise(&self, v: &EntityVars) {
        if let Some(sound) = v.moving_noise() {
            v.engine()
                .build_sound()
                .channel_static()
                .volume(self.volume)
                .emit_dyn(sound, v);
        }
    }

    pub fn stop_moving_noise(&self, v: &EntityVars) {
        if let Some(sound) = v.moving_noise() {
            v.engine().build_sound().channel_static().stop(sound, v);
        }
    }

    pub fn emit_moving_stop_noise(&self, v: &EntityVars) {
        self.stop_moving_noise(v);
        if let Some(sound) = v.moving_stop_noise() {
            v.engine()
                .build_sound()
                .channel_static()
                .volume(self.volume)
                .emit_dyn(sound, v);
        }
    }
}

pub fn play_cd_track(engine: &ServerEngine, track: i32) {
    let Some(client) = engine.get_single_player() else {
        return;
    };

    match track {
        -1 => {
            engine.client_command(&client, c"cd stop\n");
        }
        0..=30 => {
            engine.client_command(&client, format_args!("cd play {track}\n"));
        }
        _ => {
            warn!("play_cd_track: track {track} is out of range");
        }
    }
}
