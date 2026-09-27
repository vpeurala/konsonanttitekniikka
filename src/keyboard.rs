//! Typing input that ignores the operating system's keyboard layout.
//!
//! Keys are read by their physical position and mapped as on a Finnish
//! keyboard, so Ä and Ö work even when the OS layout is, say, US English.

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::miniquad::{EventHandler, KeyCode, KeyMods};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Backspace,
}

pub struct Keyboard {
    subscriber: usize,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            subscriber: register_input_subscriber(),
        }
    }

    /// The keys typed since the previous call, in order. Must be called
    /// every frame.
    pub fn typed(&self) -> Vec<Key> {
        let mut collector = Collector(Vec::new());
        repeat_all_miniquad_input(&mut collector, self.subscriber);
        collector.0
    }
}

struct Collector(Vec<Key>);

impl EventHandler for Collector {
    fn update(&mut self) {}
    fn draw(&mut self) {}

    fn key_down_event(&mut self, keycode: KeyCode, mods: KeyMods, repeat: bool) {
        if mods.ctrl || mods.logo || mods.alt {
            return;
        }
        if keycode == KeyCode::Backspace {
            self.0.push(Key::Backspace);
        } else if !repeat && let Some(c) = finnish_char(keycode) {
            self.0.push(Key::Char(c));
        }
    }
}

/// The lowercase character a physical key produces on a Finnish keyboard.
pub fn finnish_char(keycode: KeyCode) -> Option<char> {
    use KeyCode::*;
    let c = match keycode {
        A => 'a',
        B => 'b',
        C => 'c',
        D => 'd',
        E => 'e',
        F => 'f',
        G => 'g',
        H => 'h',
        I => 'i',
        J => 'j',
        K => 'k',
        L => 'l',
        M => 'm',
        N => 'n',
        O => 'o',
        P => 'p',
        Q => 'q',
        R => 'r',
        S => 's',
        T => 't',
        U => 'u',
        V => 'v',
        W => 'w',
        X => 'x',
        Y => 'y',
        Z => 'z',
        // Ö and Ä sit where US keyboards have ; and '.
        Semicolon => 'ö',
        Apostrophe => 'ä',
        Key0 | Kp0 => '0',
        Key1 | Kp1 => '1',
        Key2 | Kp2 => '2',
        Key3 | Kp3 => '3',
        Key4 | Kp4 => '4',
        Key5 | Kp5 => '5',
        Key6 | Kp6 => '6',
        Key7 | Kp7 => '7',
        Key8 | Kp8 => '8',
        Key9 | Kp9 => '9',
        _ => return None,
    };
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_finnish_letters_by_position() {
        assert_eq!(finnish_char(KeyCode::Semicolon), Some('ö'));
        assert_eq!(finnish_char(KeyCode::Apostrophe), Some('ä'));
        assert_eq!(finnish_char(KeyCode::K), Some('k'));
    }

    #[test]
    fn maps_number_row_and_keypad_digits() {
        assert_eq!(finnish_char(KeyCode::Key7), Some('7'));
        assert_eq!(finnish_char(KeyCode::Kp7), Some('7'));
    }

    #[test]
    fn ignores_other_keys() {
        assert_eq!(finnish_char(KeyCode::Left), None);
        assert_eq!(finnish_char(KeyCode::Space), None);
    }
}
