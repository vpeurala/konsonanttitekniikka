//! Drawing practice mode. What it does is `PracticeScreen`, in the core
//! crate.

use macroquad::prelude::*;

use crate::gfx::fonts::draw_centered_text;
use crate::gfx::fonts::{self, Style};
use crate::gfx::pictures::draw_picture;
use crate::gfx::view::{ARENA_H, ARENA_W};
use lukuloitsu_core::geometry::Rect;
use lukuloitsu_core::screens::practice::{PracticeScreen, State};

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);
const CARD: Color = Color::new(0.14, 0.13, 0.22, 1.0);

pub fn draw(screen: &PracticeScreen, time: f32) {
    clear_background(BACKGROUND);
    let cx = ARENA_W / 2.0;
    fonts::draw_centered("Harjoittele", cx, 40.0, 44, GOLD, Style::Heading);
    let (correct, answered) = screen.score();
    let score = format!("Oikein {correct} / {answered}");
    let size = fonts::measure(&score, Style::Body, 22);
    fonts::draw(
        &score,
        ARENA_W - size.width - 20.0,
        30.0,
        22,
        WHITE,
        Style::Body,
    );

    let card = Rect::new(90.0, 80.0, ARENA_W - 180.0, 420.0);
    draw_rectangle(card.x, card.y, card.w, card.h, CARD);
    draw_rectangle_lines(
        card.x,
        card.y,
        card.w,
        card.h,
        2.0,
        Color::new(0.4, 0.35, 0.6, 1.0),
    );

    let prompt = screen.prompt();
    let answered = screen.state() != State::Asking;
    // A word's picture helps imagine it; a number's picture would give
    // its word away, so it waits until the card is answered.
    let show_picture = screen.shows_word() || answered;
    let prompt_y = if show_picture { 250.0 } else { 200.0 };
    if show_picture {
        draw_picture(screen.pair(), vec2(cx, 160.0), 110.0, time);
    }
    fonts::draw_centered(&prompt, cx, prompt_y + 20.0, 64, WHITE, Style::Heading);

    let accent = if screen.shows_word() { SKYBLUE } else { VIOLET };
    match screen.state() {
        State::Asking => {
            if screen.hint() {
                let hint = format!("= {}", screen.display_answer());
                fonts::draw_centered(&hint, cx, prompt_y + 75.0, 28, LIME, Style::Bold);
            }
            let dead_end = screen.is_dead_end();
            let slot_color = if dead_end { RED } else { accent };
            let text = if screen.typed().is_empty() {
                "·".to_owned()
            } else {
                screen.typed().to_uppercase()
            };
            let slot = Rect::new(cx - 130.0, prompt_y + 110.0, 260.0, 60.0);
            draw_rectangle(
                slot.x,
                slot.y,
                slot.w,
                slot.h,
                Color::new(0.0, 0.0, 0.0, 0.5),
            );
            draw_rectangle_lines(slot.x, slot.y, slot.w, slot.h, 3.0, slot_color);
            let c = slot.center();
            fonts::draw_centered(&text, c.x, c.y, 36, WHITE, Style::Bold);
        }
        State::Correct(_) => {
            fonts::draw_centered("Oikein!", cx, prompt_y + 90.0, 40, GREEN, Style::Heading);
            draw_pair_line(screen, cx, prompt_y + 145.0, WHITE);
        }
        State::Revealed(_) => {
            fonts::draw_centered(
                "Oikea vastaus:",
                cx,
                prompt_y + 90.0,
                28,
                YELLOW,
                Style::Bold,
            );
            draw_pair_line(screen, cx, prompt_y + 140.0, YELLOW);
        }
    }

    let help = if screen.touch() {
        "Napauta korttia: näytä vastaus     Tauko-nappi: takaisin"
    } else {
        "Välilyönti: näytä vastaus     Esc: takaisin"
    };
    draw_centered_text(
        help,
        cx,
        ARENA_H - 50.0,
        20,
        Color::new(0.6, 0.6, 0.65, 1.0),
    );
}

/// "KEKO = 22", the whole pair.
fn draw_pair_line(screen: &PracticeScreen, x: f32, y: f32, color: Color) {
    fonts::draw_centered(&screen.pair_line(), x, y, 40, color, Style::Heading);
}
