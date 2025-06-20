use core::{cell::RefCell, ffi::c_int, mem::MaybeUninit, ptr};

use alloc::collections::linked_list::LinkedList;
use csz::{CStrArray, CStrThin};
use sv::{
    cell::SyncOnceCell,
    globals,
    macros::define_field,
    raw::{edict_s, string_t, FieldType, SAVERESTOREDATA, TYPEDESCRIPTION},
};

use crate::save::{self, SaveRestore};

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

    pub fn find(&self, name: &CStrThin) -> Option<&GlobalEntity> {
        self.list.iter().find(|i| i.name() == name)
    }

    pub fn find_mut(&mut self, name: &CStrThin) -> Option<&mut GlobalEntity> {
        self.list.iter_mut().find(|i| i.name() == name)
    }

    pub fn find_string(&self, name: string_t) -> Option<&GlobalEntity> {
        self.find(globals().string(name).into())
    }

    pub fn find_string_mut(&mut self, name: string_t) -> Option<&mut GlobalEntity> {
        self.find_mut(globals().string(name).into())
    }

    pub fn add(&mut self, name: &CStrThin, map_name: &CStrThin, state: EntityState) {
        assert!(self.find(name).is_none());
        self.list
            .push_back(GlobalEntity::new(name, map_name, state));
    }

    pub fn add_string(&mut self, name: string_t, map_name: string_t, state: EntityState) {
        let globals = globals();
        let name = globals.string(name).into();
        let map_name = globals.string(map_name).into();
        self.add(name, map_name, state)
    }

    pub fn update(&mut self, name: string_t, map_name: string_t) {
        if let Some(ent) = self.find_string_mut(name) {
            ent.map_name.clear();
            ent.map_name
                .cursor()
                .write_c_str(globals().string(map_name).into())
                .unwrap();
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
    pub entities: RefCell<Entities>,
    pub last_spawn: RefCell<*mut edict_s>,
}

impl GlobalState {
    fn new() -> Self {
        Self {
            entities: RefCell::new(Entities::new()),
            last_spawn: RefCell::new(ptr::null_mut()),
        }
    }

    pub fn save(&self, save_data: &mut SAVERESTOREDATA) -> save::Result<()> {
        let _helper = SaveRestore::new(save_data);
        debug!("TODO: SaveGlobalState");
        Ok(())
    }

    pub fn restore(&self, save_data: &mut SAVERESTOREDATA) -> save::Result<()> {
        let mut helper = SaveRestore::new(save_data);
        self.reset();

        let mut global_state = GlobalStateSave { list_count: 0 };
        helper.read_fields(
            c"GLOBAL",
            &mut global_state as *mut _ as *mut _,
            &GLOBAL_FIELDS,
        )?;

        let mut entities = self.entities.borrow_mut();
        for _ in 0..global_state.list_count {
            let mut tmp = MaybeUninit::<GlobalEntity>::uninit();
            helper.read_fields(c"GENT", tmp.as_mut_ptr().cast(), &GLOBAL_ENTITY_FIELDS)?;
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

static GLOBAL_STATE: SyncOnceCell<GlobalState> = unsafe { SyncOnceCell::new() };

pub fn global_state() -> &'static GlobalState {
    GLOBAL_STATE.get_or_init(GlobalState::new)
}
