//! Making a `Frame` from the window and the clock.
//!
//! `Inputs::read` is the one place that asks them. Everything else gets a
//! `Frame` (defined in the core crate), so it can be run with made-up
//! frames, as the tests do.

use macroquad::prelude as mq;

use crate::input::keyboard::Keyboard;
use crate::input::lifecycle::{self, Lifecycle};
use crate::input::touch::TouchReader;
use lukuloitsu_core::input::{Frame, KeyCode};

/// The subscriptions to window events that `Frame`s are made from.
pub struct Inputs {
    keyboard: Keyboard,
    touches: TouchReader,
    lifecycle: Lifecycle,
}

impl Inputs {
    pub fn new() -> Self {
        Inputs {
            keyboard: Keyboard::new(),
            touches: TouchReader::new(),
            lifecycle: Lifecycle::new(),
        }
    }

    /// The current time, in seconds since 1970, for what happens before
    /// the first frame.
    pub fn now() -> f64 {
        macroquad::miniquad::date::now()
    }

    /// This frame's input. Must be called once every frame, so nothing
    /// typed or touched piles up.
    pub fn read(&self) -> Frame {
        let dt = mq::get_frame_time();
        Frame {
            dt,
            now: Self::now(),
            away: lifecycle::was_away(self.lifecycle.minimized(), dt),
            screen: mq::vec2(mq::screen_width(), mq::screen_height()),
            typed: self.keyboard.typed(),
            pressed: mq::get_keys_pressed()
                .into_iter()
                .filter_map(key_code)
                .collect(),
            down: mq::get_keys_down()
                .into_iter()
                .filter_map(key_code)
                .collect(),
            wheel: mq::mouse_wheel().1,
            touches: self.touches.read(),
        }
    }
}

/// The keys the app cares about, from the window's.
fn key_code(key: mq::KeyCode) -> Option<KeyCode> {
    Some(match key {
        mq::KeyCode::Space => KeyCode::Space,
        mq::KeyCode::Enter => KeyCode::Enter,
        mq::KeyCode::Escape => KeyCode::Escape,
        mq::KeyCode::Backspace => KeyCode::Backspace,
        mq::KeyCode::Tab => KeyCode::Tab,
        mq::KeyCode::Up => KeyCode::Up,
        mq::KeyCode::Down => KeyCode::Down,
        mq::KeyCode::Left => KeyCode::Left,
        mq::KeyCode::Right => KeyCode::Right,
        mq::KeyCode::PageUp => KeyCode::PageUp,
        mq::KeyCode::PageDown => KeyCode::PageDown,
        mq::KeyCode::Home => KeyCode::Home,
        mq::KeyCode::End => KeyCode::End,
        mq::KeyCode::E => KeyCode::E,
        mq::KeyCode::H => KeyCode::H,
        mq::KeyCode::K => KeyCode::K,
        _ => return None,
    })
}
