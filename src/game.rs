use std::collections::HashSet;

use macroquad::prelude::*;

use crate::audio::Sfx;
use crate::effects::Effects;
use crate::keyboard::{Key, Keyboard};
use crate::pairs::{self, PAIRS, Pair};
use crate::sprites::{draw_cyclops, draw_girl, draw_monster, girl_hand};

const PLAYER_SPEED: f32 = 260.0;
const PLAYER_RADIUS: f32 = 16.0;
const ENEMY_RADIUS: f32 = 20.0;
const MAX_ENEMIES: usize = 6;

/// Enemies never appear closer than this to the player.
const MIN_SPAWN_DISTANCE: f32 = 300.0;
const SPAWN_ATTEMPTS: usize = 40;

const LABEL_FONT_SIZE: u16 = 26;
const LABEL_PAD: f32 = 6.0;
/// Distance from an enemy's center to the center of its label.
const LABEL_OFFSET: f32 = ENEMY_RADIUS + 22.0;
const LABEL_HEIGHT: f32 = LABEL_FONT_SIZE as f32 + 2.0;
const HINT_FONT_SIZE: u16 = 24;
/// Extra room below the label for a first-time hint.
const HINT_SPACE: f32 = 28.0;

const MAX_ENERGY: f32 = 100.0;
const HIT_REWARD: f32 = 5.0;
const WRONG_PENALTY: f32 = 10.0;
const COLLISION_PENALTY: f32 = 20.0;

const FEEDBACK_SECONDS: f32 = 2.5;

const FIRST_LEVEL_POINTS: u32 = 10;
/// How many more points each level needs than the one before.
const LEVEL_POINTS_INCREASE: u32 = 5;
const LEVEL_BANNER_SECONDS: f32 = 2.5;
/// No new enemies appear for this long after a level starts.
const LEVEL_BREAK_SECONDS: f32 = 2.0;

const START_SPEED: f32 = 12.0;
const MAX_SPEED: f32 = 100.0;
/// Speed gained per second spent on a level.
const SPEED_GROWTH: f32 = 1.0;
const START_SPAWN_INTERVAL: f32 = 4.0;
const MIN_SPAWN_INTERVAL: f32 = 1.2;
/// Spawn interval lost per second spent on a level.
const SPAWN_INTERVAL_SHRINK: f32 = 0.05;

/// The points needed to finish `level` (counting from 1).
pub fn points_to_clear(level: u32) -> u32 {
    FIRST_LEVEL_POINTS + LEVEL_POINTS_INCREASE * (level - 1)
}

/// Enemies start slow on every level and speed up the longer it lasts.
pub fn enemy_speed(level_time: f32) -> f32 {
    (START_SPEED + SPEED_GROWTH * level_time).min(MAX_SPEED)
}

pub fn spawn_interval(level_time: f32) -> f32 {
    (START_SPAWN_INTERVAL - SPAWN_INTERVAL_SHRINK * level_time).max(MIN_SPAWN_INTERVAL)
}

const SPELL_SPEED: f32 = 700.0;
/// How long she keeps her hand raised after casting.
const CAST_SECONDS: f32 = 0.35;

const KILL_PALETTE: [Color; 4] = [ORANGE, YELLOW, GOLD, WHITE];
const SPELL_PALETTE: [Color; 4] = [PINK, MAGENTA, VIOLET, WHITE];
const COLLISION_PALETTE: [Color; 3] = [RED, MAROON, ORANGE];

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);

/// What the typed text means given the answers currently on screen.
#[derive(Debug, PartialEq, Eq)]
pub enum InputOutcome {
    /// Nothing typed yet, or the text is the start of some answer.
    Pending,
    /// The text is the complete answer for the enemies at these indices.
    Hit(Vec<usize>),
    /// The text cannot become any on-screen answer.
    DeadEnd,
}

