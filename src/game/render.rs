//! Drawing a game. The only part of `game` that touches the screen: it
//! reads a `Game` and changes nothing in it.

use macroquad::prelude::*;

use super::BACKGROUND;
use super::Game;
use super::answer::Slot;
use super::combat::SpellTarget;
use super::enemy::{Enemy, HINT_FONT_SIZE, HINT_SPACE, LABEL_FONT_SIZE, LABEL_HEIGHT};
use super::rules::*;
use crate::effects::Effects;
use crate::fonts::{self, Style, draw_centered_text};
use crate::long_numbers::{self, Question};
use crate::sprites::{
    draw_boss, draw_cyclops, draw_girl, draw_monster, draw_obstacle, draw_portal, draw_star,
};
use crate::view::{ARENA_H, ARENA_W, View};

/// The space between the two answer slots.
const SLOT_GAP: f32 = 8.0;

/// Where the number and word slots go, given how wide each is: side by side
/// under the player, or above her when she is near the bottom edge, and
/// always inside the arena.
fn slot_rects(player: Vec2, widths: [f32; 2], height: f32) -> [Rect; 2] {
    let total_width = widths[0] + SLOT_GAP + widths[1];
    let below = player.y + PLAYER_RADIUS + 14.0;
    let y = if below + height > ARENA_H {
        player.y - PLAYER_RADIUS - 20.0 - height
    } else {
        below
    };
    let left = (player.x - total_width / 2.0).clamp(0.0, (ARENA_W - total_width).max(0.0));
    [
        Rect::new(left, y, widths[0], height),
        Rect::new(left + widths[0] + SLOT_GAP, y, widths[1], height),
    ]
}

/// Draws the sparks, rings and lightning.
fn draw_effects(effects: &Effects) {
    for ring in effects.rings() {
        let color = Color {
            a: 1.0 - ring.progress(),
            ..ring.color
        };
        draw_circle_lines(ring.pos.x, ring.pos.y, ring.radius(), 3.0, color);
    }
    for p in effects.particles() {
        let fade = p.fade();
        let color = Color { a: fade, ..p.color };
        draw_circle(p.pos.x, p.pos.y, p.size * fade.max(0.3), color);
    }
    if let Some((bolts, brightness)) = effects.bolts() {
        let glow = Color::new(0.6, 0.8, 1.0, 0.35 * brightness);
        let core = Color::new(1.0, 1.0, 1.0, brightness);
        for bolt in bolts {
            for pair in bolt.points.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                draw_line(a.x, a.y, b.x, b.y, bolt.width * 4.0, glow);
                draw_line(a.x, a.y, b.x, b.y, bolt.width, core);
            }
        }
    }
}

/// Whitens the whole screen right after a lightning strike. Drawn on top
/// of everything else.
fn draw_flash(effects: &Effects, view: View) {
    if let Some(alpha) = effects.flash() {
        let screen = view.visible();
        draw_rectangle(
            screen.x,
            screen.y,
            screen.w,
            screen.h,
            Color::new(0.9, 0.95, 1.0, alpha),
        );
    }
}

/// Draws a body: a boss, a cyclops showing a word or an ordinary monster.
fn draw_enemy_body(enemy: &Enemy, time: f32, player: Vec2) {
    if let Some(lives) = &enemy.boss {
        draw_boss(enemy.pos, enemy.radius, time, enemy.phase);
        if lives.hit_flash > 0.0 {
            let alpha = lives.hit_flash / BOSS_HIT_FLASH_SECONDS * 0.8;
            draw_circle(
                enemy.pos.x,
                enemy.pos.y,
                enemy.radius * 1.3,
                Color::new(1.0, 1.0, 1.0, alpha),
            );
        }
    } else if enemy.shows_word {
        draw_cyclops(enemy.pos, enemy.radius, time, enemy.phase, player);
    } else {
        draw_monster(enemy.pos, enemy.radius, time, enemy.phase);
    }
}

