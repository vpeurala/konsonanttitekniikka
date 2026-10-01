//! Drawing the opening screen. What it does, and where things are, is
//! `TitleScreen`, in the core crate.

use macroquad::prelude::*;

use crate::gfx::fonts::draw_centered_text;
use crate::gfx::fonts::{self, Style};
use crate::gfx::pictures::draw_picture;
use crate::gfx::sprites::{draw_boss, draw_cyclops, draw_girl, draw_monster};
use crate::gfx::view::{ARENA_H, ARENA_W, View};
use crate::pairs::DIGIT_CONSONANTS;
use crate::progress::Progress;
use lukuloitsu_core::screens::title::{
    BUTTON_H, CONSONANT_TABLE_SPACE, EXPLANATION_AFTER_TABLE, EXPLANATION_BEFORE_TABLE,
    FOOTER_HEIGHT, MENU, MENU_SPACE, MENU_Y, PAIR_ROW_HEIGHT, PICTURE_MARGIN, PRIVACY_NOTE,
    SIDE_MARGIN, TitleScreen, button_rect, hardcore_rect, pair_rows,
};

const TITLE_COLOR: Color = GOLD;
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.65, 1.0);
const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);

pub fn draw(screen: &TitleScreen, progress: &Progress, hardcore: bool, view: View, time: f32) {
    clear_background(BACKGROUND);
    let cx = ARENA_W / 2.0;
    let mut y = 70.0 - screen.scroll();

    draw_logo(cx, y, time, view);
    y += 90.0;
    draw_cast(cx, y, time);
    y += 90.0;
    draw_menu(screen, progress, hardcore);
    y += MENU_SPACE;

    for line in EXPLANATION_BEFORE_TABLE {
        draw_text(line, SIDE_MARGIN, y, 22.0, TEXT_COLOR);
        y += 28.0;
    }
    y += 10.0;
    draw_consonant_table(y);
    y += CONSONANT_TABLE_SPACE;
    for line in EXPLANATION_AFTER_TABLE {
        // On touch screens she steers with the joystick instead.
        let line = if screen.touch() {
            line.replace("nuolinäppäimillä", "ohjaussauvalla")
        } else {
            line.to_string()
        };
        draw_text(&line, SIDE_MARGIN, y, 22.0, TEXT_COLOR);
        y += 28.0;
    }

    y += 30.0;
    fonts::draw_centered("Kaikki parit", cx, y, 36, TITLE_COLOR, Style::Heading);
    y += 40.0;
    draw_pair_table(y, time);
    y += PAIR_ROW_HEIGHT * pair_rows().len() as f32;

    y += 30.0;
    fonts::draw_centered("Tietosuoja", cx, y, 28, TITLE_COLOR, Style::Heading);
    y += 40.0;
    for line in PRIVACY_NOTE {
        draw_text(line, SIDE_MARGIN, y, 20.0, DIM);
        y += 28.0;
    }

    draw_scrollbar(screen);
    draw_footer(screen.touch());
}

fn draw_menu(screen: &TitleScreen, progress: &Progress, hardcore: bool) {
    for (i, (label, key, _)) in MENU.iter().enumerate() {
        let rect = button_rect(i, screen.scroll());
        let primary = i == 0;
        let (fill, edge) = if primary {
            (Color::new(0.85, 0.45, 0.1, 1.0), GOLD)
        } else {
            (
                Color::new(0.2, 0.18, 0.32, 1.0),
                Color::new(0.55, 0.45, 0.85, 1.0),
            )
        };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, edge);
        let c = rect.center();
        fonts::draw_centered(label, c.x, c.y - 4.0, 24, WHITE, Style::Heading);
        // The keyboard shortcut, for those with a keyboard.
        if !screen.touch() {
            fonts::draw_centered(key, c.x, c.y + 17.0, 13, DIM, Style::Body);
        }
    }
    let status = format!(
        "Päiviä putkeen: {}     Paras taso: {}",
        progress.streak, progress.best_level
    );
    let y = MENU_Y + BUTTON_H / 2.0 + 26.0 - screen.scroll();
    draw_centered_text(&status, ARENA_W / 2.0, y, 20, DIM);
    draw_hardcore_switch(screen, hardcore);
}

