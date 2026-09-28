use std::collections::HashMap;

use macroquad::prelude::*;

use crate::audio::Sfx;
use crate::curriculum::Curriculum;
use crate::effects::Effects;
use crate::keyboard::{Key, Keyboard};
use crate::obstacles::{Obstacle, obstacles_for_level, push_out, steer};
use crate::pairs::{self, Pair};
use crate::portals::portal_positions;
use crate::rng::{Rng, Stream};
use crate::sprites::{
    draw_boss, draw_cyclops, draw_girl, draw_monster, draw_obstacle, draw_portal, girl_hand,
};

const PLAYER_SPEED: f32 = 260.0;
const PLAYER_RADIUS: f32 = 16.0;
const ENEMY_RADIUS: f32 = 20.0;
const MAX_ENEMIES: usize = 6;

/// Enemies never appear closer than this to the player.
const MIN_SPAWN_DISTANCE: f32 = 300.0;
const SPAWN_ATTEMPTS: usize = 40;

const LABEL_FONT_SIZE: u16 = 26;
const LABEL_PAD: f32 = 6.0;
/// Gap between an enemy's body and the center of its label.
const LABEL_GAP: f32 = 22.0;
const LABEL_HEIGHT: f32 = LABEL_FONT_SIZE as f32 + 2.0;
const HINT_FONT_SIZE: u16 = 24;
/// Extra room below the label for a hint.
const HINT_SPACE: f32 = 28.0;

/// A pair's first appearances in a game get an early hint.
const HINTED_APPEARANCES: u32 = 3;
/// How long an early hint waits after the pair is shown.
const EARLY_HINT_SECONDS: f32 = 2.0;
/// Any other enemy shows its answer once it is this fast, as a fraction of
/// its top speed. With the current speeds that takes about 22 seconds.
const HINT_SPEED_FRACTION: f32 = 0.3;

/// Whether an enemy shows its answer. Every enemy starts without a hint.
/// One of the first appearances of its pair in this game
/// (`earlier_appearances`) gets it soon after the pair is shown
/// (`shown_for` seconds ago); any other only once it has reached
/// `HINT_SPEED_FRACTION` of its top speed (`speed_fraction`).
pub fn shows_hint(earlier_appearances: u32, shown_for: f32, speed_fraction: f32) -> bool {
    if earlier_appearances < HINTED_APPEARANCES {
        shown_for >= EARLY_HINT_SECONDS
    } else {
        speed_fraction >= HINT_SPEED_FRACTION
    }
}

const MAX_ENERGY: f32 = 100.0;
const HIT_REWARD: f32 = 5.0;
const WRONG_PENALTY: f32 = 10.0;
/// Wrong keys never take energy below this, so only collisions can end
/// the game.
const LOW_ENERGY: f32 = 20.0;
const COLLISION_PENALTY: f32 = 20.0;

const FEEDBACK_SECONDS: f32 = 2.5;

const FIRST_LEVEL_POINTS: u32 = 10;
/// How many more points each level needs than the one before.
const LEVEL_POINTS_INCREASE: u32 = 5;
const BANNER_SECONDS: f32 = 2.5;
/// No new enemies appear for this long after a level starts.
const LEVEL_BREAK_SECONDS: f32 = 2.0;

/// A new enemy's speed.
const START_SPEED: f32 = 12.0;
/// Speed an enemy gains per second on screen, up to the player's speed.
const SPEED_GROWTH: f32 = 3.0;
const START_SPAWN_INTERVAL: f32 = 4.0;
const MIN_SPAWN_INTERVAL: f32 = 1.2;
/// Spawn interval lost per second spent on a level.
const SPAWN_INTERVAL_SHRINK: f32 = 0.05;

const BOSS_RADIUS: f32 = 36.0;
/// The first level's boss takes this many hits.
const BOSS_FIRST_HITS: usize = 3;
/// Bosses take one more hit every this many levels.
const BOSS_LEVELS_PER_EXTRA_HIT: u32 = 2;
const BOSS_MAX_HITS: usize = 10;
/// The boss moves at this fraction of a normal enemy's speed.
const BOSS_SPEED_FACTOR: f32 = 0.6;
/// How far a spell pushes the boss back.
const BOSS_HIT_KNOCKBACK: f32 = 50.0;
/// How far the boss bounces back after running into the player.
const BOSS_COLLISION_KNOCKBACK: f32 = 220.0;
const BOSS_HIT_FLASH_SECONDS: f32 = 0.25;
/// After running into the player, the boss can't hurt her again for this
/// long, even if it is stuck next to her against an edge.
const BOSS_HARMLESS_SECONDS: f32 = 1.5;

