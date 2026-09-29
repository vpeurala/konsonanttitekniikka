//! Bringing monsters and bosses into the arena.

use macroquad::prelude::{VIOLET, Vec2, vec2};

use super::enemy::Enemy;
use super::rules::*;
use super::{Banner, Game};
use crate::audio::Sfx;
use crate::long_numbers::{self, Question};
use crate::pairs::Pair;
use crate::rng::Rng;
use crate::view::{ARENA_H, ARENA_W};

impl Game {
    /// The unlocked pairs not currently on screen, including those of
    /// enemies still waiting for a spell to hit them.
    pub(super) fn available_pairs(&self) -> Vec<Pair> {
        let on_screen: Vec<Pair> = self
            .enemies
            .iter()
            .chain(self.spells.iter().filter_map(|s| s.doomed()))
            .flat_map(|e| e.question.pairs().iter().copied())
            .collect();
        self.curriculum
            .unlocked()
            .iter()
            .filter(|p| !on_screen.contains(p))
            .copied()
            .collect()
    }

    /// Spawns an enemy just outside a screen edge, away from the player and
    /// the other enemies, never repeating a pair that is already on screen.
    pub(super) fn spawn_enemy(&mut self) {
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
        // Pairs she knows less well come up more often.
        let now = self.now;
        let pair = pool[self
            .rng
            .weighted_index(pool, |p| self.memory.weight(p, now))];
        let question = Question::single(pair);
        let earlier = self.count_appearance(&question);
        let shows_word = self.rng.chance(0.5);
        let phase = self.rng.range(0.0, 100.0);
        let mut enemy = Enemy::new(question, shows_word, earlier, phase, self.text_width);
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

    pub(super) fn summon_boss(&mut self) {
        let mut available = self.available_pairs();
        let count = boss_hits(self.level).min(available.len());
        if count == 0 {
            return;
        }
        let now = self.now;
        let long = long_numbers::long_hits(self.level, count);
        let easy = long_numbers::easy_long_numbers(self.level);
        let mut numbers = Vec::with_capacity(count);
        for _ in long..count {
            let i = self
                .rng
                .weighted_index(&available, |p| self.memory.weight(p, now));
            numbers.push(Question::single(available.swap_remove(i)));
        }
        // The long numbers come last. At first they are made of the pairs
        // she knows best, later of the ones due for practice.
        let unlocked = self.curriculum.unlocked().to_vec();
        for _ in 0..long {
            let memory = &self.memory;
            let question = if easy {
                long_numbers::random_long_number(self.level, &unlocked, &mut self.rng, |p| {
                    (1.0 - memory.difficulty(p)).powi(3) + 0.01
                })
            } else {
                long_numbers::random_long_number(self.level, &unlocked, &mut self.rng, |p| {
                    memory.weight(p, now)
                })
            };
            numbers.push(question);
        }
        let earlier = self.count_appearance(&numbers[0]);
        let phase = self.rng.range(0.0, 100.0);
        let mut boss = Enemy::boss(&numbers, easy, earlier, phase, self.text_width);
        // The boss is slow, so without a portal it starts just inside the
        // edge.
        boss.pos = match self.portal_spawn_position() {
            Some(pos) => pos,
            None => self.spawn_position(&boss),
        };
        boss.keep_on_screen();
        self.enemies.push(boss);

        self.boss_fight = true;
        self.out.sfx.push(Sfx::Boss);
        let subtitle = if long > 0 && self.level == long_numbers::first_long_level() {
            format!("Tarvitaan {count} osumaa. Viimeinen on pitkä luku!")
        } else if long > 0 {
            format!("Tarvitaan {count} osumaa, niistä {long} pitkää lukua")
        } else {
            format!("Tarvitaan {count} osumaa")
        };
        self.banner = Some(Banner {
            title: "Pomo saapuu!".to_owned(),
            subtitle,
            color: VIOLET,
            stars: None,
            seconds_left: BANNER_SECONDS,
        });
    }
}

/// A random point just outside a screen edge, `margin` beyond it.
fn random_edge_point(rng: &mut Rng, margin: f32) -> Vec2 {
    let (w, h) = (ARENA_W, ARENA_H);
    match rng.index(0..4) {
        0 => vec2(rng.range(0.0, w), -margin),
        1 => vec2(rng.range(0.0, w), h + margin),
        2 => vec2(-margin, rng.range(0.0, h)),
        _ => vec2(w + margin, rng.range(0.0, h)),
    }
}
