use shared::ffi::common::kbutton_t;

use crate::prelude::*;

pub use shared::input::*;

// TODO: add safe wrapper for kbutton_t and remove this trait
pub trait KeyButtonExt {
    fn key_down(&mut self);
    fn key_up(&mut self);
    fn key_state(&mut self) -> f32;
}

impl KeyButtonExt for kbutton_t {
    fn key_down(&mut self) {
        let s = engine().cmd_argv(1);
        let k = if !s.is_empty() {
            s.to_str().ok().and_then(|s| s.parse().ok()).unwrap_or(0)
        } else {
            // typed manually at the console for continuous down
            -1
        };

        if !self.down.contains(&k) {
            if let Some(i) = self.down.iter_mut().find(|i| **i == 0) {
                *i = k;

                if !self.is_down() {
                    self.state_mut()
                        .insert(KeyState::DOWN | KeyState::IMPULSE_DOWN);
                }
            }
        }
    }

    fn key_up(&mut self) {
        let s = engine().cmd_argv(1);
        if !s.is_empty() {
            let k = s.to_str().ok().and_then(|s| s.parse().ok()).unwrap_or(0);

            if let Some(i) = self.down.iter_mut().find(|i| **i == k) {
                *i = 0;

                if self.is_down() && !self.down.iter().any(|i| *i != 0) {
                    self.state_mut().remove(KeyState::DOWN);
                    self.state_mut().insert(KeyState::IMPULSE_UP);
                }
            }
        } else {
            // typed manually at the console, assume for unsticking, so clear all
            self.down.fill(0);
            self.state_mut().insert(KeyState::IMPULSE_UP);
        }
    }

    fn key_state(&mut self) -> f32 {
        let mut val = 0.0;
        let impulsedown = self.is_impulse_down();
        let impulseup = self.is_impulse_up();
        let down = self.is_down();

        if impulsedown && !impulseup && down {
            // pressed and held this frame?
            val = 0.5;
        }

        if impulseup && !impulsedown && down {
            // released this frame?
            val = 0.0;
        }

        if !impulsedown && !impulseup && down {
            // held the entire frame?
            val = 1.0;
        }

        if impulsedown && impulseup {
            if down {
                // released and re-pressed this frame
                val = 0.75;
            } else {
                // pressed and released this frame
                val = 0.25;
            }
        }

        // clear impulses
        *self.state_mut() &= KeyState::DOWN;

        val
    }
}
