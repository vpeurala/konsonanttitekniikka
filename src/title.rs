//! The opening screen: the title, a few monsters, a short explanation of
//! the system and every number-word pair, scrollable, so the player can
//! study before starting.

use macroquad::prelude::*;

use crate::fonts::{self, Style};
use crate::game::draw_centered_text;
use crate::pairs::{DIGIT_CONSONANTS, PAIRS, Pair};
use crate::pictures::draw_picture;
use crate::sprites::{draw_boss, draw_cyclops, draw_girl, draw_monster};

/// Pixels scrolled per second while an arrow key is held.
const KEY_SCROLL_SPEED: f32 = 500.0;
const FOOTER_HEIGHT: f32 = 44.0;
const SIDE_MARGIN: f32 = 60.0;

const TITLE_COLOR: Color = GOLD;
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.65, 1.0);
const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);

const EXPLANATION_BEFORE_TABLE: &[&str] = &[
    "Konsonanttitekniikalla muistat minkä tahansa luvun.",
    "Jokainen numero vastaa yhtä konsonanttia:",
];

const EXPLANATION_AFTER_TABLE: &[&str] = &[
    "Vokaalit ovat pelkkää täytettä. Muita kirjaimia ei käytetä.",
    "Näin jokaiselle luvulle löytyy sana, jonka voi kuvitella:",
    "KEKO = K ja K = 2 ja 2 = 22.",
    "",
    "Pelissä hirviöissä on luku tai sana. Kirjoita sen vastine",
    "ennen kuin hirviö saa sinut kiinni! Liiku nuolinäppäimillä.",
];

pub struct TitleScreen {
    /// How far the content is scrolled up, in pixels.
    scroll: f32,
}

/// What the title screen wants to happen next.
pub enum TitleAction {
    Stay,
    StartGame,
}

impl TitleScreen {
    pub fn new() -> Self {
        TitleScreen { scroll: 0.0 }
    }

    pub fn update(&mut self) -> TitleAction {
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            return TitleAction::StartGame;
        }

        let dt = get_frame_time();
        let page = screen_height() - FOOTER_HEIGHT;
        if is_key_down(KeyCode::Down) {
            self.scroll += KEY_SCROLL_SPEED * dt;
        }
        if is_key_down(KeyCode::Up) {
            self.scroll -= KEY_SCROLL_SPEED * dt;
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.scroll += page * 0.9;
        }
        if is_key_pressed(KeyCode::PageUp) {
            self.scroll -= page * 0.9;
        }
        if is_key_pressed(KeyCode::Home) {
            self.scroll = 0.0;
        }
        if is_key_pressed(KeyCode::End) {
            self.scroll = f32::INFINITY;
        }
        // The wheel reports how far the content should move down.
        self.scroll -= mouse_wheel().1;

        self.scroll = self.scroll.clamp(0.0, self.max_scroll());
        TitleAction::Stay
    }

    fn max_scroll(&self) -> f32 {
        (content_height() - (screen_height() - FOOTER_HEIGHT)).max(0.0)
    }

    pub fn draw(&self) {
        clear_background(BACKGROUND);
        let time = get_time() as f32;
        let cx = screen_width() / 2.0;
        let mut y = 70.0 - self.scroll;

        draw_logo(cx, y, time);
        y += 90.0;
        draw_cast(cx, y, time);
        y += 90.0;

        for line in EXPLANATION_BEFORE_TABLE {
            draw_text(line, SIDE_MARGIN, y, 22.0, TEXT_COLOR);
            y += 28.0;
        }
        y += 10.0;
        draw_consonant_table(y);
        y += CONSONANT_TABLE_SPACE;
        for line in EXPLANATION_AFTER_TABLE {
            draw_text(line, SIDE_MARGIN, y, 22.0, TEXT_COLOR);
            y += 28.0;
        }

        y += 30.0;
        fonts::draw_centered("Kaikki parit", cx, y, 36, TITLE_COLOR, Style::Heading);
        y += 40.0;
        draw_pair_table(y, time);

        self.draw_scrollbar();
        draw_footer();
    }

    fn draw_scrollbar(&self) {
        let max = self.max_scroll();
        if max <= 0.0 {
            return;
        }
        let track = screen_height() - FOOTER_HEIGHT - 20.0;
        let visible = (screen_height() - FOOTER_HEIGHT) / content_height();
        let thumb = (track * visible).max(30.0);
        let top = 10.0 + (track - thumb) * self.scroll / max;
        let x = screen_width() - 10.0;
        draw_rectangle(x, 10.0, 4.0, track, Color::new(1.0, 1.0, 1.0, 0.1));
        draw_rectangle(x, top, 4.0, thumb, Color::new(1.0, 1.0, 1.0, 0.4));
    }
}