/// The energy left after a wrong key: the penalty, but never below
/// `LOW_ENERGY`.
pub fn after_wrong_key(energy: f32) -> f32 {
    if energy <= LOW_ENERGY {
        energy
    } else {
        (energy - WRONG_PENALTY).max(LOW_ENERGY)
    }
}

/// The points needed to summon the boss of `level` (counting from 1).
pub fn points_to_clear(level: u32) -> u32 {
    FIRST_LEVEL_POINTS + LEVEL_POINTS_INCREASE * (level - 1)
}

/// How many numbers (and so hits) the boss of `level` has.
pub fn boss_hits(level: u32) -> usize {
    let extra = ((level - 1) / BOSS_LEVELS_PER_EXTRA_HIT) as usize;
    (BOSS_FIRST_HITS + extra).min(BOSS_MAX_HITS)
}

/// Every enemy starts slow and speeds up as it ages, until it is as fast
/// as the player.
pub fn enemy_speed(age: f32) -> f32 {
    (START_SPEED + SPEED_GROWTH * age).min(PLAYER_SPEED)
}

/// Difficulty comes from the spawn rate: enemies appear more often the
/// longer a level lasts, and a new level starts calm again.
pub fn spawn_interval(level_time: f32) -> f32 {
    (START_SPAWN_INTERVAL - SPAWN_INTERVAL_SHRINK * level_time).max(MIN_SPAWN_INTERVAL)
}

const SPELL_SPEED: f32 = 700.0;
/// How long she keeps her hand raised after casting.
const CAST_SECONDS: f32 = 0.35;

const KILL_PALETTE: [Color; 4] = [ORANGE, YELLOW, GOLD, WHITE];
const SPELL_PALETTE: [Color; 4] = [PINK, MAGENTA, VIOLET, WHITE];
const COLLISION_PALETTE: [Color; 3] = [RED, MAROON, ORANGE];

/// How often a new monster uses one of the level's new pairs, when one is
/// free, so new pairs get extra practice.
const NEW_PAIR_SHARE: f32 = 0.5;

/// How often a new monster comes out of a portal rather than a screen
/// edge.
const PORTAL_SPAWN_SHARE: f32 = 0.75;
/// Monsters don't come out of a portal closer than this to the player.
const MIN_PORTAL_SPAWN_DISTANCE: f32 = 200.0;
const PORTAL_PALETTE: [Color; 3] = [VIOLET, SKYBLUE, WHITE];

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

/// A boss's extra lives: the pairs it shows after its current one.
struct BossLives {
    queue: Vec<Pair>,
    total: usize,
    /// Seconds left of the white flash after being hit.
    hit_flash: f32,
    /// Seconds left before it can hurt the player again.
    harmless_for: f32,
}

struct Enemy {
    pos: Vec2,
    radius: f32,
    pair: Pair,
    shows_word: bool,
    label: String,
    label_width: f32,
    /// The answer, as shown in a hint.
    hint: String,
    /// How many times the current pair had appeared in the game before it
    /// was shown here.
    earlier_appearances: u32,
    /// Seconds on screen; sets the speed.
    age: f32,
    /// Seconds the current pair has been shown; sets the hint.
    shown_for: f32,
    /// Offsets the animation so enemies don't move in sync.
    phase: f32,
    boss: Option<BossLives>,
}

impl Enemy {
    /// `phase` offsets its animation from other enemies'.
    fn new(pair: Pair, shows_word: bool, earlier_appearances: u32, phase: f32) -> Self {
        let mut enemy = Enemy {
            pos: Vec2::ZERO,
            radius: ENEMY_RADIUS,
            pair,
            shows_word,
            label: String::new(),
            label_width: 0.0,
            hint: String::new(),
            earlier_appearances: 0,
            age: 0.0,
            shown_for: 0.0,
            phase,
            boss: None,
        };
        enemy.show(pair, earlier_appearances);
        enemy
    }

