mod inventory;

mod ammo;
mod battery;
mod death_notice;
mod flashlight;
mod geiger;
mod health;
mod history;
mod menu;
mod message;
mod say_text;
mod scoreboard;
mod text_message;
mod train;
pub mod weapon_menu;

use core::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    cmp,
    ffi::c_int,
    fmt::Write,
};

use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use bitflags::bitflags;
use cl::{
    cell::SyncOnceCell,
    color::RGB,
    consts::MAX_PLAYERS,
    macros::{hook_command, spr_load},
    message::{hook_message, hook_message_flag},
    prelude::*,
    raw::{client_data_s, vec3_t, wrect_s},
    sprite::SpriteHandle,
};
use csz::{CStrArray, CStrBox};

use crate::{
    hud::{
        health::Health, inventory::Inventory, menu::Menu, scoreboard::ScoreBoard,
        text_message::TextMessage, weapon_menu::WeaponMenu,
    },
    input::{input, input_mut},
};

pub const MAX_WEAPONS: usize = 32;

const MIN_ALPHA: u8 = 100;

const FADE_TIME: f32 = 100.0;
const FADE_TIME_AMMO: f32 = 200.0;

// const DEFAULT_COLOR: RGB = RGB::YELLOWISH;
const DEFAULT_COLOR: RGB = RGB::new(255, 0, 255); // TODO: remove me

const MAX_PLAYER_NAME_LENGTH: usize = 32;

mod cvar {
    cl::cvar::define! {
        pub static zoom_sensitivity_ratio(c"1.2", ARCHIVE);
        pub static cl_autowepswitch(c"1", ARCHIVE.union(USERINFO));
        pub static default_fov(c"90", ARCHIVE);
        pub static hud_capturemouse(c"1", ARCHIVE);
        pub static hud_draw(c"1", ARCHIVE);
        pub static hud_color(c"", ARCHIVE);
    }
}

fn lower_sprite_resolution(res: u32) -> u32 {
    match res {
        2560 => 1280,
        1280 => 640,
        640 => 320,
        320 => 320,
        _ => {
            warn!("unexpected sprite resolution {res}");
            320
        }
    }
}

