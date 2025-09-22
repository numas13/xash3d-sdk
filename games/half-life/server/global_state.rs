use core::{cell::RefCell, ffi::c_int, mem::MaybeUninit, ptr};

use alloc::collections::linked_list::LinkedList;
use csz::{CStrArray, CStrThin};
use sv::{
    ffi::server::{edict_s, SAVERESTOREDATA, TYPEDESCRIPTION},
    macros::define_field,
    prelude::*,
    save::{self, FieldType, SaveReader, SaveWriter},
    str::MapString,
};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum EntityState {
    Off,
    On,
    Dead,
}

#[derive(Copy, Clone)]
pub struct GlobalEntity {
    name: CStrArray<64>,
    map_name: CStrArray<32>,
    state: EntityState,
}

impl Default for GlobalEntity {
    fn default() -> Self {
        Self {
            name: CStrArray::new(),
            map_name: CStrArray::new(),
            state: EntityState::Dead,
        }
    }
}

#[allow(dead_code)]
impl GlobalEntity {
    fn new(name: &CStrThin, map_name: &CStrThin, state: EntityState) -> Self {
        Self {
            name: name.try_into().unwrap(),
            map_name: map_name.try_into().unwrap(),
            state,
        }
    }

    pub fn name(&self) -> &CStrThin {
        self.name.as_thin()
    }

    pub fn map_name(&self) -> &CStrThin {
        self.map_name.as_thin()
    }

    pub fn state(&self) -> EntityState {
        self.state
    }

    pub fn is_off(&self) -> bool {
        self.state == EntityState::Off
    }

    pub fn is_on(&self) -> bool {
        self.state == EntityState::On
    }

    pub fn is_dead(&self) -> bool {
        self.state == EntityState::Dead
    }
}

pub struct Entities {
    list: LinkedList<GlobalEntity>,
}

impl Entities {
    fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    #[inline(never)]
    fn find_impl(&self, name: &CStrThin) -> Option<&GlobalEntity> {
        self.list.iter().find(|i| i.name() == name)
    }

    #[inline(never)]
    fn find_mut_impl(&mut self, name: &CStrThin) -> Option<&mut GlobalEntity> {
        self.list.iter_mut().find(|i| i.name() == name)
    }

    pub fn find(&self, name: impl AsRef<CStrThin>) -> Option<&GlobalEntity> {
        self.find_impl(name.as_ref())
    }

    pub fn find_mut(&mut self, name: impl AsRef<CStrThin>) -> Option<&mut GlobalEntity> {
        self.find_mut_impl(name.as_ref())
    }

    #[inline(never)]
    fn add_impl(&mut self, name: &CStrThin, map_name: &CStrThin, state: EntityState) {
        assert!(self.find(name).is_none());
        self.list
            .push_back(GlobalEntity::new(name, map_name, state));
    }

    pub fn add(
        &mut self,
        name: impl AsRef<CStrThin>,
        map_name: impl AsRef<CStrThin>,
        state: EntityState,
    ) {
        self.add_impl(name.as_ref(), map_name.as_ref(), state);
    }

    pub fn update(&mut self, name: MapString, map_name: MapString) {
        if let Some(ent) = self.find_mut(name) {
            ent.map_name.clear();
            ent.map_name.cursor().write_c_str(&map_name).unwrap();
        }
    }

    fn clear(&mut self) {
        self.list.clear();
    }
}

struct GlobalStateSave {
    list_count: c_int,
}

// Global Savedata for Delay
const GLOBAL_FIELDS: [TYPEDESCRIPTION; 1] = [define_field!(
    GlobalStateSave,
    list_count,
    FieldType::INTEGER
)];

// Global Savedata for Delay
const GLOBAL_ENTITY_FIELDS: [TYPEDESCRIPTION; 3] = [
    define_field!(GlobalEntity, name, FieldType::CHARACTER, 64),
    define_field!(GlobalEntity, map_name, FieldType::CHARACTER, 32),
    define_field!(GlobalEntity, state, FieldType::INTEGER),
];

pub struct GlobalState {
    engine: ServerEngineRef,
    pub entities: RefCell<Entities>,
    pub last_spawn: RefCell<*mut edict_s>,
}

impl GlobalState {
    pub fn new(engine: ServerEngineRef) -> Self {
        Self {
            engine,
            entities: RefCell::new(Entities::new()),
            last_spawn: RefCell::new(ptr::null_mut()),
        }
    }

    pub fn save(&self, save_data: &mut SAVERESTOREDATA) -> save::Result<()> {
        let _writer = SaveWriter::new(self.engine, save_data);
        debug!("TODO: SaveGlobalState");
        Ok(())
    }

    pub fn restore(&self, save_data: &mut SAVERESTOREDATA) -> save::Result<()> {
        let mut reader = SaveReader::new(self.engine, save_data);
        self.reset();

        let mut global_state = GlobalStateSave { list_count: 0 };
        reader.read_fields(
            c"GLOBAL",
            &mut global_state as *mut _ as *mut _,
            &GLOBAL_FIELDS,
        )?;

        let mut entities = self.entities.borrow_mut();
        for _ in 0..global_state.list_count {
            let mut tmp = MaybeUninit::<GlobalEntity>::uninit();
            reader.read_fields(c"GENT", tmp.as_mut_ptr().cast(), &GLOBAL_ENTITY_FIELDS)?;
            let tmp = unsafe { tmp.assume_init() };
            let name = tmp.name.as_thin();
            let map_name = tmp.map_name.as_thin();
            entities.add(name, map_name, tmp.state);
        }

        Ok(())
    }

    pub fn reset(&self) {
        self.entities.borrow_mut().clear();
        // TODO: init_hud = true
    }
}
