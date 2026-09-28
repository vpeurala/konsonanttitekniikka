//! Touch controls: a floating joystick in the arena for moving, and a
//! keypad beside the arena with exactly the keys the game needs. On a
//! computer, the mouse stands in for a finger, which makes the controls
//! testable without a phone.

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::miniquad::{self, EventHandler};
use macroquad::prelude::*;

use crate::fonts::{self, Style};
use crate::keyboard::Key;
use crate::pairs::{DIGIT_CONSONANTS, VOWELS};
use crate::view::{ARENA_H, ARENA_W, View};

/// A key's size, and the gaps around keys and panel edges.
const KEY: f32 = 60.0;
const GAP: f32 = 6.0;
const PAD: f32 = 10.0;

/// The keypad is split like a tablet keyboard: the left hand's keys on a
/// panel left of the arena, the right hand's on a panel right of it.
const LEFT_COLUMNS: f32 = 5.0;
const RIGHT_COLUMNS: f32 = 6.0;
const LEFT_W: f32 = 2.0 * PAD + LEFT_COLUMNS * KEY + (LEFT_COLUMNS - 1.0) * GAP;
const RIGHT_W: f32 = 2.0 * PAD + RIGHT_COLUMNS * KEY + (RIGHT_COLUMNS - 1.0) * GAP;

/// Everything the game shows, in virtual units: the arena, with the keypad
/// panels on both sides when using touch controls.
pub fn content_rect(touch: bool) -> Rect {
    if touch {
        Rect::new(-LEFT_W, 0.0, LEFT_W + ARENA_W + RIGHT_W, ARENA_H)
    } else {
        Rect::new(0.0, 0.0, ARENA_W, ARENA_H)
    }
}

/// Whether to show touch controls: always on phones, and on a computer
/// when the `LUKULOITSU_TOUCH` environment variable is set.
pub fn enabled() -> bool {
    cfg!(any(target_os = "ios", target_os = "android"))
        || std::env::var_os("LUKULOITSU_TOUCH").is_some()
}

/// A finger or the mouse, in virtual units.
#[derive(Debug, Clone, Copy)]
pub struct Pointer {
    pub id: u64,
    pub pos: Vec2,
    pub phase: TouchPhase,
}

const MOUSE_ID: u64 = u64::MAX;

/// Reads touches as the ordered stream of events they arrive in, so a
/// quick tap that starts and ends within one frame still counts.
pub struct TouchReader {
    subscriber: usize,
}

struct TouchCollector(Vec<(TouchPhase, u64, Vec2)>);

impl EventHandler for TouchCollector {
    fn update(&mut self) {}
    fn draw(&mut self) {}

    fn touch_event(&mut self, phase: miniquad::TouchPhase, id: u64, x: f32, y: f32) {
        self.0.push((phase.into(), id, vec2(x, y)));
    }
}

impl TouchReader {
    pub fn new() -> Self {
        TouchReader {
            subscriber: register_input_subscriber(),
        }
    }

    /// The touches since the previous call, in order, and the mouse acting
    /// as one more finger. Must be called every frame.
    pub fn pointers(&self, view: &View) -> Vec<Pointer> {
        let mut collector = TouchCollector(Vec::new());
        repeat_all_miniquad_input(&mut collector, self.subscriber);
        // Touches arrive in physical pixels; the view works in points.
        let dpi = miniquad::window::dpi_scale();
        let mut pointers: Vec<Pointer> = collector
            .0
            .into_iter()
            .map(|(phase, id, pos)| Pointer {
                id,
                pos: view.to_virtual(pos / dpi),
                phase,
            })
            .collect();
        pointers.extend(mouse_pointer(view));
        pointers
    }
}