fn try_spr_load<F>(init: u32, load: F) -> Option<SpriteHandle>
where
    F: Fn(u32) -> Option<SpriteHandle>,
{
    let mut res = init;
    loop {
        match load(res) {
            Some(hspr) => return Some(hspr),
            None => {
                let lower = lower_sprite_resolution(res);
                if res == lower {
                    // lowest resolution
                    return None;
                }
                // try with lower resolution
                res = lower;
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Sprite {
    hspr: SpriteHandle,
    rect: wrect_s,
}

impl Sprite {
    fn new(hspr: SpriteHandle, rect: wrect_s) -> Self {
        Sprite { hspr, rect }
    }
}

#[derive(Copy, Clone)]
struct Fade {
    value: f32,
    time: f32,
}

impl Default for Fade {
    fn default() -> Self {
        Self::new(FADE_TIME)
    }
}

impl Fade {
    fn new(time: f32) -> Self {
        Self { value: 0.0, time }
    }

    fn start(&mut self) {
        self.value = self.time;
    }

    fn stop(&mut self) {
        self.value = 0.0;
    }

    fn alpha(&mut self, delta: f64) -> u8 {
        let mut a = MIN_ALPHA;
        if self.value != 0.0 {
            if self.value > self.time {
                self.value = self.time;
            }
            self.value -= (delta * 20.0) as f32;
            if self.value <= 0.0 {
                self.value = 0.0;
            }
            a += (self.value / self.time * 128.0) as u8;
        }
        a
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    #[repr(transparent)]
    pub struct HudFlags: u32 {
        const NONE = 0;
        const ACTIVE = 1;
        const INTERMISSION = 2;
    }
}

#[allow(unused_variables)]
pub trait HudItem: Any {
    fn flags(&self) -> HudFlags {
        HudFlags::ACTIVE
    }

    fn vid_init(&mut self, state: &mut State) {}

    fn init_hud_data(&mut self, state: &mut State) {}

    fn reset(&mut self) {}

    fn think(&mut self, state: &mut State) {}

    fn draw(&mut self, state: &mut State) {}
}

bitflags! {
    /// Flags to hide HUD items.
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Hide: u32 {
        /// Hide weapons.
        const WEAPONS       = 1 << 0;
        /// Hide flashlight.
        const FLASHLIGHT    = 1 << 1;
        /// Hide all.
        const ALL           = 1 << 2;
        /// Hide health.
        const HEALTH        = 1 << 3;
    }
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct Weapons: u32 {
        const SUIT          = 1 << 31;
    }
}

#[derive(Copy, Clone, Default)]
pub struct PlayerInfoExtra {
    pub frags: i16,
    pub deaths: i16,
    pub teamnumber: i16,
}

pub struct State {
    /// Default color for HUD items.
    color: RGB,
    /// Is the game in an intermission/pause?
    pub intermission: bool,
    /// Previous engine time.
    time_old: f32,
    /// Current engine time.
    time: f32,
    /// The difference between current and previous engine time.
    time_delta: f64,
    /// Default sprite resolution to load.
    res: u32,
    /// Current hide flags for HUD items.
    hide: Hide,
    /// The position of player in the world.
    pub origin: vec3_t,
    /// The camera angles.
    pub angles: vec3_t,
    /// Player inventory.
    inv: Inventory,
    /// HUD sprites names.
    sprites_names: Vec<Box<str>>,
    /// HUD sprites.
    sprites: Vec<Sprite>,
    /// HUD sprites for numbers.
    numbers: Vec<Sprite>,
    /// The width of number sprites.
    num_width: c_int,
    /// The height of number sprites.
    num_height: c_int,
    last_fov: u8,
    fov: u8,
    mouse_sensitivity: f32,
    key_bits: u32,

    server_name: CStrBox,
    player_info_extra: [Option<PlayerInfoExtra>; MAX_PLAYERS + 1],
}

impl State {
    fn new() -> Self {
        Self {
            color: DEFAULT_COLOR,
            intermission: false,
            time_old: 0.0,
            time: 1.0,
            time_delta: 0.0,
            res: 0,
            hide: Hide::empty(),
            origin: vec3_t::ZERO,
            angles: vec3_t::ZERO,
            inv: Inventory::new(),
            sprites_names: Vec::new(),
            sprites: Vec::new(),
            numbers: Vec::with_capacity(10),
            num_width: 0,
            num_height: 0,
            last_fov: 0,
            fov: 0,
            mouse_sensitivity: 0.0,
            key_bits: 0,

            server_name: Default::default(),
            player_info_extra: [None; MAX_PLAYERS + 1],
        }
    }

    fn vid_init(&mut self) {
        let engine = engine();

        let screen_info = engine.get_screen_info();
        self.res = screen_info.sprite_resolution();

        let sprite_list = engine.spr_get_list("sprites/hud.txt");
        let sprite_list = sprite_list.as_slice();
        self.sprites_names.clear();
        self.sprites.clear();
        for i in sprite_list.iter().filter(|i| i.res as u32 == self.res) {
            let Some(hspr) = spr_load!("sprites/{}.spr", i.sprite) else {
                continue;
            };
            self.sprites_names.push(i.name.to_str().unwrap().into());
            let sprite = Sprite::new(hspr, i.rc);
            self.sprites.push(sprite);
        }

        self.numbers.clear();
        for i in 0..10 {
            let mut buf = CStrArray::<64>::new();
            write!(buf.cursor(), "number_{i}").ok();
            let sprite = self.find_sprite(buf.to_str().unwrap()).unwrap();
            self.numbers.push(sprite);
        }
        self.num_width = self.numbers[0].rect.width();
        self.num_height = self.numbers[0].rect.height();

        self.inv.vid_init();

        if !engine.is_spectator_only() && !self.is_hidden(Hide::WEAPONS | Hide::ALL) {
            self.inv.set_crosshair();
        }
    }

    fn reset(&mut self) {
        self.inv.reset();
        self.fov = 90;
        self.last_fov = cvar::default_fov.value() as u8;
    }

    fn think(&mut self) {
        self.inv.think();
    }

    fn find_sprite_index(&self, name: &str) -> Option<usize> {
        self.sprites_names.iter().position(|i| i.as_ref() == name)
    }

    fn find_sprite(&self, name: &str) -> Option<Sprite> {
        self.find_sprite_index(name)
            .map(|i| &self.sprites[i])
            .copied()
    }

    pub fn is_hidden(&self, value: Hide) -> bool {
        self.hide.intersects(value)
    }

    fn has_suit(&self) -> bool {
        self.inv.weapons().contains(Weapons::SUIT)
    }

    fn draw_number(&self, number: c_int) -> DrawNumber<'_> {
        DrawNumber {
            state: self,
            width: 0,
            color: self.color,
            number,
            zero: true,
            string: false,
            reverse: false,
        }
    }

    fn get_client_color(&self, _client: c_int) -> [f32; 3] {
        const BLUE: [f32; 3] = [0.6, 0.8, 1.0];
        const RED: [f32; 3] = [1.0, 0.25, 0.25];
        const GREEN: [f32; 3] = [0.6, 1.0, 0.6];
        const YELLOW: [f32; 3] = [1.0, 0.7, 0.0];
        const GREY: [f32; 3] = [0.8, 0.8, 0.8];

        let teamnumber = 0; // TODO: extra player info
        match teamnumber {
            0 => YELLOW,
            1 => BLUE,
            2 => RED,
            3 => YELLOW,
            4 => GREEN,
            _ => GREY,
        }
    }
}

#[must_use]
struct DrawNumber<'a> {
    state: &'a State,
    width: usize,
    number: c_int,
    color: RGB,
    zero: bool,
    string: bool,
    reverse: bool,
}

impl DrawNumber<'_> {
    fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    // fn no_zero(mut self) -> Self {
    //     self.zero = false;
    //     self
    // }

    fn color(mut self, color: RGB) -> Self {
        self.color = color;
        self
    }

    fn string(mut self, string: bool) -> Self {
        self.string = string;
        self
    }

    fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    fn at(self, x: c_int, y: c_int) -> c_int {
        if self.number == 0 && !self.zero {
            return x + self.state.num_width * self.width as c_int;
        }

        let mut buf = CStrArray::<64>::new();
        write!(buf.cursor(), "{:1$}", self.number, self.width).ok();

        let engine = engine();

        if self.string {
            if self.reverse {
                return engine.draw_string_reverse(x, y, buf.as_c_str(), self.color);
            } else {
                return engine.draw_string(x, y, buf.as_c_str(), self.color);
            }
        }

        if self.reverse {
            todo!("draw number reverse is not implemented for sprites");
        }

        let mut x = x;
        for i in buf.bytes() {
            if i.is_ascii_digit() {
                let s = &self.state.numbers[i as usize - b'0' as usize];
                engine.spr_set(s.hspr, self.color);
                engine.spr_draw_additive_rect(0, x, y, s.rect);
            }
            x += self.state.num_width;
        }
        x
    }
}

type RcCell<T> = Rc<RefCell<T>>;

#[derive(Default)]
pub struct Items {
    items: Vec<(TypeId, RcCell<dyn HudItem>)>,
}

impl Items {
    pub fn add<T: HudItem + 'static>(&mut self, value: T) -> &mut Self {
        let id = value.type_id();
        let item = Rc::new(RefCell::new(value));
        match self.items.binary_search_by_key(&id, |(id, _)| *id) {
            Ok(index) => self.items[index].1 = item,
            Err(index) => self.items.insert(index, (id, item)),
        }
        self
    }

    pub fn find<T: Any>(&self) -> Option<&RcCell<T>> {
        let id = TypeId::of::<T>();
        match self.items.binary_search_by_key(&id, |(i, _)| *i) {
            Ok(i) => {
                let item = &self.items[i].1 as *const RcCell<dyn HudItem>;
                Some(unsafe { &*(item as *const RcCell<T>) })
            }
            Err(_) => None,
        }
    }

    pub fn get<T: Any>(&self) -> Ref<'_, T> {
        self.find::<T>().unwrap().borrow()
    }

    pub fn get_mut<T: Any>(&self) -> RefMut<'_, T> {
        self.find::<T>().unwrap().borrow_mut()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = RefMut<'_, dyn HudItem>> {
        self.items.iter_mut().map(|(_, i)| i.borrow_mut())
    }
}

