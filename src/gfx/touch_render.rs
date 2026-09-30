//! Drawing the touch controls: the keypad panels beside the arena, the
//! buttons and the joystick. What they do is `lukuloitsu_core::touch`.

use macroquad::prelude::*;

use crate::gfx::fonts::{self, Style};
use crate::gfx::view::{ARENA_H, ARENA_W};
use crate::pairs::{DIGIT_CONSONANTS, VOWELS};
use lukuloitsu_core::geometry::Rect;
use lukuloitsu_core::touch::{
    Button, GAP, KEY, KeySpec, LEFT_W, Label, RIGHT_W, STICK_HOME, STICK_RADIUS, TouchControls,
    button_rect, key_rect, keypad,
};

/// Draws the keypad panels beside the arena, and the joystick over the
/// arena if `show_stick` (practice mode has nothing to move).
pub fn draw(controls: &TouchControls, music_on: bool, paused: bool, show_stick: bool) {
    if show_stick {
        draw_stick(controls);
    }
    let panel = Color::new(0.06, 0.06, 0.09, 1.0);
    let edge = Color::new(0.3, 0.3, 0.4, 1.0);
    draw_rectangle(-LEFT_W, 0.0, LEFT_W, ARENA_H, panel);
    draw_rectangle(ARENA_W, 0.0, RIGHT_W, ARENA_H, panel);
    draw_line(0.0, 0.0, 0.0, ARENA_H, 2.0, edge);
    draw_line(ARENA_W, 0.0, ARENA_W, ARENA_H, 2.0, edge);

    draw_button(button_rect(Button::Pause), |r| {
        draw_pause_icon(r.center(), paused)
    });
    draw_button(button_rect(Button::Music), |r| {
        draw_speaker_icon(r.center(), music_on);
    });

    let keys = keypad();
    for (i, key) in keys.iter().enumerate() {
        let flashing = controls.flashing().any(|f| f == i);
        draw_key(key, key_rect(key), flashing, 30);
    }
    // Pressed keys pop up enlarged above the finger that covers them.
    for i in controls.flashing() {
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

fn draw_stick(controls: &TouchControls) {
    let (base, knob, alpha) = match controls.stick() {
        Some((base, knob)) => (base, knob, 0.5),
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

/// Play when paused, pause when playing.
fn draw_pause_icon(c: Vec2, paused: bool) {
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
}

/// A loudspeaker, crossed out when the sound is off.
fn draw_speaker_icon(c: Vec2, music_on: bool) {
    let color = if music_on { WHITE } else { GRAY };
    // A loudspeaker: a box, a cone and two waves.
    draw_rectangle(c.x - 15.0, c.y - 5.0, 8.0, 10.0, color);
    draw_triangle(
        vec2(c.x - 8.0, c.y - 5.0),
        vec2(c.x - 8.0, c.y + 5.0),
        vec2(c.x + 1.0, c.y + 14.0),
        color,
    );
    draw_triangle(
        vec2(c.x - 8.0, c.y - 5.0),
        vec2(c.x + 1.0, c.y - 14.0),
        vec2(c.x + 1.0, c.y + 14.0),
        color,
    );
    for radius in [8.0, 14.0] {
        draw_wave(vec2(c.x + 1.0, c.y), radius, color);
    }
    if !music_on {
        draw_line(c.x - 16.0, c.y - 16.0, c.x + 16.0, c.y + 16.0, 3.0, RED);
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
        Label::Unused(_) => (
            Color::new(0.1, 0.1, 0.12, 1.0),
            Color::new(0.2, 0.2, 0.24, 1.0),
        ),
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
        Label::Unused(ch) => {
            let text = ch.to_uppercase().to_string();
            let dim = Color::new(0.32, 0.32, 0.37, 1.0);
            fonts::draw_centered(&text, c.x, c.y, font_size, dim, Style::Bold);
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

/// An arc of a sound wave, `radius` from `centre`, opening to the right.
fn draw_wave(centre: Vec2, radius: f32, color: Color) {
    const STEPS: usize = 5;
    const SPREAD: f32 = 0.9;
    let point = |i: usize| {
        let angle = -SPREAD + 2.0 * SPREAD * i as f32 / STEPS as f32;
        centre + vec2(angle.cos(), angle.sin()) * radius
    };
    for i in 0..STEPS {
        let (a, b) = (point(i), point(i + 1));
        draw_line(a.x, a.y, b.x, b.y, 2.5, color);
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