impl Game {
    pub fn draw(&self, view: View) {
        clear_background(BACKGROUND);
        let time = self.play_time as f32;
        // Under everything else, so monsters are never hidden behind it.
        self.draw_new_pairs();
        for &portal in &self.portals {
            draw_portal(portal, time);
        }
        for obstacle in &self.obstacles {
            draw_obstacle(obstacle, time);
        }

        for enemy in &self.enemies {
            draw_enemy_body(enemy, time, self.player);
        }
        for spell in &self.spells {
            // Doomed monsters freeze and flash while the spell flies.
            if let SpellTarget::Doomed(target) = &spell.target {
                draw_enemy_body(target, time, self.player);
                let flash = 0.35 + 0.25 * (time * 40.0).sin();
                draw_circle(
                    target.pos.x,
                    target.pos.y,
                    target.radius * 1.2,
                    Color::new(1.0, 1.0, 1.0, flash),
                );
            }
        }
        draw_girl(
            self.player,
            time,
            self.player_moving,
            self.display.cast.map(|(toward, _)| toward),
        );
        for enemy in &self.enemies {
            draw_label(enemy);
        }
        draw_effects(&self.display.effects);
        if !self.is_over() {
            self.draw_slots();
        }
        for spell in &self.spells {
            draw_spell(spell.pos, time);
        }

        self.draw_hud();
        draw_flash(&self.display.effects, view);

        if self.is_over() {
            draw_rectangle(0.0, 0.0, ARENA_W, ARENA_H, Color::new(0.0, 0.0, 0.0, 0.7));
            let (cx, cy) = (ARENA_W / 2.0, ARENA_H / 2.0);
            fonts::draw_centered("Peli päättyi!", cx, cy - 60.0, 56, WHITE, Style::Heading);
            draw_centered_text(&format!("Pisteet: {}", self.score), cx, cy - 5.0, 36, WHITE);
            draw_centered_text(&format!("Taso {}", self.level), cx, cy + 35.0, 36, WHITE);
            let hint = if self.touch {
                "Napauta aloittaaksesi alusta"
            } else {
                "Paina Enter"
            };
            draw_centered_text(hint, cx, cy + 85.0, 28, LIGHTGRAY);
        } else if self.paused {
            draw_rectangle(0.0, 0.0, ARENA_W, ARENA_H, Color::new(0.0, 0.0, 0.0, 0.6));
            let (cx, cy) = (ARENA_W / 2.0, ARENA_H / 2.0);
            fonts::draw_centered("Tauko", cx, cy - 20.0, 64, WHITE, Style::Heading);
            let hint = if self.touch {
                "Jatka napauttamalla"
            } else {
                "Jatka välilyönnillä"
            };
            draw_centered_text(hint, cx, cy + 35.0, 28, LIGHTGRAY);
        } else if let Some(banner) = &self.display.banner {
            let alpha = (banner.seconds_left / 0.5).min(1.0);
            let (cx, cy) = (ARENA_W / 2.0, ARENA_H / 3.0);
            fonts::draw_centered(
                &banner.title,
                cx,
                cy,
                72,
                Color {
                    a: alpha,
                    ..banner.color
                },
                Style::Heading,
            );
            draw_centered_text(
                &banner.subtitle,
                cx,
                cy + 50.0,
                28,
                Color { a: alpha, ..WHITE },
            );
            // The stars earned on the level just finished.
            if let Some(stars) = banner.stars
                && alpha > 0.5
            {
                for i in 0..3u8 {
                    let x = cx + (f32::from(i) - 1.0) * 46.0;
                    draw_star(vec2(x, cy + 100.0), 20.0, i < stars);
                }
            }
        }
    }