pub struct Hud {
    pub state: State,
    pub items: Items,

    #[allow(dead_code)]
    pub text_message: TextMessage,

    logo: bool,
    logo_hspr: Option<SpriteHandle>,
    old_hud_color: String,
}

impl Hud {
    fn new() -> Self {
        hook_messages_and_commands();

        let mut items = Items::default();
        items
            .add(ammo::Ammo::new())
            .add(history::History::new())
            .add(weapon_menu::WeaponMenu::new())
            .add(health::Health::new())
            .add(battery::Battery::new())
            .add(flashlight::Flashlight::new())
            .add(geiger::Geiger::new())
            .add(train::Train::new())
            .add(death_notice::DeathNotice::new())
            .add(say_text::SayText::new())
            .add(menu::Menu::new())
            .add(message::HudMessage::new())
            .add(scoreboard::ScoreBoard::new());

        Self {
            state: State::new(),
            items,

            text_message: TextMessage::new(),

            logo: false,
            logo_hspr: None,
            old_hud_color: String::new(),
        }
    }

    pub fn get_sensitivity(&self) -> f32 {
        self.state.mouse_sensitivity
    }

    pub fn set_fov(&mut self, fov: u8) {
        let default_fov = cvar::default_fov.value() as u8;
        self.state.fov = if fov == 0 { default_fov } else { fov };

        self.state.mouse_sensitivity = if self.state.fov != default_fov {
            let zsr = cvar::zoom_sensitivity_ratio.value();
            input().get_mouse_sensitivity() * (fov as f32 / default_fov as f32) * zsr
        } else {
            0.0
        };
    }

