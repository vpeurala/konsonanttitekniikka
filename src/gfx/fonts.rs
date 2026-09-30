//! The game's fonts: Nunito for text, and the rounder Fredoka for
//! headings. Both are built into the program.

use std::sync::OnceLock;

use macroquad::prelude::*;

pub(crate) const NUNITO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Nunito-Regular.ttf");
pub(crate) const NUNITO_BOLD: &[u8] = include_bytes!("../../assets/fonts/Nunito-Bold.ttf");
pub(crate) const FREDOKA_SEMIBOLD: &[u8] =
    include_bytes!("../../assets/fonts/Fredoka-SemiBold.ttf");

static BOLD: OnceLock<Font> = OnceLock::new();
static HEADING: OnceLock<Font> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Ordinary text, in Nunito.
    Body,
    /// Labels that must stand out, in bold Nunito.
    Bold,
    /// Titles and banners, in Fredoka.
    Heading,
}

fn load(bytes: &[u8]) -> Font {
    load_ttf_font_from_bytes(bytes).expect("built-in font should load")
}

/// Loads the fonts and makes Nunito the default for all text. Call once
/// at startup, before drawing any text.
pub fn init() {
    set_default_font(load(NUNITO_REGULAR));
    BOLD.get_or_init(|| load(NUNITO_BOLD));
    HEADING.get_or_init(|| load(FREDOKA_SEMIBOLD));
}

/// The font for `style`; `None` means the default font.
pub fn font(style: Style) -> Option<&'static Font> {
    match style {
        Style::Body => None,
        Style::Bold => BOLD.get(),
        Style::Heading => HEADING.get(),
    }
}

pub fn measure(text: &str, style: Style, size: u16) -> TextDimensions {
    measure_text(text, font(style), size, 1.0)
}

/// Draws text with its baseline starting at (x, y).
pub fn draw(text: &str, x: f32, y: f32, size: u16, color: Color, style: Style) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: font(style),
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

/// Draws text centered horizontally and vertically on (x, y).
///
/// Vertical centering uses the height of a capital letter rather than of
/// the text itself, so texts with and without accents (like "Ä") share a
/// baseline.
pub fn draw_centered(text: &str, x: f32, y: f32, size: u16, color: Color, style: Style) {
    let width = measure(text, style, size).width;
    let cap_height = measure("H", style, size).offset_y;
    draw(
        text,
        x - width / 2.0,
        y + cap_height / 2.0,
        size,
        color,
        style,
    );
}

/// Draws ordinary text centered on (x, y).
pub fn draw_centered_text(text: &str, x: f32, y: f32, size: u16, color: Color) {
    draw_centered(text, x, y, size, color, Style::Body);
}
