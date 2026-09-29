//! The game itself: monsters carry numbers or words toward the heroine,
//! and typing the matching word or number casts a spell at them.
//!
//! Everything but `render` is a pure simulation. `Game::update` gets what
//! the outside world says this frame (`Input`: elapsed time, the clock,
//! keys) and reports what should happen outside (`Outputs`: sounds and
//! events). It reads no global state, so a game can be played in a test
//! by feeding it inputs. `render` only reads a `Game`.

mod answer;
mod combat;
mod display;
mod enemy;
mod metrics;
mod render;
mod rules;
mod spawn;

use std::collections::HashMap;

use macroquad::prelude::{Color, GOLD, GREEN, RED, Vec2, vec2};

use crate::audio::Sfx;
use crate::curriculum::Curriculum;
use crate::keyboard::Key;
use crate::long_numbers::Question;
use crate::memory::Memory;
use crate::obstacles::{Obstacle, obstacles_for_level, push_out};
use crate::pairs::{self, PairId};
use crate::portals::portal_positions;
use crate::rng::{Rng, Stream};
use crate::view::{ARENA_H, ARENA_W};

pub use answer::{InputOutcome, resolve_input};
use answer::{Slot, Typed};
use combat::Spell;
use display::Display;
use enemy::{Enemy, EnemyId};
use rules::*;

/// What the outside world says happened this frame.
#[derive(Debug, Default, Clone)]
pub struct Input {
    /// Seconds since the previous frame.
    pub dt: f32,
    /// The current time, in seconds since 1970, for spaced repetition.
    pub now: f64,
    /// The app was in the background since the previous frame.
    pub away: bool,
    /// The characters typed, in order.
    pub keys: Vec<Key>,
    /// The arrow keys held down, each axis -1, 0 or 1.
    pub arrows: Vec2,
    /// The touch joystick's direction; its length, up to 1, is how far it
    /// is pushed.
    pub stick: Vec2,
    /// Space, or the on-screen pause button, was pressed.
    pub pause: bool,
    /// Enter was pressed.
    pub confirm: bool,
    /// Taps in the arena that didn't start the joystick moving.
    pub taps: usize,
}

/// What the game asks the outside world to do.
#[derive(Debug, Default)]
pub struct Outputs {
    /// Sound effects to play.
    pub sfx: Vec<Sfx>,
    /// Things to save or count.
    pub events: Vec<GameEvent>,
}

impl Outputs {
    fn extend(&mut self, later: Outputs) {
        self.sfx.extend(later.sfx);
        self.events.extend(later.events);
    }
}

/// Something the game wants saved, reported in `Outputs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    /// A game started (or started again) from this level.
    Started { level: u32 },
    /// The game ended on this level.
    Over { level: u32 },
    /// A level was finished (its boss beaten) with this many stars.
    LevelCompleted { level: u32, stars: u8 },
}

const BACKGROUND: Color = Color::new(0.09, 0.09, 0.125, 1.0);

/// What counts appearances of a pair: each pair on its own, and all long
/// numbers together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Appearance {
    Pair(PairId),
    Long,
}

/// What was typed or tapped since the game last advanced. A frame can be
/// shorter than a step, so these wait for the step that acts on them.
#[derive(Debug, Default)]
struct Pending {
    keys: Vec<Key>,
    taps: usize,
    confirm: bool,
}

/// How the current level is going; starts over with every level.
#[derive(Debug, Default)]
struct Stage {
    /// Points scored.
    points: u32,
    /// Seconds spent.
    time: f32,
    /// Whether the boss has been summoned and not yet beaten.
    boss_fight: bool,
}

pub struct Game {
    player: Vec2,
    player_moving: bool,
    enemies: Vec<Enemy>,
    /// The id the next enemy to join gets.
    next_enemy_id: u32,
    /// How many times each pair has appeared in this game.
    appearances: HashMap<Appearance, u32>,
    spells: Vec<Spell>,
    typed: Typed,
    energy: f32,
    score: u32,
    level: u32,
    /// How the current level is going.
    stage: Stage,
    spawn_timer: f32,
    /// What to tell the outside world, until `update` hands it over.
    out: Outputs,
    /// What is shown without affecting play.
    display: Display,
    /// Every gameplay decision's randomness, the same in every game.
    rng: Rng,
    curriculum: Curriculum,
    /// The current level's portals.
    portals: Vec<Vec2>,
    /// The current level's stones, trees and lakes.
    obstacles: Vec<Obstacle>,
    /// Whether the game is paused with the space bar.
    paused: bool,
    /// Whether she plays with touch controls, which changes some texts.
    touch: bool,
    /// How well she knows each pair; kept across games.
    memory: Memory,
    /// The level the game started from, where it starts again after a
    /// game over.
    start_level: u32,
    /// The time of the latest frame, as `Input::now`.
    now: f64,
    /// Cuts frames into the fixed steps the game advances in.
    timestep: Timestep,
    /// Typing and taps not acted on yet.
    pending: Pending,
    /// Seconds the game has run, not counting pauses. Animations follow
    /// this rather than the clock, so a pause freezes them too.
    play_time: f64,
}