    /// A boss showing the numbers of `pairs`, one after another. Its
    /// `earlier_appearances` are for the first pair.
    fn boss(pairs: &[Pair], earlier_appearances: u32, phase: f32) -> Self {
        let mut boss = Enemy::new(pairs[0], false, earlier_appearances, phase);
        boss.radius = BOSS_RADIUS;
        boss.boss = Some(BossLives {
            queue: pairs[1..].to_vec(),
            total: pairs.len(),
            hit_flash: 0.0,
            harmless_for: 0.0,
        });
        boss
    }

    /// Switches to showing `pair`.
    fn show(&mut self, pair: Pair, earlier_appearances: u32) {
        let (label, hint) = if self.shows_word {
            (pair.word.to_uppercase(), pair.number.to_owned())
        } else {
            (pair.number.to_owned(), pair.word.to_uppercase())
        };
        self.label_width = measure_text(&label, None, LABEL_FONT_SIZE, 1.0).width + 2.0 * LABEL_PAD;
        self.label = label;
        self.hint = format!("= {hint}");
        self.pair = pair;
        self.earlier_appearances = earlier_appearances;
        self.shown_for = 0.0;
    }

    fn is_boss(&self) -> bool {
        self.boss.is_some()
    }

    fn speed(&self) -> f32 {
        let speed = enemy_speed(self.age);
        if self.is_boss() {
            speed * BOSS_SPEED_FACTOR
        } else {
            speed
        }
    }

    /// Distance from the enemy's center to the center of its label.
    fn label_offset(&self) -> f32 {
        self.radius + LABEL_GAP
    }

    /// The radius of the circle enemies keep clear of each other: wide
    /// enough to cover both the body and the label hanging below it.
    fn reach(&self) -> f32 {
        let hint_width = if self.shows_hint() {
            measure_text(&self.hint, None, HINT_FONT_SIZE, 1.0).width
        } else {
            0.0
        };
        (self.radius * 1.6 + self.hint_space() / 2.0)
            .max(self.label_width / 2.0 + 4.0)
            .max(hint_width / 2.0 + 4.0)
    }

    fn shows_hint(&self) -> bool {
        // The boss's speed factor applies to its top speed as well, so it
        // cancels out of the fraction.
        let speed_fraction = enemy_speed(self.age) / PLAYER_SPEED;
        shows_hint(self.earlier_appearances, self.shown_for, speed_fraction)
    }

    fn hint_space(&self) -> f32 {
        if self.shows_hint() { HINT_SPACE } else { 0.0 }
    }

    /// Moves the enemy so its body, label and any boss pips are on screen.
    fn keep_on_screen(&mut self) {
        let top = self.radius * 1.6 + 16.0;
        let bottom = self.label_offset() + LABEL_HEIGHT / 2.0 + self.hint_space();
        // A boss's wings reach well past its body.
        let body_half_width = if self.is_boss() {
            self.radius * 1.7
        } else {
            self.radius
        };
        let half_width = body_half_width.max(self.label_width / 2.0);
        self.pos.x = self
            .pos
            .x
            .clamp(half_width, (screen_width() - half_width).max(half_width));
        self.pos.y = self.pos.y.clamp(top, (screen_height() - bottom).max(top));
    }

    /// The point `reach` is measured from, between the body and the label.
    fn reach_center(&self) -> Vec2 {
        self.pos + vec2(0.0, (self.label_offset() + self.hint_space()) / 2.0)
    }

