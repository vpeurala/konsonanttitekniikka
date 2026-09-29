//! The progress map: every pair coloured by how well the player knows it,
//! with counts of pairs learned and stars earned.

use macroquad::prelude::*;

use crate::fonts::draw_centered_text;
use crate::fonts::{self, Style};
use crate::frame::Frame;
use crate::memory::Memory;
use crate::pairs::{PAIRS, Pair};
use crate::save::SaveData;
use crate::sprites::draw_star;
use crate::title::pair_rows;
use crate::touch::Pointer;
use crate::view::{ARENA_H, ARENA_W};

/// A pair counts as learned once its difficulty is below this.
const LEARNED: f32 = 0.25;
/// ...and as nearly learned below this.
const NEARLY: f32 = 0.5;

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const UNSEEN: Color = Color::new(0.2, 0.2, 0.26, 1.0);
const RED: Color = Color::new(0.8, 0.22, 0.2, 1.0);
const YELLOW: Color = Color::new(0.92, 0.75, 0.2, 1.0);
const GREEN: Color = Color::new(0.25, 0.7, 0.3, 1.0);

const GRID_TOP: f32 = 100.0;
const ROW_H: f32 = 40.0;
const MARGIN: f32 = 40.0;

/// A colour from red (unknown) through yellow to green (learned).
pub fn mastery_color(difficulty: f32) -> Color {
    let d = difficulty.clamp(0.0, 1.0);
    let lerp = |a: Color, b: Color, t: f32| {
        Color::new(
            a.r + (b.r - a.r) * t,
            a.g + (b.g - a.g) * t,
            a.b + (b.b - a.b) * t,
            1.0,
        )
    };
    if d < NEARLY {
        lerp(GREEN, YELLOW, d / NEARLY)
    } else {
        lerp(YELLOW, RED, (d - NEARLY) / (1.0 - NEARLY))
    }
}

/// How many pairs count as learned.
pub fn learned_count(memory: &Memory) -> usize {
    PAIRS
        .iter()
        .filter(|p| memory.record(p).is_some_and(|r| r.difficulty < LEARNED))
        .count()
}

#[derive(Default)]
pub struct ProgressScreen;

impl ProgressScreen {
    /// Returns true when the player wants to go back to the title.
    pub fn update(&mut self, frame: &Frame, pointers: &[Pointer]) -> bool {
        let tapped = pointers.iter().any(|p| p.phase == TouchPhase::Ended);
        tapped
            || frame.pressed(KeyCode::Escape)
            || frame.pressed(KeyCode::Enter)
            || frame.pressed(KeyCode::Space)
            || frame.pressed(KeyCode::Backspace)
    }

    pub fn draw(&self, data: &SaveData, touch: bool) {
        clear_background(BACKGROUND);
        let memory = data.memory();
        let cx = ARENA_W / 2.0;
        fonts::draw_centered("Edistyminen", cx, 36.0, 44, GOLD, Style::Heading);

        let stars: u32 = data.stars.values().map(|&s| u32::from(s)).sum();
        let stats = format!(
            "Opittu {} / {}     Päiviä putkeen {}",
            learned_count(&memory),
            PAIRS.len(),
            data.streak
        );
        // The total of stars earned follows the other numbers, and the
        // whole line is centred.
        let stars_text = format!("{stars}");
        let stats_w = fonts::measure(&stats, Style::Body, 22).width;
        let stars_w = fonts::measure(&stars_text, Style::Body, 22).width;
        let gap = 40.0;
        let left = cx - (stats_w + gap + 16.0 + stars_w) / 2.0;
        fonts::draw(&stats, left, 82.0, 22, WHITE, Style::Body);
        let star_x = left + stats_w + gap;
        draw_star(vec2(star_x, 74.0), 11.0, true);
        fonts::draw(&stars_text, star_x + 16.0, 82.0, 22, WHITE, Style::Body);

        let cell = (ARENA_W - 2.0 * MARGIN) / 10.0;
        for (i, row) in pair_rows().iter().enumerate() {
            let y = GRID_TOP + i as f32 * ROW_H;
            for pair in row {
                let column = pair
                    .number
                    .chars()
                    .last()
                    .and_then(|c| c.to_digit(10))
                    .unwrap_or(0) as f32;
                let x = MARGIN + column * cell;
                draw_cell(
                    pair,
                    &memory,
                    Rect::new(x + 2.0, y + 2.0, cell - 4.0, ROW_H - 4.0),
                );
            }
        }

        draw_legend(GRID_TOP + 11.0 * ROW_H + 22.0);
        let back = if touch {
            "Napauta palataksesi"
        } else {
            "Esc tai Enter: takaisin"
        };
        draw_centered_text(
            back,
            cx,
            ARENA_H - 16.0,
            18,
            Color::new(0.6, 0.6, 0.65, 1.0),
        );
    }
}

fn draw_cell(pair: &Pair, memory: &Memory, rect: Rect) {
    let (fill, text) = match memory.record(pair) {
        Some(record) => (mastery_color(record.difficulty), BLACK),
        None => (UNSEEN, Color::new(0.6, 0.6, 0.65, 1.0)),
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    let c = rect.center();
    fonts::draw_centered(pair.number, c.x, c.y - 7.0, 16, text, Style::Bold);
    fonts::draw_centered(
        &pair.word.to_uppercase(),
        c.x,
        c.y + 9.0,
        11,
        text,
        Style::Bold,
    );
}

fn draw_legend(y: f32) {
    let items = [
        (UNSEEN, "Ei vielä nähty"),
        (RED, "Harjoittele"),
        (YELLOW, "Melkein"),
        (GREEN, "Osaat"),
    ];
    let mut x = MARGIN;
    for (color, label) in items {
        draw_rectangle(x, y - 8.0, 16.0, 16.0, color);
        fonts::draw(label, x + 22.0, y + 6.0, 18, WHITE, Style::Body);
        x += 26.0 + fonts::measure(label, Style::Body, 18).width + 26.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mastery_colours_run_from_green_through_yellow_to_red() {
        let close = |a: Color, b: Color| {
            (a.r - b.r).abs() < 0.001 && (a.g - b.g).abs() < 0.001 && (a.b - b.b).abs() < 0.001
        };
        assert!(close(mastery_color(0.0), GREEN));
        assert!(close(mastery_color(NEARLY), YELLOW));
        assert!(close(mastery_color(1.0), RED));
    }

    #[test]
    fn only_well_known_pairs_count_as_learned() {
        let mut memory = Memory::default();
        assert_eq!(learned_count(&memory), 0);
        for _ in 0..10 {
            memory.record_answer(PAIRS[0], 1.0, false, 0.0);
        }
        memory.record_miss(PAIRS[1], 0.0);
        assert_eq!(learned_count(&memory), 1);
    }
}