/// Resolves typed text against the answers on screen.
///
/// A complete answer wins immediately, even when it is also the start of a
/// longer answer: with both "0" and "00" on screen, typing "0" hits "0".
pub fn resolve_input<'a>(
    typed: &str,
    answers: impl IntoIterator<Item = &'a str> + Clone,
) -> InputOutcome {
    if typed.is_empty() {
        return InputOutcome::Pending;
    }
    let hits: Vec<usize> = answers
        .clone()
        .into_iter()
        .enumerate()
        .filter(|(_, answer)| *answer == typed)
        .map(|(i, _)| i)
        .collect();
    if !hits.is_empty() {
        InputOutcome::Hit(hits)
    } else if answers.into_iter().any(|answer| answer.starts_with(typed)) {
        InputOutcome::Pending
    } else {
        InputOutcome::DeadEnd
    }
}

/// Where typed characters go: digits to the number slot, letters to the
/// word slot. Each slot answers the enemies showing the other kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Number,
    Word,
}

impl Slot {
    /// The slot's color, which the monsters it answers share.
    fn accent(self) -> Color {
        match self {
            Slot::Number => SKYBLUE,
            Slot::Word => VIOLET,
        }
    }
}

struct Enemy {
    pos: Vec2,
    pair: Pair,
    shows_word: bool,
    label: String,
    label_width: f32,
    /// The answer, shown the first time a pair appears in a game.
    hint: Option<String>,
    /// Offsets the animation so enemies don't move in sync.
    phase: f32,
}

impl Enemy {
    fn new(pos: Vec2, pair: Pair, shows_word: bool, with_hint: bool) -> Self {
        let (label, hint) = if shows_word {
            (pair.word.to_uppercase(), pair.number.to_owned())
        } else {
            (pair.number.to_owned(), pair.word.to_uppercase())
        };
        let label_width = measure_text(&label, None, LABEL_FONT_SIZE, 1.0).width + 2.0 * LABEL_PAD;
        Enemy {
            pos,
            pair,
            shows_word,
            label,
            label_width,
            hint: with_hint.then(|| format!("= {hint}")),
            phase: rand::gen_range(0.0, 100.0),
        }
    }

    /// The radius of the circle enemies keep clear of each other: wide
    /// enough to cover both the body and the label hanging below it.
    fn reach(&self) -> f32 {
        let hint_width = self.hint.as_ref().map_or(0.0, |hint| {
            measure_text(hint, None, HINT_FONT_SIZE, 1.0).width
        });
        (ENEMY_RADIUS * 1.6 + self.hint_space() / 2.0)
            .max(self.label_width / 2.0 + 4.0)
            .max(hint_width / 2.0 + 4.0)
    }

    fn hint_space(&self) -> f32 {
        if self.hint.is_some() { HINT_SPACE } else { 0.0 }
    }

    /// The point `reach` is measured from, between the body and the label.
    fn reach_center(&self) -> Vec2 {
        self.pos + vec2(0.0, (LABEL_OFFSET + self.hint_space()) / 2.0)
    }

    fn draw_body(&self, time: f32, player: Vec2) {
        if self.shows_word {
            draw_cyclops(self.pos, ENEMY_RADIUS, time, self.phase, player);
        } else {
            draw_monster(self.pos, ENEMY_RADIUS, time, self.phase);
        }
    }

    fn answer_slot(&self) -> Slot {
        if self.shows_word {
            Slot::Number
        } else {
            Slot::Word
        }
    }

    fn answer(&self) -> String {
        if self.shows_word {
            self.pair.number.to_owned()
        } else {
            self.pair.word.to_lowercase()
        }
    }
}

/// A magic bolt flying toward an enemy that has already been answered.
/// The enemy is out of play and explodes when the bolt reaches it.
struct Spell {
    pos: Vec2,
    target: Enemy,
}

struct Feedback {
    text: String,
    color: Color,
    seconds_left: f32,
}

pub struct Game {
    keyboard: Keyboard,
    player: Vec2,
    player_moving: bool,
    enemies: Vec<Enemy>,
    /// The pairs that have appeared in this game, by number.
    seen: HashSet<&'static str>,
    spells: Vec<Spell>,
    /// Where she is casting toward, and for how much longer.
    cast: Option<(Vec2, f32)>,
    number_typed: String,
    word_typed: String,
    last_slot: Slot,
    energy: f32,
    score: u32,
    level: u32,
    /// Points scored on the current level.
    level_points: u32,
    /// Seconds spent on the current level.
    level_time: f32,
    /// Seconds left to show the "next level" banner.
    level_banner: f32,
    spawn_timer: f32,
    feedback: Option<Feedback>,
    /// Sound effects triggered since the last `take_sfx`.
    sfx: Vec<Sfx>,
    effects: Effects,
}

