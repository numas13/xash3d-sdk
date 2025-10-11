use core::{cmp, fmt::Write};

use alloc::vec::Vec;
use csz::{CStrArray, CStrSlice, CStrThin};
use xash3d_shared::str::ByteSliceExt;

use crate::engine::ServerEngineRef;

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