    fn draw_body(&self, time: f32, player: Vec2) {
        if let Some(lives) = &self.boss {
            draw_boss(self.pos, self.radius, time, self.phase);
            if lives.hit_flash > 0.0 {
                let alpha = lives.hit_flash / BOSS_HIT_FLASH_SECONDS * 0.8;
                draw_circle(
                    self.pos.x,
                    self.pos.y,
                    self.radius * 1.3,
                    Color::new(1.0, 1.0, 1.0, alpha),
                );
            }
        } else if self.shows_word {
            draw_cyclops(self.pos, self.radius, time, self.phase, player);
        } else {
            draw_monster(self.pos, self.radius, time, self.phase);
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

/// What a spell is flying toward.
enum SpellTarget {
    /// An enemy that is already out of play and explodes when hit.
    Doomed(Enemy),
    /// The boss, which is still in play and only loses a life.
    Boss,
}

/// A magic bolt flying toward an answered enemy.
struct Spell {
    pos: Vec2,
    target: SpellTarget,
    /// Where the target was last seen, in case the boss dies first.
    target_pos: Vec2,
}

struct Feedback {
    text: String,
    color: Color,
    seconds_left: f32,
}

/// A big message in the middle of the screen, like "Taso 2!".
struct Banner {
    title: String,
    subtitle: String,
    color: Color,
    seconds_left: f32,
}

pub struct Game {
    keyboard: Keyboard,
    player: Vec2,
    player_moving: bool,
    enemies: Vec<Enemy>,
    /// How many times each pair has appeared in this game, by number.
    appearances: HashMap<&'static str, u32>,
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
    /// Whether this level's boss has been summoned and not yet beaten.
    boss_fight: bool,
    banner: Option<Banner>,
    spawn_timer: f32,
    feedback: Option<Feedback>,
    /// Sound effects triggered since the last `take_sfx`.
    sfx: Vec<Sfx>,
    effects: Effects,
    /// Every gameplay decision's randomness, the same in every game.
    rng: Rng,
    curriculum: Curriculum,
    /// The current level's portals.
    portals: Vec<Vec2>,
    /// The current level's stones, trees and lakes.
    obstacles: Vec<Obstacle>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            keyboard: Keyboard::new(),
            player: vec2(screen_width() / 2.0, screen_height() / 2.0),
            player_moving: false,
            enemies: Vec::new(),
            appearances: HashMap::new(),
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
            boss_fight: false,
            banner: None,
            spawn_timer: 1.0,
            feedback: None,
            sfx: Vec::new(),
            effects: Effects::default(),
            rng: Rng::new(Stream::Gameplay, 0),
            curriculum: Curriculum::new(),
            portals: portal_positions(1, screen_width(), screen_height()),
            obstacles: Vec::new(),
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
        if let Some(banner) = &mut self.banner {
            banner.seconds_left -= dt;
            if banner.seconds_left <= 0.0 {
                self.banner = None;
            }
        }

        self.move_player(dt);
        for key in keys {
            self.handle_key(key);
        }
        self.move_enemies(dt);

        // No new monsters join a boss fight.
        if !self.boss_fight {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = spawn_interval(self.level_time);
                self.spawn_enemy();
            }
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
        self.player = push_out(self.player, PLAYER_RADIUS, &self.obstacles);
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
            self.energy = after_wrong_key(self.energy);
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
                self.hit_enemy(candidates[hit].0);
            }
            self.energy = (self.energy + HIT_REWARD * hits.len() as f32).min(MAX_ENERGY);
            self.show_pair(first, GREEN);
            self.slot_mut(slot).clear();
            self.add_points(hits.len() as u32);
        }
    }

    /// Casts a spell at the enemy at `index`. Ordinary enemies and a boss
    /// on its last life leave play at once; a boss with lives left moves on
    /// to its next pair.
    fn hit_enemy(&mut self, index: usize) {
        let next = self.enemies[index]
            .boss
            .as_mut()
            .and_then(|lives| (!lives.queue.is_empty()).then(|| lives.queue.remove(0)));
        match next {
            Some(pair) => {
                let target_pos = self.enemies[index].pos;
                let earlier = self.count_appearance(pair);
                self.enemies[index].show(pair, earlier);
                self.cast_spell(SpellTarget::Boss, target_pos);
            }
            None => {
                let enemy = self.enemies.remove(index);
                let target_pos = enemy.pos;
                self.cast_spell(SpellTarget::Doomed(enemy), target_pos);
            }
        }
    }

    /// Records that `pair` is appearing, returning how many times it had
    /// appeared before.
    fn count_appearance(&mut self, pair: Pair) -> u32 {
        let count = self.appearances.entry(pair.number).or_default();
        *count += 1;
        *count - 1
    }