impl Game {
    pub fn new() -> Self {
        Game {
            keyboard: Keyboard::new(),
            player: vec2(screen_width() / 2.0, screen_height() / 2.0),
            player_moving: false,
            enemies: Vec::new(),
            seen: HashSet::new(),
            spells: Vec::new(),
            cast: None,
            number_typed: String::new(),
            word_typed: String::new(),
            last_slot: Slot::Word,
            energy: MAX_ENERGY,
            score: 0,
            level: 1,
            level_points: 0,
            level_time: 0.0,
            level_banner: 0.0,
            spawn_timer: 1.0,
            feedback: None,
            sfx: Vec::new(),
            effects: Effects::default(),
        }
    }

    fn restart(&mut self) {
        let keyboard = std::mem::replace(&mut self.keyboard, Keyboard::new());
        *self = Game::new();
        // Reuse the input subscription instead of registering a new one.
        self.keyboard = keyboard;
    }

    /// The sound effects triggered since the previous call.
    pub fn take_sfx(&mut self) -> Vec<Sfx> {
        std::mem::take(&mut self.sfx)
    }

    pub fn is_over(&self) -> bool {
        self.energy <= 0.0
    }

    pub fn update(&mut self) {
        let dt = get_frame_time();
        self.effects.update(dt);
        self.update_spells(dt);
        let keys = self.keyboard.typed();

        if self.is_over() {
            if is_key_pressed(KeyCode::Enter) {
                self.restart();
            }
            return;
        }

        self.level_time += dt;
        self.level_banner = (self.level_banner - dt).max(0.0);

        self.move_player(dt);
        for key in keys {
            self.handle_key(key);
        }
        self.move_enemies(dt);

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = spawn_interval(self.level_time);
            self.spawn_enemy();
        }