impl Game {
    /// A new game starting from `start_level`, knowing what `memory` says
    /// about each pair.
    pub fn new(touch: bool, memory: Memory, start_level: u32) -> Self {
        let mut game = Game {
            player: vec2(ARENA_W / 2.0, ARENA_H / 2.0),
            player_moving: false,
            enemies: Vec::new(),
            next_enemy_id: 0,
            appearances: HashMap::new(),
            spells: Vec::new(),
            typed: Typed::default(),
            energy: MAX_ENERGY,
            score: 0,
            level: 1,
            stage: Stage::default(),
            spawn_timer: 1.0,
            out: Outputs::default(),
            display: Display::default(),
            rng: Rng::new(Stream::Gameplay, 0),
            curriculum: Curriculum::new(),
            portals: portal_positions(1, ARENA_W, ARENA_H),
            obstacles: Vec::new(),
            paused: false,
            touch,
            memory,
            start_level: 1,
            now: 0.0,
            timestep: Timestep::default(),
            pending: Pending::default(),
            play_time: 0.0,
        };
        game.start_at(start_level);
        game.out
            .events
            .push(GameEvent::Started { level: game.level });
        game
    }

    /// Skips ahead to `level`, with every pair of the levels before it
    /// already met.
    fn start_at(&mut self, level: u32) {
        self.start_level = level.max(1);
        while self.level < self.start_level {
            self.level += 1;
            self.curriculum.next_level();
        }
        if self.level > 1 {
            self.portals = portal_positions(self.level, ARENA_W, ARENA_H);
            self.obstacles = obstacles_for_level(self.level, ARENA_W, ARENA_H, &self.portals);
            self.player = push_out(self.player, PLAYER_RADIUS, &self.obstacles);
            self.display.announce(
                format!("Taso {}", self.level),
                "Onnea matkaan!".to_owned(),
                GOLD,
                None,
            );
        }
    }

    /// Starts over from the level this game started from. What she has
    /// learned carries over to the new game.
    fn restart(&mut self) {
        let memory = std::mem::take(&mut self.memory);
        let mut out = std::mem::take(&mut self.out);
        *self = Game::new(self.touch, memory, self.start_level);
        out.extend(std::mem::take(&mut self.out));
        self.out = out;
    }

