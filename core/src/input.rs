//! Everything the outside world tells the app in one frame: how long it
//! was, what time it is, which keys and fingers did what. The shell makes
//! a `Frame` from the window each frame (`Inputs::read`, the one place
//! that asks the window and the clock); everything else gets it as an
//! argument, so it can be run with made-up frames, as the tests do.

use glam::{Vec2, vec2};

use crate::geometry::Rect;
pub use crate::key::Key;
use crate::view::View;

/// The physical keys the app reacts to, apart from typing (see `Key`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Space,
    Enter,
    Escape,
    Backspace,
    Tab,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    E,
    H,
    K,
    /// A key that does nothing, for tests.
    A,
}

/// What a finger, or the mouse, is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Started,
    Moved,
    Stationary,
    Ended,
    Cancelled,
}

/// A finger or the mouse.
#[derive(Debug, Clone, Copy)]
pub struct Pointer {
    pub id: u64,
    pub pos: Vec2,
    pub phase: Phase,
}

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

    /// The touches and clicks as the screen shows `content`: positions in
    /// virtual units.
    pub fn pointers(&self, content: Rect) -> Vec<Pointer> {
        let view = View::fit(content, self.screen);
        self.touches
            .iter()
            .map(|p| Pointer {
                pos: view.to_virtual(p.pos),
                ..*p
            })
            .collect()
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
        let frame = Frame {
            screen: vec2(1600.0, 1200.0),
            touches: vec![Pointer {
                id: 7,
                pos: vec2(800.0, 600.0),
                phase: Phase::Started,
            }],
            ..Frame::default()
        };
        let pointers = frame.pointers(Rect::new(0.0, 0.0, 800.0, 600.0));
        assert_eq!(pointers[0].id, 7);
        assert_eq!(pointers[0].pos, vec2(400.0, 300.0));
        assert_eq!(pointers[0].phase, Phase::Started);
    }
}
