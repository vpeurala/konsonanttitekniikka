//! Touch controls: a floating joystick in the arena for moving, and a
//! keypad beside the arena with exactly the keys the game needs. On a
//! computer, the mouse stands in for a finger, which makes the controls
//! testable without a phone. Reading the touches is the shell's job, and
//! so is drawing the controls.

use glam::{Vec2, vec2};

use crate::arena::{ARENA_H, ARENA_W};
use crate::geometry::Rect;
use crate::input::{Key, Phase, Pointer};
use crate::pairs::is_answer_char;

/// A key's size, and the gaps around keys and panel edges.
pub const KEY: f32 = 60.0;
pub const GAP: f32 = 6.0;
const PAD: f32 = 10.0;

/// The keypad is split like a tablet keyboard: the left hand's keys on a
/// panel left of the arena, the right hand's on a panel right of it.
const LEFT_COLUMNS: f32 = 5.0;
const RIGHT_COLUMNS: f32 = 6.0;
pub const LEFT_W: f32 = 2.0 * PAD + LEFT_COLUMNS * KEY + (LEFT_COLUMNS - 1.0) * GAP;
pub const RIGHT_W: f32 = 2.0 * PAD + RIGHT_COLUMNS * KEY + (RIGHT_COLUMNS - 1.0) * GAP;

/// Everything the game shows, in virtual units: the arena, with the keypad
/// panels on both sides when using touch controls.
pub fn content_rect(touch: bool) -> Rect {
    if touch {
        Rect::new(-LEFT_W, 0.0, LEFT_W + ARENA_W + RIGHT_W, ARENA_H)
    } else {
        Rect::new(0.0, 0.0, ARENA_W, ARENA_H)
    }
}

/// On-screen buttons besides the keypad keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Pause,
    Music,
}

/// What the touch controls did this frame.
#[derive(Debug, Default)]
pub struct TouchInput {
    /// Joystick direction; its length, up to 1, is how far it is pushed.
    pub movement: Vec2,
    pub keys: Vec<Key>,
    pub buttons: Vec<Button>,
    /// Taps in the arena that didn't start the joystick moving.
    pub arena_taps: usize,
}