        if let Some(feedback) = &mut self.feedback {
            feedback.seconds_left -= dt;
            if feedback.seconds_left <= 0.0 {
                self.feedback = None;
            }
        }
    }

    fn move_player(&mut self, dt: f32) {
        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::Left) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            dir.x += 1.0;
        }
        if is_key_down(KeyCode::Up) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            dir.y += 1.0;
        }
        self.player_moving = dir != Vec2::ZERO;
        self.player += dir.normalize_or_zero() * PLAYER_SPEED * dt;
        self.player.x = self
            .player
            .x
            .clamp(PLAYER_RADIUS, screen_width() - PLAYER_RADIUS);
        self.player.y = self
            .player
            .y
            .clamp(PLAYER_RADIUS * 1.5, screen_height() - PLAYER_RADIUS * 1.5);
    }

    fn handle_key(&mut self, key: Key) {
        match key {
            Key::Backspace => {
                self.slot_mut(self.last_slot).pop();
            }
            Key::Char(c) if pairs::is_answer_char(c) => {
                let slot = if c.is_ascii_digit() {
                    Slot::Number
                } else {
                    Slot::Word
                };
                self.last_slot = slot;
                self.type_into(slot, c);
            }
            Key::Char(_) => {}
        }
    }

    fn slot(&self, slot: Slot) -> &str {
        match slot {
            Slot::Number => &self.number_typed,
            Slot::Word => &self.word_typed,
        }
    }

    fn slot_mut(&mut self, slot: Slot) -> &mut String {
        match slot {
            Slot::Number => &mut self.number_typed,
            Slot::Word => &mut self.word_typed,
        }
    }

    /// The enemies this slot can answer, as (enemy index, answer).
    fn candidates(&self, slot: Slot) -> Vec<(usize, String)> {
        self.enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.answer_slot() == slot)
            .map(|(i, e)| (i, e.answer()))
            .collect()
    }

    fn outcome(&self, slot: Slot) -> (Vec<(usize, String)>, InputOutcome) {
        let candidates = self.candidates(slot);
        let answers = candidates.iter().map(|(_, answer)| answer.as_str());
        let outcome = resolve_input(self.slot(slot), answers);
        (candidates, outcome)
    }

    fn is_dead_end(&self, slot: Slot) -> bool {
        self.outcome(slot).1 == InputOutcome::DeadEnd
    }

    /// A dead-end slot is only shown in red at first. Typing past it costs
    /// energy, which leaves room to fix a typo with backspace.
    fn type_into(&mut self, slot: Slot, c: char) {
        if self.is_dead_end(slot) {
            self.energy -= WRONG_PENALTY;
            self.sfx.push(Sfx::Wrong);
            self.feedback = Some(Feedback {
                text: format!("Väärin: {}", self.slot(slot).to_uppercase()),
                color: RED,
                seconds_left: FEEDBACK_SECONDS,
            });
            self.slot_mut(slot).clear();
            return;
        }

        self.slot_mut(slot).push(c);
        self.sfx.push(Sfx::Type);
        let (candidates, outcome) = self.outcome(slot);
        if let InputOutcome::Hit(hits) = outcome {
            let first = self.enemies[candidates[hits[0]].0].pair;
            // Candidates are in enemy order, so removing from the back
            // keeps the remaining indices valid.
            for &hit in hits.iter().rev() {
                let enemy = self.enemies.remove(candidates[hit].0);
                self.cast_spell(enemy);
            }
            self.energy = (self.energy + HIT_REWARD * hits.len() as f32).min(MAX_ENERGY);
            self.show_pair(first, GREEN);
            self.slot_mut(slot).clear();
            self.add_points(hits.len() as u32);
        }
    }

    fn add_points(&mut self, points: u32) {
        self.score += points;
        self.level_points += points;
        if self.level_points >= points_to_clear(self.level) {
            self.level += 1;
            self.level_points = 0;
            self.level_time = 0.0;
            self.level_banner = LEVEL_BANNER_SECONDS;
            self.sfx.push(Sfx::LevelUp);
            self.spawn_timer = LEVEL_BREAK_SECONDS;
        }
    }

    fn cast_spell(&mut self, target: Enemy) {
        self.sfx.push(Sfx::Cast);
        self.cast = Some((target.pos, CAST_SECONDS));
        self.spells.push(Spell {
            pos: girl_hand(self.player, target.pos),
            target,
        });
    }

    fn update_spells(&mut self, dt: f32) {
        if let Some((_, seconds_left)) = &mut self.cast {
            *seconds_left -= dt;
            if *seconds_left <= 0.0 {
                self.cast = None;
            }
        }

        let step = SPELL_SPEED * dt;
        let effects = &mut self.effects;
        let sfx = &mut self.sfx;
        self.spells.retain_mut(|spell| {
            let to_target = spell.target.pos - spell.pos;
            if to_target.length() <= step {
                effects.explode(spell.target.pos, ENEMY_RADIUS, &KILL_PALETTE);
                sfx.push(Sfx::Explode);
                effects.explode(spell.target.pos, ENEMY_RADIUS * 0.5, &SPELL_PALETTE);
                return false;
            }
            spell.pos += to_target.normalize() * step;
            effects.trail(spell.pos, &SPELL_PALETTE);
            true
        });
    }

    fn move_enemies(&mut self, dt: f32) {
        let speed = enemy_speed(self.level_time);
        let player = self.player;
        for enemy in &mut self.enemies {
            enemy.pos += (player - enemy.pos).normalize_or_zero() * speed * dt;
        }
        self.separate_enemies();

        let hit_distance = PLAYER_RADIUS + ENEMY_RADIUS;
        let (collided, remaining): (Vec<Enemy>, Vec<Enemy>) = self
            .enemies
            .drain(..)
            .partition(|e| e.pos.distance(player) < hit_distance);
        self.enemies = remaining;
        for enemy in &collided {
            self.energy -= COLLISION_PENALTY;
            self.sfx.extend([Sfx::Explode, Sfx::Hurt]);
            self.effects
                .explode(enemy.pos, ENEMY_RADIUS, &COLLISION_PALETTE);
            // Show the pair so a collision still teaches something.
            self.show_pair(enemy.pair, YELLOW);
        }
        if !collided.is_empty() {
            self.number_typed.clear();
            self.word_typed.clear();
        }
    }

    /// Pushes overlapping enemies apart so their labels stay readable.
    fn separate_enemies(&mut self) {
        for _ in 0..4 {
            for i in 0..self.enemies.len() {
                for j in i + 1..self.enemies.len() {
                    let (a, b) = (&self.enemies[i], &self.enemies[j]);
                    let offset = b.reach_center() - a.reach_center();
                    let min_distance = a.reach() + b.reach();
                    let distance = offset.length();
                    if distance >= min_distance {
                        continue;
                    }
                    let dir = if distance > 0.001 {
                        offset / distance
                    } else {
                        Vec2::from_angle(rand::gen_range(0.0, std::f32::consts::TAU))
                    };
                    let push = dir * (min_distance - distance) / 2.0;
                    self.enemies[i].pos -= push;
                    self.enemies[j].pos += push;
                }
            }
        }
    }

    fn show_pair(&mut self, pair: Pair, color: Color) {
        self.feedback = Some(Feedback {
            text: format!("{} = {}", pair.word.to_uppercase(), pair.number),
            color,
            seconds_left: FEEDBACK_SECONDS,
        });
    }

    /// Spawns an enemy just outside a screen edge, away from the player and
    /// the other enemies, never repeating a pair that is already on screen.
    fn spawn_enemy(&mut self) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let available: Vec<Pair> = PAIRS
            .iter()
            .filter(|p| {
                let mut on_screen = self
                    .enemies
                    .iter()
                    .chain(self.spells.iter().map(|s| &s.target));
                on_screen.all(|e| e.pair != **p)
            })
            .copied()
            .collect();
        if available.is_empty() {
            return;
        }
        let pair = available[rand::gen_range(0, available.len())];
        let first_time = !self.seen.contains(pair.number);
        let mut enemy = Enemy::new(Vec2::ZERO, pair, rand::gen_range(0, 2) == 0, first_time);

        // Try random edge points; settle for the farthest one from the
        // player if none is clear of everything.
        let mut best: Option<(Vec2, f32)> = None;
        for _ in 0..SPAWN_ATTEMPTS {
            enemy.pos = random_edge_point(enemy.reach());
            let player_distance = enemy.pos.distance(self.player);
            let clear = player_distance >= MIN_SPAWN_DISTANCE
                && self.enemies.iter().all(|other| {
                    other.reach_center().distance(enemy.reach_center())
                        >= other.reach() + enemy.reach()
                });
            if clear {
                best = Some((enemy.pos, f32::INFINITY));
                break;
            }
            if best.is_none_or(|(_, d)| player_distance > d) {
                best = Some((enemy.pos, player_distance));
            }
        }
        if let Some((pos, _)) = best {
            enemy.pos = pos;
            self.seen.insert(pair.number);
            self.enemies.push(enemy);
        }
    }

    pub fn draw(&self) {
        clear_background(BACKGROUND);
        let time = get_time() as f32;

        for enemy in &self.enemies {
            enemy.draw_body(time, self.player);
        }
        for spell in &self.spells {
            // Doomed monsters freeze and flash while the spell flies.
            let target = &spell.target;
            target.draw_body(time, self.player);
            let flash = 0.35 + 0.25 * (time * 40.0).sin();
            draw_circle(
                target.pos.x,
                target.pos.y,
                ENEMY_RADIUS * 1.2,
                Color::new(1.0, 1.0, 1.0, flash),
            );
        }
        draw_girl(
            self.player,
            time,
            self.player_moving,
            self.cast.map(|(toward, _)| toward),
        );
        for enemy in &self.enemies {
            draw_label(enemy);
        }
        self.effects.draw();
        if !self.is_over() {
            self.draw_slots();
        }
        for spell in &self.spells {
            draw_spell(spell.pos, time);
        }

        self.draw_hud();

        if self.is_over() {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 0.7),
            );
            let (cx, cy) = (screen_width() / 2.0, screen_height() / 2.0);
            draw_centered_text("Peli päättyi!", cx, cy - 60.0, 56, WHITE);
            draw_centered_text(&format!("Pisteet: {}", self.score), cx, cy - 5.0, 36, WHITE);
            draw_centered_text(&format!("Taso {}", self.level), cx, cy + 35.0, 36, WHITE);
            draw_centered_text("Paina Enter", cx, cy + 85.0, 28, LIGHTGRAY);
        } else if self.level_banner > 0.0 {
            let alpha = (self.level_banner / 0.5).min(1.0);
            let (cx, cy) = (screen_width() / 2.0, screen_height() / 3.0);
            draw_centered_text(
                &format!("Taso {}!", self.level),
                cx,
                cy,
                72,
                Color { a: alpha, ..GOLD },
            );
            draw_centered_text(
                "Hirviöt hidastuvat",
                cx,
                cy + 50.0,
                28,
                Color { a: alpha, ..WHITE },
            );
        }
    }

    fn draw_hud(&self) {
        let bar_width = 200.0;
        let fill = (self.energy / MAX_ENERGY).clamp(0.0, 1.0);
        draw_rectangle(16.0, 16.0, bar_width, 16.0, DARKGRAY);
        draw_rectangle(16.0, 16.0, bar_width * fill, 16.0, GREEN);
        draw_text("Energia", 16.0, 50.0, 22.0, LIGHTGRAY);
        draw_text("Tab: musiikki", 16.0, screen_height() - 16.0, 18.0, GRAY);

        let score = format!("Pisteet: {}", self.score);
        let size = measure_text(&score, None, 28, 1.0);
        draw_text(
            &score,
            screen_width() - size.width - 16.0,
            32.0,
            28.0,
            WHITE,
        );

        // Level progress bar under the score.
        let needed = points_to_clear(self.level);
        let right = screen_width() - 16.0;
        let progress = self.level_points as f32 / needed as f32;
        draw_rectangle(right - bar_width, 44.0, bar_width, 10.0, DARKGRAY);
        draw_rectangle(right - bar_width, 44.0, bar_width * progress, 10.0, GOLD);
        let level = format!("Taso {}: {}/{}", self.level, self.level_points, needed);
        let size = measure_text(&level, None, 22, 1.0);
        draw_text(&level, right - size.width, 76.0, 22.0, LIGHTGRAY);

        let (cx, bottom) = (screen_width() / 2.0, screen_height());
        if let Some(feedback) = &self.feedback {
            draw_centered_text(&feedback.text, cx, bottom - 30.0, 32, feedback.color);
        }
    }

    /// Draws the number and word slots side by side under the player, or
    /// above it when the player is near the bottom edge. A dead-end slot
    /// turns red.
    fn draw_slots(&self) {
        const FONT_SIZE: u16 = 24;
        const PAD: f32 = 6.0;
        const GAP: f32 = 8.0;
        const MIN_WIDTH: f32 = 36.0;

        let texts = [Slot::Number, Slot::Word].map(|slot| {
            let typed = self.slot(slot);
            let text = if typed.is_empty() {
                "_".to_owned()
            } else {
                typed.to_uppercase()
            };
            let width = measure_text(&text, None, FONT_SIZE, 1.0).width + 2.0 * PAD;
            (slot, text, width.max(MIN_WIDTH), typed.is_empty())
        });
        let height = FONT_SIZE as f32 + PAD;
        let total_width = texts[0].2 + GAP + texts[1].2;

        let below = self.player.y + PLAYER_RADIUS + 14.0;
        let y = if below + height > screen_height() {
            self.player.y - PLAYER_RADIUS - 20.0 - height
        } else {
            below
        };
        let mut x =
            (self.player.x - total_width / 2.0).clamp(0.0, (screen_width() - total_width).max(0.0));

        for (slot, text, width, empty) in texts {
            let dead_end = self.is_dead_end(slot);
            let accent = if dead_end { RED } else { slot.accent() };
            let background = if dead_end {
                Color::new(0.5, 0.0, 0.0, 0.7)
            } else {
                Color::new(0.0, 0.0, 0.0, 0.6)
            };
            draw_rectangle(x, y, width, height, background);
            draw_rectangle_lines(x, y, width, height, 2.0, accent);
            let color = if empty { GRAY } else { WHITE };
            draw_centered_text(&text, x + width / 2.0, y + height / 2.0, FONT_SIZE, color);
            x += width + GAP;
        }
    }
}