    fn draw_hud(&self) {
        let bar_width = 200.0;
        let fill = (self.energy / MAX_ENERGY).clamp(0.0, 1.0);
        draw_rectangle(16.0, 16.0, bar_width, 16.0, DARKGRAY);
        draw_rectangle(16.0, 16.0, bar_width * fill, 16.0, GREEN);
        // Marks the level wrong keys can't take her below.
        let low_x = 16.0 + bar_width * LOW_ENERGY / MAX_ENERGY;
        draw_line(low_x, 12.0, low_x, 36.0, 2.0, WHITE);
        draw_text("Energia", 16.0, 54.0, 18.0, LIGHTGRAY);
        // The touch panel has buttons for these instead.
        if !self.touch {
            draw_text(
                "Tab: äänet   Välilyönti: tauko",
                16.0,
                ARENA_H - 16.0,
                16.0,
                GRAY,
            );
        }

        let score = format!("Pisteet: {}", self.score);
        let size = measure_text(&score, None, 24, 1.0);
        draw_text(&score, ARENA_W - size.width - 16.0, 32.0, 24.0, WHITE);

        // Level progress bar under the score.
        let needed = points_to_clear(self.level);
        let right = ARENA_W - 16.0;
        let progress = self.stage.points as f32 / needed as f32;
        let (bar_color, level) = if self.stage.boss_fight {
            (VIOLET, format!("Taso {}: POMO", self.level))
        } else {
            (
                GOLD,
                format!("Taso {}: {}/{}", self.level, self.stage.points, needed),
            )
        };
        draw_rectangle(right - bar_width, 44.0, bar_width, 10.0, DARKGRAY);
        draw_rectangle(
            right - bar_width,
            44.0,
            bar_width * progress,
            10.0,
            bar_color,
        );
        let size = measure_text(&level, None, 18, 1.0);
        draw_text(&level, right - size.width, 74.0, 18.0, LIGHTGRAY);

        let (cx, bottom) = (ARENA_W / 2.0, ARENA_H);
        if let Some(feedback) = &self.display.feedback {
            fonts::draw_centered(
                &feedback.text,
                cx,
                bottom - 30.0,
                30,
                feedback.color,
                Style::Bold,
            );
        }
    }

    /// Lists the pairs introduced on this level down the right edge. The
    /// panel is gone once a level brings nothing new.
    fn draw_new_pairs(&self) {
        const WIDTH: f32 = 150.0;
        const ROW: f32 = 24.0;
        const FONT_SIZE: u16 = 18;
        let new = self.curriculum.new_pairs();
        if new.is_empty() {
            if long_numbers::easy_long_numbers(self.level) {
                draw_long_number_tip();
            }
            return;
        }
        let x = ARENA_W - WIDTH - 16.0;
        let y = 100.0;
        let height = 40.0 + ROW * new.len() as f32;
        draw_rectangle(x, y, WIDTH, height, Color::new(0.0, 0.0, 0.0, 0.55));
        draw_rectangle_lines(x, y, WIDTH, height, 2.0, GOLD);
        fonts::draw_centered(
            "Uudet parit",
            x + WIDTH / 2.0,
            y + 18.0,
            FONT_SIZE,
            GOLD,
            Style::Heading,
        );
        for (i, pair) in new.iter().enumerate() {
            let row_y = y + 44.0 + ROW * i as f32;
            let number = measure_text(pair.number, None, FONT_SIZE, 1.0);
            // Numbers are right-aligned so the words line up.
            draw_text(
                pair.number,
                x + 40.0 - number.width,
                row_y,
                FONT_SIZE as f32,
                WHITE,
            );
            draw_text(
                pair.word.to_uppercase(),
                x + 56.0,
                row_y,
                FONT_SIZE as f32,
                LIME,
            );
        }
    }

