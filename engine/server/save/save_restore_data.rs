use core::{
    ffi::{c_char, c_int, c_uint, c_ushort, CStr},
    ops::{Deref, DerefMut},
    ptr,
};

use csz::CStrThin;
use xash3d_shared::{
    ffi::{
        common::vec3_t,
        server::{edict_s, ENTITYTABLE, SAVERESTOREDATA},
    },
    utils::{slice_from_raw_parts_or_empty, slice_from_raw_parts_or_empty_mut},
};

use crate::save::{SaveError, SaveResult, Token};

#[repr(transparent)]
pub struct SaveRestoreBuffer {
    data: SAVERESTOREDATA,
}

impl SaveRestoreBuffer {
    pub fn offset(&self) -> usize {
        self.data.size as usize
    }

    pub fn capacity(&self) -> usize {
        self.data.bufferSize as usize
    }

    pub fn available(&self) -> usize {
        self.capacity() - self.offset()
    }

    pub fn is_empty(&self) -> bool {
        let diff = unsafe { self.data.pCurrentData.offset_from(self.data.pBaseData) };
        diff >= self.capacity() as isize
    }

    pub fn as_slice(&self) -> &[u8] {
        let data = self.data.pCurrentData.cast();
        let len = self.available();
        unsafe { slice_from_raw_parts_or_empty(data, len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        let data = self.data.pCurrentData.cast();
        let len = self.available();
        unsafe { slice_from_raw_parts_or_empty_mut(data, len) }
    }

    pub fn advance(&mut self, len: usize) -> SaveResult<()> {
        let new_size = self.data.size + len as c_int;
        if new_size < self.data.bufferSize {
            self.data.pCurrentData = self.data.pCurrentData.wrapping_add(len);
            self.data.size += len as c_int;
            Ok(())
        } else {
            Err(SaveError::Overflow)
        }
    }
}

#[repr(transparent)]
pub struct SaveRestoreState {
    data: SAVERESTOREDATA,
}

impl SaveRestoreState {
    pub fn time(&self) -> f32 {
        self.data.time
    }

    pub fn use_landmark_offset(&self) -> Option<vec3_t> {
        if self.data.fUseLandmark != 0 {
            Some(self.data.vecLandmarkOffset)
        } else {
            None
        }
    }

    pub fn landmark_offset(&self) -> vec3_t {
        self.data.vecLandmarkOffset
    }

    pub fn set_landmark_offset(&mut self, landmark_offset: vec3_t) {
        self.data.vecLandmarkOffset = landmark_offset;
    }

    pub fn current_map_name(&self) -> &CStrThin {
        unsafe { CStrThin::from_ptr(self.data.szCurrentMapName.as_ptr()) }
    }

    pub fn current_index(&self) -> usize {
        self.data.currentIndex as usize
    }

    pub fn table(&self) -> &[ENTITYTABLE] {
        let len = self.data.tableCount as usize;
        unsafe { slice_from_raw_parts_or_empty(self.data.pTable, len) }
    }

    pub fn table_mut(&mut self) -> &mut [ENTITYTABLE] {
        let len = self.data.tableCount as usize;
        unsafe { slice_from_raw_parts_or_empty_mut(self.data.pTable, len) }
    }

    fn tokens(&self) -> &[*mut c_char] {
        let len = self.data.tokenCount as usize;
        unsafe { slice_from_raw_parts_or_empty(self.data.pTokens, len) }
    }

    fn tokens_mut(&mut self) -> &mut [*mut c_char] {
        let len = self.data.tokenCount as usize;
        unsafe { slice_from_raw_parts_or_empty_mut(self.data.pTokens, len) }
    }

    pub fn token_str(&self, token: Token) -> Option<&CStrThin> {
        let s = self.tokens().get(token.to_usize())?;
        Some(unsafe { CStrThin::from_ptr(*s) })
    }

    pub fn token_hash(&mut self, token: &CStr) -> Token {
        fn hash_string(token: &CStr) -> c_uint {
            token
                .to_bytes()
                .iter()
                .fold(0, |hash, &byte| hash.rotate_right(4) ^ (byte as c_uint))
        }

        let tokens = self.tokens_mut();
        let hash = (hash_string(token) % (tokens.len() as c_uint)) as c_ushort;
        for i in 0..tokens.len() {
            let mut index = i + hash as usize;
            if index >= tokens.len() {
                index -= tokens.len();
            }
            if tokens[index].is_null() || token == unsafe { CStr::from_ptr(tokens[index]) } {
                tokens[index] = token.as_ptr() as *mut c_char;
                return Token::new(index as u16);
            }
        }

        error!("Save::token_hash is COMPLETELY FULL!");
        Token::new(0)
    }

    pub fn entity_index(&self, ent: *mut edict_s) -> Option<usize> {
        if !ent.is_null() {
            self.table().iter().position(|i| i.pent == ent)
        } else {
            None
        }
    }

    pub fn entity_from_index(&self, index: i32) -> *mut edict_s {
        if index < 0 {
            return ptr::null_mut();
        }
        self.table()
            .iter()
            .find(|i| i.id == index)
            .map_or(ptr::null_mut(), |i| i.pent)
    }

    pub fn entity_flags_set(&mut self, index: usize, flags: c_int) -> c_int {
        if let Some(i) = self.table_mut().get_mut(index) {
            i.flags |= flags;
            i.flags
        } else {
            0
        }
    }
}

#[repr(transparent)]
pub struct SaveRestoreData {
    data: SAVERESTOREDATA,
}

impl SaveRestoreData {
    pub fn new(data: &mut SAVERESTOREDATA) -> &mut Self {
        unsafe { &mut *(data as *mut _ as *mut Self) }
    }

    pub fn split(&self) -> (&SaveRestoreBuffer, &SaveRestoreState) {
        let data = &self.data as *const SAVERESTOREDATA;
        let buffer = unsafe { &*data.cast::<SaveRestoreBuffer>() };
        let state = unsafe { &*data.cast::<SaveRestoreState>() };
        (buffer, state)
    }

    pub fn split_mut(&mut self) -> (&mut SaveRestoreBuffer, &mut SaveRestoreState) {
        let data = &mut self.data as *mut SAVERESTOREDATA;
        let buffer = unsafe { &mut *data.cast::<SaveRestoreBuffer>() };
        let state = unsafe { &mut *data.cast::<SaveRestoreState>() };
        (buffer, state)
    }

    pub fn restore_save_pointers(&mut self) {
        self.data.size = self.table()[self.current_index()].location;
        self.data.pCurrentData = self.data.pBaseData.wrapping_add(self.data.size as usize);
    }

    pub fn offset(&self) -> usize {
        self.data.size as usize
    }
}

impl Deref for SaveRestoreData {
    type Target = SaveRestoreState;

    fn deref(&self) -> &Self::Target {
        self.split().1
    }
}

impl DerefMut for SaveRestoreData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.split_mut().1
    }
}