/// The switch for hardcore mode, a game without hints.
fn draw_hardcore_switch(screen: &TitleScreen, on: bool) {
    let rect = hardcore_rect(screen.scroll());
    let (fill, edge) = if on {
        (Color::new(0.6, 0.1, 0.1, 1.0), RED)
    } else {
        (Color::new(0.15, 0.14, 0.24, 1.0), DIM)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, edge);
    let label = format!(
        "Ankara tila (ei vihjeitä): {}",
        if on { "päällä" } else { "pois" }
    );
    let c = rect.center();
    fonts::draw_centered(&label, c.x, c.y - 1.0, 18, WHITE, Style::Body);
    if !screen.touch() {
        fonts::draw_centered("A", rect.x + rect.w + 16.0, c.y - 1.0, 13, DIM, Style::Body);
    }
}

fn draw_scrollbar(screen: &TitleScreen) {
    let max = TitleScreen::max_scroll();
    if max <= 0.0 {
        return;
    }
    let track = ARENA_H - FOOTER_HEIGHT - 20.0;
    let visible = (ARENA_H - FOOTER_HEIGHT) / lukuloitsu_core::screens::title::content_height();
    let thumb = (track * visible).max(30.0);
    let top = 10.0 + (track - thumb) * screen.scroll() / max;
    let x = ARENA_W - 10.0;
    draw_rectangle(x, 10.0, 4.0, track, Color::new(1.0, 1.0, 1.0, 0.1));
    draw_rectangle(x, top, 4.0, thumb, Color::new(1.0, 1.0, 1.0, 0.4));
}

const LOGO_TEXT: &str = "Lukuloitsu";
const LOGO_SIZE: u16 = 100;
const LOGO_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.45);
const LOGO_OUTLINE: Color = Color::new(0.28, 0.1, 0.03, 1.0);
const LOGO_FILL: Color = Color::new(1.0, 0.5, 0.1, 1.0);
const LOGO_GLOSS: Color = Color::new(1.0, 0.86, 0.3, 1.0);
const LOGO_SHINE: Color = Color::new(1.0, 0.97, 0.8, 1.0);

/// The game's name as a logo: chunky letters with a dark outline and a
/// drop shadow, an orange fill with a glossy golden top, bobbing in a
/// gentle wave. Centered on (cx, y).
fn draw_logo(cx: f32, y: f32, time: f32, view: View) {
    let measure = |text: &str| fonts::measure(text, Style::Heading, LOGO_SIZE);
    let whole = measure(LOGO_TEXT);
    let left = cx - whole.width / 2.0;
    let baseline = y + whole.offset_y / 2.0;

    // Each letter's position, measured from the width of the text before
    // it so the spacing matches the font's.
    let letters: Vec<(String, f32, f32)> = LOGO_TEXT
        .char_indices()
        .enumerate()
        .map(|(i, (byte, ch))| {
            let x = left + measure(&LOGO_TEXT[..byte]).width;
            let bob = (time * 2.5 - i as f32 * 0.35).sin() * 4.0;
            (ch.to_string(), x, baseline + bob)
        })
        .collect();
    let draw_all = |dx: f32, dy: f32, color: Color| {
        for (letter, x, y) in &letters {
            fonts::draw(letter, x + dx, y + dy, LOGO_SIZE, color, Style::Heading);
        }
    };

    draw_all(5.0, 7.0, LOGO_SHADOW);
    // Many copies around each letter, at two distances, make a smooth
    // outline even at large sizes.
    for radius in [3.0, 6.0] {
        for i in 0..32 {
            let a = i as f32 / 32.0 * std::f32::consts::TAU;
            draw_all(a.cos() * radius, a.sin() * radius, LOGO_OUTLINE);
        }
    }
    draw_all(0.0, 0.0, LOGO_FILL);

    // The gloss: the same letters in gold, clipped to a band over the top
    // of the lowercase letters, with a pale shine at its very top.
    let x_height = measure("o").offset_y;
    let top = baseline - whole.offset_y - 8.0;
    let gloss_bottom = baseline - x_height * 0.5;
    with_clip(
        view,
        left - 10.0,
        top,
        whole.width + 20.0,
        gloss_bottom - top,
        || {
            draw_all(0.0, 0.0, LOGO_GLOSS);
        },
    );
    let shine_bottom = baseline - x_height * 0.85;
    with_clip(
        view,
        left - 10.0,
        top,
        whole.width + 20.0,
        shine_bottom - top,
        || {
            draw_all(0.0, 0.0, LOGO_SHINE);
        },
    );
}