    fn add_points(&mut self, points: u32) {
        self.score += points;
        if self.boss_fight {
            return;
        }
        self.level_points += points;
        if self.level_points >= points_to_clear(self.level) {
            self.level_points = points_to_clear(self.level);
            self.summon_boss();
        }
    }

    fn summon_boss(&mut self) {
        let mut available = self.available_pairs();
        let count = boss_hits(self.level).min(available.len());
        if count == 0 {
            return;
        }
        let mut pairs = Vec::with_capacity(count);
        for _ in 0..count {
            pairs.push(self.rng.take(&mut available));
        }
        let earlier = self.count_appearance(pairs[0]);
        let phase = self.rng.range(0.0, 100.0);
        let mut boss = Enemy::boss(&pairs, earlier, phase);
        // The boss is slow, so without a portal it starts just inside the
        // edge.
        boss.pos = match self.portal_spawn_position() {
            Some(pos) => pos,
            None => self.spawn_position(&boss),
        };
        boss.keep_on_screen();
        self.enemies.push(boss);

        self.boss_fight = true;
        self.sfx.push(Sfx::Boss);
        self.banner = Some(Banner {
            title: "Pomo saapuu!".to_owned(),
            subtitle: format!("Tarvitaan {count} osumaa"),
            color: VIOLET,
            seconds_left: BANNER_SECONDS,
        });
    }

    /// Starts the next level with a lightning strike at `pos`, where the
    /// boss fell. Any monsters left on screen explode with it, so the next
    /// level starts from a clear slate.
    fn complete_level(&mut self, pos: Vec2) {
        self.level += 1;
        self.level_points = 0;
        self.level_time = 0.0;
        self.boss_fight = false;
        self.spawn_timer = LEVEL_BREAK_SECONDS;
        for enemy in self.enemies.drain(..) {
            self.effects.explode(enemy.pos, enemy.radius, &KILL_PALETTE);
        }
        self.number_typed.clear();
        self.word_typed.clear();
        self.effects.lightning(pos);
        self.sfx.extend([Sfx::Explode, Sfx::Thunder, Sfx::LevelUp]);
        let new_pairs = self.curriculum.next_level();
        let (width, height) = (screen_width(), screen_height());
        self.portals = portal_positions(self.level, width, height);
        self.obstacles = obstacles_for_level(self.level, width, height, &self.portals);
        // She may be standing where a new obstacle appeared.
        self.player = push_out(self.player, PLAYER_RADIUS, &self.obstacles);
        self.banner = Some(Banner {
            title: format!("Taso {}!", self.level),
            subtitle: if new_pairs > 0 {
                format!("Hienoa! {new_pairs} uutta paria")
            } else {
                "Hienoa!".to_owned()
            },
            color: GOLD,
            seconds_left: BANNER_SECONDS,
        });
    }

    fn cast_spell(&mut self, target: SpellTarget, target_pos: Vec2) {
        self.sfx.push(Sfx::Cast);
        self.cast = Some((target_pos, CAST_SECONDS));
        self.spells.push(Spell {
            pos: girl_hand(self.player, target_pos),
            target,
            target_pos,
        });
    }