    pub fn set_last_fov(&mut self, fov: u8) {
        self.state.last_fov = fov;
    }

    pub fn get_last_fov(&self) -> u8 {
        // TODO: demo api
        self.state.last_fov
    }

    pub fn get_fov(&self) -> u8 {
        self.state.fov
    }

    pub fn score_info(
        &mut self,
        cl: u8,
        frags: i16,
        deaths: i16,
        _player_class: i16,
        teamnumber: i16,
    ) {
        if cl > 0 && cl <= MAX_PLAYERS as u8 {
            let index = cl as usize;
            let extra = PlayerInfoExtra {
                frags,
                deaths,
                teamnumber: cmp::max(0, teamnumber),
            };
            self.items.get_mut::<ScoreBoard>().score_info(cl, &extra);
            self.state.player_info_extra[index] = Some(extra);
        }
    }

    pub fn show_score_board(&mut self, show: bool) {
        if engine().is_multiplayer() {
            self.items.get_mut::<ScoreBoard>().show(show);
        }
    }

    pub fn vid_init(&mut self) {
        self.logo_hspr = None;

        self.state.vid_init();

        for mut i in self.items.iter_mut() {
            i.vid_init(&mut self.state);
        }
    }

    fn init_hud(&mut self) {
        for mut i in self.items.iter_mut() {
            i.init_hud_data(&mut self.state);
        }

        self.reset();
    }

    pub fn reset(&mut self) {
        self.state.reset();

        for mut i in self.items.iter_mut() {
            i.reset();
        }
    }

