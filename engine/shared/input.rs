use core::{ffi::c_int, mem};

use bitflags::bitflags;
use xash3d_ffi::common::kbutton_t;

bitflags! {
    #[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct KeyState: c_int {
        const DOWN = 1 << 0;
        const IMPULSE_DOWN = 1 << 1;
        const ANY_DOWN = Self::DOWN.bits() | Self::IMPULSE_DOWN.bits();
        const IMPULSE_UP = 1 << 2;
    }
}

// TODO: add safe wrapper for kbutton_t and remove this trait
pub trait KButtonExt {
    fn new() -> Self;

    fn state(&self) -> &KeyState;

    fn state_mut(&mut self) -> &mut KeyState;

    fn is_down(&self) -> bool {
        self.state().contains(KeyState::DOWN)
    }

    fn is_up(&self) -> bool {
        !self.is_down()
    }

    fn is_impulse_down(&self) -> bool {
        self.state().intersects(KeyState::IMPULSE_DOWN)
    }

    fn is_impulse_up(&self) -> bool {
        self.state().intersects(KeyState::IMPULSE_UP)
    }
}

impl KButtonExt for kbutton_t {
    fn new() -> Self {
        kbutton_t {
            down: [0; 2],
            state: 0,
        }
    }

    fn state(&self) -> &KeyState {
        unsafe { mem::transmute(&self.state) }
    }

    fn state_mut(&mut self) -> &mut KeyState {
        unsafe { mem::transmute(&mut self.state) }
    }
}
