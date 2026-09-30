//! Drawing the little notice at the top of the screen when a badge is
//! earned. What is shown, and when, is `Toasts`, in the core crate.

use macroquad::prelude::*;

use crate::gfx::badge_art::draw_medal;
use crate::gfx::fonts::{self, Style};
use lukuloitsu_core::toast::Toasts;

const WIDTH: f32 = 340.0;
const HEIGHT: f32 = 50.0;
const TOP: f32 = 8.0;

/// Draws the notice, if there is one, at the top of the arena.
pub fn draw(toasts: &Toasts, arena_width: f32) {
    let Some((badge, alpha)) = toasts.current() else {
        return;
    };
    let x = (arena_width - WIDTH) / 2.0;
    draw_rectangle(
        x,
        TOP,
        WIDTH,
        HEIGHT,
        Color::new(0.1, 0.1, 0.16, 0.92 * alpha),
    );
    draw_rectangle_lines(
        x,
        TOP,
        WIDTH,
        HEIGHT,
        2.0,
        Color::new(0.96, 0.77, 0.2, alpha),
    );
    draw_medal(vec2(x + 30.0, TOP + 19.0), 15.0, badge, true, alpha);
    let text_x = x + 62.0;
    let fade = |color: Color| Color::new(color.r, color.g, color.b, alpha);
    fonts::draw(
        "Uusi kunniamerkki!",
        text_x,
        TOP + 20.0,
        16,
        fade(GOLD),
        Style::Body,
    );
    fonts::draw(badge.name, text_x, TOP + 41.0, 22, fade(WHITE), Style::Bold);
}
