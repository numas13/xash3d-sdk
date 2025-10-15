use core::{
    cell::{Cell, Ref, RefCell, RefMut},
    ffi::{c_int, CStr},
    mem::MaybeUninit,
};

use alloc::{boxed::Box, collections::linked_list::LinkedList};
use csz::{CStrArray, CStrThin};
use xash3d_shared::{engine::EngineRef, export::impl_unsync_global, ffi::server::TYPEDESCRIPTION};

use crate::{
    define_fields,
    engine::ServerEngineRef,
    entity::EntityHandle,
    game_rules::{GameRules, StubGameRules},
    save::{FieldType, SaveFields, SaveReader, SaveRestoreData, SaveResult, SaveWriter},
    sound::Sentences,
    str::MapString,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub enum EntityState {
    #[default]
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

    pub fn set_state(&mut self, state: EntityState) {
        self.state = state;
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

unsafe impl SaveFields for GlobalEntity {
    const SAVE_NAME: &'static CStr = c"GENT";

    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![
        name,
        map_name,
        state => unsafe FieldType::INTEGER,
    ];
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

unsafe impl SaveFields for GlobalStateSave {
    const SAVE_NAME: &'static CStr = c"GLOBAL";

    const SAVE_FIELDS: &'static [TYPEDESCRIPTION] = &define_fields![list_count];
}

pub struct GlobalState {
    engine: ServerEngineRef,
    entities: RefCell<Entities>,
    game_rules: RefCell<Box<dyn GameRules>>,
    last_spawn: Cell<Option<EntityHandle>>,
    init_hud: Cell<bool>,
    sentences: RefCell<Option<Sentences>>,
}

impl_unsync_global!(GlobalState);

impl GlobalState {
    pub fn new(engine: ServerEngineRef) -> Self {
        Self {
            engine,
            entities: RefCell::new(Entities::new()),
            game_rules: RefCell::new(Box::new(StubGameRules::new(engine))),
            last_spawn: Cell::new(None),
            init_hud: Cell::new(true),
            sentences: RefCell::new(None),
        }
    }

    pub fn entities(&self) -> Ref<'_, Entities> {
        self.entities.borrow()
    }

    pub fn entities_mut(&self) -> RefMut<'_, Entities> {
        self.entities.borrow_mut()
    }

    pub fn game_rules(&self) -> Ref<'_, dyn GameRules> {
        Ref::map(self.game_rules.borrow(), |i| i.as_ref())
    }

    pub fn game_rules_mut(&self) -> RefMut<'_, dyn GameRules> {
        RefMut::map(self.game_rules.borrow_mut(), |i| i.as_mut())
    }

    pub fn set_game_rules<T: GameRules>(&self, game_rules: T) {
        self.game_rules.replace(Box::new(game_rules));
    }

    pub fn last_spawn(&self) -> Option<EntityHandle> {
        self.last_spawn.get()
    }

    pub fn set_last_spawn(&self, ent: Option<EntityHandle>) {
        self.last_spawn.set(ent);
    }

    pub fn save(&self, save_data: &mut SaveRestoreData) -> SaveResult<()> {
        let mut writer = SaveWriter::new(self.engine);
        let entities = self.entities.borrow();
        let global_state = GlobalStateSave {
            list_count: entities.list.len() as i32,
        };
        writer.write_fields(save_data, &global_state)?;
        for ent in &entities.list {
            writer.write_fields(save_data, ent)?;
        }
        Ok(())
    }

    pub fn restore(&self, save_data: &mut SaveRestoreData) -> SaveResult<()> {
        let mut reader = SaveReader::new(self.engine);
        self.reset();

        let mut global_state = GlobalStateSave { list_count: 0 };
        reader.read_fields(save_data, &mut global_state)?;

        let mut entities = self.entities.borrow_mut();
        for _ in 0..global_state.list_count {
            let mut tmp = MaybeUninit::<GlobalEntity>::uninit();
            reader.read_fields(save_data, unsafe { tmp.assume_init_mut() })?;
            let tmp = unsafe { tmp.assume_init() };
            let name = tmp.name.as_thin();
            let map_name = tmp.map_name.as_thin();
            entities.add(name, map_name, tmp.state);
        }

        Ok(())
    }

    pub fn reset(&self) {
        self.entities_mut().clear();
        self.init_hud.set(true);
    }

    pub fn entity_state(&self, name: MapString) -> EntityState {
        self.entities()
            .find(name)
            .map(|ent| ent.state())
            .unwrap_or_default()
    }

    pub fn set_entity_state(&self, name: MapString, state: EntityState) {
        if let Some(ent) = self.entities_mut().find_mut(name) {
            ent.set_state(state);
        }
    }

    /// Returns `true` if the client HUD needs to be initialized.
    pub fn init_hud(&self) -> bool {
        self.init_hud.get()
    }

    /// Call with `false` after initializing the client HUD.
    pub fn set_init_hud(&self, value: bool) {
        self.init_hud.set(value);
    }

    /// Initialize sentences.
    ///
    /// Must be called by world spawn.
    pub fn sentence_init(&self) {
        let mut sentences = self.sentences.borrow_mut();
        if sentences.is_none() {
            *sentences = Some(Sentences::new(self.engine));
        }
    }

    pub fn sentences(&self) -> Ref<'_, Sentences> {
        Ref::filter_map(self.sentences.borrow(), |i| i.as_ref())
            .ok()
            .expect("sentences must be initialized by world spawn")
    }
}

pub type GlobalStateRef = EngineRef<GlobalState>;