/// A random point just outside a screen edge, `margin` beyond it.
fn random_edge_point(margin: f32) -> Vec2 {
    let (w, h) = (screen_width(), screen_height());
    match rand::gen_range(0, 4) {
        0 => vec2(rand::gen_range(0.0, w), -margin),
        1 => vec2(rand::gen_range(0.0, w), h + margin),
        2 => vec2(-margin, rand::gen_range(0.0, h)),
        _ => vec2(w + margin, rand::gen_range(0.0, h)),
    }
}

/// A glowing magic orb.
fn draw_spell(pos: Vec2, time: f32) {
    let pulse = 1.0 + 0.2 * (time * 30.0).sin();
    draw_circle(pos.x, pos.y, 12.0 * pulse, Color::new(1.0, 0.4, 0.8, 0.3));
    draw_circle(pos.x, pos.y, 7.0 * pulse, Color::new(1.0, 0.6, 0.9, 0.8));
    draw_circle(pos.x, pos.y, 3.5, WHITE);
}

/// Draws an enemy's word or number on a plate below its body.
fn draw_label(enemy: &Enemy) {
    let height = LABEL_HEIGHT;
    let center = enemy.pos + vec2(0.0, LABEL_OFFSET);
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
    draw_centered_text(&enemy.label, center.x, center.y, LABEL_FONT_SIZE, WHITE);

    if let Some(hint) = &enemy.hint {
        let hint_y = center.y + height / 2.0 + HINT_SPACE / 2.0;
        draw_centered_text(hint, center.x, hint_y, HINT_FONT_SIZE, LIME);
    }
}

