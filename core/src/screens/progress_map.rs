//! The progress map: every pair coloured by how well the player knows it,
//! with counts of pairs learned and stars earned. This is where things
//! are and what a key does; colouring and drawing it is the shell's job.

use glam::{Vec2, vec2};

use crate::arena::ARENA_W;
use crate::geometry::Rect;
use crate::input::{Frame, KeyCode, Phase, Pointer};
use crate::memory::LEARNED;
use crate::pairs::Pair;

/// The colour is pure yellow here; a pair is "nearly" learned around it.
pub const NEARLY: f32 = 0.5;
/// Halfway between yellow and red: past this a pair still needs practice.
const PRACTISE: f32 = 0.75;

pub const GRID_TOP: f32 = 100.0;
pub const ROW_H: f32 = 40.0;
pub const MARGIN: f32 = 40.0;

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
    let tapped = pointers.iter().any(|p| p.phase == Phase::Ended);
    tapped
        || frame.pressed(KeyCode::Escape)
        || frame.pressed(KeyCode::Enter)
        || frame.pressed(KeyCode::Space)
        || frame.pressed(KeyCode::Backspace)
}

/// Where the cell of `pair` goes, in grid row `row`: the columns are the
/// last digit of the number.
pub fn cell_rect(row: usize, pair: &Pair) -> Rect {
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
pub fn meter_center(cell: Rect) -> Vec2 {
    vec2(cell.right() - 11.5, cell.y + 7.5)
}

/// How far the dots of a meter reach from their centre sideways.
pub const METER_REACH: f32 = 6.5 + 2.4;

/// The legend: the dots of each colour, and a label for it.
pub const LEGEND: [(Option<u8>, &str); 4] = [
    (None, "Ei vielä nähty"),
    (Some(1), "Harjoittele"),
    (Some(2), "Melkein"),
    (Some(3), "Osaat"),
];

/// Where the legend's line is, under the last row of the grid.
pub const LEGEND_Y: f32 = GRID_TOP + 11.0 * ROW_H + 22.0;

/// Where each legend entry starts, and where the last one ends, given how
/// wide `text_width` says the labels are.
pub fn legend_layout(text_width: impl Fn(&str, u16) -> f32) -> ([f32; 4], f32) {
    let mut xs = [0.0; 4];
    let mut x = MARGIN;
    for (i, (_, label)) in LEGEND.iter().enumerate() {
        xs[i] = x;
        x += 44.0 + text_width(label, 18);
        if i + 1 < LEGEND.len() {
            x += 18.0;
        }
    }
    (xs, x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Memory, learned_count};
    use crate::pairs::PAIRS;
    use crate::screens::title::pair_rows;

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
        let legend_y = LEGEND_Y;
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
        for (i, label) in LEGEND.iter().map(|(_, l)| l).enumerate().take(3) {
            assert!(xs[i] + 40.0 + width(label, 18) < xs[i + 1], "{label}");
        }
    }

    fn tap() -> Vec<Pointer> {
        vec![Pointer {
            id: 1,
            pos: vec2(100.0, 100.0),
            phase: Phase::Ended,
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
        assert!(!update(&pressed(KeyCode::Unused), &[]));
    }

    #[test]
    fn a_finger_only_touching_down_does_not_close_it() {
        let started = [Pointer {
            phase: Phase::Started,
            ..tap()[0]
        }];
        assert!(!update(&Frame::default(), &started));
    }
}
