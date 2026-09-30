//! What is in the arena: monsters, spells in flight, portals and obstacles.
//!
//! A `World` moves things and keeps its own books, but knows nothing of
//! energy, sounds, sparks or what she has learned. When something happens
//! it says so in a report (`Hit`, `Contact`, `SpellEvent`), and the game
//! decides what that means for her.

use std::collections::HashMap;

use glam::Vec2;

use super::enemy::{Enemy, EnemyId};
use super::rules::*;
use crate::arena::{ARENA_H, ARENA_W, girl_hand};
use crate::long_numbers::Question;
use crate::obstacles::{Obstacle, obstacles_for_level, push_out, steer};
use crate::pairs::PairId;
use crate::portals::portal_positions;
use crate::rng::Rng;

/// What a spell is flying toward.
pub enum SpellTarget {
    /// An enemy that is already out of play and explodes when hit.
    Doomed(Enemy),
    /// The boss, which is still in play and only loses a life.
    Boss,
}

/// A magic bolt flying toward an answered enemy.
pub struct Spell {
    pub pos: Vec2,
    pub target: SpellTarget,
    /// Where the target was last seen, in case the boss dies first.
    pub target_pos: Vec2,
}

impl Spell {
    /// The enemy that is out of play and waiting for this spell.
    pub fn doomed(&self) -> Option<&Enemy> {
        match &self.target {
            SpellTarget::Doomed(enemy) => Some(enemy),
            SpellTarget::Boss => None,
        }
    }
}

/// What counts appearances of a pair: each pair on its own, and all long
/// numbers together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Appearance {
    Pair(PairId),
    Long,
}

/// An enemy was answered.
pub struct Hit {
    /// The enemy as it was when answered, for what she learns from it.
    pub answered: Enemy,
    pub outcome: HitOutcome,
}

pub enum HitOutcome {
    /// A boss with lives left moved on to its next number.
    Wounded,
    /// The enemy left play; a spell is on its way to finish it.
    Defeated { boss: bool },
}

/// A monster or boss reached her.
pub struct Contact {
    /// The enemy as it was when it touched her.
    pub enemy: Enemy,
    pub kind: ContactKind,
}

pub enum ContactKind {
    /// A boss touched her at this point, and bounced away. It stays.
    Boss { at: Vec2 },
    /// An ordinary monster touched her and left play.
    Ordinary,
}

/// One thing a spell did while flying.
pub enum SpellEvent {
    /// It moved on, to this point.
    Trail(Vec2),
    Impact(Impact),
}

/// A spell reached its target.
pub struct Impact {
    pub pos: Vec2,
    pub target: ImpactTarget,
}

pub enum ImpactTarget {
    /// An enemy already out of play exploded.
    Doomed { radius: f32, boss: bool },
    /// The boss lost a life and was knocked back.
    Boss,
    /// The boss it followed was gone.
    Nothing,
}

pub struct World {
    enemies: Vec<Enemy>,
    spells: Vec<Spell>,
    /// The id the next enemy to join gets.
    next_enemy_id: u32,
    /// How many times each pair has appeared in this game.
    appearances: HashMap<Appearance, u32>,
    /// The current level's portals.
    portals: Vec<Vec2>,
    /// The current level's stones, trees and lakes.
    obstacles: Vec<Obstacle>,
}

impl World {
    /// The arena of `level`, with nobody in it.
    pub fn for_level(level: u32) -> World {
        let (portals, obstacles) = layout(level);
        World {
            enemies: Vec::new(),
            spells: Vec::new(),
            next_enemy_id: 0,
            appearances: HashMap::new(),
            portals,
            obstacles,
        }
    }

    pub fn enemies(&self) -> &[Enemy] {
        &self.enemies
    }

    pub fn spells(&self) -> &[Spell] {
        &self.spells
    }

    pub fn portals(&self) -> &[Vec2] {
        &self.portals
    }

    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    /// The enemy called `id`, if it is still in play.
    #[cfg(test)]
    pub fn enemy(&self, id: EnemyId) -> Option<&Enemy> {
        self.enemies.iter().find(|e| e.id == id)
    }

    #[cfg(test)]
    pub(super) fn enemies_mut(&mut self) -> &mut Vec<Enemy> {
        &mut self.enemies
    }

    #[cfg(test)]
    pub(super) fn appearances(&self) -> &HashMap<Appearance, u32> {
        &self.appearances
    }

    /// Adds `enemy` to the world, giving it an id of its own.
    pub fn admit(&mut self, mut enemy: Enemy) {
        enemy.id = EnemyId(self.next_enemy_id);
        self.next_enemy_id += 1;
        self.enemies.push(enemy);
    }

    /// Records that `question` is appearing, returning how many times it
    /// had appeared before. Long numbers are counted together, so the first
    /// few in a game get an early hint.
    pub(super) fn count_appearance(&mut self, question: &Question) -> u32 {
        let key = if question.is_long() {
            Appearance::Long
        } else {
            Appearance::Pair(question.first().id)
        };
        let count = self.appearances.entry(key).or_default();
        *count += 1;
        *count - 1
    }

    /// Moves on to the arena of `level`. Every monster in play leaves, and
    /// is handed back; spells already flying carry on.
    pub fn next_level(&mut self, level: u32) -> Vec<Enemy> {
        let (portals, obstacles) = layout(level);
        self.portals = portals;
        self.obstacles = obstacles;
        std::mem::take(&mut self.enemies)
    }