    fn think(&mut self) {
        self.state.think();

        for mut i in self.items.iter_mut() {
            i.think(&mut self.state);
        }

        self.set_fov(self.get_last_fov());
        if self.state.fov == 0 {
            self.state.fov = cmp::max(90, cvar::default_fov.value() as u8);
        }
    }

    pub fn show_score(&self) -> bool {
        self.items.get::<Health>().is_dead() || self.state.intermission
    }

    pub fn update_client_data(&mut self, data: &mut client_data_s, _time: f32) -> bool {
        self.state.origin = data.origin;
        self.state.angles = data.viewangles;

        let show_score = self.show_score();
        self.state.key_bits = input_mut().button_bits(false, show_score);

        self.state
            .inv
            .weapons_set(Weapons::from_bits_retain(data.iWeaponBits as u32));

        self.think();

        data.fov = self.get_fov() as f32;

        input_mut().reset_button_bits(self.state.key_bits, show_score);

        // data has been changed
        true
    }

    fn update_hud_color(&mut self) {
        const COLOR_MAP: [(&str, RGB); 16] = [
            ("black", RGB::BLACK),
            ("silver", RGB::SILVER),
            ("gray", RGB::GRAY),
            ("white", RGB::WHITE),
            ("maroon", RGB::MAROON),
            ("red", RGB::RED),
            ("green", RGB::GREEN),
            ("lime", RGB::LIME),
            ("navy", RGB::NAVY),
            ("blue", RGB::BLUE),
            ("yellowish", RGB::YELLOWISH),
            ("redish", RGB::REDISH),
            ("greenish", RGB::GREENISH),
            ("purple", RGB::PURPLE),
            ("fuchsia", RGB::FUCHSIA),
            ("cyan", RGB::CYAN),
        ];

        let s = cvar::hud_color.value_str();
        let Ok(s) = s.to_str() else { return };

        if self.old_hud_color == s {
            return;
        }
        self.old_hud_color.clear();
        self.old_hud_color.push_str(s);

        if s.is_empty() {
            self.state.color = DEFAULT_COLOR;
            return;
        }

        if s == "help" {
            info!("  empty (default color), hex value (RGB, RRGGBB) or color name:");
            for (color, _) in &COLOR_MAP {
                info!("    {color}");
            }
            return;
        }

        self.state.color = match COLOR_MAP.iter().find(|i| i.0 == s) {
            Some((_, color)) => *color,
            None => match parse_color(s) {
                Some(c) => c,
                None => {
                    warn!("invalid hud_color {s:?}");
                    DEFAULT_COLOR
                }
            },
        };
    }

    fn draw_logo(&mut self) {
        const MAX_LOGO_FRAMES: usize = 56;
        const LOGO_FRAME: [c_int; MAX_LOGO_FRAMES] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 13, 13, 13, 13, 13, 12, 11, 10, 9, 8, 14,
            15, 16, 17, 18, 19, 20, 20, 20, 20, 20, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 29, 29,
            29, 29, 29, 28, 27, 26, 25, 24, 30, 31,
        ];

        if !self.logo {
            return;
        }

        if self.logo_hspr.is_none() {
            self.logo_hspr =
                try_spr_load(self.state.res, |res| spr_load!("sprites/{res}_logo.spr"));
        }

        let Some(hspr) = self.logo_hspr else { return };

        let engine = engine();
        let info = engine.get_screen_info();
        let (w, h) = engine.spr_size(hspr, 0);
        let x = info.width - w;
        let y = h / 2;
        let frame = (self.state.time * 20.0) as usize % MAX_LOGO_FRAMES;
        let i = LOGO_FRAME[frame] - 1;