/// The mouse as a finger, for trying touch controls on a computer.
fn mouse_pointer(view: &View) -> Option<Pointer> {
    let phase = if is_mouse_button_pressed(MouseButton::Left) {
        TouchPhase::Started
    } else if is_mouse_button_released(MouseButton::Left) {
        TouchPhase::Ended
    } else if is_mouse_button_down(MouseButton::Left) {
        TouchPhase::Moved
    } else {
        return None;
    };
    Some(Pointer {
        id: MOUSE_ID,
        pos: view.to_virtual(mouse_position().into()),
        phase,
    })
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

enum Label {
    Char(char),
    Backspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

struct KeySpec {
    label: Label,
    side: Side,
    /// Column within its panel, and row: 0 for the digits, then the
    /// keyboard's Q, A and Z rows.
    col: f32,
    row: f32,
}

/// The keys in their places on a Finnish QWERTY keyboard, split between
/// the hands, with gaps (`.`) for letters the game never uses.
const LEFT_ROWS: [&str; 4] = ["12345", "..ert", "as...", "...v."];
const RIGHT_ROWS: [&str; 4] = ["67890<", "yuiop.", "hjklöä", ".m...."];

fn keypad() -> Vec<KeySpec> {
    let mut keys = Vec::new();
    for (side, rows) in [(Side::Left, LEFT_ROWS), (Side::Right, RIGHT_ROWS)] {
        for (row, chars) in rows.iter().enumerate() {
            for (col, c) in chars.chars().enumerate() {
                let label = match c {
                    '.' => continue,
                    // Backspace sits right of 0, as on a real keyboard.
                    '<' => Label::Backspace,
                    c => Label::Char(c),
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

fn key_rect(key: &KeySpec) -> Rect {
    Rect::new(
        panel_left(key.side) + key.col * (KEY + GAP),
        KEYS_TOP + key.row * (KEY + GAP),
        KEY,
        KEY,
    )
}

/// Pause in the top left corner, music in the top right.
fn button_rect(button: Button) -> Rect {
    let w = 2.0 * KEY + GAP;
    let x = match button {
        Button::Pause => panel_left(Side::Left),
        Button::Music => ARENA_W + RIGHT_W - PAD - w,
    };
    Rect::new(x, PAD, w, KEY)
}

const STICK_RADIUS: f32 = 60.0;
/// Pushes shorter than this fraction of the radius don't move.
const STICK_DEAD_ZONE: f32 = 0.15;
/// Where the joystick is hinted at before the first touch.
const STICK_HOME: Vec2 = vec2(110.0, ARENA_H - 110.0);
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
                TouchPhase::Started if in_arena => {
                    input.arena_taps += 1;
                    if self.stick.is_none() {
                        self.stick = Some(Stick {
                            id: p.id,
                            base: p.pos,
                            knob: p.pos,
                        });
                    }
                }
                TouchPhase::Started => {
                    if let Some(i) = keys.iter().position(|k| key_rect(k).contains(p.pos)) {
                        input.keys.push(match keys[i].label {
                            Label::Char(c) => Key::Char(c),
                            Label::Backspace => Key::Backspace,
                        });
                        self.flashes.push((i, PRESS_FLASH_SECONDS));
                    }
                    for button in [Button::Pause, Button::Music] {
                        if button_rect(button).contains(p.pos) {
                            input.buttons.push(button);
                        }
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => match &mut self.stick {
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
                TouchPhase::Ended | TouchPhase::Cancelled => {
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

    /// Draws the keypad panels beside the arena, and the joystick over the
    /// arena if `show_stick` (practice mode has nothing to move).
    pub fn draw(&self, music_on: bool, paused: bool, show_stick: bool) {
        if show_stick {
            self.draw_stick();
        }
        let panel = Color::new(0.06, 0.06, 0.09, 1.0);
        let edge = Color::new(0.3, 0.3, 0.4, 1.0);
        draw_rectangle(-LEFT_W, 0.0, LEFT_W, ARENA_H, panel);
        draw_rectangle(ARENA_W, 0.0, RIGHT_W, ARENA_H, panel);
        draw_line(0.0, 0.0, 0.0, ARENA_H, 2.0, edge);
        draw_line(ARENA_W, 0.0, ARENA_W, ARENA_H, 2.0, edge);

        draw_button(button_rect(Button::Pause), |r| {
            let c = r.center();
            if paused {
                draw_triangle(
                    c + vec2(-8.0, -12.0),
                    c + vec2(-8.0, 12.0),
                    c + vec2(12.0, 0.0),
                    WHITE,
                );
            } else {
                draw_rectangle(c.x - 10.0, c.y - 12.0, 7.0, 24.0, WHITE);
                draw_rectangle(c.x + 3.0, c.y - 12.0, 7.0, 24.0, WHITE);
            }
        });
        draw_button(button_rect(Button::Music), |r| {
            let c = r.center();
            let color = if music_on { WHITE } else { GRAY };
            // An eighth note.
            draw_circle(c.x - 5.0, c.y + 9.0, 7.0, color);
            draw_line(c.x + 1.5, c.y + 9.0, c.x + 1.5, c.y - 14.0, 3.0, color);
            draw_line(c.x + 1.5, c.y - 14.0, c.x + 11.0, c.y - 7.0, 3.0, color);
            if !music_on {
                draw_line(c.x - 16.0, c.y - 16.0, c.x + 16.0, c.y + 16.0, 3.0, RED);
            }
        });

        let keys = keypad();
        for (i, key) in keys.iter().enumerate() {
            let flashing = self.flashes.iter().any(|f| f.0 == i);
            draw_key(key, key_rect(key), flashing, 30);
        }
        // Pressed keys pop up enlarged above the finger that covers them.
        for &(i, _) in &self.flashes {
            let rect = key_rect(&keys[i]);
            let size = KEY * 1.5;
            let bubble = Rect::new(
                rect.center().x - size / 2.0,
                (rect.y - size - GAP).max(0.0),
                size,
                size,
            );
            draw_key(&keys[i], bubble, true, 48);
        }
    }

    fn draw_stick(&self) {
        let (base, knob, alpha) = match &self.stick {
            Some(stick) => (stick.base, stick.knob, 0.5),
            None => (STICK_HOME, STICK_HOME, 0.18),
        };
        draw_circle(
            base.x,
            base.y,
            STICK_RADIUS,
            Color::new(1.0, 1.0, 1.0, alpha * 0.3),
        );
        draw_circle_lines(
            base.x,
            base.y,
            STICK_RADIUS,
            3.0,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
        draw_circle(
            knob.x,
            knob.y,
            STICK_RADIUS * 0.45,
            Color::new(1.0, 1.0, 1.0, alpha),
        );
    }
}

/// A key's colors, by kind: digits blue like the number slot, consonants
/// violet like the word slot, vowels amber, so the eye can search one
/// group at a time. Returns (fill, accent).
fn key_colors(label: &Label) -> (Color, Color) {
    match label {
        Label::Char(c) if c.is_ascii_digit() => (Color::new(0.1, 0.2, 0.32, 1.0), SKYBLUE),
        Label::Char(c) if VOWELS.contains(c) => (
            Color::new(0.28, 0.2, 0.06, 1.0),
            Color::new(0.95, 0.72, 0.25, 1.0),
        ),
        Label::Char(_) => (Color::new(0.2, 0.12, 0.3, 1.0), VIOLET),
        Label::Backspace => (Color::new(0.25, 0.25, 0.3, 1.0), LIGHTGRAY),
    }
}

/// Draws `key` filling `rect`, lit up if `pressed`.
fn draw_key(key: &KeySpec, rect: Rect, pressed: bool, font_size: u16) {
    let (fill, accent) = key_colors(&key.label);
    let fill = if pressed { accent } else { fill };
    let text_color = if pressed { BLACK } else { WHITE };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, accent);
    let c = rect.center();
    let scale = rect.w / KEY;
    match key.label {
        Label::Char(ch) => {
            let text = ch.to_uppercase().to_string();
            fonts::draw_centered(&text, c.x, c.y, font_size, text_color, Style::Bold);
            // Consonants show the digit they stand for, in the corner.
            if let Some(digit) = DIGIT_CONSONANTS.iter().position(|&d| d == ch) {
                let badge = if pressed { BLACK } else { SKYBLUE };
                fonts::draw_centered(
                    &digit.to_string(),
                    rect.x + rect.w - 10.0 * scale,
                    rect.y + 11.0 * scale,
                    (15.0 * scale) as u16,
                    badge,
                    Style::Bold,
                );
            }
        }
        Label::Backspace => {
            // A left-pointing arrow with a cross.
            let s = scale * 0.8;
            draw_triangle(
                c + vec2(-24.0, 0.0) * s,
                c + vec2(-10.0, -13.0) * s,
                c + vec2(-10.0, 13.0) * s,
                text_color,
            );
            draw_rectangle(
                c.x - 10.0 * s,
                c.y - 13.0 * s,
                32.0 * s,
                26.0 * s,
                text_color,
            );
            draw_line(
                c.x + 1.0 * s,
                c.y - 7.0 * s,
                c.x + 15.0 * s,
                c.y + 7.0 * s,
                3.0,
                fill,
            );
            draw_line(
                c.x + 15.0 * s,
                c.y - 7.0 * s,
                c.x + 1.0 * s,
                c.y + 7.0 * s,
                3.0,
                fill,
            );
        }
    }
}

fn draw_button(rect: Rect, icon: impl FnOnce(Rect)) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.2, 0.2, 0.26, 1.0),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, LIGHTGRAY);
    icon(rect);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairs::is_answer_char;

    #[test]
    fn the_keypad_has_every_answer_character_once() {
        let chars: Vec<char> = keypad()
            .iter()
            .filter_map(|k| match k.label {
                Label::Char(c) => Some(c),
                Label::Backspace => None,
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
    fn letters_keep_their_qwerty_order_within_each_row() {
        let qwerty = ["1234567890", "qwertyuiopå", "asdfghjklöä", "zxcvbnm"];
        let keys = keypad();
        for (row, order) in qwerty.iter().enumerate() {
            // Positions along the row, left panel first.
            let mut placed: Vec<(f32, char)> = keys
                .iter()
                .filter(|k| k.row == row as f32)
                .filter_map(|k| match k.label {
                    Label::Char(c) => {
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
            let indexes: Vec<usize> = placed
                .iter()
                .map(|(_, c)| order.chars().position(|o| o == *c).unwrap())
                .collect();
            assert!(indexes.is_sorted(), "row {row}: {placed:?}");
        }
    }

    fn touch(id: u64, pos: Vec2, phase: TouchPhase) -> Pointer {
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
        let input = controls.update(
            &[touch(1, key_rect(k).center(), TouchPhase::Started)],
            0.016,
        );
        assert_eq!(input.keys, vec![Key::Char('k')]);
    }

    #[test]
    fn the_joystick_follows_a_finger_dragged_in_the_arena() {
        let mut controls = TouchControls::default();
        let start = vec2(200.0, 300.0);
        controls.update(&[touch(7, start, TouchPhase::Started)], 0.016);
        let input = controls.update(
            &[touch(7, start + vec2(30.0, 0.0), TouchPhase::Moved)],
            0.016,
        );
        assert!(
            (input.movement - vec2(0.5, 0.0)).length() < 0.001,
            "{}",
            input.movement
        );
        // Pushing past the edge is full speed, not faster.
        let input = controls.update(
            &[touch(7, start + vec2(0.0, -500.0), TouchPhase::Moved)],
            0.016,
        );
        assert!((input.movement.length() - 1.0).abs() < 0.001);
        let input = controls.update(&[touch(7, start, TouchPhase::Ended)], 0.016);
        assert_eq!(input.movement, Vec2::ZERO);
    }

    #[test]
    fn a_finger_whose_start_was_missed_still_steers() {
        let mut controls = TouchControls::default();
        controls.update(&[touch(3, vec2(200.0, 300.0), TouchPhase::Moved)], 0.016);
        let input = controls.update(&[touch(3, vec2(260.0, 300.0), TouchPhase::Moved)], 0.016);
        assert!(input.movement.x > 0.9, "{}", input.movement);
    }

    #[test]
    fn a_second_finger_can_type_while_the_first_steers() {
        let mut controls = TouchControls::default();
        controls.update(&[touch(1, vec2(200.0, 300.0), TouchPhase::Started)], 0.016);
        let keys = keypad();
        let seven = keys
            .iter()
            .find(|k| matches!(k.label, Label::Char('7')))
            .unwrap();
        let input = controls.update(
            &[
                touch(1, vec2(260.0, 300.0), TouchPhase::Moved),
                touch(2, key_rect(seven).center(), TouchPhase::Started),
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