const LOGO_TEXT: &str = "Konsonanttitekniikka";
const LOGO_SIZE: u16 = 68;
const LOGO_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.45);
const LOGO_OUTLINE: Color = Color::new(0.28, 0.1, 0.03, 1.0);
const LOGO_FILL: Color = Color::new(1.0, 0.5, 0.1, 1.0);
const LOGO_GLOSS: Color = Color::new(1.0, 0.86, 0.3, 1.0);
const LOGO_SHINE: Color = Color::new(1.0, 0.97, 0.8, 1.0);

/// The game's name as a logo: chunky letters with a dark outline and a
/// drop shadow, an orange fill with a glossy golden top, bobbing in a
/// gentle wave. Centered on (cx, y).
fn draw_logo(cx: f32, y: f32, time: f32) {
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
    for i in 0..16 {
        let a = i as f32 / 16.0 * std::f32::consts::TAU;
        draw_all(a.cos() * 4.5, a.sin() * 4.5, LOGO_OUTLINE);
    }
    draw_all(0.0, 0.0, LOGO_FILL);

    // The gloss: the same letters in gold, clipped to a band over the top
    // of the lowercase letters, with a pale shine at its very top.
    let x_height = measure("o").offset_y;
    let top = baseline - whole.offset_y - 8.0;
    let gloss_bottom = baseline - x_height * 0.5;
    with_clip(
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
fn with_clip(x: f32, y: f32, w: f32, h: f32, draw: impl FnOnce()) {
    let dpi = macroquad::miniquad::window::dpi_scale();
    let rect = (
        (x * dpi) as i32,
        (y * dpi) as i32,
        (w * dpi).max(0.0) as i32,
        (h * dpi).max(0.0) as i32,
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
    let cell = (screen_width() - 2.0 * SIDE_MARGIN) / 10.0;
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

/// The pairs grouped into rows: the single digits, then one row per
/// first digit of the two-digit numbers, each pair in the column of its
/// last digit.
fn pair_rows() -> Vec<Vec<Pair>> {
    let mut rows: Vec<Vec<Pair>> = Vec::new();
    let singles: Vec<Pair> = PAIRS
        .iter()
        .filter(|p| p.number.len() == 1)
        .copied()
        .collect();
    if !singles.is_empty() {
        rows.push(singles);
    }
    for first in '0'..='9' {
        let row: Vec<Pair> = PAIRS
            .iter()
            .filter(|p| p.number.len() == 2 && p.number.starts_with(first))
            .copied()
            .collect();
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

const PAIR_ROW_HEIGHT: f32 = 118.0;
/// The size of each word's picture, leaving a gap between cells.
const PICTURE_MARGIN: f32 = 10.0;
/// The consonant table's height plus room before the next line.
const CONSONANT_TABLE_SPACE: f32 = 90.0;

fn draw_pair_table(y: f32, time: f32) {
    let cell = (screen_width() - 2.0 * SIDE_MARGIN) / 10.0;
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
            draw_picture(pair.number, vec2(x, row_y + size / 2.0), size, time);
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

/// The height of everything that scrolls, matching `draw`.
fn content_height() -> f32 {
    let explanation =
        28.0 * (EXPLANATION_BEFORE_TABLE.len() + EXPLANATION_AFTER_TABLE.len()) as f32;
    70.0 + 90.0
        + 90.0
        + explanation
        + 10.0
        + CONSONANT_TABLE_SPACE
        + 30.0
        + 40.0
        + PAIR_ROW_HEIGHT * pair_rows().len() as f32
        + 20.0
}

const AUTHOR: &str = "Ville Peurala";

/// The controls on the left and the author's name in the lower right
/// corner, fixed below the scrolling content.
fn draw_footer() {
    const FONT_SIZE: u16 = 20;
    let top = screen_height() - FOOTER_HEIGHT;
    draw_rectangle(0.0, top, screen_width(), FOOTER_HEIGHT, BACKGROUND);
    draw_line(0.0, top, screen_width(), top, 1.0, DIM);

    let controls = "Enter tai välilyönti: aloita peli   Nuolet tai hiiri: selaa";
    let size = measure_text(controls, None, FONT_SIZE, 1.0);
    let baseline = top + FOOTER_HEIGHT / 2.0 + size.offset_y / 2.0;
    draw_text(controls, 16.0, baseline, FONT_SIZE as f32, GOLD);

    let author = measure_text(AUTHOR, None, FONT_SIZE, 1.0);
    draw_text(
        AUTHOR,
        screen_width() - author.width - 16.0,
        baseline,
        FONT_SIZE as f32,
        DIM,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pair_is_in_the_table_once() {
        let listed: usize = pair_rows().iter().map(Vec::len).sum();
        assert_eq!(listed, PAIRS.len());
    }

    #[test]
    fn the_example_in_the_explanation_is_a_real_pair() {
        assert!(PAIRS.iter().any(|p| p.number == "22" && p.word == "keko"));
    }
}