/// Draws text centered horizontally and vertically on (x, y).
fn draw_centered_text(text: &str, x: f32, y: f32, font_size: u16, color: Color) {
    let size = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        x - size.width / 2.0,
        y + size.offset_y / 2.0,
        font_size as f32,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_level_needs_ten_points() {
        assert_eq!(points_to_clear(1), 10);
    }

    #[test]
    fn each_level_needs_more_points_than_the_last() {
        for level in 1..50 {
            assert!(points_to_clear(level + 1) > points_to_clear(level));
        }
    }

    #[test]
    fn enemies_speed_up_within_a_level_up_to_a_cap() {
        assert_eq!(enemy_speed(0.0), START_SPEED);
        assert!(enemy_speed(10.0) > enemy_speed(0.0));
        assert_eq!(enemy_speed(1000.0), MAX_SPEED);
    }

    #[test]
    fn spawns_get_more_frequent_within_a_level_down_to_a_floor() {
        assert_eq!(spawn_interval(0.0), START_SPAWN_INTERVAL);
        assert!(spawn_interval(10.0) < spawn_interval(0.0));
        assert_eq!(spawn_interval(1000.0), MIN_SPAWN_INTERVAL);
    }

    #[test]
    fn empty_input_is_pending() {
        assert_eq!(resolve_input("", ["kuu"]), InputOutcome::Pending);
    }

    #[test]
    fn prefix_of_an_answer_is_pending() {
        assert_eq!(resolve_input("ku", ["kuu", "3"]), InputOutcome::Pending);
    }

    #[test]
    fn complete_answer_hits_every_matching_enemy() {
        assert_eq!(
            resolve_input("2", ["2", "kuu", "2"]),
            InputOutcome::Hit(vec![0, 2])
        );
    }

    #[test]
    fn complete_answer_wins_over_longer_answer() {
        assert_eq!(resolve_input("0", ["00", "0"]), InputOutcome::Hit(vec![1]));
    }

    #[test]
    fn prefix_of_longer_answer_waits() {
        assert_eq!(resolve_input("0", ["00", "5"]), InputOutcome::Pending);
    }

    #[test]
    fn text_matching_nothing_is_a_dead_end() {
        assert_eq!(resolve_input("ka", ["kuu", "3"]), InputOutcome::DeadEnd);
    }

    #[test]
    fn typing_with_no_enemies_is_a_dead_end() {
        assert_eq!(resolve_input("1", []), InputOutcome::DeadEnd);
    }
}
