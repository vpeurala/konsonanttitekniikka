//! The user instruction booklet, `--render-booklet FILE.html`: one
//! self-contained HTML file, in Finnish, meant to be printed to PDF on A4
//! pages (`scripts/booklet.sh` does it with Chrome).
//!
//! The booklet is made from the same program as the game, so it can't drift
//! from it: the numbers, words and consonants come from `pairs`, the levels
//! from `curriculum` and `long_numbers`, and the pictures are drawn by the
//! game's own drawing code. This file and `pages` are pure: they turn
//! `Assets` into text. `images` is the shell that draws the pictures.

mod images;
mod pages;
mod style;
#[cfg(test)]
mod tests;

use crate::pairs::{DIGIT_CONSONANTS, PAIR_COUNT};

pub use images::write;

/// Everything the booklet embeds, as `data:` URIs so the HTML needs no
/// other files.
pub struct Assets {
    pub nunito_regular: String,
    pub nunito_bold: String,
    pub fredoka: String,
    /// The picture of each pair, by `PairId` index.
    pub pictures: Vec<String>,
    pub characters: Characters,
}

/// The game's characters, which decorate the pages.
pub struct Characters {
    pub girl: String,
    pub girl_casting: String,
    pub monster: String,
    pub cyclops: String,
    pub boss: String,
    pub portal: String,
    pub star: String,
}

/// The whole booklet as an HTML document.
pub fn html(assets: &Assets) -> String {
    assert_eq!(
        assets.pictures.len(),
        PAIR_COUNT,
        "a picture for every pair"
    );
    let mut out = String::new();
    out += "<!doctype html>\n<html lang=\"fi\">\n<head>\n<meta charset=\"utf-8\">\n";
    out += "<title>Lukuloitsu – opas</title>\n<style>\n";
    out += &fonts_css(assets);
    out += &digit_colors_css();
    out += &characters_css(&assets.characters);
    out += style::CSS;
    out += "</style>\n</head>\n<body>\n";
    for (i, page) in pages::pages(assets).into_iter().enumerate() {
        out += &page.render(i + 1);
    }
    out += "</body>\n</html>\n";
    out
}

fn fonts_css(assets: &Assets) -> String {
    format!(
        "@font-face {{ font-family: Nunito; font-weight: 400; src: url({}) format('truetype'); }}\n\
         @font-face {{ font-family: Nunito; font-weight: 700; src: url({}) format('truetype'); }}\n\
         @font-face {{ font-family: Fredoka; font-weight: 600; src: url({}) format('truetype'); }}\n",
        assets.nunito_regular, assets.nunito_bold, assets.fredoka
    )
}

/// `.d0` to `.d9`: the colour of a digit, which its consonant shares.
fn digit_colors_css() -> String {
    style::DIGIT_COLORS
        .iter()
        .enumerate()
        .map(|(digit, (background, text))| {
            format!(".d{digit} {{ --c: {background}; --t: {text}; }}\n")
        })
        .collect()
}

/// The characters are drawn as backgrounds, so a picture used on many pages
/// is only in the file once.
fn characters_css(characters: &Characters) -> String {
    [
        ("girl", &characters.girl),
        ("girl-casting", &characters.girl_casting),
        ("monster", &characters.monster),
        ("cyclops", &characters.cyclops),
        ("boss", &characters.boss),
        ("portal", &characters.portal),
        ("star", &characters.star),
    ]
    .iter()
    .map(|(name, uri)| format!(".{name} {{ background-image: url({uri}); }}\n"))
    .collect()
}

/// A page of the booklet: its class (`dark` for the covers) and its content.
pub struct Page {
    pub class: &'static str,
    pub body: String,
}

impl Page {
    fn render(&self, number: usize) -> String {
        // The covers carry no page number.
        let foot = if self.class == "dark" {
            String::new()
        } else {
            format!("<div class=\"foot\"><span>Lukuloitsu – opas</span><span>{number}</span></div>")
        };
        format!(
            "<section class=\"page {}\">\n{}\n{foot}\n</section>\n",
            self.class, self.body
        )
    }
}

/// The digit a consonant stands for, if it is one of the ten.
pub fn digit_of(consonant: char) -> Option<usize> {
    DIGIT_CONSONANTS.iter().position(|&c| c == consonant)
}

/// The letters of `word` as tiles: a consonant in its digit's colour with
/// the digit under it, a vowel in grey, so it is plain which letters count.
pub fn letter_tiles(word: &str) -> String {
    let tiles: String = word
        .chars()
        .map(|c| {
            let upper: String = c.to_uppercase().collect();
            match digit_of(c) {
                Some(digit) => {
                    format!("<span class=\"tile d{digit}\"><b>{upper}</b><i>{digit}</i></span>")
                }
                None => format!("<span class=\"tile vowel\"><b>{upper}</b><i>&nbsp;</i></span>"),
            }
        })
        .collect();
    format!("<div class=\"tiles\">{tiles}</div>")
}

/// Escapes text for HTML.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding.
pub fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(BASE64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// A `data:` URI for `bytes` of the given media type.
pub fn data_uri(media_type: &str, bytes: &[u8]) -> String {
    format!("data:{media_type};base64,{}", base64(bytes))
}