        engine.spr_set(hspr, RGB::new(250, 250, 250));
        engine.spr_draw_additive(i, x, y);
    }

    pub fn draw(&mut self, time: f32, intermission: bool) -> bool {
        self.state.time_old = self.state.time;
        self.state.time = time;
        self.state.time_delta = time as f64 - self.state.time_old as f64;
        if self.state.time_delta < 0.0 {
            self.state.time_delta = 0.0;
        }
        self.state.intermission = intermission;

        if cvar::hud_draw.value() != 0.0 {
            self.update_hud_color();
            for mut i in self.items.iter_mut() {
                let flags = i.flags();
                if !intermission {
                    if flags.contains(HudFlags::ACTIVE) && !self.state.is_hidden(Hide::ALL) {
                        i.draw(&mut self.state);
                    }
                } else if flags.contains(HudFlags::INTERMISSION) {
                    i.draw(&mut self.state);
                }
            }
        }

        self.draw_logo();
        true
    }
}

fn hex(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

fn parse_color(s: &str) -> Option<RGB> {
    if !s
        .chars()
        .all(|i| i.is_ascii_digit() || ('a'..='f').contains(&i) || ('A'..='F').contains(&i))
    {
        return None;
    }
    let s = s.as_bytes();
    if ![3, 6].contains(&s.len()) {
        return None;
    }
    let mut c = [0; 3];
    if s.len() == 3 {
        for i in 0..3 {
            let x = hex(s[i]);
            c[i] = x | (x << 4);
        }
    } else {
        for (i, c) in c.iter_mut().enumerate() {
            let j = i * 2;
            *c = (hex(s[j]) << 4) | hex(s[j]);
        }
    }
    Some(c.into())
}

fn hook_messages_and_commands() {
    hook_message_flag!(Logo, hud_mut().logo);

    hook_message!(InitHUD, {
        hud_mut().init_hud();
        true
    });

    hook_message!(GameMode, {
        trace!("message GameMode is not implemented");
        true
    });

    hook_message!(SetFOV, |msg| {
        let fov = msg.read_u8()?;
        let mut hud = hud_mut();
        hud.set_last_fov(fov);
        hud.set_fov(fov);
        Ok(())
    });

    hook_message!(ScoreInfo, |msg| {
        let cl = msg.read_u8()?;
        let frags = msg.read_i16()?;
        let deaths = msg.read_i16()?;
        let player_class = msg.read_i16()?;
        let teamnumber = msg.read_i16()?;
        hud_mut().score_info(cl, frags, deaths, player_class, teamnumber);
        Ok(())
    });

    hook_message!(ServerName, |msg| {
        hud_mut().state.server_name = msg.read_cstr()?.into();
        Ok(())
    });

    hook_message!(ResetHUD, |msg| {
        if !matches!(msg.data(), &[0]) {
            warn!("ResetHUD: unexpected user message data");
        }
        hud_mut().reset();
        true
    });

    fn cmd_slot(slot: u32) {
        let hud = &mut *hud_mut();
        let mut menu = hud.items.get_mut::<Menu>();
        if menu.is_displayed() {
            menu.select_menu_item(slot);
        } else {
            hud.items
                .get_mut::<WeaponMenu>()
                .select_slot(&mut hud.state, slot);
        }
    }

    hook_command!(c"slot1", cmd_slot(1));
    hook_command!(c"slot2", cmd_slot(2));
    hook_command!(c"slot3", cmd_slot(3));
    hook_command!(c"slot4", cmd_slot(4));
    hook_command!(c"slot5", cmd_slot(5));
    hook_command!(c"slot6", cmd_slot(6));
    hook_command!(c"slot7", cmd_slot(7));
    hook_command!(c"slot8", cmd_slot(8));
    hook_command!(c"slot9", cmd_slot(9));
    hook_command!(c"slot10", cmd_slot(10));
}

static HUD: SyncOnceCell<RefCell<Hud>> = unsafe { SyncOnceCell::new() };

fn hud_global() -> &'static RefCell<Hud> {
    HUD.get_or_init(|| RefCell::new(Hud::new()))
}

pub fn hud<'a>() -> Ref<'a, Hud> {
    hud_global().borrow()
}

pub fn hud_mut<'a>() -> RefMut<'a, Hud> {
    hud_global().borrow_mut()
}

pub fn init() {
    hud_global();
}