pub enum Label {
    Char(char),
    Backspace,
    /// A key of the full keyboard the game never uses, shown greyed out so
    /// the layout looks familiar, but doing nothing.
    Unused(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct KeySpec {
    pub label: Label,
    pub side: Side,
    /// Column within its panel, and row: 0 for the digits, then the
    /// keyboard's Q, A and Z rows.
    pub col: f32,
    pub row: f32,
}

/// A whole Finnish QWERTY keyboard, split between the hands. Keys the game
/// never uses are greyed out.
const LEFT_ROWS: [&str; 4] = ["12345", "qwert", "asdfg", "zxcvb"];
const RIGHT_ROWS: [&str; 4] = ["67890<", "yuiopå", "hjklöä", "nm,.-"];

pub fn keypad() -> Vec<KeySpec> {
    let mut keys = Vec::new();
    for (side, rows) in [(Side::Left, LEFT_ROWS), (Side::Right, RIGHT_ROWS)] {
        for (row, chars) in rows.iter().enumerate() {
            for (col, c) in chars.chars().enumerate() {
                let label = match c {
                    // Backspace sits right of 0, as on a real keyboard.
                    '<' => Label::Backspace,
                    c if is_answer_char(c) => Label::Char(c),
                    c => Label::Unused(c),
                };
                keys.push(KeySpec {
                    label,
                    side,
                    col: col as f32,
                    row: row as f32,
                });
            }
        }
    }
    keys
}

/// The top of the digit row. The keys sit low, where thumbs rest.
const KEYS_TOP: f32 = ARENA_H - PAD - 4.0 * KEY - 3.0 * GAP;

fn panel_left(side: Side) -> f32 {
    match side {
        Side::Left => -LEFT_W + PAD,
        Side::Right => ARENA_W + PAD,
    }
}

pub fn key_rect(key: &KeySpec) -> Rect {
    Rect::new(
        panel_left(key.side) + key.col * (KEY + GAP),
        KEYS_TOP + key.row * (KEY + GAP),
        KEY,
        KEY,
    )
}

/// Pause in the top left corner, music in the top right.
pub fn button_rect(button: Button) -> Rect {
    let w = 2.0 * KEY + GAP;
    let x = match button {
        Button::Pause => panel_left(Side::Left),
        Button::Music => ARENA_W + RIGHT_W - PAD - w,
    };
    Rect::new(x, PAD, w, KEY)
}

pub const STICK_RADIUS: f32 = 60.0;
/// Pushes shorter than this fraction of the radius don't move.
const STICK_DEAD_ZONE: f32 = 0.15;
/// Where the joystick is hinted at before the first touch.
pub const STICK_HOME: Vec2 = vec2(110.0, ARENA_H - 110.0);
/// How long a pressed key stays highlighted and magnified.
const PRESS_FLASH_SECONDS: f32 = 0.3;

struct Stick {
    id: u64,
    base: Vec2,
    knob: Vec2,
}

#[derive(Default)]
pub struct TouchControls {
    stick: Option<Stick>,
    /// Keys recently pressed, with seconds left of their highlight.
    flashes: Vec<(usize, f32)>,
}

impl TouchControls {
    /// Reads this frame's `pointers`; `dt` is the frame time in seconds.
    pub fn update(&mut self, pointers: &[Pointer], dt: f32) -> TouchInput {
        for flash in &mut self.flashes {
            flash.1 -= dt;
        }
        self.flashes.retain(|f| f.1 > 0.0);

        let mut input = TouchInput::default();
        let keys = keypad();
        for p in pointers {
            let in_arena = (0.0..ARENA_W).contains(&p.pos.x);
            match p.phase {
                Phase::Started if in_arena => {
                    input.arena_taps += 1;
                    if self.stick.is_none() {
                        self.stick = Some(Stick {
                            id: p.id,
                            base: p.pos,
                            knob: p.pos,
                        });
                    }
                }
                Phase::Started => {
                    let pressed = keys.iter().enumerate().find_map(|(i, k)| {
                        let key = match k.label {
                            Label::Char(c) => Key::Char(c),
                            Label::Backspace => Key::Backspace,
                            Label::Unused(_) => return None,
                        };
                        key_rect(k).contains(p.pos).then_some((i, key))
                    });
                    if let Some((i, key)) = pressed {
                        input.keys.push(key);
                        self.flashes.push((i, PRESS_FLASH_SECONDS));
                    }
                    for button in [Button::Pause, Button::Music] {
                        if button_rect(button).contains(p.pos) {
                            input.buttons.push(button);
                        }
                    }
                }
                Phase::Moved | Phase::Stationary => match &mut self.stick {
                    Some(stick) if stick.id == p.id => {
                        let offset = (p.pos - stick.base).clamp_length_max(STICK_RADIUS);
                        stick.knob = stick.base + offset;
                    }
                    // A finger moving in the arena whose start was missed
                    // (say, it landed as the game began) takes the stick.
                    None if in_arena => {
                        self.stick = Some(Stick {
                            id: p.id,
                            base: p.pos,
                            knob: p.pos,
                        });
                    }
                    _ => {}
                },
                Phase::Ended | Phase::Cancelled => {
                    if self.stick.as_ref().is_some_and(|s| s.id == p.id) {
                        self.stick = None;
                    }
                }
            }
        }

        if let Some(stick) = &self.stick {
            let push = (stick.knob - stick.base) / STICK_RADIUS;
            if push.length() > STICK_DEAD_ZONE {
                input.movement = push;
            }
        }
        input
    }

    /// Where the joystick is, as its base and knob, while a finger is on it.
    pub fn stick(&self) -> Option<(Vec2, Vec2)> {
        self.stick.as_ref().map(|s| (s.base, s.knob))
    }

    /// The keys recently pressed, as positions in `keypad()`.
    pub fn flashing(&self) -> impl Iterator<Item = usize> + '_ {
        self.flashes.iter().map(|&(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keypad_has_every_answer_character_once() {
        let chars: Vec<char> = keypad()
            .iter()
            .filter_map(|k| match k.label {
                Label::Char(c) => Some(c),
                Label::Backspace | Label::Unused(_) => None,
            })
            .collect();
        let expected: Vec<char> = "0123456789hjklmprstvaeiouyäö".chars().collect();
        assert_eq!(chars.len(), expected.len());
        for c in expected {
            assert!(is_answer_char(c), "{c}");
            assert_eq!(chars.iter().filter(|&&k| k == c).count(), 1, "{c}");
        }
    }

    #[test]
    fn the_keys_follow_a_finnish_keyboard_row_by_row() {
        let qwerty = ["1234567890", "qwertyuiopå", "asdfghjklöä", "zxcvbnm,.-"];
        let keys = keypad();
        for (row, order) in qwerty.iter().enumerate() {
            // Positions along the row, left panel first.
            let mut placed: Vec<(f32, char)> = keys
                .iter()
                .filter(|k| k.row == row as f32)
                .filter_map(|k| match k.label {
                    Label::Char(c) | Label::Unused(c) => {
                        let x = if k.side == Side::Left {
                            k.col
                        } else {
                            100.0 + k.col
                        };
                        Some((x, c))
                    }
                    Label::Backspace => None,
                })
                .collect();
            placed.sort_by(|a, b| a.0.total_cmp(&b.0));
            // Every key of the row, in order, as on a real keyboard.
            let row_keys: String = placed.iter().map(|(_, c)| c).collect();
            assert_eq!(&row_keys, order, "row {row}");
        }
    }

    fn touch(id: u64, pos: Vec2, phase: Phase) -> Pointer {
        Pointer { id, pos, phase }
    }

    #[test]
    fn tapping_a_key_types_it() {
        let mut controls = TouchControls::default();
        let keys = keypad();
        let k = keys
            .iter()
            .find(|k| matches!(k.label, Label::Char('k')))
            .unwrap();
        let input = controls.update(&[touch(1, key_rect(k).center(), Phase::Started)], 0.016);
        assert_eq!(input.keys, vec![Key::Char('k')]);
    }

    #[test]
    fn greyed_out_keys_do_nothing() {
        let mut controls = TouchControls::default();
        let keys = keypad();
        let q = keys
            .iter()
            .find(|k| matches!(k.label, Label::Unused('q')))
            .unwrap();
        let input = controls.update(&[touch(1, key_rect(q).center(), Phase::Started)], 0.016);
        assert!(input.keys.is_empty());
        assert!(controls.flashing().next().is_none());
    }

    #[test]
    fn the_joystick_follows_a_finger_dragged_in_the_arena() {
        let mut controls = TouchControls::default();
        let start = vec2(200.0, 300.0);
        controls.update(&[touch(7, start, Phase::Started)], 0.016);
        let input = controls.update(&[touch(7, start + vec2(30.0, 0.0), Phase::Moved)], 0.016);
        assert!(
            (input.movement - vec2(0.5, 0.0)).length() < 0.001,
            "{}",
            input.movement
        );
        // Pushing past the edge is full speed, not faster.
        let input = controls.update(&[touch(7, start + vec2(0.0, -500.0), Phase::Moved)], 0.016);
        assert!((input.movement.length() - 1.0).abs() < 0.001);
        let input = controls.update(&[touch(7, start, Phase::Ended)], 0.016);
        assert_eq!(input.movement, Vec2::ZERO);
    }

    #[test]
    fn a_finger_whose_start_was_missed_still_steers() {
        let mut controls = TouchControls::default();
        controls.update(&[touch(3, vec2(200.0, 300.0), Phase::Moved)], 0.016);
        let input = controls.update(&[touch(3, vec2(260.0, 300.0), Phase::Moved)], 0.016);
        assert!(input.movement.x > 0.9, "{}", input.movement);
    }

    #[test]
    fn a_second_finger_can_type_while_the_first_steers() {
        let mut controls = TouchControls::default();
        controls.update(&[touch(1, vec2(200.0, 300.0), Phase::Started)], 0.016);
        let keys = keypad();
        let seven = keys
            .iter()
            .find(|k| matches!(k.label, Label::Char('7')))
            .unwrap();
        let input = controls.update(
            &[
                touch(1, vec2(260.0, 300.0), Phase::Moved),
                touch(2, key_rect(seven).center(), Phase::Started),
            ],
            0.016,
        );
        assert_eq!(input.keys, vec![Key::Char('7')]);
        assert!(input.movement.x > 0.9);
    }

    #[test]
    fn keys_and_buttons_fit_the_panel_without_overlapping() {
        let mut rects: Vec<Rect> = keypad().iter().map(key_rect).collect();
        rects.push(button_rect(Button::Pause));
        rects.push(button_rect(Button::Music));
        for (i, r) in rects.iter().enumerate() {
            let in_left = r.x >= -LEFT_W && r.x + r.w <= 0.0;
            let in_right = r.x >= ARENA_W && r.x + r.w <= ARENA_W + RIGHT_W;
            assert!(in_left || in_right, "{r:?}");
            assert!(r.y >= 0.0 && r.y + r.h <= ARENA_H, "{r:?}");
            for other in &rects[i + 1..] {
                assert!(!r.overlaps(other), "{r:?} {other:?}");
            }
        }
    }
}