/// Runs `draw` with drawing clipped to the given screen rectangle.
fn with_clip(view: View, x: f32, y: f32, w: f32, h: f32, draw: impl FnOnce()) {
    // Clipping works in physical pixels, so convert from virtual units.
    let dpi = macroquad::miniquad::window::dpi_scale();
    let top_left = view.to_screen(vec2(x, y)) * dpi;
    let bottom_right = view.to_screen(vec2(x + w, y + h)) * dpi;
    let size = (bottom_right - top_left).max(Vec2::ZERO);
    let rect = (
        top_left.x as i32,
        top_left.y as i32,
        size.x as i32,
        size.y as i32,
    );
    // Safe as long as nothing else holds the internal GL context, which
    // is true inside ordinary drawing code.
    unsafe { get_internal_gl() }.quad_gl.scissor(Some(rect));
    draw();
    unsafe { get_internal_gl() }.quad_gl.scissor(None);
}

/// The heroine among some of the monsters she will meet.
fn draw_cast(cx: f32, y: f32, time: f32) {
    draw_boss(vec2(cx - 250.0, y), 32.0, time, 0.0);
    draw_monster(vec2(cx - 115.0, y + 10.0), 20.0, time, 1.3);
    draw_girl(vec2(cx, y + 6.0), time, false, None);
    let girl = vec2(cx, y);
    draw_cyclops(vec2(cx + 115.0, y + 10.0), 20.0, time, 2.1, girl);
    draw_cyclops(vec2(cx + 250.0, y + 6.0), 22.0, time, 4.7, girl);
}

/// Each digit above the consonant that stands for it.
fn draw_consonant_table(y: f32) {
    let cell = (ARENA_W - 2.0 * SIDE_MARGIN) / 10.0;
    for (digit, consonant) in DIGIT_CONSONANTS.iter().enumerate() {
        let x = SIDE_MARGIN + cell * (digit as f32 + 0.5);
        draw_rectangle_lines(x - cell / 2.0 + 3.0, y - 4.0, cell - 6.0, 58.0, 1.5, DIM);
        draw_centered_text(&digit.to_string(), x, y + 12.0, 22, WHITE);
        fonts::draw_centered(
            &consonant.to_uppercase().to_string(),
            x,
            y + 38.0,
            24,
            LIME,
            Style::Bold,
        );
    }
}

fn draw_pair_table(y: f32, time: f32) {
    let cell = (ARENA_W - 2.0 * SIDE_MARGIN) / 10.0;
    for (i, row) in pair_rows().iter().enumerate() {
        let row_y = y + i as f32 * PAIR_ROW_HEIGHT;
        if i % 2 == 0 {
            draw_rectangle(
                SIDE_MARGIN,
                row_y - 6.0,
                cell * 10.0,
                PAIR_ROW_HEIGHT - 4.0,
                Color::new(1.0, 1.0, 1.0, 0.04),
            );
        }
        for pair in row {
            let last_digit = pair.number.chars().last().and_then(|c| c.to_digit(10));
            let column = last_digit.unwrap_or(0) as f32;
            let x = SIDE_MARGIN + cell * (column + 0.5);
            let size = cell - PICTURE_MARGIN;
            draw_picture(*pair, vec2(x, row_y + size / 2.0), size, time);
            draw_centered_text(pair.number, x, row_y + size + 14.0, 16, DIM);
            fonts::draw_centered(
                &pair.word.to_uppercase(),
                x,
                row_y + size + 34.0,
                14,
                WHITE,
                Style::Bold,
            );
        }
    }
}

const AUTHOR: &str = "Ville Peurala";

/// The controls on the left and the author's name in the lower right
/// corner, fixed below the scrolling content.
fn draw_footer(touch: bool) {
    const FONT_SIZE: u16 = 20;
    let top = ARENA_H - FOOTER_HEIGHT;
    draw_rectangle(0.0, top, ARENA_W, FOOTER_HEIGHT, BACKGROUND);
    draw_line(0.0, top, ARENA_W, top, 1.0, DIM);

    let controls = if touch {
        "Vedä: selaa"
    } else {
        "Nuolet tai hiiri: selaa"
    };
    let size = measure_text(controls, None, FONT_SIZE, 1.0);
    let baseline = top + FOOTER_HEIGHT / 2.0 + size.offset_y / 2.0;
    draw_text(controls, 16.0, baseline, FONT_SIZE as f32, GOLD);

    let author = measure_text(AUTHOR, None, FONT_SIZE, 1.0);
    draw_text(
        AUTHOR,
        ARENA_W - author.width - 16.0,
        baseline,
        FONT_SIZE as f32,
        DIM,
    );
}