    fn update_spells(&mut self, dt: f32) {
        if let Some((_, seconds_left)) = &mut self.cast {
            *seconds_left -= dt;
            if *seconds_left <= 0.0 {
                self.cast = None;
            }
        }
        for enemy in &mut self.enemies {
            if let Some(lives) = &mut enemy.boss {
                lives.hit_flash = (lives.hit_flash - dt).max(0.0);
            }
        }

        let step = SPELL_SPEED * dt;
        let player = self.player;
        let effects = &mut self.effects;
        let sfx = &mut self.sfx;
        let enemies = &mut self.enemies;
        let mut boss_fell_at = None;
        self.spells.retain_mut(|spell| {
            // A spell at the boss follows it around.
            let boss = enemies.iter_mut().find(|e| e.is_boss());
            if let (SpellTarget::Boss, Some(boss)) = (&spell.target, &boss) {
                spell.target_pos = boss.pos;
            }

            let to_target = spell.target_pos - spell.pos;
            if to_target.length() > step {
                spell.pos += to_target.normalize() * step;
                effects.trail(spell.pos, &SPELL_PALETTE);
                return true;
            }

            let pos = spell.target_pos;
            sfx.push(Sfx::Explode);
            effects.explode(pos, ENEMY_RADIUS * 0.5, &SPELL_PALETTE);
            match (&spell.target, boss) {
                (SpellTarget::Doomed(enemy), _) => {
                    effects.explode(pos, enemy.radius, &KILL_PALETTE);
                    if enemy.is_boss() {
                        // A boss goes out with a bigger bang.
                        effects.explode_around(pos, 30.0, 3, enemy.radius, &KILL_PALETTE);
                        boss_fell_at = Some(pos);
                    }
                }
                (SpellTarget::Boss, Some(boss)) => {
                    if let Some(lives) = &mut boss.boss {
                        lives.hit_flash = BOSS_HIT_FLASH_SECONDS;
                    }
                    boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_HIT_KNOCKBACK;
                    boss.keep_on_screen();
                }
                // The boss fell before this spell arrived.
                (SpellTarget::Boss, None) => {}
            }
            false
        });

        if let Some(pos) = boss_fell_at
            && !self.is_over()
        {
            self.complete_level(pos);
        }
    }