    /// What the game knows about each pair, for saving.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn is_over(&self) -> bool {
        self.energy <= 0.0
    }

    /// Pauses the game, unless it is already over.
    fn pause(&mut self) {
        if !self.is_over() {
            self.paused = true;
        }
    }

    /// Plays one frame and returns what the outside world should do about
    /// it.
    pub fn update(&mut self, input: &Input) -> Outputs {
        self.now = input.now;
        // Coming back after being away finds the game paused, not lost.
        if input.away {
            self.pause();
        }
        let pause_pressed = input.pause
            // A tap anywhere in the arena also resumes.
            || (self.paused && input.taps > 0);
        if pause_pressed && !self.is_over() {
            self.paused = !self.paused;
        }
        // Keys typed during a pause are dropped along with the frame.
        if self.paused {
            self.pending = Pending::default();
        } else {
            self.pending.keys.extend(&input.keys);
            self.pending.taps += input.taps;
            self.pending.confirm |= input.confirm;
            let steps = self.timestep.steps_due(input.dt);
            for step in 0..steps {
                // What was typed is acted on in the first step only.
                let acts = (step == 0).then(|| std::mem::take(&mut self.pending));
                self.advance(STEP_SECONDS, input, acts);
            }
        }
        std::mem::take(&mut self.out)
    }

    /// Advances the game by `dt` seconds, acting on what was typed or
    /// tapped (`acts`) if there is anything new.
    fn advance(&mut self, dt: f32, input: &Input, acts: Option<Pending>) {
        self.play_time += f64::from(dt);
        self.display.update_motion(dt);
        self.update_spells(dt);

        if self.is_over() {
            if acts.is_some_and(|a| a.confirm || a.taps > 0) {
                self.restart();
            }
            return;
        }

        self.stage.time += dt;
        self.display.update_messages(dt);

        self.move_player(dt, input.arrows, input.stick);
        for key in acts.into_iter().flat_map(|a| a.keys) {
            self.handle_key(key);
        }
        self.move_enemies(dt);
        if self.is_over() {
            self.out.events.push(GameEvent::Over { level: self.level });
        }

        // No new monsters join a boss fight.
        if !self.stage.boss_fight {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = spawn_interval(self.stage.time);
                self.spawn_enemy();
            }
        }
    }

    /// Moves her with the arrow keys, or else with the touch joystick's
    /// `stick` direction, whose length says how fast.
    fn move_player(&mut self, dt: f32, arrows: Vec2, stick: Vec2) {
        let velocity = if arrows != Vec2::ZERO {
            arrows.normalize()
        } else {
            stick.clamp_length_max(1.0)
        };
        self.player_moving = velocity != Vec2::ZERO;
        self.player += velocity * PLAYER_SPEED * dt;
        self.player.x = self.player.x.clamp(PLAYER_RADIUS, ARENA_W - PLAYER_RADIUS);
        self.player.y = self
            .player
            .y
            .clamp(PLAYER_RADIUS * 1.5, ARENA_H - PLAYER_RADIUS * 1.5);
        self.player = push_out(self.player, PLAYER_RADIUS, &self.obstacles);
    }

    fn handle_key(&mut self, key: Key) {
        match key {
            // Backspace starts over: it empties both slots.
            Key::Backspace => self.clear_typed(),
            Key::Char(c) if pairs::is_answer_char(c) => self.type_into(Slot::of_char(c), c),
            Key::Char(_) => {}
        }
    }

    fn clear_typed(&mut self) {
        self.typed.clear();
    }

    /// The enemy called `id`, if it is still in play.
    fn enemy(&self, id: EnemyId) -> Option<&Enemy> {
        self.enemies.iter().find(|e| e.id == id)
    }

    /// Adds `enemy` to the game, giving it an id of its own.
    fn admit(&mut self, mut enemy: Enemy) {
        enemy.id = EnemyId(self.next_enemy_id);
        self.next_enemy_id += 1;
        self.enemies.push(enemy);
    }

    /// The enemies this slot can answer, with their answers.
    fn candidates(&self, slot: Slot) -> Vec<(EnemyId, String)> {
        self.enemies
            .iter()
            .filter(|e| e.answer_slot() == slot)
            .map(|e| (e.id, e.answer()))
            .collect()
    }

    fn outcome(&self, slot: Slot) -> (Vec<(EnemyId, String)>, InputOutcome) {
        let candidates = self.candidates(slot);
        let answers = candidates.iter().map(|(_, answer)| answer.as_str());
        let outcome = resolve_input(self.typed.get(slot), answers);
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
            self.out.sfx.push(Sfx::Wrong);
            self.display.say(
                format!("Väärin: {}", self.typed.get(slot).to_uppercase()),
                RED,
            );
            self.typed.get_mut(slot).clear();
            return;
        }

        self.typed.get_mut(slot).push(c);
        self.out.sfx.push(Sfx::Type);
        let (candidates, outcome) = self.outcome(slot);
        if let InputOutcome::Hit(hits) = outcome {
            let ids: Vec<EnemyId> = hits.iter().map(|&hit| candidates[hit].0).collect();
            let first = self
                .enemy(ids[0])
                .map(|e| e.question.clone())
                .expect("a hit enemy is in the game");
            for id in ids {
                self.hit_enemy(id);
            }
            self.energy = (self.energy + HIT_REWARD * hits.len() as f32).min(MAX_ENERGY);
            self.show_question(&first, GREEN);
            self.typed.get_mut(slot).clear();
            self.add_points(hits.len() as u32);
        }
    }

    /// Records that `question` is appearing, returning how many times it
    /// had appeared before. Long numbers are counted together, so the first
    /// few in a game get an early hint.
    fn count_appearance(&mut self, question: &Question) -> u32 {
        let key = if question.is_long() {
            Appearance::Long
        } else {
            Appearance::Pair(question.first().id)
        };
        let count = self.appearances.entry(key).or_default();
        *count += 1;
        *count - 1
    }

    fn add_points(&mut self, points: u32) {
        self.score += points;
        if self.stage.boss_fight {
            return;
        }
        self.stage.points += points;
        if self.stage.points >= points_to_clear(self.level) {
            self.stage.points = points_to_clear(self.level);
            self.summon_boss();
        }
    }

    /// Starts the next level with a lightning strike at `pos`, where the
    /// boss fell. Any monsters left on screen explode with it, so the next
    /// level starts from a clear slate.
    fn complete_level(&mut self, pos: Vec2) {
        let stars = stars_for(self.energy / MAX_ENERGY);
        self.out.events.push(GameEvent::LevelCompleted {
            level: self.level,
            stars,
        });
        self.level += 1;
        self.stage.points = 0;
        self.stage.time = 0.0;
        self.stage.boss_fight = false;
        self.spawn_timer = LEVEL_BREAK_SECONDS;
        for enemy in self.enemies.drain(..) {
            self.display
                .effects
                .explode(enemy.pos, enemy.radius, &KILL_PALETTE);
        }
        self.clear_typed();
        self.display.effects.lightning(pos);
        self.out
            .sfx
            .extend([Sfx::Explode, Sfx::Thunder, Sfx::LevelUp]);
        let new_pairs = self.curriculum.next_level();
        self.portals = portal_positions(self.level, ARENA_W, ARENA_H);
        self.obstacles = obstacles_for_level(self.level, ARENA_W, ARENA_H, &self.portals);
        // She may be standing where a new obstacle appeared.
        self.player = push_out(self.player, PLAYER_RADIUS, &self.obstacles);
        let subtitle = if new_pairs > 0 {
            format!("Hienoa! {new_pairs} uutta paria")
        } else {
            "Hienoa!".to_owned()
        };
        self.display
            .announce(format!("Taso {}!", self.level), subtitle, GOLD, Some(stars));
    }

    fn show_question(&mut self, question: &Question, color: Color) {
        self.display.say(
            format!("{} = {}", question.words(), question.number(false)),
            color,
        );
    }
}

#[cfg(test)]
mod tests;