    /// Draws the number and word slots side by side under the player, or
    /// above it when the player is near the bottom edge. A dead-end slot
    /// turns red.
    fn draw_slots(&self) {
        const FONT_SIZE: u16 = 20;
        const PAD: f32 = 6.0;
        const MIN_WIDTH: f32 = 36.0;

        let texts = [Slot::Number, Slot::Word].map(|slot| {
            let typed = self.typed.get(slot);
            let text = if typed.is_empty() {
                "·".to_owned()
            } else {
                typed.to_uppercase()
            };
            let width = fonts::measure(&text, Style::Bold, FONT_SIZE).width + 2.0 * PAD;
            (slot, text, width.max(MIN_WIDTH), typed.is_empty())
        });
        let widths = [texts[0].2, texts[1].2];
        let height = FONT_SIZE as f32 + PAD;
        let rects = slot_rects(self.player, widths, height);
        for ((slot, text, _, empty), rect) in texts.into_iter().zip(rects) {
            let dead_end = self.is_dead_end(slot);
            let accent = if dead_end { RED } else { slot.accent() };
            let background = if dead_end {
                Color::new(0.5, 0.0, 0.0, 0.7)
            } else {
                Color::new(0.0, 0.0, 0.0, 0.6)
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, accent);
            let color = if empty { GRAY } else { WHITE };
            let center = rect.center();
            fonts::draw_centered(&text, center.x, center.y, FONT_SIZE, color, Style::Bold);
        }
    }
}

/// Explains long numbers down the right edge, on the first levels with
/// them, where the panel of new pairs would be.
fn draw_long_number_tip() {
    const WIDTH: f32 = 150.0;
    const ROW: f32 = 24.0;
    const FONT_SIZE: u16 = 18;
    let Some(example) = Question::for_number("201") else {
        return;
    };
    let rows = [
        ("Kaksi numeroa".to_owned(), WHITE),
        ("kerrallaan:".to_owned(), WHITE),
        (example.number(false), WHITE),
        (format!("= {}", example.number(true)), WHITE),
        (format!("= {}", example.words()), LIME),
    ];
    let x = ARENA_W - WIDTH - 16.0;
    let y = 100.0;
    let height = 40.0 + ROW * rows.len() as f32;
    draw_rectangle(x, y, WIDTH, height, Color::new(0.0, 0.0, 0.0, 0.55));
    draw_rectangle_lines(x, y, WIDTH, height, 2.0, GOLD);
    let cx = x + WIDTH / 2.0;
    fonts::draw_centered(
        "Pitkät luvut",
        cx,
        y + 18.0,
        FONT_SIZE,
        GOLD,
        Style::Heading,
    );
    for (i, (text, color)) in rows.iter().enumerate() {
        let row_y = y + 44.0 + ROW * i as f32;
        fonts::draw_centered(text, cx, row_y - 5.0, FONT_SIZE, *color, Style::Bold);
    }
}

/// A glowing magic orb.
fn draw_spell(pos: Vec2, time: f32) {
    let pulse = 1.0 + 0.2 * (time * 30.0).sin();
    draw_circle(pos.x, pos.y, 12.0 * pulse, Color::new(1.0, 0.4, 0.8, 0.3));
    draw_circle(pos.x, pos.y, 7.0 * pulse, Color::new(1.0, 0.6, 0.9, 0.8));
    draw_circle(pos.x, pos.y, 3.5, WHITE);
}