    /// Answers the enemy called `id`, if it is still in play: a boss with
    /// lives left moves on to its next number, anything else leaves play.
    /// Either way a spell flies from `player`'s hand toward it.
    pub fn hit(&mut self, id: EnemyId, player: Vec2) -> Option<Hit> {
        let index = self.enemies.iter().position(|e| e.id == id)?;
        let answered = self.enemies[index].clone();
        let target_pos = answered.pos;
        let next = self.enemies[index]
            .boss
            .as_mut()
            .and_then(|lives| lives.queue.pop_front());
        let (target, outcome) = match next {
            Some(question) => {
                let earlier = self.count_appearance(&question);
                self.enemies[index].show(question, earlier);
                (SpellTarget::Boss, HitOutcome::Wounded)
            }
            None => {
                let enemy = self.enemies.remove(index);
                let boss = enemy.is_boss();
                (SpellTarget::Doomed(enemy), HitOutcome::Defeated { boss })
            }
        };
        self.spells.push(Spell {
            pos: girl_hand(player, target_pos),
            target,
            target_pos,
        });
        Some(Hit { answered, outcome })
    }

    /// Flies every spell on by `dt` seconds, and says what they did.
    pub fn advance_spells(&mut self, dt: f32, player: Vec2) -> Vec<SpellEvent> {
        for enemy in &mut self.enemies {
            if let Some(lives) = &mut enemy.boss {
                lives.hit_flash = (lives.hit_flash - dt).max(0.0);
            }
        }

        let step = SPELL_SPEED * dt;
        let mut events = Vec::new();
        let enemies = &mut self.enemies;
        self.spells.retain_mut(|spell| {
            // A spell at the boss follows it around.
            let boss = enemies.iter_mut().find(|e| e.is_boss());
            if let (SpellTarget::Boss, Some(boss)) = (&spell.target, &boss) {
                spell.target_pos = boss.pos;
            }

            let to_target = spell.target_pos - spell.pos;
            if to_target.length() > step {
                spell.pos += to_target.normalize() * step;
                events.push(SpellEvent::Trail(spell.pos));
                return true;
            }

            let target = match (&spell.target, boss) {
                (SpellTarget::Doomed(enemy), _) => ImpactTarget::Doomed {
                    radius: enemy.radius,
                    boss: enemy.is_boss(),
                },
                (SpellTarget::Boss, Some(boss)) => {
                    if let Some(lives) = &mut boss.boss {
                        lives.hit_flash = BOSS_HIT_FLASH_SECONDS;
                    }
                    boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_HIT_KNOCKBACK;
                    boss.keep_on_screen();
                    ImpactTarget::Boss
                }
                (SpellTarget::Boss, None) => ImpactTarget::Nothing,
            };
            events.push(SpellEvent::Impact(Impact {
                pos: spell.target_pos,
                target,
            }));
            false
        });
        events
    }

    /// Moves every monster on by `dt` seconds toward `player`, and says
    /// which reached her. Monsters that touch her leave play, apart from a
    /// boss, which bounces away and is harmless for a while.
    pub fn advance_enemies(&mut self, dt: f32, player: Vec2, rng: &mut Rng) -> Vec<Contact> {
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
        self.separate_enemies(rng);
        for enemy in &mut self.enemies {
            if enemy.is_boss() {
                enemy.keep_on_screen();
            }
            enemy.pos = push_out(enemy.pos, enemy.radius, &self.obstacles);
        }

        let touches = |e: &Enemy| e.pos.distance(player) < PLAYER_RADIUS + e.radius;
        let mut contacts = Vec::new();

        // The boss bounces off her and starts slow again.
        for boss in &mut self.enemies {
            let touching = touches(boss);
            let Some(lives) = &mut boss.boss else {
                continue;
            };
            lives.harmless_for = (lives.harmless_for - dt).max(0.0);
            if !touching || lives.harmless_for > 0.0 {
                continue;
            }
            lives.harmless_for = BOSS_HARMLESS_SECONDS;
            contacts.push(Contact {
                enemy: boss.clone(),
                kind: ContactKind::Boss {
                    at: (boss.pos + player) / 2.0,
                },
            });
            boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_COLLISION_KNOCKBACK;
            boss.keep_on_screen();
            boss.age = 0.0;
        }

        let (collided, remaining): (Vec<Enemy>, Vec<Enemy>) = std::mem::take(&mut self.enemies)
            .into_iter()
            .partition(|e| !e.is_boss() && touches(e));
        self.enemies = remaining;
        contacts.extend(collided.into_iter().map(|enemy| Contact {
            enemy,
            kind: ContactKind::Ordinary,
        }));
        contacts
    }

    /// Pushes overlapping enemies apart, a few passes so a crowd settles.
    fn separate_enemies(&mut self, rng: &mut Rng) {
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
                        Vec2::from_angle(rng.range(0.0, std::f32::consts::TAU))
                    };
                    let push = dir * (min_distance - distance) / 2.0;
                    self.enemies[i].pos -= push;
                    self.enemies[j].pos += push;
                }
            }
        }
    }
}

/// The portals and obstacles of a level. The first level has no obstacles.
fn layout(level: u32) -> (Vec<Vec2>, Vec<Obstacle>) {
    let portals = portal_positions(level, ARENA_W, ARENA_H);
    let obstacles = if level > 1 {
        obstacles_for_level(level, ARENA_W, ARENA_H, &portals)
    } else {
        Vec::new()
    };
    (portals, obstacles)
}