    fn move_enemies(&mut self, dt: f32) {
        let player = self.player;
        for enemy in &mut self.enemies {
            let speed = enemy.speed();
            let toward = (player - enemy.pos).normalize_or_zero();
            // Enemies with an even phase go left around obstacles, the rest
            // right, so they don't all bunch up on one side.
            let prefer_left = (enemy.phase as u32).is_multiple_of(2);
            let dir = steer(
                enemy.pos,
                enemy.radius,
                toward,
                &self.obstacles,
                prefer_left,
            );
            enemy.pos += dir * speed * dt;
            enemy.age += dt;
            enemy.shown_for += dt;
        }
        self.separate_enemies();
        for enemy in &mut self.enemies {
            if enemy.is_boss() {
                enemy.keep_on_screen();
            }
            enemy.pos = push_out(enemy.pos, enemy.radius, &self.obstacles);
        }

        let touches = |e: &Enemy| e.pos.distance(player) < PLAYER_RADIUS + e.radius;
        let mut hurt = false;

        // The boss bounces off her and starts slow again.
        for boss in self.enemies.iter_mut() {
            let touching = touches(boss);
            let Some(lives) = &mut boss.boss else {
                continue;
            };
            lives.harmless_for = (lives.harmless_for - dt).max(0.0);
            if !touching || lives.harmless_for > 0.0 {
                continue;
            }
            lives.harmless_for = BOSS_HARMLESS_SECONDS;
            hurt = true;
            self.energy -= COLLISION_PENALTY;
            self.sfx.push(Sfx::Hurt);
            self.effects
                .explode((boss.pos + player) / 2.0, ENEMY_RADIUS, &COLLISION_PALETTE);
            boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_COLLISION_KNOCKBACK;
            boss.keep_on_screen();
            boss.age = 0.0;
        }

        let (collided, remaining): (Vec<Enemy>, Vec<Enemy>) = self
            .enemies
            .drain(..)
            .partition(|e| !e.is_boss() && touches(e));
        self.enemies = remaining;
        for enemy in &collided {
            hurt = true;
            self.energy -= COLLISION_PENALTY;
            self.sfx.extend([Sfx::Explode, Sfx::Hurt]);
            self.effects
                .explode(enemy.pos, enemy.radius, &COLLISION_PALETTE);
            // Show the pair so a collision still teaches something.
            self.show_pair(enemy.pair, YELLOW);
        }
        if hurt {
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
                        Vec2::from_angle(self.rng.range(0.0, std::f32::consts::TAU))
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

    /// The unlocked pairs not currently on screen, including those of
    /// enemies still waiting for a spell to hit them.
    fn available_pairs(&self) -> Vec<Pair> {
        let doomed = self.spells.iter().filter_map(|s| match &s.target {
            SpellTarget::Doomed(enemy) => Some(enemy),
            SpellTarget::Boss => None,
        });
        let on_screen: Vec<Pair> = self.enemies.iter().chain(doomed).map(|e| e.pair).collect();
        self.curriculum
            .unlocked()
            .iter()
            .filter(|p| !on_screen.contains(p))
            .copied()
            .collect()
    }

    /// Spawns an enemy just outside a screen edge, away from the player and
    /// the other enemies, never repeating a pair that is already on screen.
    fn spawn_enemy(&mut self) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let available = self.available_pairs();
        if available.is_empty() {
            return;
        }
        let new: Vec<Pair> = available
            .iter()
            .filter(|p| self.curriculum.is_new(p))
            .copied()
            .collect();
        let pool = if !new.is_empty() && self.rng.chance(NEW_PAIR_SHARE) {
            &new
        } else {
            &available
        };
        let pair = *self.rng.pick(pool);
        let earlier = self.count_appearance(pair);
        let shows_word = self.rng.chance(0.5);
        let phase = self.rng.range(0.0, 100.0);
        let mut enemy = Enemy::new(pair, shows_word, earlier, phase);
        let portal = if self.rng.chance(PORTAL_SPAWN_SHARE) {
            self.portal_spawn_position()
        } else {
            None
        };
        enemy.pos = match portal {
            Some(pos) => pos,
            None => self.spawn_position(&enemy),
        };
        self.enemies.push(enemy);
    }

    /// A random portal far enough from the player to spawn from, if any,
    /// with a burst of sparks as something comes through.
    fn portal_spawn_position(&mut self) -> Option<Vec2> {
        let usable: Vec<Vec2> = self
            .portals
            .iter()
            .filter(|p| p.distance(self.player) >= MIN_PORTAL_SPAWN_DISTANCE)
            .copied()
            .collect();
        if usable.is_empty() {
            return None;
        }
        let pos = *self.rng.pick(&usable);
        self.effects.explode(pos, 24.0, &PORTAL_PALETTE);
        Some(pos)
    }

    /// A random point just outside a screen edge, away from the player and
    /// clear of the other enemies, or else the farthest one from the player
    /// that was tried.
    fn spawn_position(&mut self, enemy: &Enemy) -> Vec2 {
        let reach = enemy.reach();
        let center_offset = enemy.reach_center() - enemy.pos;
        let mut best = (Vec2::ZERO, f32::NEG_INFINITY);
        for _ in 0..SPAWN_ATTEMPTS {
            let pos = random_edge_point(&mut self.rng, reach);
            let player_distance = pos.distance(self.player);
            let clear = player_distance >= MIN_SPAWN_DISTANCE
                && self.enemies.iter().all(|other| {
                    other.reach_center().distance(pos + center_offset) >= other.reach() + reach
                });
            if clear {
                return pos;
            }
            if player_distance > best.1 {
                best = (pos, player_distance);
            }
        }
        best.0
    }

    pub fn draw(&self) {
        clear_background(BACKGROUND);
        let time = get_time() as f32;
        // Under everything else, so monsters are never hidden behind it.
        self.draw_new_pairs();
        for &portal in &self.portals {
            draw_portal(portal, time);
        }
        for obstacle in &self.obstacles {
            draw_obstacle(obstacle, time);
        }

        for enemy in &self.enemies {
            enemy.draw_body(time, self.player);
        }
        for spell in &self.spells {
            // Doomed monsters freeze and flash while the spell flies.
            if let SpellTarget::Doomed(target) = &spell.target {
                target.draw_body(time, self.player);
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
        self.effects.draw_flash();

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
        } else if let Some(banner) = &self.banner {
            let alpha = (banner.seconds_left / 0.5).min(1.0);
            let (cx, cy) = (screen_width() / 2.0, screen_height() / 3.0);
            draw_centered_text(
                &banner.title,
                cx,
                cy,
                72,
                Color {
                    a: alpha,
                    ..banner.color
                },
            );
            draw_centered_text(
                &banner.subtitle,
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
        // Marks the level wrong keys can't take her below.
        let low_x = 16.0 + bar_width * LOW_ENERGY / MAX_ENERGY;
        draw_line(low_x, 12.0, low_x, 36.0, 2.0, WHITE);
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
        let (bar_color, level) = if self.boss_fight {
            (VIOLET, format!("Taso {}: POMO", self.level))
        } else {
            (
                GOLD,
                format!("Taso {}: {}/{}", self.level, self.level_points, needed),
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
        let size = measure_text(&level, None, 22, 1.0);
        draw_text(&level, right - size.width, 76.0, 22.0, LIGHTGRAY);

        let (cx, bottom) = (screen_width() / 2.0, screen_height());
        if let Some(feedback) = &self.feedback {
            draw_centered_text(&feedback.text, cx, bottom - 30.0, 32, feedback.color);
        }
    }

    /// Lists the pairs introduced on this level down the right edge. The
    /// panel is gone once a level brings nothing new.
    fn draw_new_pairs(&self) {
        const WIDTH: f32 = 150.0;
        const ROW: f32 = 24.0;
        const FONT_SIZE: u16 = 22;
        let new = self.curriculum.new_pairs();
        if new.is_empty() {
            return;
        }
        let x = screen_width() - WIDTH - 16.0;
        let y = 100.0;
        let height = 40.0 + ROW * new.len() as f32;
        draw_rectangle(x, y, WIDTH, height, Color::new(0.0, 0.0, 0.0, 0.55));
        draw_rectangle_lines(x, y, WIDTH, height, 2.0, GOLD);
        draw_centered_text("Uudet parit", x + WIDTH / 2.0, y + 18.0, FONT_SIZE, GOLD);
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
fn random_edge_point(rng: &mut Rng, margin: f32) -> Vec2 {
    let (w, h) = (screen_width(), screen_height());
    match rng.index(0..4) {
        0 => vec2(rng.range(0.0, w), -margin),
        1 => vec2(rng.range(0.0, w), h + margin),
        2 => vec2(-margin, rng.range(0.0, h)),
        _ => vec2(w + margin, rng.range(0.0, h)),
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
    draw_centered_text(&enemy.label, center.x, center.y, LABEL_FONT_SIZE, WHITE);

    if enemy.shows_hint() {
        let hint_y = center.y + height / 2.0 + HINT_SPACE / 2.0;
        draw_centered_text(&enemy.hint, center.x, hint_y, HINT_FONT_SIZE, LIME);
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
    fn every_enemy_appears_without_a_hint() {
        for earlier in 0..10 {
            assert!(!shows_hint(earlier, 0.0, enemy_speed(0.0) / PLAYER_SPEED));
        }
    }

    #[test]
    fn first_appearances_of_a_pair_get_a_hint_soon() {
        for earlier in 0..HINTED_APPEARANCES {
            assert!(!shows_hint(earlier, EARLY_HINT_SECONDS - 0.1, 0.0));
            assert!(shows_hint(earlier, EARLY_HINT_SECONDS, 0.0));
        }
    }

    #[test]
    fn later_appearances_get_a_hint_only_when_fast() {
        let earlier = HINTED_APPEARANCES;
        assert!(!shows_hint(earlier, 1000.0, HINT_SPEED_FRACTION - 0.01));
        assert!(shows_hint(earlier, 0.0, HINT_SPEED_FRACTION));
    }

    #[test]
    fn wrong_keys_cost_energy_above_the_low_level() {
        assert_eq!(after_wrong_key(100.0), 100.0 - WRONG_PENALTY);
    }

    #[test]
    fn wrong_keys_never_go_below_the_low_level() {
        assert_eq!(after_wrong_key(LOW_ENERGY + 1.0), LOW_ENERGY);
        assert_eq!(after_wrong_key(LOW_ENERGY), LOW_ENERGY);
        assert_eq!(after_wrong_key(5.0), 5.0);
    }

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
    fn first_boss_takes_three_hits() {
        assert_eq!(boss_hits(1), 3);
    }

    #[test]
    fn bosses_get_tougher_up_to_a_limit() {
        for level in 1..100 {
            assert!(boss_hits(level + 1) >= boss_hits(level));
        }
        assert!(boss_hits(10) > boss_hits(1));
        assert_eq!(boss_hits(1000), BOSS_MAX_HITS);
    }

    #[test]
    fn enemies_speed_up_with_age_to_the_players_speed() {
        assert_eq!(enemy_speed(0.0), START_SPEED);
        assert!(enemy_speed(10.0) > enemy_speed(0.0));
        assert_eq!(enemy_speed(1000.0), PLAYER_SPEED);
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
