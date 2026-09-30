//! The progress map: every pair coloured by how well the player knows it,
//! with counts of pairs learned and stars earned.

use macroquad::prelude::*;

use crate::gfx::fonts::draw_centered_text;
use crate::gfx::fonts::{self, Style};
use crate::gfx::sprites::draw_star;
use crate::gfx::view::{ARENA_H, ARENA_W};
use crate::input::frame::Frame;
use crate::input::touch::Pointer;
use crate::memory::{LEARNED, Memory, learned_count};
use crate::pairs::{PAIRS, Pair};
use crate::platform::save::SaveData;
use crate::screens::title::pair_rows;

/// The colour is pure yellow here; a pair is "nearly" learned around it.
const NEARLY: f32 = 0.5;
/// Halfway between yellow and red: past this a pair still needs practice.
const PRACTISE: f32 = 0.75;

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

/// How well a pair is known as a number of dots, so the map can be read
/// without telling colours apart: 3 for learned, 2 for nearly, 1 for a pair
/// that needs practice.
pub fn mastery_tier(difficulty: f32) -> u8 {
    if difficulty < LEARNED {
        3
    } else if difficulty < PRACTISE {
        2
    } else {
        1
    }
}

/// Returns true when the player wants to go back to the title.
pub fn update(frame: &Frame, pointers: &[Pointer]) -> bool {
    let tapped = pointers.iter().any(|p| p.phase == TouchPhase::Ended);
    tapped
        || frame.pressed(KeyCode::Escape)
        || frame.pressed(KeyCode::Enter)
        || frame.pressed(KeyCode::Space)
        || frame.pressed(KeyCode::Backspace)
}

