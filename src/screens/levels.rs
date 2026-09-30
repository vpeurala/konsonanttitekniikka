//! Drawing the level choice. What it does is `LevelSelect`, in the core
//! crate.

use macroquad::prelude::*;

use crate::gfx::fonts::draw_centered_text;
use crate::gfx::fonts::{self, Style};
use crate::gfx::view::{ARENA_H, ARENA_W};
use lukuloitsu_core::screens::levels::LevelSelect;

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const DIM: Color = Color::new(0.6, 0.6, 0.65, 1.0);

pub fn draw(levels: &LevelSelect) {
    clear_background(BACKGROUND);
    let cx = ARENA_W / 2.0;
    fonts::draw_centered("Valitse taso", cx, 50.0, 44, GOLD, Style::Heading);
    draw_centered_text(
        "Aloita alusta tai tasolta, jolle olet jo päässyt",
        cx,
        95.0,
        20,
        DIM,
    );
    for (i, (level, caption)) in levels.levels().iter().zip(levels.captions()).enumerate() {
        let rect = levels.button_rect(i);
        let chosen = i == levels.selected() && !levels.touch();
        let (fill, edge) = if chosen {
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
        let title = format!("Taso {level}");
        fonts::draw_centered(&title, c.x, c.y - 9.0, 26, WHITE, Style::Heading);
        let caption_color = if chosen { WHITE } else { DIM };
        fonts::draw_centered(caption, c.x, c.y + 18.0, 15, caption_color, Style::Body);
    }
    let help = if levels.touch() {
        "Napauta tasoa aloittaaksesi"
    } else {
        "Nuolet: valitse     Enter: aloita     Esc: takaisin"
    };
    draw_centered_text(help, cx, ARENA_H - 30.0, 20, DIM);
}
