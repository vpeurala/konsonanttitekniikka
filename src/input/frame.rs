//! Everything the outside world tells the game in one frame: how long it
//! was, what time it is, which keys and fingers did what.
//!
//! `Inputs::read` is the one place that asks the window and the clock.
//! Everything else gets a `Frame`, so it can be run with made-up frames,
//! as the tests do.

use macroquad::prelude::*;

use crate::gfx::view::View;
use crate::input::keyboard::{Key, Keyboard};
use crate::input::lifecycle::{self, Lifecycle};
use crate::input::touch::{Pointer, TouchReader};

/// One frame's worth of input.
#[derive(Debug, Default, Clone)]
pub struct Frame {
    /// Seconds since the previous frame.
    pub dt: f32,
    /// The current time, in seconds since 1970.
    pub now: f64,
    /// The app was in the background since the previous frame.
    pub away: bool,
    /// The size of the window, in screen points.
    pub screen: Vec2,
    /// The characters typed and Backspaces pressed, in order.
    pub typed: Vec<Key>,
    /// The keys that went down this frame.
    pub pressed: Vec<KeyCode>,
    /// The keys held down.
    pub down: Vec<KeyCode>,
    /// How far the mouse wheel moved the content down.
    pub wheel: f32,
    /// Touches and mouse clicks, in screen points.
    pub touches: Vec<Pointer>,
}

impl Frame {
    /// Whether `key` went down this frame.
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Whether `key` is held down.
    pub fn down(&self, key: KeyCode) -> bool {
        self.down.contains(&key)
    }

    /// Whether any of `keys` went down this frame.
    pub fn any_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&key| self.pressed(key))
    }

    /// The direction the arrow keys held down point in, each axis -1, 0
    /// or 1.
    pub fn arrows(&self) -> Vec2 {
        let axis = |negative, positive| {
            f32::from(i8::from(self.down(positive)) - i8::from(self.down(negative)))
        };
        vec2(
            axis(KeyCode::Left, KeyCode::Right),
            axis(KeyCode::Up, KeyCode::Down),
        )
    }

    /// The touches and clicks as the screen `view` shows them: positions
    /// in virtual units.
    pub fn pointers(&self, view: &View) -> Vec<Pointer> {
        self.touches
            .iter()
            .map(|p| Pointer {
                pos: view.to_virtual(p.pos),
                ..*p
            })
            .collect()
    }
}

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

    /// This frame's input. Must be called once every frame, so nothing
    /// typed or touched piles up.
    pub fn read(&self) -> Frame {
        let dt = get_frame_time();
        Frame {
            dt,
            now: miniquad::date::now(),
            away: lifecycle::was_away(self.lifecycle.minimized(), dt),
            screen: vec2(screen_width(), screen_height()),
            typed: self.keyboard.typed(),
            pressed: get_keys_pressed().into_iter().collect(),
            down: get_keys_down().into_iter().collect(),
            wheel: mouse_wheel().1,
            touches: self.touches.read(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holding(keys: &[KeyCode]) -> Frame {
        Frame {
            down: keys.to_vec(),
            ..Frame::default()
        }
    }

    #[test]
    fn no_arrow_keys_point_nowhere() {
        assert_eq!(holding(&[]).arrows(), Vec2::ZERO);
    }

    #[test]
    fn arrow_keys_point_where_they_say() {
        assert_eq!(holding(&[KeyCode::Right]).arrows(), vec2(1.0, 0.0));
        assert_eq!(holding(&[KeyCode::Left]).arrows(), vec2(-1.0, 0.0));
        assert_eq!(holding(&[KeyCode::Up]).arrows(), vec2(0.0, -1.0));
        assert_eq!(holding(&[KeyCode::Down]).arrows(), vec2(0.0, 1.0));
    }

    #[test]
    fn opposite_arrow_keys_cancel_out() {
        assert_eq!(
            holding(&[KeyCode::Left, KeyCode::Right]).arrows(),
            Vec2::ZERO
        );
    }

    #[test]
    fn diagonals_combine() {
        assert_eq!(
            holding(&[KeyCode::Right, KeyCode::Up]).arrows(),
            vec2(1.0, -1.0)
        );
    }

    #[test]
    fn pointers_are_mapped_into_virtual_units() {
        let view = View::fit(Rect::new(0.0, 0.0, 800.0, 600.0), vec2(1600.0, 1200.0));
        let frame = Frame {
            touches: vec![Pointer {
                id: 7,
                pos: vec2(800.0, 600.0),
                phase: TouchPhase::Started,
            }],
            ..Frame::default()
        };
        let pointers = frame.pointers(&view);
        assert_eq!(pointers[0].id, 7);
        assert_eq!(pointers[0].pos, vec2(400.0, 300.0));
        assert_eq!(pointers[0].phase, TouchPhase::Started);
    }
}