pub fn draw(data: &SaveData, memory: &Memory, touch: bool) {
    clear_background(BACKGROUND);
    let cx = ARENA_W / 2.0;
    fonts::draw_centered("Edistyminen", cx, 36.0, 44, GOLD, Style::Heading);

    let stars: u32 = data.stars.values().map(|&s| u32::from(s)).sum();
    let stats = format!(
        "Opittu {} / {}     Päiviä putkeen {}",
        learned_count(memory),
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

    for (i, row) in pair_rows().iter().enumerate() {
        for pair in row {
            draw_cell(pair, memory, cell_rect(i, pair));
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

/// Where the cell of `pair` goes, in grid row `row`: the columns are the
/// last digit of the number.
fn cell_rect(row: usize, pair: &Pair) -> Rect {
    let cell = (ARENA_W - 2.0 * MARGIN) / 10.0;
    let column = pair
        .number
        .chars()
        .last()
        .and_then(|c| c.to_digit(10))
        .unwrap_or(0) as f32;
    let (x, y) = (MARGIN + column * cell, GRID_TOP + row as f32 * ROW_H);
    Rect::new(x + 2.0, y + 2.0, cell - 4.0, ROW_H - 4.0)
}

/// Where the dots of a cell are centred: in its top right corner, clear of
/// the number in the middle.
fn meter_center(cell: Rect) -> Vec2 {
    vec2(cell.right() - 11.5, cell.y + 7.5)
}

fn draw_cell(pair: &Pair, memory: &Memory, rect: Rect) {
    let (fill, text, tier) = match memory.record(pair) {
        Some(record) => (
            mastery_color(record.difficulty),
            BLACK,
            Some(mastery_tier(record.difficulty)),
        ),
        None => (UNSEEN, Color::new(0.6, 0.6, 0.65, 1.0), None),
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_meter(meter_center(rect), tier, text);
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

/// How far the dots of a meter reach from their centre sideways.
const METER_REACH: f32 = 6.5 + 2.4;

/// Three dots in a row, centred on `center`: as many filled, from the left,
/// as `tier` says, the rest empty. Nothing is filled for a pair not yet
/// seen.
fn draw_meter(center: Vec2, tier: Option<u8>, color: Color) {
    const SPACING: f32 = 6.5;
    const RADIUS: f32 = 2.4;
    debug_assert!((SPACING + RADIUS - METER_REACH).abs() < 1e-6);
    for i in 0..3u8 {
        let x = center.x + (f32::from(i) - 1.0) * SPACING;
        if tier.is_some_and(|tier| i < tier) {
            draw_circle(x, center.y, RADIUS, color);
        } else {
            draw_circle_lines(x, center.y, RADIUS, 1.2, color);
        }
    }
}

/// The legend: a swatch, its dots and a label for each of the colours.
const LEGEND: [(Color, Option<u8>, &str); 4] = [
    (UNSEEN, None, "Ei vielä nähty"),
    (RED, Some(1), "Harjoittele"),
    (YELLOW, Some(2), "Melkein"),
    (GREEN, Some(3), "Osaat"),
];

/// Where each legend entry starts, and where the last one ends, given how
/// wide `text_width` says the labels are.
fn legend_layout(text_width: impl Fn(&str, u16) -> f32) -> ([f32; 4], f32) {
    let mut xs = [0.0; 4];
    let mut x = MARGIN;
    for (i, (_, _, label)) in LEGEND.iter().enumerate() {
        xs[i] = x;
        x += 44.0 + text_width(label, 18);
        if i + 1 < LEGEND.len() {
            x += 18.0;
        }
    }
    (xs, x)
}

fn draw_legend(y: f32) {
    let (xs, _) = legend_layout(|text, size| fonts::measure(text, Style::Body, size).width);
    for ((color, tier, label), x) in LEGEND.into_iter().zip(xs) {
        draw_rectangle(x, y - 10.0, 34.0, 20.0, color);
        let dots = if tier.is_some() {
            BLACK
        } else {
            Color::new(0.6, 0.6, 0.65, 1.0)
        };
        draw_meter(vec2(x + 17.0, y), tier, dots);
        fonts::draw(label, x + 40.0, y + 6.0, 18, WHITE, Style::Body);
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

    #[test]
    fn dots_run_from_one_for_hard_pairs_to_three_for_learned_ones() {
        assert_eq!(mastery_tier(0.0), 3);
        assert_eq!(mastery_tier(NEARLY), 2);
        assert_eq!(mastery_tier(1.0), 1);
        for i in 0..100 {
            let d = i as f32 / 100.0;
            assert!(
                mastery_tier(d + 0.01) <= mastery_tier(d),
                "harder means fewer dots"
            );
        }
    }

    #[test]
    fn three_dots_means_learned() {
        for i in 0..=100 {
            let d = i as f32 / 100.0;
            assert_eq!(mastery_tier(d) == 3, d < LEARNED, "{d}");
        }
    }

    #[test]
    fn each_legend_colour_has_the_dots_of_its_tier() {
        assert_eq!(mastery_tier(0.0), 3, "green");
        assert_eq!(mastery_tier(NEARLY), 2, "yellow");
        assert_eq!(mastery_tier(1.0), 1, "red");
    }

    /// A stand-in for measuring text.
    fn width(text: &str, size: u16) -> f32 {
        text.chars().count() as f32 * f32::from(size) * 0.6
    }

    fn all_cells() -> Vec<(Pair, Rect)> {
        pair_rows()
            .iter()
            .enumerate()
            .flat_map(|(row, pairs)| pairs.iter().map(move |p| (*p, cell_rect(row, p))))
            .collect()
    }

    #[test]
    fn every_pair_has_a_cell_and_none_overlap() {
        let cells = all_cells();
        assert_eq!(cells.len(), PAIRS.len());
        for (i, (a, ra)) in cells.iter().enumerate() {
            for (b, rb) in &cells[i + 1..] {
                let overlap = ra.x < rb.right()
                    && rb.x < ra.right()
                    && ra.y < rb.bottom()
                    && rb.y < ra.bottom();
                assert!(!overlap, "{} and {} overlap", a.number, b.number);
            }
        }
    }

    #[test]
    fn the_grid_fits_the_arena_above_the_legend() {
        let legend_y = GRID_TOP + 11.0 * ROW_H + 22.0;
        for (pair, rect) in all_cells() {
            assert!(rect.x >= 0.0 && rect.right() <= ARENA_W, "{}", pair.number);
            assert!(
                rect.y >= GRID_TOP && rect.bottom() < legend_y - 10.0,
                "{}",
                pair.number
            );
        }
    }

    #[test]
    fn a_pairs_column_is_the_last_digit_of_its_number() {
        for (pair, rect) in all_cells() {
            let digit = pair.number.chars().last().unwrap().to_digit(10).unwrap() as f32;
            let cell = (ARENA_W - 2.0 * MARGIN) / 10.0;
            assert!((rect.x - (MARGIN + digit * cell + 2.0)).abs() < 1e-4);
        }
    }

    #[test]
    fn the_dots_fit_in_the_cell_clear_of_the_number() {
        for (pair, rect) in all_cells() {
            let dots = meter_center(rect);
            let (left, right) = (dots.x - METER_REACH, dots.x + METER_REACH);
            assert!(left > rect.x && right < rect.right(), "{}", pair.number);
            assert!(
                dots.y > rect.y && dots.y < rect.center().y,
                "{}",
                pair.number
            );
            // The number is centred in the cell.
            let number_right = rect.center().x + width(pair.number, 16) / 2.0;
            assert!(
                left > number_right,
                "{}: {left} vs {number_right}",
                pair.number
            );
        }
    }

    #[test]
    fn the_legend_fits_on_one_line() {
        let (xs, end) = legend_layout(width);
        assert!(xs.is_sorted());
        assert!(end <= ARENA_W - MARGIN, "the legend ends at {end}");
    }

    #[test]
    fn the_legend_entries_do_not_run_into_each_other() {
        let (xs, _) = legend_layout(width);
        for (i, label) in LEGEND.iter().map(|(_, _, l)| l).enumerate().take(3) {
            assert!(xs[i] + 40.0 + width(label, 18) < xs[i + 1], "{label}");
        }
    }

    fn tap() -> Vec<Pointer> {
        vec![Pointer {
            id: 1,
            pos: vec2(100.0, 100.0),
            phase: TouchPhase::Ended,
        }]
    }

    fn pressed(key: KeyCode) -> Frame {
        Frame {
            pressed: vec![key],
            ..Frame::default()
        }
    }

    #[test]
    fn the_progress_screen_closes_on_a_tap_or_a_leaving_key() {
        assert!(!update(&Frame::default(), &[]));
        assert!(update(&Frame::default(), &tap()));
        for key in [
            KeyCode::Escape,
            KeyCode::Enter,
            KeyCode::Space,
            KeyCode::Backspace,
        ] {
            assert!(update(&pressed(key), &[]), "{key:?}");
        }
        assert!(!update(&pressed(KeyCode::A), &[]));
    }

    #[test]
    fn a_finger_only_touching_down_does_not_close_it() {
        let started = [Pointer {
            phase: TouchPhase::Started,
            ..tap()[0]
        }];
        assert!(!update(&Frame::default(), &started));
    }
}