/// Draws an enemy's word or number on a plate below its body, its hint
/// when shown, and a boss's remaining lives above its head.
fn draw_label(enemy: &Enemy) {
    let height = LABEL_HEIGHT;
    let center = enemy.pos + vec2(0.0, enemy.label_offset());
    let (x, y) = (center.x - enemy.label_width / 2.0, center.y - height / 2.0);
    draw_rectangle(
        x,
        y,
        enemy.label_width,
        height,
        Color::new(0.0, 0.0, 0.0, 0.75),
    );
    draw_rectangle_lines(
        x,
        y,
        enemy.label_width,
        height,
        2.0,
        enemy.answer_slot().accent(),
    );
    fonts::draw_centered(
        &enemy.label,
        center.x,
        center.y,
        LABEL_FONT_SIZE,
        WHITE,
        Style::Bold,
    );

    if enemy.shows_hint() {
        let hint_y = center.y + height / 2.0 + HINT_SPACE / 2.0;
        fonts::draw_centered(
            &enemy.hint,
            center.x,
            hint_y,
            HINT_FONT_SIZE,
            LIME,
            Style::Bold,
        );
    }

    if let Some(lives) = &enemy.boss {
        const PIP_RADIUS: f32 = 5.0;
        const PIP_SPACING: f32 = 14.0;
        let left = lives.queue.len() + 1;
        let row_width = PIP_SPACING * (lives.total - 1) as f32;
        let y = enemy.pos.y - enemy.radius * 1.6 - 8.0;
        for i in 0..lives.total {
            let x = enemy.pos.x - row_width / 2.0 + i as f32 * PIP_SPACING;
            if i < left {
                draw_circle(x, y, PIP_RADIUS, RED);
            }
            draw_circle_lines(x, y, PIP_RADIUS, 1.5, WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEIGHT: f32 = 26.0;

    fn overlaps(a: Rect, b: Rect) -> bool {
        a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
    }

    /// Whether a rectangle touches a circle.
    fn touches_circle(rect: Rect, center: Vec2, radius: f32) -> bool {
        let nearest = center.clamp(rect.point(), rect.point() + rect.size());
        nearest.distance(center) < radius
    }

    fn positions() -> impl Iterator<Item = Vec2> {
        (0..=20).flat_map(|i| {
            (0..=15).map(move |j| vec2(i as f32 * ARENA_W / 20.0, j as f32 * ARENA_H / 15.0))
        })
    }

    #[test]
    fn the_slots_stay_inside_the_arena_wherever_she_stands() {
        for player in positions() {
            for widths in [[36.0, 36.0], [36.0, 150.0], [80.0, 240.0]] {
                let [a, b] = slot_rects(player, widths, HEIGHT);
                for rect in [a, b] {
                    assert!(
                        rect.x >= 0.0 && rect.right() <= ARENA_W,
                        "{player} {widths:?} {rect:?}"
                    );
                    assert!(
                        rect.y >= 0.0 && rect.bottom() <= ARENA_H,
                        "{player} {rect:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_slots_sit_side_by_side_without_overlapping() {
        for player in positions() {
            let [a, b] = slot_rects(player, [36.0, 90.0], HEIGHT);
            assert!(!overlaps(a, b));
            assert_eq!(a.y, b.y);
            assert!((b.x - a.right() - SLOT_GAP).abs() < 1e-4);
        }
    }

    #[test]
    fn the_slots_never_cover_her() {
        for player in positions() {
            let player = player.clamp(
                vec2(PLAYER_RADIUS, PLAYER_RADIUS * 1.5),
                vec2(ARENA_W - PLAYER_RADIUS, ARENA_H - PLAYER_RADIUS * 1.5),
            );
            for rect in slot_rects(player, [36.0, 150.0], HEIGHT) {
                assert!(
                    !touches_circle(rect, player, PLAYER_RADIUS),
                    "{player} {rect:?}"
                );
            }
        }
    }

    #[test]
    fn the_slots_are_below_her_unless_she_is_near_the_bottom() {
        let [mid, _] = slot_rects(vec2(400.0, 300.0), [36.0, 36.0], HEIGHT);
        assert!(mid.y > 300.0);
        let [low, _] = slot_rects(vec2(400.0, ARENA_H - 24.0), [36.0, 36.0], HEIGHT);
        assert!(low.bottom() < ARENA_H - 24.0);
    }

    #[test]
    fn the_slots_are_centred_under_her_when_there_is_room() {
        let [a, b] = slot_rects(vec2(400.0, 200.0), [40.0, 80.0], HEIGHT);
        let middle = (a.x + b.right()) / 2.0;
        assert!((middle - 400.0).abs() < 1e-4);
    }
}
