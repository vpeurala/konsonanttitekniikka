//! Drawing the progress map. Where things are, and what the tiers mean,
//! is `screens::progress_map`, in the core crate.

use macroquad::prelude::*;

use crate::gfx::fonts::draw_centered_text;
use crate::gfx::fonts::{self, Style};
use crate::gfx::sprites::draw_star;
use crate::gfx::view::{ARENA_H, ARENA_W};
use crate::memory::{Memory, learned_count};
use crate::pairs::{PAIRS, Pair};
use crate::progress::Progress;
use lukuloitsu_core::geometry::Rect;
use lukuloitsu_core::screens::progress_map::{
    LEGEND, LEGEND_Y, METER_REACH, NEARLY, cell_rect, legend_layout, mastery_tier, meter_center,
};
use lukuloitsu_core::screens::title::pair_rows;

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const UNSEEN: Color = Color::new(0.2, 0.2, 0.26, 1.0);
const RED: Color = Color::new(0.8, 0.22, 0.2, 1.0);
const YELLOW: Color = Color::new(0.92, 0.75, 0.2, 1.0);
const GREEN: Color = Color::new(0.25, 0.7, 0.3, 1.0);

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
/// The colour of a legend entry: the colour of a pair at that tier.
fn tier_color(tier: Option<u8>) -> Color {
    match tier {
        None => UNSEEN,
        Some(1) => RED,
        Some(2) => YELLOW,
        _ => GREEN,
    }
}

pub fn draw(data: &Progress, hardcore: bool, touch: bool) {
    let memory = &data.memory;
    clear_background(BACKGROUND);
    let cx = ARENA_W / 2.0;
    fonts::draw_centered("Edistyminen", cx, 36.0, 44, GOLD, Style::Heading);
    fonts::draw_mode_tag(hardcore);

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

    draw_legend(LEGEND_Y);
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

fn draw_legend(y: f32) {
    let (xs, _) = legend_layout(|text, size| fonts::measure(text, Style::Body, size).width);
    for ((tier, label), x) in LEGEND.into_iter().zip(xs) {
        let color = tier_color(tier);
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
}
