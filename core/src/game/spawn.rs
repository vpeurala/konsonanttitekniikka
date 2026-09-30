//! Choosing which monsters and bosses appear, and where.

use glam::{Vec2, vec2};

use super::enemy::Enemy;
use super::rules::{
    MAX_ENEMIES, MIN_PORTAL_SPAWN_DISTANCE, MIN_SPAWN_DISTANCE, NEW_PAIR_SHARE, PORTAL_SPAWN_SHARE,
    SPAWN_ATTEMPTS, boss_hits, speed_growth,
};
use super::world::World;
use crate::arena::{ARENA_H, ARENA_W};
use crate::curriculum::Curriculum;
use crate::long_numbers::{self, Question};
use crate::memory::Memory;
use crate::pairs::Pair;
use crate::rng::Rng;

/// What choosing a newcomer depends on, besides the world and luck.
pub struct SpawnContext<'a> {
    pub curriculum: &'a Curriculum,
    /// What she knows, which decides what she is asked about more often.
    pub memory: &'a Memory,
    pub now: f64,
    pub player: Vec2,
    pub level: u32,
}

/// A boss was summoned.
pub struct Summoned {
    /// How many numbers, and so hits, it has.
    pub count: usize,
    /// How many of them are long numbers.
    pub long: usize,
    /// The portal it came through, if any.
    pub portal: Option<Vec2>,
}

impl World {
    /// The pairs that could appear now: unlocked, and not already on the
    /// screen, so nobody is asked about the same pair twice at once.
    fn available_pairs(&self, ctx: &SpawnContext) -> Vec<Pair> {
        let on_screen: Vec<Pair> = self
            .enemies()
            .iter()
            .chain(self.spells().iter().filter_map(|s| s.doomed()))
            .flat_map(|e| e.question.pairs().iter().copied())
            .collect();
        ctx.curriculum
            .unlocked()
            .iter()
            .filter(|p| !on_screen.contains(p))
            .copied()
            .collect()
    }

    /// Adds a monster, unless there are enough already or no pair is
    /// free. Returns the portal it came through, if any.
    pub(super) fn spawn_enemy(&mut self, ctx: &SpawnContext, rng: &mut Rng) -> Option<Vec2> {
        if self.enemies().len() >= MAX_ENEMIES {
            return None;
        }
        let available = self.available_pairs(ctx);
        if available.is_empty() {
            return None;
        }
        let new: Vec<Pair> = available
            .iter()
            .filter(|p| ctx.curriculum.is_new(p))
            .copied()
            .collect();
        let pool = if !new.is_empty() && rng.chance(NEW_PAIR_SHARE) {
            &new
        } else {
            &available
        };
        // Pairs she knows less well come up more often.
        let pair = pool[rng.weighted_index(pool, |p| ctx.memory.weight(p, ctx.now))];
        let question = Question::single(pair);
        let earlier = self.count_appearance(&question);
        let shows_word = rng.chance(0.5);
        let phase = rng.range(0.0, 100.0);
        let mut enemy = Enemy::new(question, shows_word, earlier, phase);
        enemy.speed_growth = speed_growth(ctx.level);
        let portal = if rng.chance(PORTAL_SPAWN_SHARE) {
            self.portal_spawn_position(ctx, rng)
        } else {
            None
        };
        enemy.pos = match portal {
            Some(pos) => pos,
            None => self.spawn_position(ctx, &enemy, rng),
        };
        self.admit(enemy);
        portal
    }

    /// A portal a newcomer can come through: one that is not too close to
    /// her.
    fn portal_spawn_position(&self, ctx: &SpawnContext, rng: &mut Rng) -> Option<Vec2> {
        let usable: Vec<Vec2> = self
            .portals()
            .iter()
            .filter(|p| p.distance(ctx.player) >= MIN_PORTAL_SPAWN_DISTANCE)
            .copied()
            .collect();
        if usable.is_empty() {
            return None;
        }
        Some(*rng.pick(&usable))
    }

    /// Somewhere off the edge, far from her and from other monsters, or
    /// else the farthest of several tries.
    fn spawn_position(&self, ctx: &SpawnContext, enemy: &Enemy, rng: &mut Rng) -> Vec2 {
        let reach = enemy.reach();
        let center_offset = enemy.reach_center() - enemy.pos;
        let mut best = (Vec2::ZERO, f32::NEG_INFINITY);
        for _ in 0..SPAWN_ATTEMPTS {
            let pos = random_edge_point(rng, reach);
            let player_distance = pos.distance(ctx.player);
            let clear = player_distance >= MIN_SPAWN_DISTANCE
                && self.enemies().iter().all(|other| {
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

    /// Summons the level's boss, unless there is no pair free to ask
    /// about. It shows several numbers, one after another, the last of
    /// them long numbers when the level has any.
    pub(super) fn summon_boss(&mut self, ctx: &SpawnContext, rng: &mut Rng) -> Option<Summoned> {
        let mut available = self.available_pairs(ctx);
        let count = boss_hits(ctx.level).min(available.len());
        if count == 0 {
            return None;
        }
        let (level, memory, now) = (ctx.level, ctx.memory, ctx.now);
        let long = long_numbers::long_hits(level, count);
        let easy = long_numbers::easy_long_numbers(level);
        let mut numbers = Vec::with_capacity(count);
        for _ in long..count {
            let i = rng.weighted_index(&available, |p| memory.weight(p, now));
            numbers.push(Question::single(available.swap_remove(i)));
        }
        // The long numbers come last. At first they are made of the pairs
        // she knows best, later of the ones due for practice.
        let unlocked = ctx.curriculum.unlocked();
        for _ in 0..long {
            let question = if easy {
                long_numbers::random_long_number(level, unlocked, rng, |p| {
                    (1.0 - memory.difficulty(p)).powi(3) + 0.01
                })
            } else {
                long_numbers::random_long_number(level, unlocked, rng, |p| memory.weight(p, now))
            };
            numbers.push(question);
        }
        let earlier = self.count_appearance(&numbers[0]);
        let phase = rng.range(0.0, 100.0);
        let mut boss = Enemy::boss(&numbers, easy, earlier, phase);
        boss.speed_growth = speed_growth(ctx.level);
        // The boss is slow, so without a portal it starts just inside the
        // edge.
        let portal = self.portal_spawn_position(ctx, rng);
        boss.pos = match portal {
            Some(pos) => pos,
            None => self.spawn_position(ctx, &boss, rng),
        };
        boss.keep_on_screen();
        self.admit(boss);
        Some(Summoned {
            count,
            long,
            portal,
        })
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
